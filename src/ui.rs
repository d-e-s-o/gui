// ui.rs

// *************************************************************************
// * Copyright (C) 2018-2020 Daniel Mueller (deso@posteo.net)              *
// *                                                                       *
// * This program is free software: you can redistribute it and/or modify  *
// * it under the terms of the GNU General Public License as published by  *
// * the Free Software Foundation, either version 3 of the License, or     *
// * (at your option) any later version.                                   *
// *                                                                       *
// * This program is distributed in the hope that it will be useful,       *
// * but WITHOUT ANY WARRANTY; without even the implied warranty of        *
// * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the         *
// * GNU General Public License for more details.                          *
// *                                                                       *
// * You should have received a copy of the GNU General Public License     *
// * along with this program.  If not, see <http://www.gnu.org/licenses/>. *
// *************************************************************************

use std::any::Any;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use std::mem::replace;
use std::ops::Deref;
use std::rc::Rc;
use std::slice::Iter;
#[cfg(debug_assertions)]
use std::sync::atomic::AtomicUsize;
#[cfg(debug_assertions)]
use std::sync::atomic::Ordering;

use crate::BBox;
use crate::ChainEvent;
use crate::CustomEvent;
use crate::EventChain;
use crate::OptionChain;
use crate::Placeholder;
use crate::Renderer;
use crate::UiEvent;
use crate::UiEvents;
use crate::UnhandledEvent;
use crate::UnhandledEvents;
use crate::Widget;


/// An `Index` is our internal representation of an `Id`. `Id`s can
/// belong to different `Ui` objects and a validation step converts them
/// into an `Index`.
#[derive(Clone, Copy, Debug, Eq, Ord, Hash, PartialEq, PartialOrd)]
struct Index {
  idx: usize,
}

impl Index {
  fn new(idx: usize) -> Self {
    Self { idx }
  }
}

impl Display for Index {
  /// Format the `Index` into the given formatter.
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    write!(f, "{}", self.idx)
  }
}


/// An `Id` uniquely representing a widget.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Id {
  #[cfg(debug_assertions)]
  ui_id: usize,
  idx: Index,
}

impl Id {
  #[allow(unused_variables)]
  fn new<E>(idx: usize, ui: &Ui<E>) -> Id
  where
    E: 'static + Debug,
  {
    Self {
      #[cfg(debug_assertions)]
      ui_id: ui.id,
      idx: Index::new(idx),
    }
  }
}

impl Display for Id {
  /// Format the `Id` into the given formatter.
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    write!(f, "{}", self.idx)
  }
}


/// An iterator over the children of a widget.
pub(crate) type ChildIter<'widget> = Iter<'widget, Id>;

type NewDataFn = dyn FnOnce() -> Box<dyn Any>;
type NewWidgetFn<E> = dyn FnOnce(Id, &mut dyn MutCap<E>) -> Box<dyn Widget<E>>;
// Note that we only pass a non-mutable Cap object to the handler. We do
// not want to allow operations such as changing of the input focus or
// overwriting of the event hook itself from the event hook handler.
type EventHookFn<E> = &'static dyn Fn(&dyn Widget<E>, &dyn Cap, &E) -> Option<UiEvents<E>>;


/// A capability allowing for various widget related operations.
pub trait Cap: Debug {
  /// Retrieve a reference to a widget's data.
  fn data(&self, widget: Id) -> &dyn Any;

  /// Retrieve an iterator over the children. Iteration happens in
  /// z-order, from highest to lowest.
  fn children(&self, widget: Id) -> ChildIter<'_>;

  /// Retrieve the `Id` of the root widget.
  fn root_id(&self) -> Id;

  /// Retrieve the parent of the given widget.
  fn parent_id(&self, widget: Id) -> Option<Id>;

  /// Check whether a widget has its visibility flag set.
  ///
  /// Note that a return value of `true` does not necessary mean that
  /// the widget is actually visible. A widget is only visible if all
  /// its parents have the visibility flag set, too. The `is_displayed`
  /// method can be used to check for actual visibility.
  fn is_visible(&self, widget: Id) -> bool;

  /// Check whether a widget is actually being displayed.
  ///
  /// This method checks whether the referenced widget is actually being
  /// displayed, that is, whether its own as well as its parents'
  /// visibility flags are all set.
  fn is_displayed(&self, widget: Id) -> bool;

  /// Retrieve the currently focused widget.
  fn focused(&self) -> Option<Id>;

  /// Check whether the widget with the given `Id` is focused.
  fn is_focused(&self, widget: Id) -> bool;
}


/// A mutable capability allowing for various widget related operations.
pub trait MutCap<E>: Cap + Deref<Target = dyn Cap>
where
  E: Debug,
{
  /// Retrieve a mutable reference to a widget's data.
  fn data_mut(&mut self, widget: Id) -> &mut dyn Any;

  /// Add a widget to the `Ui` represented by the capability.
  // TODO: We should not require a Box here conceptually, but omitting
  //       it will require the unboxed closures feature to stabilize.
  fn add_widget(
    &mut self,
    parent: Id,
    new_data: Box<NewDataFn>,
    new_widget: Box<NewWidgetFn<E>>,
  ) -> Id;

  /// Show a widget, i.e., set its and its parents' visibility flag.
  ///
  /// This method sets the referenced widget's visibility flag as well
  /// as those of all its parents.
  fn show(&mut self, widget: Id);

  /// Hide a widget, i.e., unset its visibility flag.
  ///
  /// This method makes sure that widget referenced is no longer
  /// displayed. If the widget has children, all those children will
  /// also be hidden.
  fn hide(&mut self, widget: Id);

  /// Focus a widget.
  ///
  /// The focused widget is the one receiving certain types of events
  /// (such as key events) first but may also be rendered in a different
  /// color or be otherwise highlighted. Note that being focused implies
  /// being visible. This invariant is enforced internally.
  fn focus(&mut self, widget: Id);

  /// Install or remove an event hook handler.
  ///
  /// The event hook handler is a call back function that is invoked for
  /// all events originating outside of the UI, i.e., those that come in
  /// through the `Ui::handle` method. For such events, the event hook
  /// handler gets to inspect the event before any widget gets a chance
  /// to handle it "officially" through the `Handleable::handle` method.
  ///
  /// Event hook handlers are allowed to emit events on its own, just as
  /// "normal" event handlers. It is guaranteed that these emitted
  /// events will reach the widget after the event that was hooked.
  ///
  /// Note that event hook functions are only able to inspect events and
  /// not change or discard them. That restriction prevents conflicts
  /// due to what effectively comes down to shared global state: widgets
  /// could be racing to install an event hook handler and the order in
  /// which these handlers end up being installed could influence the
  /// handling of events.
  ///
  /// A widget (identified by the given `Id`) may only register one
  /// handler and subsequent requests will overwrite the previously
  /// installed one. The method returns the handler that was previously
  /// installed, if any.
  fn hook_events(&mut self, widget: Id, hook_fn: Option<EventHookFn<E>>) -> Option<EventHookFn<E>>;
}


#[cfg(debug_assertions)]
fn get_next_ui_id() -> usize {
  static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

  NEXT_ID.fetch_add(1, Ordering::Relaxed)
}


/// This type contains data that is common to all widgets.
#[derive(Debug)]
struct WidgetData<E>
where
  E: 'static,
{
  /// The `Id` of the parent widget.
  ///
  /// This value may only be `None` for the root widget.
  parent_idx: Option<Index>,
  /// The data associated with the widget.
  data: Box<dyn Any>,
  /// Vector of all the children that have this widget as a parent.
  // Note that unfortunately there is no straight forward way to make
  // this a Vec<Index> because we cannot use an impl trait return type
  // for the `children` method present in `Cap`.
  children: Vec<Id>,
  /// An optional event hook that may be registered for the widget.
  event_hook: Option<EventHook<E>>,
  /// Flag indicating the widget's visibility state.
  visible: bool,
}

impl<E> WidgetData<E> {
  fn new(parent_idx: Option<Index>, data: Box<dyn Any>) -> Self {
    Self {
      parent_idx,
      data,
      children: Default::default(),
      event_hook: None,
      visible: true,
    }
  }
}


/// A struct wrapping an `EventHookFn` while implementing `Debug`.
struct EventHook<E>(EventHookFn<E>)
where
  E: 'static;

impl<E> Debug for EventHook<E> {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    write!(f, "{:p}", self.0)
  }
}


/// A `Ui` is a container for related widgets.
#[derive(Debug, Default)]
pub struct Ui<E>
where
  E: 'static + Debug,
{
  #[cfg(debug_assertions)]
  id: usize,
  #[allow(clippy::type_complexity)]
  widgets: Vec<(WidgetData<E>, Option<Rc<dyn Widget<E>>>)>,
  hooked: Vec<Index>,
  focused: Option<Index>,
}

impl<E> Ui<E>
where
  E: 'static + Debug,
{
  /// Create a new `Ui` instance containing one widget that acts as the
  /// root widget.
  #[allow(clippy::new_ret_no_self)]
  pub fn new<D, W>(new_data: D, new_root_widget: W) -> (Self, Id)
  where
    D: FnOnce() -> Box<dyn Any>,
    W: FnOnce(Id, &mut dyn MutCap<E>) -> Box<dyn Widget<E>>,
  {
    let mut ui = Self {
      #[cfg(debug_assertions)]
      id: get_next_ui_id(),
      widgets: Default::default(),
      hooked: Default::default(),
      focused: None,
    };

    let id = ui._add_widget(None, new_data, new_root_widget);
    debug_assert_eq!(id.idx.idx, 0);
    (ui, id)
  }

  /// Add a widget to the `Ui`.
  ///
  /// This method fulfills the exact same purpose as
  /// `MutCap::add_widget`, but it does not require boxing up the
  /// provided `FnOnce`.
  // TODO: This method should be removed once we no longer require
  //       boxing up of `FnOnce` closures.
  pub fn add_ui_widget<D, W>(&mut self, parent: Id, new_data: D, new_widget: W) -> Id
  where
    D: FnOnce() -> Box<dyn Any>,
    W: FnOnce(Id, &mut dyn MutCap<E>) -> Box<dyn Widget<E>>,
  {
    let parent_idx = self.validate(parent);
    self._add_widget(Some(parent_idx), new_data, new_widget)
  }

  /// Add a widget to the `Ui`.
  fn _add_widget<D, W>(&mut self, parent_idx: Option<Index>, new_data: D, new_widget: W) -> Id
  where
    D: FnOnce() -> Box<dyn Any>,
    W: FnOnce(Id, &mut dyn MutCap<E>) -> Box<dyn Widget<E>>,
  {
    let idx = Index::new(self.widgets.len());
    let id = Id::new(idx.idx, self);

    // We require some trickery here to allow for dynamic widget
    // creation from within the constructor of another widget. In
    // particular, we install a "dummy" widget that acts as a container
    // to which newly created child widgets can be registered.
    let dummy = Placeholder::new();
    let data = new_data();
    let data = WidgetData::new(parent_idx, data);
    self.widgets.push((data, Some(Rc::new(dummy))));

    // The widget is already linked to its parent but the parent needs to
    // know about the child as well. We do that registration before the
    // widget is actually fully constructed to preserve the invariant
    // that a widget's ID is part of the list of IDs managed by its
    // parent.
    if let Some(parent_idx) = parent_idx {
      self.widgets[parent_idx.idx].0.children.push(id)
    }

    // TODO: Consider making NewWidgetFn return an Rc instead of a Box
    //       to begin with as Rc::from(Box) is a non-trivial operation.
    let widget = Rc::from(new_widget(id, self));
    // Replace our placeholder with the actual widget we just created.
    // Note that because we store the children separately as part of an
    // `WidgetData` object there is no need for us to do anything about
    // them. Note furthermore that this implies that the Widget trait's
    // `add_child` method must not have any side effects.
    self.with(idx, |_, _| (widget, ()));
    id
  }

  /// Validate an `Id`, converting it into the internally used `Index`.
  #[inline]
  fn validate(&self, id: Id) -> Index {
    #[cfg(debug_assertions)]
    debug_assert_eq!(id.ui_id, self.id, "The given Id belongs to a different Ui");
    id.idx
  }

  /// Lookup a widget from an `Index`.
  fn lookup(&self, idx: Index) -> &dyn Widget<E> {
    match &self.widgets[idx.idx].1 {
      Some(widget) => widget.as_ref(),
      None => panic!("Widget {} is currently taken", idx),
    }
  }

  fn children(&self, idx: Index) -> ChildIter<'_> {
    self.widgets[idx.idx].0.children.iter()
  }

  /// Show the widget with the given `Index` and recursively all its parents.
  ///
  /// Note that the given reordering function needs to be idempotent
  /// with respect to repeated reordering of the same widgets.
  fn show<F>(&mut self, idx: Index, reorder_fn: F)
  where
    F: Fn(&mut Ui<E>, Index),
  {
    // Always run before making the widget visible. The reorder function
    // may check for visibility internally and relies in the value being
    // that before the change.
    reorder_fn(self, idx);

    let data = &mut self.widgets[idx.idx].0;
    data.visible = true;

    if let Some(parent_idx) = data.parent_idx {
      self.show(parent_idx, reorder_fn)
    }
  }

  /// Reorder the widget with the given `Index` as the last visible one.
  fn reorder<F>(&mut self, idx: Index, new_idx_fn: F)
  where
    F: FnOnce(&Ui<E>, &Vec<Id>) -> usize,
  {
    if let Some(parent_idx) = self.widgets[idx.idx].0.parent_idx {
      // First retrieve the index of the widget we are interested in
      // from its parent's list of children.
      let children = &self.widgets[parent_idx.idx].0.children;
      let id = Id::new(idx.idx, self);
      let cur_idx = children.iter().position(|x| *x == id).unwrap();

      // Now remove said widget from the list of children.
      let id = self.widgets[parent_idx.idx].0.children.remove(cur_idx);
      // Next find the spot where to insert the widget as the first
      // hidden child.
      let new_idx = new_idx_fn(self, &self.widgets[parent_idx.idx].0.children);
      // And reinsert it at this spot.
      self.widgets[parent_idx.idx].0.children.insert(new_idx, id)
    } else {
      // No parent. Nothing to do.
    }
  }

  /// Reorder the widget with the given `Index` as the last visible one.
  fn reorder_as_focused(&mut self, idx: Index) {
    // Reordering to the top is an idempotent operations already, but it
    // potentially involves allocation and deallocation and so don't do
    // it unless necessary.
    if !self.is_top_most_child(idx) {
      self.reorder(idx, |_, _| 0);
    }
  }

  /// Reorder the widget with the given `Index` as the last visible one.
  fn reorder_as_visible(&mut self, idx: Index) {
    // In order to appear idempotent, only reorder the given widget in
    // the parent's list of children if it is not already visible.
    if !self.is_visible(idx) {
      self.reorder(idx, |ui, children| {
        children
          .iter()
          .rev()
          .position(|x| Cap::is_visible(ui, *x))
          .map(|x| x + 1)
          .unwrap_or_else(|| children.len())
      })
    }
  }

  /// Reorder the widget with the given `Index` as the first hidden one.
  fn reorder_as_hidden(&mut self, idx: Index) {
    if self.is_visible(idx) {
      self.reorder(idx, |ui, children| {
        children
          .iter()
          .position(|x| !Cap::is_visible(ui, *x))
          .unwrap_or_else(|| children.len())
      })
    }
  }

  fn is_visible(&self, idx: Index) -> bool {
    self.widgets[idx.idx].0.visible
  }

  fn is_displayed(&self, idx: Index) -> bool {
    let data = &self.widgets[idx.idx].0;
    data.visible && data.parent_idx.map_or(true, |x| self.is_displayed(x))
  }

  fn is_top_most_child(&self, idx: Index) -> bool {
    let parent_idx = self.widgets[idx.idx].0.parent_idx;

    if let Some(parent_idx) = parent_idx {
      let children = &self.widgets[parent_idx.idx].0.children;
      children[0].idx == idx
    } else {
      true
    }
  }

  fn focus(&mut self, idx: Index) {
    // We want to provide the invariant that a focused widget needs to
    // be visible.
    self.show(idx, Ui::reorder_as_focused);
    self.focused = Some(idx);
  }

  fn with<F, R>(&mut self, idx: Index, with_widget: F) -> R
  where
    F: FnOnce(&mut Ui<E>, Rc<dyn Widget<E>>) -> (Rc<dyn Widget<E>>, R),
  {
    match self.widgets[idx.idx].1.take() {
      Some(widget) => {
        let (widget, result) = with_widget(self, widget);
        self.widgets[idx.idx].1 = Some(widget);
        result
      },
      None => panic!("Widget {} is currently taken", idx),
    }
  }

  /// Render the `Ui` with the given `Renderer`.
  pub fn render(&self, renderer: &dyn Renderer) {
    // We cannot simply iterate through all widgets in `self.widgets`
    // when rendering, because we need to take parent-child
    // relationships into account in case widgets cover each other.
    let idx = self.validate(self.root_id());
    let root = self.lookup(idx);
    let bbox = renderer.renderable_area();

    renderer.pre_render();
    self.render_all(idx, root, renderer, bbox);
    renderer.post_render();
  }

  /// Recursively render the given widget and its children.
  fn render_all(&self, idx: Index, widget: &dyn Widget<E>, renderer: &dyn Renderer, bbox: BBox) {
    if self.is_visible(idx) {
      // TODO: Ideally we would want to go without the recursion stuff we
      //       have. This may not be possible (efficiently) with safe
      //       Rust, though. Not sure.
      let bbox = widget.render(self, renderer, bbox);

      for child_id in self.children(idx).rev() {
        let child_idx = self.validate(*child_id);
        let child = self.lookup(child_idx);
        self.render_all(child_idx, child, renderer, bbox)
      }
    }
  }

  /// Invoke all registered event hooks for the given event.
  fn invoke_event_hooks(&mut self, event: &E) -> Option<UiEvents<E>> {
    let mut result = None;

    // Note that we deliberately iterate over the vector by means of
    // indices. We cannot acquire an immutable reference to it because
    // we require a mutable one to self below. By using indices for the
    // iteration we side step this problem. That is safe, though,
    // because a widget cannot modify the event hooks from the hook
    // function, because we only provide it with an immutable `Cap`.
    for i in 0..self.hooked.len() {
      let idx = self.hooked[i];

      self.with(idx, |ui, widget| {
        match &ui.widgets[idx.idx].0.event_hook {
          Some(hook_fn) => {
            let event = hook_fn.0(widget.as_ref(), ui, event);
            let prev = result.take();
            let _ = replace(&mut result, OptionChain::chain(prev, event));
          },
          None => debug_assert!(false, "Widget registered as hooked but no hook func found"),
        };
        (widget, ())
      })
    }
    result
  }

  /// Handle an event.
  ///
  /// This function performs the initial determination of which widget
  /// is supposed to handle the given event and then passes it down to
  /// the actual event handler.
  pub fn handle<T>(&mut self, event: T) -> Option<UnhandledEvents<E>>
  where
    T: Into<UiEvent<E>>,
  {
    let ui_event = event.into();

    let ui_events = if let UiEvent::Event(event) = &ui_event {
      // Invoke the hooks before passing the event to the widgets on the
      // "official" route.
      self.invoke_event_hooks(event)
    } else {
      None
    };

    // Determine the target widget from where to start handling.
    let idx = match ui_event {
      // All currently defined "ordinary" events go to the currently
      // focused widget.
      UiEvent::Event(_) |
      UiEvent::Custom(_) => self.focused,
      // All others either carry an explicit target with them (e.g.,
      // some custom event) or have no target at all (for example the
      // Quit event).
      _ => None,
    };

    // Note that we guarantee that the event as it came in is received
    // by the widget before additional events as emitted by the hook.
    let events = ui_event.chain_opt(ui_events);
    self.handle_ui_events(idx, events)
  }

  /// Bubble up an event until it is handled by some `Widget`.
  fn handle_event(&mut self, idx: Index, event: E) -> Option<UnhandledEvents<E>> {
    // To enable a mutable borrow of the Ui as well as the widget we
    // temporarily remove the widget from the internally used
    // vector. This means that now we would panic if we were to
    // access the widget recursively (because that's what we do if
    // the Option is None). The only way this can happen is if the
    // widget's handle method uses the provided `Cap` object. All
    // the methods of this object are carefully chosen in a way to
    // not call into the widget itself.
    let (events, parent_idx) = self.with(idx, |ui, widget| {
      let events = widget.handle(ui, event);
      let parent_idx = ui.widgets[idx.idx].0.parent_idx;
      (widget, (events, parent_idx))
    });

    if let Some(events) = events {
      self.handle_ui_events(parent_idx, events)
    } else {
      // The event got handled.
      None
    }
  }

  /// Handle a chain of `UiEvent` objects.
  fn handle_ui_events(&mut self,
                      idx: Option<Index>,
                      events: UiEvents<E>) -> Option<UnhandledEvents<E>> {
    match events {
      ChainEvent::Event(event) => self.handle_ui_event(idx, event),
      ChainEvent::Chain(event, chain) => OptionChain::chain(
        self.handle_ui_event(idx, event),
        self.handle_ui_events(idx, *chain),
      ),
    }
  }

  /// Handle a custom event.
  fn handle_custom_event(&mut self,
                         idx: Index,
                         event: CustomEvent<'_>) -> Option<UnhandledEvents<E>> {
    let (events, parent_idx) = self.with(idx, |ui, widget| {
      let events = match event {
        CustomEvent::Owned(event) => widget.handle_custom(ui, event),
        CustomEvent::Borrowed(event) => widget.handle_custom_ref(ui, event),
      };
      let parent_idx = ui.widgets[idx.idx].0.parent_idx;
      (widget, (events, parent_idx))
    });

    if let Some(events) = events {
      self.handle_ui_events(parent_idx, events)
    } else {
      // The event got handled.
      None
    }
  }

  /// Handle a `UiEvent`.
  fn handle_ui_event(&mut self,
                     idx: Option<Index>,
                     event: UiEvent<E>) -> Option<UnhandledEvents<E>> {
    match event {
      UiEvent::Event(event) => {
        if let Some(idx) = idx {
          self.handle_event(idx, event)
        } else {
          // There is no receiver for this event. That could have many
          // reasons, for example, event propagation could have reached the
          // root widget which does not contain a parent or we were trying
          // to send an event to the focused widget and no widget had the
          // focus. In any case, return the event as-is.
          Some(UnhandledEvent::Event(event).into())
        }
      },
      UiEvent::Custom(event) => {
        if let Some(idx) = idx {
          let event = CustomEvent::Owned(event);
          self.handle_custom_event(idx, event)
        } else {
          Some(UnhandledEvent::Custom(event).into())
        }
      },
      UiEvent::Directed(id, event) => {
        let idx = self.validate(id);
        let event = CustomEvent::Owned(event);
        self.handle_custom_event(idx, event)
      },
      UiEvent::Returnable(src, dst, mut any) => {
        // First let the widget handle the event.
        let events1 = {
          let event = CustomEvent::Borrowed(any.as_mut());
          let idx = self.validate(dst);
          self.handle_custom_event(idx, event)
        };

        // Then pass the event back to the widget that originally
        // emitted it.
        let events2 = {
          let event = CustomEvent::Owned(any);
          let idx = self.validate(src);
          self.handle_custom_event(idx, event)
        };
        OptionChain::chain(events1, events2)
      },
      UiEvent::Quit => Some(UnhandledEvent::Quit.into()),
    }
  }
}

impl<E> Cap for Ui<E>
where
  E: 'static + Debug,
{
  /// Retrieve a reference to a widget's data.
  fn data(&self, widget: Id) -> &dyn Any {
    let idx = self.validate(widget);
    self.widgets[idx.idx].0.data.as_ref()
  }

  /// Retrieve an iterator over the children. Iteration happens in
  /// z-order, from highest to lowest.
  fn children(&self, widget: Id) -> ChildIter<'_> {
    self.children(self.validate(widget))
  }

  /// Retrieve the `Id` of the root widget.
  fn root_id(&self) -> Id {
    debug_assert!(!self.widgets.is_empty());
    // We do not unconditionally unwrap the Option returned by as_ref()
    // here as it is possible that it is empty and we do not want to
    // panic here. This is mostly important for unit testing.
    debug_assert_eq!(self.widgets[0].1.as_ref().map_or(0, |x| self.validate(x.id()).idx), 0);

    Id::new(0, self)
  }

  /// Retrieve the parent of the given widget.
  fn parent_id(&self, widget: Id) -> Option<Id> {
    let idx = self.validate(widget);
    let parent_idx = self.widgets[idx.idx].0.parent_idx;
    let parent_id = parent_idx.map(|x| Id::new(x.idx, self));
    debug_assert!(parent_id.map_or(true, |x| Cap::children(self, x).any(|x| *x == widget)));
    parent_id
  }

  /// Check whether a widget has its visibility flag set.
  fn is_visible(&self, widget: Id) -> bool {
    self.is_visible(self.validate(widget))
  }

  /// Check whether a widget is actually being displayed.
  fn is_displayed(&self, widget: Id) -> bool {
    self.is_displayed(self.validate(widget))
  }

  /// Retrieve the currently focused widget.
  fn focused(&self) -> Option<Id> {
    self.focused.map(|x| Id::new(x.idx, self))
  }

  /// Check whether the given widget is focused.
  fn is_focused(&self, widget: Id) -> bool {
    let idx = self.validate(widget);
    let result = self.focused == Some(idx);
    debug_assert!(result && self.is_displayed(idx) || !result);
    debug_assert!(result && self.is_top_most_child(idx) || !result);
    result
  }
}

impl<E> MutCap<E> for Ui<E>
where
  E: 'static + Debug,
{
  /// Retrieve a mutable reference to a widget's data.
  fn data_mut(&mut self, widget: Id) -> &mut dyn Any {
    let idx = self.validate(widget);
    self.widgets[idx.idx].0.data.as_mut()
  }

  /// Add a widget to the `Ui`.
  fn add_widget(
    &mut self,
    parent: Id,
    new_data: Box<NewDataFn>,
    new_widget: Box<NewWidgetFn<E>>,
  ) -> Id {
    self.add_ui_widget(parent, new_data, new_widget)
  }

  /// Show a widget, i.e., set its and its parents' visibility flag.
  fn show(&mut self, widget: Id) {
    let idx = self.validate(widget);
    self.show(idx, Ui::reorder_as_visible);
  }

  /// Hide a widget, i.e., unset its visibility flag.
  fn hide(&mut self, widget: Id) {
    if self.is_focused(widget) {
      self.focused = None
    }

    let idx = self.validate(widget);
    self.reorder_as_hidden(idx);
    self.widgets[idx.idx].0.visible = false;
  }

  /// Focus a widget.
  fn focus(&mut self, widget: Id) {
    let idx = self.validate(widget);
    self.focus(idx)
  }

  /// Install or remove an event hook handler.
  fn hook_events(&mut self, widget: Id, hook_fn: Option<EventHookFn<E>>) -> Option<EventHookFn<E>> {
    let idx = self.validate(widget);
    let data = &mut self.widgets[idx.idx].0;
    let result = self.hooked.binary_search(&idx);

    debug_assert_eq!(result.is_ok(), data.event_hook.is_some());

    match hook_fn {
      Some(_) => {
        if let Err(i) = result {
          self.hooked.insert(i, idx);
        }
      },
      None => {
        if let Ok(i) = result {
          let _ = self.hooked.remove(i);
        }
      },
    };

    let prev_hook = data.event_hook.take();
    data.event_hook = hook_fn.map(|x| EventHook(x));
    prev_hook.map(|x| x.0)
  }
}

impl<E> Deref for Ui<E>
where
  E: 'static + Debug,
{
  type Target = dyn Cap;

  fn deref(&self) -> &Self::Target {
    self
  }
}

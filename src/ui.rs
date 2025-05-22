// Copyright (C) 2018-2025 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::any::Any;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::rc::Rc;
use std::slice::Iter;
#[cfg(debug_assertions)]
use std::sync::atomic::AtomicUsize;
#[cfg(debug_assertions)]
use std::sync::atomic::Ordering;

use async_trait::async_trait;

use crate::BBox;
use crate::Mergeable;
use crate::Placeholder;
use crate::Renderer;
use crate::Widget;


/// An [`Index`] is our internal representation of an [`Id`]. `Id`s can
/// belong to different [`Ui`] objects and a validation step converts
/// them into an `Index`.
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
  fn new<E, M>(idx: usize, ui: &Ui<E, M>) -> Id {
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


/// An internally used trait for abstracting over the invocation of
/// event hooks.
#[async_trait(?Send)]
trait Hooker<E, M> {
  async fn invoke(
    &self,
    ui: &mut Ui<E, M>,
    pre_hook_event: Option<E>,
    unhandled: Option<E>,
    event: Option<&E>,
  ) -> Option<E>;
}

fn merge<E>(e1: Option<E>, e2: Option<E>) -> Option<E>
where
  E: Mergeable,
{
  match (e1, e2) {
    (Some(e1), Some(e2)) => Some(e1.merge_with(e2)),
    (Some(e), None) | (None, Some(e)) => Some(e),
    (None, None) => None,
  }
}

struct Hooked {}

#[async_trait(?Send)]
impl<E, M> Hooker<E, M> for Hooked
where
  E: Mergeable,
{
  async fn invoke(
    &self,
    ui: &mut Ui<E, M>,
    pre_hook_event: Option<E>,
    unhandled: Option<E>,
    event: Option<&E>,
  ) -> Option<E> {
    let mut result = None;

    for idx in ui.hooked.clone().as_ref() {
      match &ui.widgets[idx.idx].0.event_hook {
        Some(hook_fn) => {
          let widget = ui.widgets[idx.idx].1.clone();
          let event = hook_fn.0(widget.as_ref(), ui, event).await;

          result = merge(result, event);
        },
        None => debug_assert!(false, "Widget registered as hooked but no hook func found"),
      };
    }
    merge(merge(pre_hook_event, unhandled), result)
  }
}

struct NotHooked {}

#[async_trait(?Send)]
impl<E, M> Hooker<E, M> for NotHooked {
  async fn invoke(
    &self,
    _ui: &mut Ui<E, M>,
    pre_hook_event: Option<E>,
    unhandled: Option<E>,
    _event: Option<&E>,
  ) -> Option<E> {
    debug_assert!(pre_hook_event.is_none());
    unhandled
  }
}


/// An iterator over the children of a widget.
pub(crate) type ChildIter<'widget> = Iter<'widget, Id>;

type NewDataFn = dyn FnOnce() -> Box<dyn Any>;
type NewWidgetFn<E, M> = dyn FnOnce(Id, &mut dyn MutCap<E, M>) -> Box<dyn Widget<E, M>>;
type EventHookFn<E, M> = &'static dyn for<'f> Fn(
  &'f dyn Widget<E, M>,
  &'f mut dyn MutCap<E, M>,
  Option<&'f E>,
) -> Pin<Box<dyn Future<Output = Option<E>> + 'f>>;

mod private {
  pub trait Sealed {}
}

/// A capability allowing for various widget related operations.
pub trait Cap: Debug + private::Sealed {
  /// Retrieve a reference to a widget's data.
  fn data(&self, widget: Id) -> &dyn Any;

  /// Retrieve an iterator over the children. Iteration happens in
  /// z-order, from highest to lowest.
  fn children(&self, widget: Id) -> ChildIter<'_>;

  /// Retrieve the [`Id`] of the root widget.
  fn root_id(&self) -> Id;

  /// Retrieve the parent of the given widget.
  fn parent_id(&self, widget: Id) -> Option<Id>;

  /// Check whether a widget has its visibility flag set.
  ///
  /// Note that a return value of `true` does not necessary mean that
  /// the widget is actually visible. A widget is only visible if all
  /// its parents have the visibility flag set, too. The
  /// [`is_displayed`][Self::is_displayed] method can be used to check
  /// for actual visibility.
  fn is_visible(&self, widget: Id) -> bool;

  /// Check whether a widget is actually being displayed.
  ///
  /// This method checks whether the referenced widget is actually being
  /// displayed, that is, whether its own as well as its parents'
  /// visibility flags are all set.
  fn is_displayed(&self, widget: Id) -> bool;

  /// Retrieve the currently focused widget.
  fn focused(&self) -> Option<Id>;

  /// Check whether the widget with the given [`Id`] is focused.
  fn is_focused(&self, widget: Id) -> bool;
}


/// A mutable capability allowing for various widget related operations.
#[async_trait(?Send)]
pub trait MutCap<E, M>: Cap + Deref<Target = dyn Cap> {
  /// Retrieve a mutable reference to a widget's data.
  fn data_mut(&mut self, widget: Id) -> &mut dyn Any;

  /// Add a widget to the `Ui` represented by the capability.
  // TODO: We should not require a Box here conceptually, but omitting
  //       it will require the unboxed closures feature to stabilize.
  fn add_widget(
    &mut self,
    parent: Id,
    new_data: Box<NewDataFn>,
    new_widget: Box<NewWidgetFn<E, M>>,
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
  /// to handle it "officially" through the `Handleable::handle` method
  /// (pre-hook).
  /// Furthermore, after widgets handled the event, the hook will be
  /// invoked again, this time without an actual event (post-hook).
  ///
  /// Event hook handlers are allowed to emit an event on their own,
  /// just as "normal" event handlers. The events of all hooks get
  /// merged into a single event. As such, they must be mergeable. Note
  /// that the order in which multiple event hooks are invoked relative
  /// to each other is unspecified, and that should be taken into
  /// account when providing a `Mergeable` implementation for the
  /// provided event type.
  /// Furthermore, the final merged event is not passed to widgets, but
  /// returned straight back.
  ///
  /// Note that event hook functions are only able to inspect events and
  /// not change or discard them.
  ///
  /// A widget (identified by the given `Id`) may only register one
  /// handler and subsequent requests will overwrite the previously
  /// installed one. The method returns the handler that was previously
  /// installed, if any.
  fn hook_events(
    &mut self,
    widget: Id,
    hook_fn: Option<EventHookFn<E, M>>,
  ) -> Option<EventHookFn<E, M>>
  where
    E: Mergeable;

  /// Send the provided message to the given widget.
  async fn send(&mut self, widget: Id, message: M) -> Option<M>;

  /// Send the provided message to the given widget, without
  /// transferring ownership of the message.
  async fn call(&mut self, widget: Id, message: &mut M) -> Option<M>;
}


#[cfg(debug_assertions)]
fn get_next_ui_id() -> usize {
  static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

  NEXT_ID.fetch_add(1, Ordering::Relaxed)
}


/// This type contains data that is common to all widgets.
#[derive(Debug)]
struct WidgetData<E, M>
where
  E: 'static,
  M: 'static,
{
  /// The `Id` of the parent widget.
  ///
  /// This value may only be `None` for the root widget.
  parent_idx: Option<Index>,
  /// The data associated with the widget.
  data: Box<dyn Any>,
  /// Vector of all the children that have this widget as a parent.
  ///
  /// Children are ordered by their z-index. The widget at index zero
  /// has the highest z-index (i.e., covers all below), the last one the
  /// lowest. If one of the children is the focused one, it will be at
  /// index zero.
  // Note that unfortunately there is no straight forward way to make
  // this a Vec<Index> because we cannot use an impl trait return type
  // for the `children` method present in `Cap`.
  children: Vec<Id>,
  /// An optional event hook that may be registered for the widget.
  event_hook: Option<EventHook<E, M>>,
  /// Flag indicating the widget's visibility state.
  visible: bool,
}

impl<E, M> WidgetData<E, M> {
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


/// A struct wrapping an [`EventHookFn`] while implementing [`Debug`].
struct EventHook<E, M>(EventHookFn<E, M>)
where
  E: 'static,
  M: 'static;

impl<E, M> Debug for EventHook<E, M> {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    write!(f, "{:p}", self.0)
  }
}


/// A `Ui` is a container for related widgets.
pub struct Ui<E, M>
where
  E: 'static,
  M: 'static,
{
  #[cfg(debug_assertions)]
  id: usize,
  #[allow(clippy::type_complexity)]
  widgets: Vec<(WidgetData<E, M>, Rc<dyn Widget<E, M>>)>,
  hooker: &'static dyn Hooker<E, M>,
  hooked: Rc<Vec<Index>>,
  focused: Option<Index>,
}

impl<E, M> Ui<E, M> {
  /// Create a new `Ui` instance containing one widget that acts as the
  /// root widget.
  #[allow(clippy::new_ret_no_self)]
  pub fn new<D, W>(new_data: D, new_root_widget: W) -> (Self, Id)
  where
    D: FnOnce() -> Box<dyn Any>,
    W: FnOnce(Id, &mut dyn MutCap<E, M>) -> Box<dyn Widget<E, M>>,
  {
    static NOT_HOOKED: NotHooked = NotHooked {};

    let mut ui = Self {
      #[cfg(debug_assertions)]
      id: get_next_ui_id(),
      widgets: Default::default(),
      hooker: &NOT_HOOKED,
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
    W: FnOnce(Id, &mut dyn MutCap<E, M>) -> Box<dyn Widget<E, M>>,
  {
    let parent_idx = self.validate(parent);
    self._add_widget(Some(parent_idx), new_data, new_widget)
  }

  /// Add a widget to the `Ui`.
  fn _add_widget<D, W>(&mut self, parent_idx: Option<Index>, new_data: D, new_widget: W) -> Id
  where
    D: FnOnce() -> Box<dyn Any>,
    W: FnOnce(Id, &mut dyn MutCap<E, M>) -> Box<dyn Widget<E, M>>,
  {
    let idx = Index::new(self.widgets.len());
    let id = Id::new(idx.idx, self);

    // Because we have not created the actual widget yet, just install a
    // placeholder in its stead.
    let dummy = Placeholder;
    let data = new_data();
    let data = WidgetData::new(parent_idx, data);
    self.widgets.push((data, Rc::new(dummy)));

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
    let widget = Rc::<dyn Widget<E, M>>::from(new_widget(id, self));
    // As a help for the user, check that the widget's ID is actually
    // the correct one that we provided.
    debug_assert_eq!(widget.id(), id, "Created widget does not have provided Id");
    // Replace our placeholder with the actual widget we just created.
    // Note that because we store the children separately as part of an
    // `WidgetData` object there is no need for us to do anything about
    // them.
    self.widgets[idx.idx].1 = widget;
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
  fn lookup(&self, idx: Index) -> &dyn Widget<E, M> {
    self.widgets[idx.idx].1.as_ref()
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
    F: Fn(&mut Ui<E, M>, Index),
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

  /// Reorder the widget with the given `Index` according to the
  /// provided function's result.
  fn reorder<F>(&mut self, idx: Index, new_idx_fn: F)
  where
    F: FnOnce(&Ui<E, M>, &[Id]) -> usize,
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

  /// Render the `Ui` with the given `Renderer`.
  pub fn render(&self, renderer: &dyn Renderer) {
    // We cannot simply iterate through all widgets in `self.widgets`
    // when rendering, because we need to take parent-child
    // relationships into account in case widgets cover each other.
    let idx = self.validate(self.root_id());
    let root = self.lookup(idx);

    let () = renderer.pre_render();
    let bbox = renderer.renderable_area();

    self.render_all(idx, root, renderer, bbox);
    let () = renderer.post_render();
  }

  /// Recursively render the given widget and its children.
  fn render_all(&self, idx: Index, widget: &dyn Widget<E, M>, renderer: &dyn Renderer, bbox: BBox) {
    if self.is_visible(idx) {
      // TODO: Ideally we would want to go without the recursion stuff we
      //       have. This may not be possible (efficiently) with safe
      //       Rust, though. Not sure.
      let inner_bbox = widget.render(self, renderer, bbox);

      if inner_bbox.w != 0 && inner_bbox.h != 0 {
        // We start rendering with the widget with the lowest z-index,
        // i.e., the one the furthest in the background.
        for child_id in self.children(idx).rev() {
          let child_idx = self.validate(*child_id);
          let child = self.lookup(child_idx);
          self.render_all(child_idx, child, renderer, inner_bbox)
        }
      }

      let () = widget.render_done(self, renderer, bbox);
    }
  }

  /// Handle an event.
  ///
  /// This function performs the initial determination of which widget
  /// is supposed to handle the given event and then passes it down to
  /// the actual event handler.
  pub async fn handle<T>(&mut self, event: T) -> Option<E>
  where
    T: Into<E>,
  {
    let event = event.into();

    // Invoke the hooks before passing the event to the widgets on the
    // "official" route.
    let hook_event = self.hooker.invoke(self, None, None, Some(&event)).await;
    // All events go to the focused widget first.
    let idx = self.focused;
    // Any hook emitted events are not passed to the widgets themselves,
    // but just returned.
    let unhandled = self.try_handle_event(idx, event).await;

    self.hooker.invoke(self, hook_event, unhandled, None).await
  }

  /// Bubble up an event until it is handled by some `Widget`.
  fn handle_event(
    &mut self,
    idx: Index,
    event: E,
  ) -> Pin<Box<dyn Future<Output = Option<E>> + '_>> {
    Box::pin(async move {
      // The clone we perform here allows us to decouple the Widget from
      // the Ui, which in turn makes it possible to pass a mutable Ui
      // reference (in the form of a MutCap) to an immutable widget. It is
      // nothing more but a reference count bump, though.
      let widget = self.widgets[idx.idx].1.clone();
      let event = widget.handle(self, event).await;
      let parent_idx = self.widgets[idx.idx].0.parent_idx;

      if let Some(event) = event {
        self.try_handle_event(parent_idx, event).await
      } else {
        // The event got handled.
        None
      }
    })
  }

  /// Handle an event.
  async fn try_handle_event(&mut self, idx: Option<Index>, event: E) -> Option<E> {
    if let Some(idx) = idx {
      self.handle_event(idx, event).await
    } else {
      // There is no receiver for this event. That could have many
      // reasons, for example, event propagation could have reached the
      // root widget which does not contain a parent or we were trying
      // to send an event to the focused widget and no widget had the
      // focus. In any case, return the event as-is.
      Some(event)
    }
  }
}

impl<E, M> private::Sealed for Ui<E, M> {}

impl<E, M> Cap for Ui<E, M> {
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
    debug_assert_eq!(self.validate(self.widgets[0].1.id()).idx, 0);

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
  #[allow(clippy::nonminimal_bool)]
  fn is_focused(&self, widget: Id) -> bool {
    let idx = self.validate(widget);
    let result = self.focused == Some(idx);
    debug_assert!(result && self.is_displayed(idx) || !result);
    debug_assert!(result && self.is_top_most_child(idx) || !result);
    result
  }
}

#[async_trait(?Send)]
impl<E, M> MutCap<E, M> for Ui<E, M> {
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
    new_widget: Box<NewWidgetFn<E, M>>,
  ) -> Id {
    self.add_ui_widget(parent, new_data, new_widget)
  }

  /// Show a widget, i.e., set its and its parents' visibility flag.
  fn show(&mut self, widget: Id) {
    let idx = self.validate(widget);
    self.show(idx, |_, _| ());
  }

  /// Hide a widget, i.e., unset its visibility flag.
  fn hide(&mut self, widget: Id) {
    if self.is_focused(widget) {
      self.focused = None
    }

    let idx = self.validate(widget);
    self.widgets[idx.idx].0.visible = false;
  }

  /// Focus a widget.
  fn focus(&mut self, widget: Id) {
    let idx = self.validate(widget);
    self.focus(idx)
  }

  /// Install or remove an event hook handler.
  fn hook_events(
    &mut self,
    widget: Id,
    hook_fn: Option<EventHookFn<E, M>>,
  ) -> Option<EventHookFn<E, M>>
  where
    E: Mergeable,
  {
    static HOOKED: Hooked = Hooked {};
    self.hooker = &HOOKED;

    let idx = self.validate(widget);
    let data = &mut self.widgets[idx.idx].0;
    let result = self.hooked.binary_search(&idx);

    debug_assert_eq!(result.is_ok(), data.event_hook.is_some());

    match hook_fn {
      Some(_) => {
        if let Err(i) = result {
          Rc::make_mut(&mut self.hooked).insert(i, idx);
        }
      },
      None => {
        if let Ok(i) = result {
          let _ = Rc::make_mut(&mut self.hooked).remove(i);
        }
      },
    };

    let prev_hook = data.event_hook.take();
    data.event_hook = hook_fn.map(|x| EventHook(x));
    prev_hook.map(|x| x.0)
  }

  /// Send the provided message to the given widget.
  async fn send(&mut self, widget: Id, message: M) -> Option<M> {
    let idx = self.validate(widget);
    let widget = self.widgets[idx.idx].1.clone();

    widget.react(message, self).await
  }

  /// Send the provided message to the given widget, without
  /// transferring ownership of the message.
  async fn call(&mut self, widget: Id, message: &mut M) -> Option<M> {
    let idx = self.validate(widget);
    let widget = self.widgets[idx.idx].1.clone();

    widget.respond(message, self).await
  }
}

impl<E, M> Debug for Ui<E, M> {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    let mut debug = f.debug_struct("Ui");
    #[cfg(debug_assertions)]
    let _ = debug.field("id", &self.id);
    debug.finish()
  }
}

impl<E, M> Deref for Ui<E, M> {
  type Target = dyn Cap;

  fn deref(&self) -> &Self::Target {
    self
  }
}

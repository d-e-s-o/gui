// ui.rs

// *************************************************************************
// * Copyright (C) 2018 Daniel Mueller (deso@posteo.net)                   *
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

use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
#[cfg(debug_assertions)]
use std::sync::atomic::AtomicUsize;
#[cfg(debug_assertions)]
use std::sync::atomic::Ordering;

use BBox;
use Event;
use Handleable;
use MetaEvent;
use Object;
use Placeholder;
use Renderable;
use Renderer;
use UiEvent;


/// A type representing a reference to a `Widget`.
///
/// A reference to a `Widget` can be represented quite literally as a
/// reference to a `Widget` but it can also just be an `Id`. Object of
/// this type help abstract away from the differences between the two
/// for the purpose of the `Ui`.
pub trait WidgetRef {
  /// Retrieve a reference to a widget.
  fn as_widget<'s, 'ui: 's>(&'s self, ui: &'ui Ui) -> &Widget;

  /// Retrieve a mutable reference to a widget.
  fn as_mut_widget<'s, 'ui: 's>(&'s mut self, ui: &'ui mut Ui) -> &mut Widget;

  /// Retrieve an `Id`.
  fn as_id(&self) -> Id;
}


/// An `Index` is our internal representation of an `Id`. `Id`s can
/// belong to different `Ui` objects and a validation step converts them
/// into an `Index`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Index {
  idx: usize,
}

impl Index {
  fn new(idx: usize) -> Self {
    Index {
      idx: idx,
    }
  }
}

impl Display for Index {
  /// Format the `Index` into the given formatter.
  fn fmt(&self, f: &mut Formatter) -> Result {
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
  fn new(idx: usize, ui: &Ui) -> Id {
    Id {
      #[cfg(debug_assertions)]
      ui_id: ui.id,
      idx: Index::new(idx),
    }
  }
}

impl Display for Id {
  /// Format the `Id` into the given formatter.
  fn fmt(&self, f: &mut Formatter) -> Result {
    write!(f, "{}", self.idx)
  }
}

impl WidgetRef for Id {
  /// Retrieve a reference to a widget.
  fn as_widget<'s, 'ui: 's>(&'s self, ui: &'ui Ui) -> &Widget {
    let idx = ui.validate(*self);
    ui.lookup(idx).as_ref()
  }

  /// Retrieve a mutable reference to a widget.
  fn as_mut_widget<'s, 'ui: 's>(&'s mut self, ui: &'ui mut Ui) -> &mut Widget {
    let idx = ui.validate(*self);
    ui.lookup_mut(idx).as_mut()
  }

  /// Retrieve an `Id`.
  fn as_id(&self) -> Id {
    *self
  }
}


/// A widget as used by a `Ui`.
///
/// In addition to taking care of `Id` management and parent-child
/// relationships, the `Ui` is responsible for dispatching events to
/// widgets and rendering them. Hence, a widget usable for the `Ui`
/// needs to implement `Handleable`, `Renderable`, `Object`, and
/// `WidgetRef`.
pub trait Widget: Handleable + Renderable + Object + Debug + WidgetRef {}


// TODO: Ideally we would want to use FnOnce here, in case callers need
//       to move data into a widget. We cannot do so with a reference
//       and using generics is not possible because NewWidgetFn is used
//       in the signature of a trait method. FnBox provides a possible
//       solution but is a nightly-only API. For now, users are advised
//       to use an Option as one of the parameters and panic if None is
//       supplied.
type NewRootWidgetFn<'f> = &'f mut FnMut(Id, &mut Cap) -> Box<Widget>;
type NewWidgetFn<'f> = &'f mut FnMut(&mut WidgetRef, Id, &mut Cap) -> Box<Widget>;


/// A capability allowing for various widget related operations.
pub trait Cap {
  /// Add a widget to the `Ui` represented by the capability.
  fn add_widget(&mut self, parent: &mut WidgetRef, new_widget: NewWidgetFn) -> Id;

  /// Retrieve the `Id` of the root widget.
  fn root_id(&self) -> Id;

  /// Retrieve the parent of the given widget.
  fn parent_id(&self, widget: &WidgetRef) -> Option<Id>;

  /// Focus a widget.
  ///
  /// The focused widget is the one receiving certain types of events
  /// (such as key events) first but may also be rendered in a different
  /// color or be otherwise highlighted.
  fn focus(&mut self, widget: &WidgetRef);

  /// Check whether the referenced widget is focused.
  fn is_focused(&self, widget: &WidgetRef) -> bool;
}


#[cfg(debug_assertions)]
fn get_next_ui_id() -> usize {
  static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

  NEXT_ID.fetch_add(1, Ordering::Relaxed)
}


/// A `Ui` is a container for related widgets.
#[derive(Debug, Default)]
pub struct Ui {
  #[cfg(debug_assertions)]
  id: usize,
  widgets: Vec<Option<Box<Widget>>>,
  focused: Option<Index>,
}

// Clippy raises a false alert due to the generic type used but not
// implementing Default.
// See https://github.com/rust-lang-nursery/rust-clippy/issues/2226
#[allow(new_without_default_derive)]
impl Ui {
  /// Create a new `Ui` instance containing one widget that acts as the
  /// root widget.
  pub fn new(new_root_widget: NewRootWidgetFn) -> (Self, Id) {
    let mut ui = Ui {
      #[cfg(debug_assertions)]
      id: get_next_ui_id(),
      widgets: Default::default(),
      focused: None,
    };

    let id = ui._add_widget(new_root_widget);
    debug_assert_eq!(id.idx.idx, 0);
    (ui, id)
  }

  /// Add a widget to the `Ui`.
  // TODO: Usage of NewRootWidgetFn here gives the wrong impression of
  //       intention as we are not necessarily adding a root widget. It
  //       could just be a normal widget. The type just happens to have
  //       the right signature.
  fn _add_widget(&mut self, new_widget: NewRootWidgetFn) -> Id {
    let idx = Index::new(self.widgets.len());
    let id = Id::new(idx.idx, self);

    // If no widget has the focus we focus the newly created widget but
    // then the focus stays unless explicitly changed.
    if self.focused.is_none() {
      self.focus(&id);
    }

    // We require some trickery here to allow for dynamic widget
    // creation from within the constructor of another widget. In
    // particular, we install a "dummy" widget that acts as a container
    // to which newly created child widgets can be registered. After the
    // widget of interest got created we transfer all those children
    // over.
    let dummy = Placeholder::new();
    self.widgets.push(Some(Box::new(dummy)));

    let mut widget = new_widget(id, self);

    for child in self.lookup(idx).iter().cloned() {
      widget.add_child(&child)
    }

    self.widgets[idx.idx] = Some(widget);
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
  #[allow(borrowed_box)]
  fn lookup(&self, idx: Index) -> &Box<Widget> {
    match self.widgets[idx.idx].as_ref() {
      Some(widget) => widget,
      None => panic!("Widget {} is currently taken", idx),
    }
  }

  /// Lookup a widget from an `Index`.
  #[allow(borrowed_box)]
  fn lookup_mut(&mut self, idx: Index) -> &mut Box<Widget> {
    match self.widgets[idx.idx].as_mut() {
      Some(widget) => widget,
      None => panic!("Widget {} is currently taken", idx),
    }
  }

  /// Render the `Ui` with the given `Renderer`.
  pub fn render(&self, renderer: &Renderer) {
    // We cannot simply iterate through all widgets in `self.widgets`
    // when rendering, because we need to take parent-child
    // relationships into account in case widgets cover each other.
    let idx = self.validate(self.root_id());
    let root = self.lookup(idx);
    let bbox = renderer.renderable_area();

    renderer.pre_render();
    self.render_all(&root, renderer, bbox);
    renderer.post_render();
  }

  /// Recursively render the given widget and its children.
  #[allow(borrowed_box)]
  fn render_all(&self, widget: &Box<Widget>, renderer: &Renderer, bbox: BBox) {
    // TODO: Ideally we would want to go without the recursion stuff we
    //       have. This may not be possible (efficiently) with safe
    //       Rust, though. Not sure.
    let bbox = widget.render(renderer, bbox);

    for child_id in widget.iter().rev() {
      let idx = self.validate(*child_id);
      let child = self.lookup(idx);
      self.render_all(&child, renderer, bbox)
    }
  }

  /// Handle an event.
  pub fn handle<E>(&mut self, event: E) -> Option<UiEvent>
  where
    E: Into<UiEvent>,
  {
    let ui_event = event.into();
    match ui_event {
      UiEvent::Event(event) => {
        if let Some(idx) = self.focused {
          self.handle_event(idx, event)
        } else {
          None
        }
      },
      _ => self.handle_ui_specific_event(ui_event),
    }.and_then(|x| Some(x.into_last()))
  }

  /// Bubble up an event until it is handled by some `Widget`.
  fn handle_event(&mut self, idx: Index, event: Event) -> Option<MetaEvent> {
    let (meta_event, id) = {
      // To enable a mutable borrow of the Ui as well as the widget we
      // temporarily remove the widget from the internally used vector.
      // This means that now we would panic if we were to access the
      // widget recursively (because that's what we do if the Option is
      // None). The only way this can happen is if the widget's handle
      // method uses the provided `Cap` object. We fudge that case by
      // requiring that the widget supply its own reference to the `Cap`
      // object, and not its own `Id`. This way we do not have to lookup
      // the widget and, hence, do not panic. This use case is enabled
      // since all methods in the `Cap` trait accept a `WidgetRef`,
      // which can be either an `Id` or an actual reference.
      match self.widgets[idx.idx].take() {
        Some(mut widget) => {
          let meta_event = widget.handle(event, self);
          let parent_id = widget.parent_id();

          self.widgets[idx.idx] = Some(widget);
          (meta_event, parent_id)
        },
        None => panic!("Widget {} is currently taken", idx),
      }
    };

    if let Some(meta_event) = meta_event {
      self.handle_meta_event(id, meta_event)
    } else {
      // The event got handled.
      None
    }
  }

  /// Handle a `MetaEvent`.
  fn handle_meta_event(&mut self, id: Option<Id>, event: MetaEvent) -> Option<MetaEvent> {
    match event {
      MetaEvent::UiEvent(ui_event) => self.handle_ui_event(id, ui_event),
      MetaEvent::Chain(ui_event, meta_event) => {
        self.handle_ui_event(id, ui_event);
        self.handle_meta_event(id, *meta_event)
      },
    }
  }

  /// Handle a `UiEvent`.
  fn handle_ui_event(&mut self, id: Option<Id>, event: UiEvent) -> Option<MetaEvent> {
    match event {
      UiEvent::Event(event) => {
        if let Some(id) = id {
          let idx = self.validate(id);
          self.handle_event(idx, event)
        } else {
          // The event has not been handled. Return it as-is.
          Some(event.into())
        }
      },
      _ => self.handle_ui_specific_event(event),
    }
  }

  /// Handle a UI specific event.
  fn handle_ui_specific_event(&mut self, event: UiEvent) -> Option<MetaEvent> {
    match event {
      UiEvent::Custom(id, any) => {
        let event = Event::Custom(any);
        let idx = self.validate(id);
        self.handle_event(idx, event)
      },
      UiEvent::Quit => Some(UiEvent::Quit.into()),
      UiEvent::Event(_) => unreachable!(),
    }
  }
}

impl Cap for Ui {
  /// Add a widget to the `Ui`.
  fn add_widget(&mut self, parent: &mut WidgetRef, new_widget: NewWidgetFn) -> Id {
    let id = self._add_widget(&mut |id, cap| new_widget(parent, id, cap));
    // The widget is already linked to its parent but the parent needs to
    // know about the child as well.
    parent.as_mut_widget(self).add_child(&id);

    id
  }

  /// Retrieve the `Id` of the root widget.
  fn root_id(&self) -> Id {
    debug_assert!(!self.widgets.is_empty());
    // We do not unconditionally unwrap the Option returned by as_ref()
    // here as it is possible that it is empty and we do not want to
    // panic here. This is mostly important for unit testing.
    debug_assert_eq!(self.widgets[0].as_ref().map_or(0, |x| self.validate(x.id()).idx), 0);

    Id::new(0, self)
  }

  /// Retrieve the parent of the given widget.
  fn parent_id(&self, widget: &WidgetRef) -> Option<Id> {
    widget.as_widget(self).parent_id()
  }

  /// Focus a widget.
  fn focus(&mut self, widget: &WidgetRef) {
    let idx = self.validate(widget.as_id());
    self.focused = Some(idx)
  }

  /// Check whether the given widget is focused.
  fn is_focused(&self, widget: &WidgetRef) -> bool {
    let idx = self.validate(widget.as_id());
    self.focused == Some(idx)
  }
}

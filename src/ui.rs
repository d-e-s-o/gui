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

use Event;
use Handleable;
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


/// An `Id` uniquely representing a widget.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Id {
  idx: usize,
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
    ui.lookup(*self).as_ref()
  }

  /// Retrieve a mutable reference to a widget.
  fn as_mut_widget<'s, 'ui: 's>(&'s mut self, ui: &'ui mut Ui) -> &mut Widget {
    ui.lookup_mut(*self).as_mut()
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


/// A `Ui` is a container for related widgets.
#[derive(Debug, Default)]
pub struct Ui {
  widgets: Vec<Option<Box<Widget>>>,
  focused: Option<Id>,
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
      widgets: Default::default(),
      focused: None,
    };
    let id = ui._add_widget(new_root_widget);
    debug_assert_eq!(id.idx, 0);
    (ui, id)
  }

  /// Add a widget to the `Ui`.
  // TODO: Usage of NewRootWidgetFn here gives the wrong impression of
  //       intention as we are not necessarily adding a root widget. It
  //       could just be a normal widget. The type just happens to have
  //       the right signature.
  fn _add_widget(&mut self, new_widget: NewRootWidgetFn) -> Id {
    let id = Id {
      idx: self.widgets.len(),
    };

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

    for child in self.lookup(id).iter().cloned() {
      widget.add_child(&child)
    }

    self.widgets[id.idx] = Some(widget);
    id
  }

  /// Lookup a widget from an `Id`.
  #[allow(borrowed_box)]
  fn lookup(&self, id: Id) -> &Box<Widget> {
    match self.widgets[id.idx].as_ref() {
      Some(widget) => widget,
      None => panic!("Widget {} is currently taken", id),
    }
  }

  /// Lookup a widget from an `Id`.
  #[allow(borrowed_box)]
  fn lookup_mut(&mut self, id: Id) -> &mut Box<Widget> {
    match self.widgets[id.idx].as_mut() {
      Some(widget) => widget,
      None => panic!("Widget {} is currently taken", id),
    }
  }

  /// Render the `Ui` with the given `Renderer`.
  pub fn render(&self, renderer: &Renderer) {
    // We cannot simply iterate through all widgets in `self.widgets`
    // when rendering, because we need to take parent-child
    // relationships into account in case widgets cover each other.
    let root = self.lookup(self.root_id());
    renderer.pre_render();
    self.render_all(&root, renderer);
    renderer.post_render();
  }

  /// Recursively render the given widget and its children.
  #[allow(borrowed_box)]
  fn render_all(&self, widget: &Box<Widget>, renderer: &Renderer) {
    // TODO: Ideally we would want to go without the recursion stuff we
    //       have. This may not be possible (efficiently) with safe
    //       Rust, though. Not sure.
    widget.render(renderer);

    for child_id in widget.iter().rev() {
      let child = self.lookup(*child_id);
      self.render_all(&child, renderer)
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
        match event {
          Event::KeyUp(_) |
          Event::KeyDown(_) => self.handle_key_event(event),
          Event::Custom(_) => self.handle_custom_event(event),
        }
      },
      _ => self.handle_ui_specific_event(ui_event),
    }
  }

  /// Send a key event to the focused widget.
  fn handle_key_event(&mut self, event: Event) -> Option<UiEvent> {
    // All key events go to the focused widget.
    if let Some(id) = self.focused {
      self.handle_event(id, event)
    } else {
      None
    }
  }

  /// Handle a custom event.
  fn handle_custom_event(&mut self, event: Event) -> Option<UiEvent> {
    if let Some(id) = self.focused {
      self.handle_event(id, event)
    } else {
      None
    }
  }

  /// Handle a focus event for the widget with the given `Id`.
  fn handle_focus_event(&mut self, id: Id) -> Option<UiEvent> {
    self.focus(&id);
    None
  }

  /// Handle a quit event, i.e., one requesting the application to exit.
  fn handle_quit_event(&self) -> Option<UiEvent> {
    Some(UiEvent::Quit)
  }

  /// Bubble up an event until it is handled by some `Widget`.
  fn handle_event(&mut self, id: Id, event: Event) -> Option<UiEvent> {
    let (ui_event, id) = {
      let mut widget = self.lookup_mut(id);
      (widget.handle(event), widget.parent_id())
    };

    if let Some(ui_event) = ui_event {
      match ui_event {
        UiEvent::Event(event) => {
          if let Some(id) = id {
            self.handle_event(id, event)
          } else {
            // The event has not been handled. Return it as-is.
            Some(event.into())
          }
        },
        _ => self.handle_ui_specific_event(ui_event),
      }
    } else {
      // The event got handled.
      None
    }
  }

  /// Handle a UI specific event.
  fn handle_ui_specific_event(&mut self, event: UiEvent) -> Option<UiEvent> {
    match event {
      UiEvent::Focus(id) => self.handle_focus_event(id),
      UiEvent::Custom(id, any) => {
        let event = Event::Custom(any);
        self.handle_event(id, event)
      },
      UiEvent::Quit => self.handle_quit_event(),
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
    debug_assert_eq!(self.widgets[0].as_ref().unwrap().id().idx, 0);

    Id {
      idx: 0,
    }
  }

  /// Retrieve the parent of the given widget.
  fn parent_id(&self, widget: &WidgetRef) -> Option<Id> {
    widget.as_widget(self).parent_id()
  }

  /// Focus a widget.
  fn focus(&mut self, widget: &WidgetRef) {
    self.focused = Some(widget.as_id())
  }

  /// Check whether the given widget is focused.
  fn is_focused(&self, widget: &WidgetRef) -> bool {
    self.focused == Some(widget.as_id())
  }
}

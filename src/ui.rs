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

use std::cell::Cell;
use std::cell::Ref;
use std::cell::RefCell;
use std::cell::RefMut;
use std::fmt::Debug;

use Event;
use Handleable;
use Object;
use Placeholder;
use Renderable;
use Renderer;
use UiEvent;


/// An `Id` uniquely representing a widget.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Id {
  idx: usize,
}


/// A widget as used by a `Ui`.
///
/// In addition to taking care of `Id` management and parent-child
/// relationships, the `Ui` is responsible for dispatching events to
/// widgets and rendering them. Hence, a widget usable for the `Ui`
/// needs to implement `Handleable`, `Renderable`, and `Object`.
pub trait Widget: Handleable + Renderable + Object + Debug {}


type NewRootWidgetFn<'f> = &'f Fn(Id, &mut Cap) -> Box<Widget>;
type NewWidgetFn<'f> = &'f Fn(Id, Id, &mut Cap) -> Box<Widget>;


/// A capability allowing for various widget related operations.
pub trait Cap {
  /// Add a widget to the `Ui` represented by the capability.
  ///
  /// Note that there is no need for an `add_root_widget` method as
  /// exposed by the `Ui` struct: A root widget is always the first
  /// widget being added to a UI and there will only ever be one in it.
  fn add_widget(&mut self, parent_id: Id, new_widget: NewWidgetFn) -> Id;

  /// Retrieve the parent of the widget with the given `Id`.
  fn parent_id(&self, id: Id) -> Option<Id>;

  /// Focus a widget.
  ///
  /// The focused widget is the one receiving certain types of events
  /// (such as key events) first but may also be rendered in a different
  /// color or be otherwise highlighted.
  fn focus(&self, id: Id);

  /// Check whether the widget with the given `Id` is focused.
  fn is_focused(&self, id: Id) -> bool;
}


/// A `Ui` is a container for related widgets.
#[derive(Debug, Default)]
pub struct Ui {
  widgets: Vec<RefCell<Box<Widget>>>,
  focused: Cell<Option<Id>>,
}

// Clippy raises a false alert due to the generic type used but not
// implementing Default.
// See https://github.com/rust-lang-nursery/rust-clippy/issues/2226
#[allow(new_without_default_derive)]
impl Ui {
  /// Create a new `Ui` instance without any widgets.
  pub fn new() -> Self {
    Ui {
      widgets: Default::default(),
      focused: Cell::new(None),
    }
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
    if self.focused.get().is_none() {
      self.focus(id);
    }

    // We require some trickery here to allow for dynamic widget
    // creation from within the constructor of another widget. In
    // particular, we install a "dummy" widget that acts as a container
    // to which newly created child widgets can be registered. After the
    // widget of interest got created we transfer all those children
    // over.
    let dummy = Placeholder::new();
    self.widgets.push(RefCell::new(Box::new(dummy)));

    let mut widget = new_widget(id, self);

    for child in self.widgets[id.idx].borrow().iter().cloned() {
      widget.add_child(child)
    }

    self.widgets[id.idx] = RefCell::new(widget);
    id
  }

  /// Add a root widget, i.e., the first widget, to the `Ui`.
  pub fn add_root_widget(&mut self, new_root_widget: NewRootWidgetFn) -> Id {
    debug_assert!(self.widgets.is_empty(), "Only one root widget may exist in a Ui");
    self._add_widget(new_root_widget)
  }

  /// Lookup a widget from an `Id`.
  #[allow(borrowed_box)]
  fn lookup(&self, id: Id) -> Ref<Box<Widget>> {
    self.widgets[id.idx].borrow()
  }

  /// Lookup a widget from an `Id`.
  #[allow(borrowed_box)]
  fn lookup_mut(&self, id: Id) -> RefMut<Box<Widget>> {
    self.widgets[id.idx].borrow_mut()
  }

  /// Render the `Ui` with the given `Renderer`.
  pub fn render(&self, renderer: &Renderer) {
    // We cannot simply iterate through all widgets in `self.widgets`
    // when rendering, because we need to take parent-child
    // relationships into account in case widgets cover each other.
    if let Some(root) = self.widgets.first() {
      renderer.pre_render();
      self.render_all(&root.borrow(), renderer);
      renderer.post_render();
    }
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
  // Note that although self could be immutable here, we declare it
  // mutable for reasons of safety around the `RefCell` functionality we
  // have in use internally.
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
  fn handle_key_event(&self, event: Event) -> Option<UiEvent> {
    // All key events go to the focused widget.
    if let Some(id) = self.focused.get() {
      let mut widget = self.lookup_mut(id);
      self.handle_event(widget, event)
    } else {
      None
    }
  }

  /// Handle a custom event.
  fn handle_custom_event(&self, event: Event) -> Option<UiEvent> {
    if let Some(id) = self.focused.get() {
      let mut widget = self.lookup_mut(id);
      self.handle_event(widget, event)
    } else {
      None
    }
  }

  /// Handle a custom event directed at a particular widget.
  fn handle_directed_custom_event(&self, id: Id, event: Event) -> Option<UiEvent> {
    let widget = self.lookup_mut(id);
    self.handle_event(widget, event)
  }

  /// Handle a focus event for the widget with the given `Id`.
  fn handle_focus_event(&self, id: Id) -> Option<UiEvent> {
    self.focus(id);
    None
  }

  /// Handle a quit event, i.e., one requesting the application to exit.
  fn handle_quit_event(&self) -> Option<UiEvent> {
    Some(UiEvent::Quit)
  }

  /// Bubble up an event until it is handled by some `Widget`.
  fn handle_event(&self, mut widget: RefMut<Box<Widget>>, event: Event) -> Option<UiEvent> {
    let ui_event = widget.handle(event);
    if let Some(ui_event) = ui_event {
      match ui_event {
        UiEvent::Event(event) => {
          let id = widget.parent_id();
          // Make sure to drop the mutable reference to the given widget
          // here, otherwise we may run into a borrow conflict should the
          // handling of the event somehow cause a borrow of the same
          // widget.
          drop(widget);

          if let Some(id) = id {
            let mut parent = self.lookup_mut(id);
            self.handle_event(parent, event)
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
  fn handle_ui_specific_event(&self, event: UiEvent) -> Option<UiEvent> {
    match event {
      UiEvent::Focus(id) => self.handle_focus_event(id),
      UiEvent::Custom(id, any) => {
        let event = Event::Custom(any);
        self.handle_directed_custom_event(id, event)
      },
      UiEvent::Quit => self.handle_quit_event(),
      UiEvent::Event(_) => unreachable!(),
    }
  }
}

impl Cap for Ui {
  /// Add a widget to the `Ui`.
  fn add_widget(&mut self, parent_id: Id, new_widget: NewWidgetFn) -> Id {
    let id = self._add_widget(&|id, cap| new_widget(parent_id, id, cap));
    // The widget is already linked to its parent but the parent needs to
    // know about the child as well.
    self.lookup_mut(parent_id).add_child(id);

    id
  }

  /// Retrieve the parent of the widget with the given `Id`.
  fn parent_id(&self, id: Id) -> Option<Id> {
    self.lookup(id).parent_id()
  }

  /// Focus a widget.
  fn focus(&self, id: Id) {
    self.focused.set(Some(id))
  }

  /// Check whether the widget with the given `Id` is focused.
  fn is_focused(&self, id: Id) -> bool {
    self.focused.get() == Some(id)
  }
}

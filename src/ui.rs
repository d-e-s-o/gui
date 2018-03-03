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

use std::cell::Ref;
use std::cell::RefCell;
use std::cell::RefMut;
use std::fmt::Debug;

use event::Event;
use handleable::Handleable;
use object::Object;
use renderable::Renderable;
use renderer::Renderer;


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
pub trait Widget<R>: Handleable + Renderable<R> + Object + Debug
where
  R: Renderer,
{
}


/// A `Ui` is a container for related widgets.
#[derive(Debug, Default)]
pub struct Ui<R> {
  widgets: Vec<RefCell<Box<Widget<R>>>>,
  focused: Option<Id>,
}

// Clippy raises a false alert due to the generic type used but not
// implementing Default.
// See https://github.com/rust-lang-nursery/rust-clippy/issues/2226
#[allow(new_without_default_derive)]
impl<R> Ui<R>
where
  R: Renderer,
{
  /// Create a new `Ui` instance without any widgets.
  pub fn new() -> Self {
    Ui {
      widgets: Default::default(),
      focused: None,
    }
  }

  /// Add a widget to the `Ui`.
  fn _add_widget<F>(&mut self, new_widget: F) -> Id
  where
    F: FnOnce(Id) -> Box<Widget<R>>,
  {
    let id = Id {
      idx: self.widgets.len(),
    };

    // If no widget has the focus we focus the newly created widget but
    // then the focus stays unless explicitly changed.
    if self.focused.is_none() {
      self.focus(id);
    }

    let widget = new_widget(id);
    self.widgets.push(RefCell::new(widget));
    id
  }

  /// Add a root widget, i.e., the first widget, to the `Ui`.
  pub fn add_root_widget<F>(&mut self, new_root_widget: F) -> Id
  where
    F: FnOnce(Id) -> Box<Widget<R>>,
  {
    debug_assert!(self.widgets.is_empty(), "Only one root widget may exist in a Ui");
    self._add_widget(new_root_widget)
  }

  /// Add a widget to the `Ui`.
  pub fn add_widget<F>(&mut self, parent_id: Id, new_widget: F) -> Id
  where
    F: FnOnce(Id, Id) -> Box<Widget<R>>,
  {
    let id = self._add_widget(|id| new_widget(parent_id, id));
    // The widget is already linked to its parent but the parent needs to
    // know about the child as well.
    self.lookup_mut(parent_id).add_child(id);

    id
  }

  /// Lookup a widget from an `Id`.
  #[allow(borrowed_box)]
  fn lookup(&self, id: Id) -> Ref<Box<Widget<R>>> {
    self.widgets[id.idx].borrow()
  }

  /// Lookup a widget from an `Id`.
  #[allow(borrowed_box)]
  fn lookup_mut(&self, id: Id) -> RefMut<Box<Widget<R>>> {
    self.widgets[id.idx].borrow_mut()
  }

  /// Render the `Ui` with the given `Renderer`.
  pub fn render(&self, renderer: &R) {
    // We cannot simply iterate through all widgets in `self.widgets`
    // when rendering, because we need to take parent-child
    // relationships into account in case widgets cover each other.
    if let Some(root) = self.widgets.first() {
      self.render_all(&root.borrow(), renderer)
    }
  }

  /// Recursively render the given widget and its children.
  #[allow(borrowed_box)]
  fn render_all(&self, widget: &Box<Widget<R>>, renderer: &R) {
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
  pub fn handle<E>(&mut self, event: E) -> Option<Event>
  where
    E: Into<Event>,
  {
    let event = event.into();
    match event {
      Event::KeyUp(_) |
      Event::KeyDown(_) => self.handle_key_event(event),
    }
  }

  /// Send a key event to the focused widget.
  fn handle_key_event(&self, event: Event) -> Option<Event> {
    // All key events go to the focused widget.
    if let Some(id) = self.focused {
      let mut widget = self.lookup_mut(id);
      self.handle_event(&mut widget, event)
    } else {
      None
    }
  }

  /// Bubble up an event until it is handled by some `Widget`.
  #[allow(borrowed_box)]
  fn handle_event(&self, widget: &mut Box<Widget<R>>, event: Event) -> Option<Event> {
    let event = widget.handle(event);
    if let Some(event) = event {
      let id = widget.parent_id();

      if let Some(id) = id {
        let mut widget = self.lookup_mut(id);
        self.handle_event(&mut widget, event)
      } else {
        // The event has not been handled.
        Some(event)
      }
    } else {
      // The event got handled.
      None
    }
  }

  /// Retrieve the parent of the widget with the given `Id`.
  pub fn parent_id(&self, id: Id) -> Option<Id> {
    self.lookup(id).parent_id()
  }

  /// Focus a widget.
  ///
  /// The focused widget is the one receiving certain types of events
  /// (such as key events) first but may also be rendered in a different
  /// color or be otherwise highlighted.
  pub fn focus(&mut self, id: Id) {
    self.focused = Some(id)
  }

  /// Check whether the widget with the given `Id` is focused.
  pub fn is_focused(&self, id: Id) -> bool {
    self.focused == Some(id)
  }
}

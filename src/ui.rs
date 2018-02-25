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

use object::Object;


/// An `Id` uniquely representing a widget.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Id {
  idx: usize,
}


/// A first-class entity managed by the `Ui`.
pub trait Widget: Object + Debug {}


/// A `Ui` is a container for related widgets.
#[derive(Debug, Default)]
pub struct Ui {
  widgets: Vec<Box<Widget>>,
}

impl Ui {
  /// Create a new `Ui` instance without any widgets.
  pub fn new() -> Self {
    Ui {
      widgets: Default::default(),
    }
  }

  /// Add a widget to the `Ui`.
  fn _add_widget<F>(&mut self, new_widget: F) -> Id
  where
    F: FnOnce(Id) -> Box<Widget>,
  {
    let id = Id {
      idx: self.widgets.len(),
    };
    let widget = new_widget(id);
    self.widgets.push(widget);
    id
  }
  /// Add a root widget, i.e., the first widget, to the `Ui`.
  pub fn add_root_widget<F>(&mut self, new_root_widget: F) -> Id
  where
    F: FnOnce(Id) -> Box<Widget>,
  {
    debug_assert!(self.widgets.is_empty(), "Only one root widget may exist in a Ui");
    self._add_widget(new_root_widget)
  }

  /// Add a widget to the `Ui`.
  pub fn add_widget<F>(&mut self, parent_id: Id, new_widget: F) -> Id
  where
    F: FnOnce(Id, Id) -> Box<Widget>,
  {
    let id = self._add_widget(|id| new_widget(parent_id, id));
    // The widget is already linked to its parent but the parent needs to
    // know about the child as well.
    self.lookup_mut(parent_id).add_child(id);

    id
  }

  /// Lookup a widget from an `Id`.
  #[allow(borrowed_box)]
  fn lookup(&self, id: Id) -> &Box<Widget> {
    &self.widgets[id.idx]
  }

  /// Lookup a widget from an `Id`.
  #[allow(borrowed_box)]
  fn lookup_mut(&mut self, id: Id) -> &mut Box<Widget> {
    &mut self.widgets[id.idx]
  }

  /// Retrieve the parent of the widget with the given `Id`.
  pub fn parent_id(&self, id: Id) -> Option<Id> {
    self.lookup(id).parent_id()
  }
}

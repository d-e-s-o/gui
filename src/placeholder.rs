// placeholder.rs

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

use BBox;
use Cap;
use ChildIter;
use Event;
use Handleable;
use Id;
use MetaEvent;
use Object;
use Renderable;
use Renderer;
use Ui;
use Widget;
use WidgetRef;


/// This class is a dummy implementation of a container style `Widget`.
/// Objects of it are used internally in the `Ui` class to allow for
/// dynamic widget creation.
#[derive(Debug)]
pub struct Placeholder {
  children: Vec<Id>,
}

impl Placeholder {
  pub fn new() -> Self {
    Placeholder {
      children: Vec::new(),
    }
  }
}

impl Renderable for Placeholder {
  fn render(&self, _renderer: &Renderer, _bbox: BBox) -> BBox {
    unreachable!()
  }
}

impl Object for Placeholder {
  fn id(&self) -> Id {
    unreachable!()
  }
  fn add_child(&mut self, widget: &WidgetRef) {
    self.children.push(widget.as_id())
  }
  fn iter(&self) -> ChildIter {
    ChildIter::with_iter(self.children.iter())
  }
}

impl Handleable for Placeholder {
  fn handle(&mut self, _event: Event, _cap: &mut Cap) -> Option<MetaEvent> {
    unreachable!()
  }
}

impl WidgetRef for Placeholder {
  /// Retrieve a reference to a widget.
  fn as_widget<'s, 'ui: 's>(&'s self, _ui: &'ui Ui) -> &Widget {
    self
  }

  /// Retrieve a mutable reference to a widget.
  fn as_mut_widget<'s, 'ui: 's>(&'s mut self, _ui: &'ui mut Ui) -> &mut Widget {
    self
  }

  /// Retrieve an `Id`.
  fn as_id(&self) -> Id {
    self.id()
  }
}

impl Widget for Placeholder {}

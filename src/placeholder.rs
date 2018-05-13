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

use event::Event;
use event::UiEvent;
use handleable::Handleable;
use object::ChildIter;
use object::Object;
use renderable::Renderable;
use renderer::Renderer;
use ui::Id;
use ui::Widget;


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

impl<R> Renderable<R> for Placeholder
where
  R: Renderer,
{
  fn render(&self, _renderer: &R) {
    unreachable!()
  }
}

impl Object for Placeholder {
  fn id(&self) -> Id {
    unreachable!()
  }
  fn parent_id(&self) -> Option<Id> {
    unreachable!()
  }
  fn add_child(&mut self, id: Id) {
    self.children.push(id)
  }
  fn iter(&self) -> ChildIter {
    ChildIter::with_iter(self.children.iter())
  }
}

impl Handleable for Placeholder {
  fn handle(&mut self, _event: Event) -> Option<UiEvent> {
    unreachable!()
  }
}

impl<R> Widget<R> for Placeholder
where
  R: Renderer,
{
}

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
use Event;
use Handleable;
use Id;
use MetaEvent;
use Object;
use Renderable;
use Renderer;
use Widget;


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
}

impl Handleable for Placeholder {
  fn handle(&mut self, _event: Event, _cap: &mut Cap) -> Option<MetaEvent> {
    unreachable!()
  }
}

impl Widget for Placeholder {}

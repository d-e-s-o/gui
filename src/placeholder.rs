// placeholder.rs

// *************************************************************************
// * Copyright (C) 2018-2019 Daniel Mueller (deso@posteo.net)              *
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

use std::any::TypeId;

use crate::BBox;
use crate::Cap;
use crate::Event;
use crate::Handleable;
use crate::Id;
use crate::Object;
use crate::Renderable;
use crate::Renderer;
use crate::UiEvents;
use crate::Widget;


/// This class is a dummy implementation of a container style `Widget`.
/// Objects of it are used internally in the `Ui` class to allow for
/// dynamic widget creation.
#[derive(Debug)]
pub(crate) struct Placeholder {
  children: Vec<Id>,
}

impl Placeholder {
  pub(crate) fn new() -> Self {
    Placeholder {
      children: Vec::new(),
    }
  }
}

impl Renderable for Placeholder {
  fn type_id(&self) -> TypeId {
    TypeId::of::<Placeholder>()
  }

  fn render(&self, _renderer: &dyn Renderer, _bbox: BBox, _cap: &dyn Cap) -> BBox {
    unreachable!()
  }
}

impl Object for Placeholder {
  fn id(&self) -> Id {
    unreachable!()
  }
}

impl Handleable for Placeholder {
  fn handle(&mut self, _event: Event, _cap: &mut dyn Cap) -> Option<UiEvents> {
    unreachable!()
  }
}

impl Widget for Placeholder {
  fn type_id(&self) -> TypeId {
    TypeId::of::<Placeholder>()
  }
}

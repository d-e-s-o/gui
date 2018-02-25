// mod.rs

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

extern crate gui;

use gui::ChildIter;
use gui::Id;
use gui::Object;
use gui::Widget;


#[derive(Debug)]
pub struct TestRootWidget {
  id: Id,
  children: Vec<Id>,
}

impl TestRootWidget {
  pub fn new(id: Id) -> Self {
    TestRootWidget {
      id: id,
      children: Vec::new(),
    }
  }
}

impl Object for TestRootWidget {
  fn id(&self) -> Id {
    self.id
  }

  fn parent_id(&self) -> Option<Id> {
    None
  }

  fn add_child(&mut self, id: Id) {
    self.children.push(id)
  }

  fn iter(&self) -> ChildIter {
    ChildIter::with_iter(self.children.iter())
  }
}

impl Widget for TestRootWidget {}


#[derive(Debug)]
pub struct TestWidget {
  id: Id,
  parent_id: Id,
}

impl TestWidget {
  pub fn new(parent_id: Id, id: Id) -> Self {
    TestWidget {
      id: id,
      parent_id: parent_id,
    }
  }
}

impl Object for TestWidget {
  fn id(&self) -> Id {
    self.id
  }

  fn parent_id(&self) -> Option<Id> {
    Some(self.parent_id)
  }
}

impl Widget for TestWidget {}


#[derive(Debug)]
pub struct TestContainer {
  id: Id,
  parent_id: Id,
  children: Vec<Id>,
}

impl TestContainer {
  pub fn new(parent_id: Id, id: Id) -> Self {
    TestContainer {
      id: id,
      parent_id: parent_id,
      children: Vec::new(),
    }
  }
}

impl Object for TestContainer {
  fn id(&self) -> Id {
    self.id
  }

  fn parent_id(&self) -> Option<Id> {
    Some(self.parent_id)
  }

  fn add_child(&mut self, id: Id) {
    self.children.push(id)
  }

  fn iter(&self) -> ChildIter {
    ChildIter::with_iter(self.children.iter())
  }
}

impl Widget for TestContainer {}

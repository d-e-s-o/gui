// test_derive.rs

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

#![deny(warnings)]

extern crate gui;
#[macro_use]
extern crate gui_derive;

mod common;

use gui::Handleable;
use gui::Id;
use gui::Ui;

use common::TestRenderer;


#[derive(Debug, GuiWidget, GuiHandleable)]
struct TestDefaultWidget {
  id: Id,
  parent_id: Id,
}

impl TestDefaultWidget {
  pub fn new(parent_id: Id, id: Id) -> Self {
    TestDefaultWidget {
      id: id,
      parent_id: parent_id,
    }
  }
}


#[derive(Debug, GuiWidget, GuiHandleable)]
#[GuiType = "Widget"]
#[GuiDefaultNew]
struct TestWidget {
  id: Id,
  parent_id: Id,
}


// Note that the deny(unused_imports) attribute exists for testing
// purposes.
#[deny(unused_imports)]
#[derive(Debug, GuiWidget)]
#[GuiType = "Container"]
struct TestContainer {
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

impl Handleable for TestContainer {}


#[derive(Debug, GuiWidget, GuiHandleable)]
#[GuiType = "RootWidget"]
#[GuiDefaultNew]
struct TestRootWidget {
  id: Id,
  children: Vec<Id>,
}


#[test]
#[should_panic(expected = "Cannot add an object to a non-container")]
fn default_type_is_widget() {
  let mut ui = Ui::<TestRenderer>::new();
  let r = ui.add_root_widget(|id| {
    Box::new(TestRootWidget::new(id))
  });
  let w = ui.add_widget(r, |parent_id, id| {
    Box::new(TestDefaultWidget::new(parent_id, id))
  });
  // We assume we got a widget. A widget cannot have a child.
  let _ = ui.add_widget(w, |parent_id, id| {
    Box::new(TestDefaultWidget::new(parent_id, id))
  });
}

#[test]
#[should_panic(expected = "Cannot add an object to a non-container")]
fn widget_type_yields_widget() {
  let mut ui = Ui::<TestRenderer>::new();
  let r = ui.add_root_widget(|id| {
    Box::new(TestRootWidget::new(id))
  });
  let w = ui.add_widget(r, |parent_id, id| {
    Box::new(TestWidget::new(parent_id, id))
  });
  let _ = ui.add_widget(w, |parent_id, id| {
    Box::new(TestWidget::new(parent_id, id))
  });
}

#[test]
fn container_type_yields_container() {
  let mut ui = Ui::<TestRenderer>::new();
  let r = ui.add_root_widget(|id| {
    Box::new(TestRootWidget::new(id))
  });
  let c = ui.add_widget(r, |parent_id, id| {
    Box::new(TestContainer::new(parent_id, id))
  });
  let _ = ui.add_widget(c, |parent_id, id| {
    Box::new(TestWidget::new(parent_id, id))
  });
}

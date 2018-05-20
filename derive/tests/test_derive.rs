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

use std::fmt::Debug;
use std::marker::PhantomData;

use gui::Cap;
use gui::Handleable;
use gui::Id;
use gui::Ui;


#[derive(Debug, GuiWidget, GuiHandleable)]
#[gui(default_new)]
struct TestWidget {
  id: Id,
  parent_id: Id,
}


// Note that the deny(unused_imports) attribute exists for testing
// purposes.
#[deny(unused_imports)]
#[derive(Debug, GuiContainer)]
#[gui(default_new)]
struct TestContainer {
  id: Id,
  parent_id: Id,
  children: Vec<Id>,
}

impl Handleable for TestContainer {}


#[derive(Debug, GuiRootWidget, GuiHandleable)]
#[gui(default_new)]
struct TestRootWidget {
  id: Id,
  children: Vec<Id>,
}


#[derive(Debug, GuiContainer, GuiHandleable)]
struct TestContainerT<T>
where
  T: 'static + Debug,
{
  id: Id,
  parent_id: Id,
  children: Vec<Id>,
  _data: PhantomData<T>,
}

impl<T> TestContainerT<T>
where
  T: 'static + Debug,
{
  pub fn new(parent_id: Id, id: Id) -> Self {
    TestContainerT {
      id: id,
      parent_id: parent_id,
      children: Vec::new(),
      _data: PhantomData,
    }
  }
}


#[test]
#[should_panic(expected = "Cannot add an object to a non-container")]
fn widget_type_yields_widget() {
  let (mut ui, r) = Ui::new(&mut |id, _cap| {
    Box::new(TestRootWidget::new(id))
  });
  let w = ui.add_widget(r, &mut |parent_id, id, _cap| {
    Box::new(TestWidget::new(parent_id, id))
  });
  let _ = ui.add_widget(w, &mut |parent_id, id, _cap| {
    Box::new(TestWidget::new(parent_id, id))
  });
}

#[test]
fn container_type_yields_container() {
  let (mut ui, r) = Ui::new(&mut |id, _cap| {
    Box::new(TestRootWidget::new(id))
  });
  let c = ui.add_widget(r, &mut |parent_id, id, _cap| {
    Box::new(TestContainer::new(parent_id, id))
  });
  let _ = ui.add_widget(c, &mut |parent_id, id, _cap| {
    Box::new(TestWidget::new(parent_id, id))
  });
}

#[test]
fn generic_container() {
  let (mut ui, r) = Ui::new(&mut |id, _cap| {
    Box::new(TestRootWidget::new(id))
  });
  let c = ui.add_widget(r, &mut |parent_id, id, _cap| {
    Box::new(TestContainerT::<u32>::new(parent_id, id))
  });
  let _ = ui.add_widget(c, &mut |parent_id, id, _cap| {
    Box::new(TestWidget::new(parent_id, id))
  });
}

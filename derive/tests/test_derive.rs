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

use std::fmt::Debug;
use std::marker::PhantomData;

use gui::Cap;
use gui::Handleable;
use gui::Id;
use gui::Ui;
use gui_derive::GuiHandleable;
use gui_derive::GuiWidget;


#[derive(Debug, GuiWidget, GuiHandleable)]
#[gui(default_new)]
struct TestWidget {
  id: Id,
}


// Note that the deny(unused_imports) attribute exists for testing
// purposes.
#[deny(unused_imports)]
#[derive(Debug, GuiWidget)]
#[gui(default_new)]
struct TestWidgetCustom {
  id: Id,
}

impl Handleable for TestWidgetCustom {}


#[derive(Debug, GuiWidget, GuiHandleable)]
struct TestWidgetT<T>
where
  T: 'static + Debug,
{
  id: Id,
  _data: PhantomData<T>,
}

impl<T> TestWidgetT<T>
where
  T: 'static + Debug,
{
  pub fn new(id: Id) -> Self {
    TestWidgetT {
      id: id,
      _data: PhantomData,
    }
  }
}


#[test]
fn various_derive_combinations() {
  let (mut ui, r) = Ui::new(&mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let _ = ui.add_widget(r, &mut |id, _cap| {
    Box::new(TestWidgetCustom::new(id))
  });
  let _ = ui.add_widget(r, &mut |id, _cap| {
    Box::new(TestWidgetT::<u32>::new(id))
  });
}

// test_derive.rs

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

#![allow(
  clippy::redundant_field_names,
)]

use std::any::TypeId;
use std::fmt::Debug;
use std::marker::PhantomData;

use gui::BBox;
use gui::Cap;
use gui::derive::Handleable;
use gui::derive::Widget;
use gui::Handleable;
use gui::Id;
use gui::MutCap;
use gui::Object;
use gui::Renderable;
use gui::Renderer;
use gui::Ui;
use gui::Widget;


#[allow(unused)]
type Event = ();


#[derive(Debug, Widget, Handleable)]
#[gui(default_new, Event = "Event")]
struct TestWidget {
  id: Id,
}


// Note that the deny(unused_imports) attribute exists for testing
// purposes.
#[deny(unused_imports)]
#[derive(Debug, Widget)]
#[gui(default_new, Event = "Event")]
struct TestWidgetCustom {
  id: Id,
}

impl Handleable<Event> for TestWidgetCustom {}


#[derive(Debug, Widget, Handleable)]
#[gui(Event = "Event")]
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


#[derive(Debug, Handleable)]
#[gui(Event = "Event")]
struct TestHandleable {
  id: Id,
}

impl Renderable for TestHandleable {
  fn type_id(&self) -> TypeId {
    TypeId::of::<TestHandleable>()
  }

  fn render(&self, renderer: &Renderer, bbox: BBox, cap: &Cap) -> BBox {
    renderer.render(self, bbox, cap)
  }
}

impl Object for TestHandleable {
  fn id(&self) -> Id {
    self.id
  }
}

impl Widget<Event> for TestHandleable {
  fn type_id(&self) -> TypeId {
    TypeId::of::<TestHandleable>()
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

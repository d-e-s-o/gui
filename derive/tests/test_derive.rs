// Copyright (C) 2018-2024 Daniel Mueller (deso@posteo.net)
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(
  clippy::redundant_field_names,
)]

use std::any::TypeId;
use std::fmt::Debug;
use std::marker::PhantomData;

use async_trait::async_trait;

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


type Event = ();
type Message = ();


#[derive(Debug, Widget, Handleable)]
#[gui(default_new, Event = ())]
struct TestWidget {
  id: Id,
}


// Note that the deny(unused_imports) attribute exists for testing
// purposes.
#[deny(unused_imports)]
#[derive(Debug, Widget)]
#[gui(default_new, Event = (), Message = ())]
struct TestWidgetCustom {
  id: Id,
}

impl Handleable<Event, Message> for TestWidgetCustom {}


#[derive(Debug, Widget, Handleable)]
#[gui(Event = Event)]
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
    Self {
      id,
      _data: PhantomData,
    }
  }
}


#[derive(Debug, Handleable)]
#[gui(Event = Event)]
struct TestHandleable {
  id: Id,
}

impl Renderable for TestHandleable {
  fn type_id(&self) -> TypeId {
    TypeId::of::<TestHandleable>()
  }

  fn render(&self, cap: &dyn Cap, renderer: &dyn Renderer, bbox: BBox) -> BBox {
    renderer.render(self, cap, bbox)
  }
}

impl Object for TestHandleable {
  fn id(&self) -> Id {
    self.id
  }
}

impl Widget<Event, Message> for TestHandleable {
  fn type_id(&self) -> TypeId {
    TypeId::of::<TestHandleable>()
  }
}


#[derive(Debug, Widget, Handleable)]
struct TestGeneric<E, M>
where
  E: Debug + 'static,
  M: Debug + 'static,
{
  id: Id,
  _data: PhantomData<(E, M)>,
}

impl<E, M> TestGeneric<E, M>
where
  E: Debug,
  M: Debug,
{
  pub fn new(id: Id) -> Self {
    Self {
      id,
      _data: PhantomData,
    }
  }
}


trait MyEvent {
  fn modify(&mut self);
}


#[derive(Debug)]
struct CustomEvent {
  value: u64,
}

impl MyEvent for CustomEvent {
  fn modify(&mut self) {
    self.value *= 2;
  }
}


#[derive(Debug, Widget)]
#[gui(Event = E)]
struct TestGenericEvent<E>
where
  E: Debug + MyEvent + 'static,
{
  id: Id,
  _data: PhantomData<E>,
}

impl<E> TestGenericEvent<E>
where
  E: Debug + MyEvent,
{
  pub fn new(id: Id) -> Self {
    Self {
      id,
      _data: PhantomData,
    }
  }
}

#[async_trait(?Send)]
impl<E, M> Handleable<E, M> for TestGenericEvent<E>
where
  E: Debug + MyEvent,
{
  async fn handle(&self, _cap: &mut dyn MutCap<E, M>, mut event: E) -> Option<E> {
    event.modify();
    Some(event)
  }
}


#[test]
fn various_derive_combinations() {
  let (mut ui, r) = Ui::new(|| Box::new(()), |id, _cap| Box::new(TestWidget::new(id)));
  let _ = ui.add_ui_widget(
    r,
    || Box::new(()),
    |id, _cap| Box::new(TestWidgetCustom::new(id)),
  );
  let _ = ui.add_ui_widget(
    r,
    || Box::new(()),
    |id, _cap| Box::new(TestWidgetT::<u32>::new(id)),
  );
}

#[tokio::test]
async fn generic_widget() {
  let (mut ui, r) = Ui::<Event, Message>::new(
    || Box::new(()),
    |id, _cap| Box::new(TestGeneric::<Event, Message>::new(id)),
  );
  let _ = ui.add_ui_widget(
    r,
    || Box::new(()),
    |id, _cap| Box::new(TestGeneric::<Event, Message>::new(id)),
  );

  ui.handle(()).await;
}

#[tokio::test]
async fn generic_event() {
  let (mut ui, r) = Ui::<CustomEvent, Message>::new(
    || Box::new(()),
    |id, _cap| Box::new(TestGenericEvent::<CustomEvent>::new(id)),
  );
  let _ = ui.add_ui_widget(
    r,
    || Box::new(()),
    |id, _cap| Box::new(TestGenericEvent::<CustomEvent>::new(id)),
  );
  ui.focus(r);

  let event = CustomEvent { value: 42 };
  let result = ui.handle(event).await.unwrap();
  assert_eq!(result.value, 84);
}

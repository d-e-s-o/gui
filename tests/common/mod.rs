// mod.rs

// *************************************************************************
// * Copyright (C) 2018-2020 Daniel Mueller (deso@posteo.net)              *
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

use std::any::Any;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result;
use std::ops::Deref;

use gui::ChainEvent;
use gui::derive::Widget;
use gui::Handleable;
use gui::Id;
use gui::MutCap;
use gui::UiEvents as GuiEvents;
use gui::UnhandledEvent;
use gui::UnhandledEvents;
use gui::Widget;


/// An event type used for testing purposes.
#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
  /// An empty event.
  Empty,
  /// An event containing a key.
  Key(char),
}

pub type UiEvents = GuiEvents<Event>;

struct Handler<T>(T);

impl<T> Debug for Handler<T> {
  fn fmt(&self, f: &mut Formatter) -> Result {
    write!(f, "common::Handler")
  }
}

impl<T> Deref for Handler<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

type EventFn = dyn Fn(Id, &mut dyn MutCap<Event>, Event) -> Option<UiEvents>;
type CustomFn = dyn Fn(Id, &mut dyn MutCap<Event>, Box<dyn Any>) -> Option<UiEvents>;
type CustomRefFn = dyn Fn(Id, &mut dyn MutCap<Event>, &mut dyn Any) -> Option<UiEvents>;

type EventHandler = Handler<Box<EventFn>>;
type CustomHandler = Handler<Box<CustomFn>>;
type CustomRefHandler = Handler<Box<CustomRefFn>>;


#[derive(Debug)]
pub struct TestWidgetData {
  event_handler: Option<EventHandler>,
  custom_handler: Option<CustomHandler>,
  custom_ref_handler: Option<CustomRefHandler>,
}

#[derive(Debug)]
pub struct TestWidgetDataBuilder {
  event_handler: Option<EventHandler>,
  custom_handler: Option<CustomHandler>,
  custom_ref_handler: Option<CustomRefHandler>,
}

#[allow(unused)]
impl TestWidgetDataBuilder {
  /// Create a new `TestWidgetDataBuilder` object.
  pub fn new() -> Self {
    Self {
      event_handler: None,
      custom_handler: None,
      custom_ref_handler: None,
    }
  }

  /// Set a handler for `Handleable::handle`.
  pub fn event_handler<F>(mut self, handler: F) -> Self
  where
    F: 'static + Fn(Id, &mut dyn MutCap<Event>, Event) -> Option<UiEvents>,
  {
    self.event_handler = Some(Handler(Box::new(handler)));
    self
  }

  /// Set a handler for `Handleable::handle_custom`.
  pub fn custom_handler<F>(mut self, handler: F) -> Self
  where
    F: 'static + Fn(Id, &mut dyn MutCap<Event>, Box<dyn Any>) -> Option<UiEvents>,
  {
    self.custom_handler = Some(Handler(Box::new(handler)));
    self
  }

  /// Set a handler for `Handleable::handle_custom_ref`.
  pub fn custom_ref_handler<F>(mut self, handler: F) -> Self
  where
    F: 'static + Fn(Id, &mut dyn MutCap<Event>, &mut dyn Any) -> Option<UiEvents>,
  {
    self.custom_ref_handler = Some(Handler(Box::new(handler)));
    self
  }

  /// Build the `TestWidget` object.
  pub fn build(self) -> Box<dyn Any> {
    let data = TestWidgetData {
      event_handler: self.event_handler,
      custom_handler: self.custom_handler,
      custom_ref_handler: self.custom_ref_handler,
    };
    Box::new(data)
  }
}

#[derive(Debug, Widget)]
#[gui(Event = Event)]
pub struct TestWidget {
  id: Id,
}

impl TestWidget {
  pub fn new(id: Id) -> Self {
    Self { id }
  }
}

impl Handleable<Event> for TestWidget {
  fn handle(&self, cap: &mut dyn MutCap<Event>, event: Event) -> Option<UiEvents> {
    // Also check that we can access the non-mutable version of the data.
    let _ = self.data::<TestWidgetData>(cap);

    let data = self.data_mut::<TestWidgetData>(cap);
    match data.event_handler.take() {
      Some(handler) => {
        let event = handler(self.id, cap, event);
        let data = self.data_mut::<TestWidgetData>(cap);
        data.event_handler = Some(handler);
        event
      },
      None => Some(event.into()),
    }
  }

  fn handle_custom(&self, cap: &mut dyn MutCap<Event>, event: Box<dyn Any>) -> Option<UiEvents> {
    let data = self.data_mut::<TestWidgetData>(cap);
    match data.custom_handler.take() {
      Some(handler) => {
        let event = handler(self.id, cap, event);
        let data = self.data_mut::<TestWidgetData>(cap);
        data.custom_handler = Some(handler);
        event
      },
      None => Some(gui::UiEvent::Custom(event).into()),
    }
  }

  fn handle_custom_ref(
    &self,
    cap: &mut dyn MutCap<Event>,
    event: &mut dyn Any,
  ) -> Option<UiEvents> {
    let data = self.data_mut::<TestWidgetData>(cap);
    match data.custom_ref_handler.take() {
      Some(handler) => {
        let event = handler(self.id, cap, event);
        let data = self.data_mut::<TestWidgetData>(cap);
        data.custom_ref_handler = Some(handler);
        event
      },
      None => None,
    }
  }
}

#[allow(unused)]
pub fn unwrap_custom<E, T>(events: UnhandledEvents<E>) -> Box<T>
where
  E: Debug,
  T: 'static,
{
  match events {
    ChainEvent::Event(event) => {
      match event {
        UnhandledEvent::Custom(event) => event.downcast::<T>().unwrap(),
        _ => panic!("Unexpected event: {:?}", event),
      }
    },
    ChainEvent::Chain(_, _) => panic!("Unexpected event: {:?}", events),
  }
}

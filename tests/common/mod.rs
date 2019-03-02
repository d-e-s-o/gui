// mod.rs

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

use std::any::Any;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result;
use std::ops::Deref;

use gui::ChainEvent;
use gui::derive::Widget;
use gui::Event;
use gui::Handleable;
use gui::Id;
use gui::MutCap;
use gui::UiEvents as GuiEvents;
use gui::UnhandledEvent;
use gui::UnhandledEvents;

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

type EventFn = Fn(Id, Event, &mut MutCap<Event>) -> Option<UiEvents>;
type CustomFn = Fn(Id, Box<Any>, &mut MutCap<Event>) -> Option<UiEvents>;
type CustomRefFn = Fn(Id, &mut Any, &mut MutCap<Event>) -> Option<UiEvents>;

type EventHandler = Handler<Box<EventFn>>;
type CustomHandler = Handler<Box<CustomFn>>;
type CustomRefHandler = Handler<Box<CustomRefFn>>;


#[derive(Debug)]
pub struct TestWidgetBuilder {
  event_handler: Option<EventHandler>,
  custom_handler: Option<CustomHandler>,
  custom_ref_handler: Option<CustomRefHandler>,
}

#[allow(unused)]
impl TestWidgetBuilder {
  /// Create a new `TestWidgetBuilder` object.
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
    F: 'static + Fn(Id, Event, &mut MutCap<Event>) -> Option<UiEvents>,
  {
    self.event_handler = Some(Handler(Box::new(handler)));
    self
  }

  /// Set a handler for `Handleable::handle_custom`.
  pub fn custom_handler<F>(mut self, handler: F) -> Self
  where
    F: 'static + Fn(Id, Box<Any>, &mut MutCap<Event>) -> Option<UiEvents>,
  {
    self.custom_handler = Some(Handler(Box::new(handler)));
    self
  }

  /// Set a handler for `Handleable::handle_custom_ref`.
  pub fn custom_ref_handler<F>(mut self, handler: F) -> Self
  where
    F: 'static + Fn(Id, &mut Any, &mut MutCap<Event>) -> Option<UiEvents>,
  {
    self.custom_ref_handler = Some(Handler(Box::new(handler)));
    self
  }

  /// Build the `TestWidget` object.
  pub fn build(self, id: Id) -> TestWidget {
    TestWidget {
      id: id,
      event_handler: self.event_handler,
      custom_handler: self.custom_handler,
      custom_ref_handler: self.custom_ref_handler,
    }
  }
}


#[derive(Debug, Widget)]
#[gui(Event = "Event")]
pub struct TestWidget {
  id: Id,
  event_handler: Option<EventHandler>,
  custom_handler: Option<CustomHandler>,
  custom_ref_handler: Option<CustomRefHandler>,
}

impl TestWidget {
  pub fn new(id: Id) -> Self {
    TestWidget {
      id: id,
      event_handler: None,
      custom_handler: None,
      custom_ref_handler: None,
    }
  }
}

impl Handleable<Event> for TestWidget {
  fn handle(&mut self, event: Event, cap: &mut MutCap<Event>) -> Option<UiEvents> {
    match self.event_handler.take() {
      Some(handler) => {
        let event = handler(self.id, event, cap);
        self.event_handler = Some(handler);
        event
      },
      None => Some(event.into()),
    }
  }

  fn handle_custom(&mut self, event: Box<Any>, cap: &mut MutCap<Event>) -> Option<UiEvents> {
    match self.custom_handler.take() {
      Some(handler) => {
        let event = handler(self.id, event, cap);
        self.custom_handler = Some(handler);
        event
      },
      None => Some(gui::UiEvent::Custom(event).into()),
    }
  }

  fn handle_custom_ref(&mut self, event: &mut Any, cap: &mut MutCap<Event>) -> Option<UiEvents> {
    match self.custom_ref_handler.take() {
      Some(handler) => {
        let event = handler(self.id, event, cap);
        self.custom_ref_handler = Some(handler);
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

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

use std::any::Any;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result;
use std::ops::Deref;

use gui::Cap;
use gui::ChainEvent;
use gui::Event;
use gui::Handleable;
use gui::Id;
use gui::MetaEvent;
use gui::UiEvent;


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

type EventFn = Fn(Id, Event, &mut Cap) -> Option<MetaEvent>;
type CustomFn = Fn(Id, Box<Any>, &mut Cap) -> Option<MetaEvent>;
type CustomRefFn = Fn(Id, &mut Any, &mut Cap) -> Option<MetaEvent>;

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
    F: 'static + Fn(Id, Event, &mut Cap) -> Option<MetaEvent>,
  {
    self.event_handler = Some(Handler(Box::new(handler)));
    self
  }

  /// Set a handler for `Handleable::handle_custom`.
  pub fn custom_handler<F>(mut self, handler: F) -> Self
  where
    F: 'static + Fn(Id, Box<Any>, &mut Cap) -> Option<MetaEvent>,
  {
    self.custom_handler = Some(Handler(Box::new(handler)));
    self
  }

  /// Set a handler for `Handleable::handle_custom_ref`.
  pub fn custom_ref_handler<F>(mut self, handler: F) -> Self
  where
    F: 'static + Fn(Id, &mut Any, &mut Cap) -> Option<MetaEvent>,
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


#[derive(Debug, GuiWidget)]
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

impl Handleable for TestWidget {
  fn handle(&mut self, event: Event, cap: &mut Cap) -> Option<MetaEvent> {
    match self.event_handler.take() {
      Some(handler) => {
        let event = handler(self.id, event, cap);
        self.event_handler = Some(handler);
        event
      },
      None => Some(event.into()),
    }
  }

  fn handle_custom(&mut self, event: Box<Any>, cap: &mut Cap) -> Option<MetaEvent> {
    match self.custom_handler.take() {
      Some(handler) => {
        let event = handler(self.id, event, cap);
        self.custom_handler = Some(handler);
        event
      },
      None => Some(UiEvent::Custom(event).into()),
    }
  }

  fn handle_custom_ref(&mut self, event: &mut Any, cap: &mut Cap) -> Option<MetaEvent> {
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
pub fn unwrap_custom<T>(event: MetaEvent) -> Box<T>
where
  T: 'static,
{
  match event {
    ChainEvent::Event(event) => {
      match event {
        UiEvent::Custom(event) => event.downcast::<T>().unwrap(),
        UiEvent::Directed(_, event) => event.downcast::<T>().unwrap(),
        _ => panic!("Unexpected event: {:?}", event),
      }
    },
    ChainEvent::Chain(_, _) => panic!("Unexpected event: {:?}", event),
  }
}

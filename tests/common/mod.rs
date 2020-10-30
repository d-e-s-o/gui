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

use async_trait::async_trait;

use gui::ChainEvent;
use gui::derive::Widget;
use gui::Handleable;
use gui::Id;
use gui::Mergeable;
use gui::MutCap;
use gui::UiEvent as GuiEvent;
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
  /// An integer.
  Int(u64),
}

#[allow(unused)]
impl Event {
  /// Unwrap the value of the `Event::Int` variant.
  pub fn unwrap_int(self) -> u64 {
    match self {
      Event::Empty | Event::Key(..) => unreachable!(),
      Event::Int(value) => value,
    }
  }
}

#[allow(unused)]
pub type UiEvent = GuiEvent<Event>;
pub type UiEvents = GuiEvents<Event>;

impl Mergeable for Event {
  fn merge_with(self, other: Self) -> Self {
    match other {
      Self::Empty => self,
      Self::Key(c1) => match self {
        Self::Empty => other,
        Self::Key(c2) => {
          assert_eq!(c1, c2);
          self
        },
        Self::Int(..) => unreachable!(),
      },
      Self::Int(i1) => match self {
        Self::Empty => other,
        Self::Key(..) => unreachable!(),
        Self::Int(i2) => Self::Int(i1 + i2),
      },
    }
  }
}

#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Message {
  /// An integer value.
  pub value: u64,
}

#[allow(unused)]
impl Message {
  pub fn new(value: u64) -> Self {
    Self { value }
  }
}

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

type EventFn = dyn Fn(Id, &mut dyn MutCap<Event, Message>, Event) -> Option<UiEvents>;
type ReactFn = dyn Fn(Message, &mut dyn MutCap<Event, Message>) -> Option<Message>;
type RespondFn = dyn Fn(&mut Message, &mut dyn MutCap<Event, Message>) -> Option<Message>;

type EventHandler = Handler<Box<EventFn>>;
type ReactHandler = Handler<Box<ReactFn>>;
type RespondHandler = Handler<Box<RespondFn>>;


#[derive(Debug)]
pub struct TestWidgetData {
  event_handler: Option<EventHandler>,
  react_handler: Option<ReactHandler>,
  respond_handler: Option<RespondHandler>,
}

#[derive(Debug)]
pub struct TestWidgetDataBuilder {
  event_handler: Option<EventHandler>,
  react_handler: Option<ReactHandler>,
  respond_handler: Option<RespondHandler>,
}

#[allow(unused)]
impl TestWidgetDataBuilder {
  /// Create a new `TestWidgetDataBuilder` object.
  pub fn new() -> Self {
    Self {
      event_handler: None,
      react_handler: None,
      respond_handler: None,
    }
  }

  /// Set a handler for `Handleable::handle`.
  pub fn event_handler<F>(mut self, handler: F) -> Self
  where
    F: 'static + Fn(Id, &mut dyn MutCap<Event, Message>, Event) -> Option<UiEvents>,
  {
    self.event_handler = Some(Handler(Box::new(handler)));
    self
  }

  /// Set a handler for `Handleable::react`.
  pub fn react_handler<F>(mut self, handler: F) -> Self
  where
    F: 'static + Fn(Message, &mut dyn MutCap<Event, Message>) -> Option<Message>,
  {
    self.react_handler = Some(Handler(Box::new(handler)));
    self
  }

  /// Set a handler for `Handleable::respond`.
  pub fn respond_handler<F>(mut self, handler: F) -> Self
  where
    F: 'static + Fn(&mut Message, &mut dyn MutCap<Event, Message>) -> Option<Message>,
  {
    self.respond_handler = Some(Handler(Box::new(handler)));
    self
  }

  /// Build the `TestWidget` object.
  pub fn build(self) -> Box<dyn Any> {
    let data = TestWidgetData {
      event_handler: self.event_handler,
      react_handler: self.react_handler,
      respond_handler: self.respond_handler,
    };
    Box::new(data)
  }
}

#[derive(Debug, Widget)]
#[gui(Event = Event, Message = Message)]
pub struct TestWidget {
  id: Id,
}

impl TestWidget {
  pub fn new(id: Id) -> Self {
    Self { id }
  }
}

#[async_trait(?Send)]
impl Handleable<Event, Message> for TestWidget {
  async fn handle(&self, cap: &mut dyn MutCap<Event, Message>, event: Event) -> Option<UiEvents> {
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

  async fn react(&self, message: Message, cap: &mut dyn MutCap<Event, Message>) -> Option<Message> {
    let data = self.data_mut::<TestWidgetData>(cap);
    match data.react_handler.take() {
      Some(handler) => {
        let message = handler(message, cap);
        let data = self.data_mut::<TestWidgetData>(cap);
        data.react_handler = Some(handler);
        message
      },
      None => None,
    }
  }

  async fn respond(
    &self,
    message: &mut Message,
    cap: &mut dyn MutCap<Event, Message>,
  ) -> Option<Message> {
    let data = self.data_mut::<TestWidgetData>(cap);
    match data.respond_handler.take() {
      Some(handler) => {
        let result = handler(message, cap);

        let data = self.data_mut::<TestWidgetData>(cap);
        data.respond_handler = Some(handler);
        result
      },
      None => None,
    }
  }
}

#[allow(unused)]
pub fn unwrap_event<E>(events: UnhandledEvents<E>) -> E
where
  E: Debug,
{
  match events {
    ChainEvent::Event(event) => {
      match event {
        UnhandledEvent::Event(event) => event,
        _ => panic!("Unexpected event: {:?}", event),
      }
    },
    ChainEvent::Chain(_, _) => panic!("Unexpected event: {:?}", events),
  }
}

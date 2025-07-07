// Copyright (C) 2018-2025 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::any::Any;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result;
use std::future::Future;
use std::ops::Deref;
use std::ops::DerefMut;
use std::pin::Pin;
use std::rc::Rc;

use async_trait::async_trait;

use gui::derive::Widget;
use gui::Handleable;
use gui::Id;
use gui::Mergeable;
use gui::MutCap;
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

impl<T> DerefMut for Handler<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}


type EventFn = dyn for<'f> Fn(
  Id,
  &'f mut dyn MutCap<Event, Message>,
  Event,
) -> Pin<Box<dyn Future<Output = Option<Event>> + 'f>>;
type ReactFn = dyn for<'f> Fn(
  Id,
  Message,
  &'f mut dyn MutCap<Event, Message>,
) -> Pin<Box<dyn Future<Output = Option<Message>> + 'f>>;
type RespondFn = dyn for<'f> Fn(
  Id,
  &'f mut Message,
  &'f mut dyn MutCap<Event, Message>,
) -> Pin<Box<dyn Future<Output = Option<Message>> + 'f>>;

type EventHandler = Handler<Rc<EventFn>>;
type ReactHandler = Handler<Rc<ReactFn>>;
type RespondHandler = Handler<Rc<RespondFn>>;


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
    F: 'static
      + for<'f> Fn(
        Id,
        &'f mut dyn MutCap<Event, Message>,
        Event,
      ) -> Pin<Box<dyn Future<Output = Option<Event>> + 'f>>,
  {
    self.event_handler = Some(Handler(Rc::new(handler)));
    self
  }

  /// Set a handler for `Handleable::react`.
  pub fn react_handler<F>(mut self, handler: F) -> Self
  where
    F: 'static
      + for<'f> Fn(
        Id,
        Message,
        &'f mut dyn MutCap<Event, Message>,
      ) -> Pin<Box<dyn Future<Output = Option<Message>> + 'f>>,
  {
    self.react_handler = Some(Handler(Rc::new(handler)));
    self
  }

  /// Set a handler for `Handleable::respond`.
  pub fn respond_handler<F>(mut self, handler: F) -> Self
  where
    F: 'static
      + for<'f> Fn(
        Id,
        &'f mut Message,
        &'f mut dyn MutCap<Event, Message>,
      ) -> Pin<Box<dyn Future<Output = Option<Message>> + 'f>>,
  {
    self.respond_handler = Some(Handler(Rc::new(handler)));
    self
  }

  /// Build the `TestWidgetData` object.
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
  async fn handle(&self, cap: &mut dyn MutCap<Event, Message>, event: Event) -> Option<Event> {
    // Also check that we can access the non-mutable version of the data.
    let _ = self.data::<TestWidgetData>(cap);

    let data = self.data_mut::<TestWidgetData>(cap);
    match &data.event_handler {
      Some(handler) => {
        let handler = Rc::clone(handler);
        handler(self.id, cap, event).await
      },
      None => Some(event),
    }
  }

  async fn react(&self, message: Message, cap: &mut dyn MutCap<Event, Message>) -> Option<Message> {
    let data = self.data_mut::<TestWidgetData>(cap);
    match &data.react_handler {
      Some(handler) => {
        let handler = Rc::clone(handler);
        handler(self.id, message, cap).await
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
    match &data.respond_handler {
      Some(handler) => {
        let handler = Rc::clone(handler);
        handler(self.id, message, cap).await
      },
      None => None,
    }
  }
}

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

use gui::Event;
use gui::Handleable;
use gui::Id;
use gui::Renderer;
use gui::UiEvent;


type HandlerBox = Box<Fn(Event) -> Option<UiEvent>>;

struct Handler(HandlerBox);

impl Debug for Handler {
  fn fmt(&self, f: &mut Formatter) -> Result {
    write!(f, "common::Handler")
  }
}

impl Deref for Handler {
  type Target = HandlerBox;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}


#[derive(Debug, GuiRootWidget)]
pub struct TestRootWidget {
  id: Id,
  children: Vec<Id>,
  handler: Option<Handler>,
}

impl TestRootWidget {
  pub fn new(id: Id) -> Self {
    TestRootWidget {
      id: id,
      children: Vec::new(),
      handler: None,
    }
  }

  #[allow(unused)]
  pub fn with_handler<F>(id: Id, handler: F) -> Self
  where
    F: 'static + Fn(Event) -> Option<UiEvent>,
  {
    TestRootWidget {
      id: id,
      children: Vec::new(),
      handler: Some(Handler(Box::new(handler))),
    }
  }
}

impl Handleable for TestRootWidget {
  fn handle(&mut self, event: Event) -> Option<UiEvent> {
    match self.handler {
      Some(ref handler) => handler(event),
      None => Some(event.into()),
    }
  }
}


#[derive(Debug, GuiWidget)]
pub struct TestWidget {
  id: Id,
  parent_id: Id,
  handler: Option<Handler>,
}

impl TestWidget {
  pub fn new(parent_id: Id, id: Id) -> Self {
    TestWidget {
      id: id,
      parent_id: parent_id,
      handler: None,
    }
  }

  #[allow(unused)]
  pub fn with_handler<F>(parent_id: Id, id: Id, handler: F) -> Self
  where
    F: 'static + Fn(Event) -> Option<UiEvent>,
  {
    TestWidget {
      id: id,
      parent_id: parent_id,
      handler: Some(Handler(Box::new(handler))),
    }
  }
}

impl Handleable for TestWidget {
  fn handle(&mut self, event: Event) -> Option<UiEvent> {
    match self.handler {
      Some(ref handler) => handler(event),
      None => Some(event.into()),
    }
  }
}


#[derive(Debug, GuiContainer)]
pub struct TestContainer {
  id: Id,
  parent_id: Id,
  children: Vec<Id>,
  handler: Option<Handler>,
}

impl TestContainer {
  #[allow(unused)]
  pub fn new(parent_id: Id, id: Id) -> Self {
    TestContainer {
      id: id,
      parent_id: parent_id,
      children: Vec::new(),
      handler: None,
    }
  }

  #[allow(unused)]
  pub fn with_handler<F>(parent_id: Id, id: Id, handler: F) -> Self
  where
    F: 'static + Fn(Event) -> Option<UiEvent>,
  {
    TestContainer {
      id: id,
      parent_id: parent_id,
      children: Vec::new(),
      handler: Some(Handler(Box::new(handler))),
    }
  }
}

impl Handleable for TestContainer {
  fn handle(&mut self, event: Event) -> Option<UiEvent> {
    match self.handler {
      Some(ref handler) => handler(event),
      None => Some(event.into()),
    }
  }
}


#[allow(unused)]
#[derive(Debug)]
pub struct TestRenderer {}

impl Renderer for TestRenderer {
  fn render(&self, _object: &Any) {}
}


#[allow(unused)]
pub fn clone_event(event: &Event) -> Event {
  match *event {
    Event::KeyUp(key) => Event::KeyUp(key),
    Event::KeyDown(key) => Event::KeyDown(key),
    Event::Custom(_) => panic!("Cannot clone custom event"),
  }
}

#[allow(unused)]
pub fn compare_events(event1: &Event, event2: &Event) -> bool {
  match *event1 {
    Event::KeyUp(key1) => {
      match *event2 {
        Event::KeyUp(key2) => key1 == key2,
        _ => false,
      }
    },
    Event::KeyDown(key1) => {
      match *event2 {
        Event::KeyDown(key2) => key1 == key2,
        _ => false,
      }
    },
    Event::Custom(_) => panic!("Cannot compare custom events"),
  }
}


#[allow(unused)]
pub fn clone_ui_event(event: &UiEvent) -> UiEvent {
  match *event {
    UiEvent::Event(ref event) => UiEvent::Event(clone_event(event)),
    UiEvent::Focus(id) => UiEvent::Focus(id),
    UiEvent::Quit => UiEvent::Quit,
    UiEvent::Custom(_, _) => panic!("Cannot clone custom event"),
  }
}

#[allow(unused)]
pub fn compare_ui_events(event1: &UiEvent, event2: &UiEvent) -> bool {
  match *event1 {
    UiEvent::Event(ref event1) => {
      match *event2 {
        UiEvent::Event(ref event2) => compare_events(event1, event2),
        _ => false,
      }
    },
    UiEvent::Focus(id1) => {
      match *event2 {
        UiEvent::Focus(id2) => id1 == id2,
        _ => false,
      }
    },
    UiEvent::Quit => {
      match *event2 {
        UiEvent::Quit => true,
        _ => false,
      }
    },
    UiEvent::Custom(_, _) => panic!("Cannot compare custom events"),
  }
}

#[allow(unused)]
pub fn unwrap_custom<T>(event: UiEvent) -> Box<T>
where
  T: 'static,
{
  match event {
    UiEvent::Event(event) => {
      match event {
        Event::Custom(data) => data.downcast::<T>().unwrap(),
        _ => panic!("Unexpected event: {:?}", event),
      }
    },
    _ => panic!("Unexpected event: {:?}", event),
  }
}

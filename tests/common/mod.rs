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

use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result;
use std::ops::Deref;

use gui::Cap;
use gui::Event;
use gui::Handleable;
use gui::Id;
use gui::MetaEvent;
use gui::UiEvent;
use gui::WidgetRef;


type HandlerBox = Box<Fn(&mut WidgetRef, Event, &mut Cap) -> Option<MetaEvent>>;

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
    F: 'static + Fn(&mut WidgetRef, Event, &mut Cap) -> Option<MetaEvent>,
  {
    TestRootWidget {
      id: id,
      children: Vec::new(),
      handler: Some(Handler(Box::new(handler))),
    }
  }
}

impl Handleable for TestRootWidget {
  fn handle(&mut self, event: Event, cap: &mut Cap) -> Option<MetaEvent> {
    match self.handler.take() {
      Some(handler) => {
        let event = handler(self, event, cap);
        self.handler = Some(handler);
        event
      },
      None => Some(event.into()),
    }
  }
}


#[derive(Debug, GuiWidget)]
pub struct TestWidget {
  id: Id,
  handler: Option<Handler>,
}

impl TestWidget {
  pub fn new(id: Id) -> Self {
    TestWidget {
      id: id,
      handler: None,
    }
  }

  #[allow(unused)]
  pub fn with_handler<F>(id: Id, handler: F) -> Self
  where
    F: 'static + Fn(&mut WidgetRef, Event, &mut Cap) -> Option<MetaEvent>,
  {
    TestWidget {
      id: id,
      handler: Some(Handler(Box::new(handler))),
    }
  }
}

impl Handleable for TestWidget {
  fn handle(&mut self, event: Event, cap: &mut Cap) -> Option<MetaEvent> {
    match self.handler.take() {
      Some(handler) => {
        let event = handler(self, event, cap);
        self.handler = Some(handler);
        event
      },
      None => Some(event.into()),
    }
  }
}


#[derive(Debug, GuiContainer)]
pub struct TestContainer {
  id: Id,
  children: Vec<Id>,
  handler: Option<Handler>,
}

impl TestContainer {
  #[allow(unused)]
  pub fn new(id: Id) -> Self {
    TestContainer {
      id: id,
      children: Vec::new(),
      handler: None,
    }
  }

  #[allow(unused)]
  pub fn with_handler<F>(id: Id, handler: F) -> Self
  where
    F: 'static + Fn(&mut WidgetRef, Event, &mut Cap) -> Option<MetaEvent>,
  {
    TestContainer {
      id: id,
      children: Vec::new(),
      handler: Some(Handler(Box::new(handler))),
    }
  }
}

impl Handleable for TestContainer {
  fn handle(&mut self, event: Event, cap: &mut Cap) -> Option<MetaEvent> {
    match self.handler.take() {
      Some(handler) => {
        let event = handler(self, event, cap);
        self.handler = Some(handler);
        event
      },
      None => Some(event.into()),
    }
  }
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


pub fn clone_ui_event(event: &UiEvent) -> UiEvent {
  match *event {
    UiEvent::Event(ref event) => UiEvent::Event(clone_event(event)),
    UiEvent::Quit => UiEvent::Quit,
    UiEvent::Custom(_, _) => panic!("Cannot clone custom event"),
  }
}

pub fn compare_ui_events(event1: &UiEvent, event2: &UiEvent) -> bool {
  match *event1 {
    UiEvent::Event(ref event1) => {
      match *event2 {
        UiEvent::Event(ref event2) => compare_events(event1, event2),
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


#[allow(unused)]
pub fn clone_meta_event(event: &MetaEvent) -> MetaEvent {
  match *event {
    MetaEvent::UiEvent(ref event) => {
      MetaEvent::UiEvent(clone_ui_event(event))
    },
    MetaEvent::Chain(ref event, ref meta) => {
      MetaEvent::Chain(clone_ui_event(event), Box::new(clone_meta_event(meta)))
    },
  }
}

#[allow(unused)]
pub fn compare_meta_events(event1: &MetaEvent, event2: &MetaEvent) -> bool {
  match *event1 {
    MetaEvent::UiEvent(ref event1) => {
      match *event2 {
        MetaEvent::UiEvent(ref event2) => compare_ui_events(event1, event2),
        _ => false,
      }
    },
    MetaEvent::Chain(ref event1, ref meta1) => {
      match *event2 {
        MetaEvent::Chain(ref event2, ref meta2) => {
          compare_ui_events(event1, event2) &&
          compare_meta_events(meta1, meta2)
        },
        _ => false,
      }
    },
  }
}

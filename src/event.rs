// event.rs

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

use std::any::Any;

use Id;


/// An object representing a key on the key board.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Key {
  /// The backspace key.
  Backspace,
  /// The character representing the key.
  Char(char),
  /// The delete key.
  Delete,
  /// The down arrow key.
  Down,
  /// The end key.
  End,
  /// The Escape key.
  Esc,
  /// The home key.
  Home,
  /// The insert key.
  Insert,
  /// The left arrow key.
  Left,
  /// The page down key.
  PageDown,
  /// The page up key.
  PageUp,
  /// The return key.
  Return,
  /// The right arrow key.
  Right,
  /// The up arrow key.
  Up,
}


/// An event that can be handled by a `Handleable`.
#[derive(Debug)]
pub enum Event {
  /// A key was pressed.
  ///
  /// A key down event is delivered to the focused widget and it is up
  /// to this widget to decide whether the event gets propagated further
  /// up.
  KeyDown(Key),
  /// A key was released.
  ///
  /// A key up event is delivered to the focused widget and it is up to
  /// this widget to decide whether the event gets propagated further
  /// up.
  KeyUp(Key),
  /// A custom event that can contain arbitrary data.
  Custom(Box<Any>),
}


/// An event that the `Ui` can process.
#[derive(Debug)]
pub enum UiEvent {
  /// An `Event` that can be handled by a `Handleable`.
  Event(Event),
  /// A custom event that can contain arbitrary data.
  ///
  /// Custom events are destined for a particular widget, described by
  /// the given `Id`. That is the only widget that will receive the
  /// event.
  Custom(Id, Box<Any>),
  /// A request to quit the application has been made.
  Quit,
}

/// A convenience conversion from `Event` to `UiEvent`.
impl From<Event> for UiEvent {
  fn from(event: Event) -> Self {
    UiEvent::Event(event)
  }
}


/// An event potentially comprising multiple `UiEvent` objects.
#[derive(Debug)]
pub enum MetaEvent {
  /// An event handleable by a `Ui`.
  UiEvent(UiEvent),
  /// A chain of events.
  ///
  /// The events will be processed in the order they are chained.
  Chain(UiEvent, Box<MetaEvent>),
}

impl MetaEvent {
  /// Convert this `MetaEvent` into the last `UiEvent` it comprises.
  pub fn into_last(self) -> UiEvent {
    match self {
      MetaEvent::UiEvent(ui_event) => ui_event,
      MetaEvent::Chain(_, meta_event) => meta_event.into_last(),
    }
  }
}

/// A convenience conversion from `Event` to `MetaEvent`.
impl From<Event> for MetaEvent {
  fn from(event: Event) -> Self {
    MetaEvent::UiEvent(event.into())
  }
}

/// A convenience conversion from `UiEvent` to `MetaEvent`.
impl From<UiEvent> for MetaEvent {
  fn from(event: UiEvent) -> Self {
    MetaEvent::UiEvent(event)
  }
}


/// A trait for chaining of events.
pub trait EventChain {
  /// Chain together two events.
  ///
  /// The given event will effectively be appended to the current one
  /// and, hence, be handled after the first one got processed.
  fn chain<E>(self, event: E) -> MetaEvent
  where
    E: Into<MetaEvent>;

  /// Chain together an event with an optional event.
  ///
  /// This method returns the chain of the first event with the second
  /// one, if present, or otherwise just returns the first one.
  fn chain_opt<E>(self, event: Option<E>) -> MetaEvent
  where
    E: Into<MetaEvent>;
}

impl<ES> EventChain for ES
where
  ES: Into<MetaEvent>,
{
  fn chain<E>(self, event: E) -> MetaEvent
  where
    E: Into<MetaEvent>
  {
    match self.into() {
      MetaEvent::UiEvent(ui_event) => {
        MetaEvent::Chain(ui_event, Box::new(event.into()))
      },
      MetaEvent::Chain(ui_event, meta_event) => {
        MetaEvent::Chain(ui_event, Box::new(meta_event.chain(event)))
      },
    }
  }

  fn chain_opt<E>(self, event: Option<E>) -> MetaEvent
  where
    E: Into<MetaEvent>,
  {
    match event {
      Some(event) => self.chain(event),
      None => self.into(),
    }
  }
}


/// A trait for chaining of optional events.
pub trait OptionChain {
  /// Chain an optional event with another optional event.
  fn chain<E>(self, event: Option<E>) -> Option<MetaEvent>
  where
    E: Into<MetaEvent>;

  /// Chain an optional event with the given event.
  fn opt_chain<E>(self, event: E) -> MetaEvent
  where
    E: Into<MetaEvent>;
}

impl<ES> OptionChain for Option<ES>
where
  ES: Into<MetaEvent>,
{
  fn chain<E>(self, event: Option<E>) -> Option<MetaEvent>
  where
    E: Into<MetaEvent>,
  {
    match self {
      Some(e1) => Some(e1.chain_opt(event)),
      None => {
        match event {
          Some(e2) => Some(e2.into()),
          None => None,
        }
      },
    }
  }

  fn opt_chain<E>(self, event: E) -> MetaEvent
  where
    E: Into<MetaEvent>,
  {
    match self {
      Some(e1) => e1.chain(event),
      None => event.into(),
    }
  }
}

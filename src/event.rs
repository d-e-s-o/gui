// event.rs

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

use std::any::Any;


/// An event type containing custom and arbitrary data.
///
/// Custom events are the means for transferring arbitrary data between
/// widgets.
#[derive(Debug)]
pub(crate) enum CustomEvent {
  /// An event that is sent in full to another widget.
  ///
  /// Ownership of the event is transferred to a widget. Custom events
  /// of this type are considered the safe default. They represent a
  /// unidirectional message from one widget to another.
  Owned(Box<dyn Any>),
}


/// An event that the `Ui` can process.
#[derive(Debug)]
pub enum UiEvent<E> {
  /// An `Event` that can be handled by a `Handleable`.
  Event(E),
  /// A custom event that can contain arbitrary data.
  Custom(Box<dyn Any>),
  /// A request to quit the application has been made.
  Quit,
}

/// A convenience conversion from `Event` to `UiEvent`.
impl<E> From<E> for UiEvent<E> {
  fn from(event: E) -> Self {
    UiEvent::Event(event)
  }
}


/// An event that the `Ui` did not process.
///
/// An unhandled event comprises the variants of a `UiEvent` that are
/// not concerned with addressing.
// Note that we do not provide a conversion from `UiEvent` because
// conversion should only happen from within the `Ui` proper and after
// making sure that `UiEvent` variants dealing solely with addressing
// are no longer present.
#[derive(Debug)]
pub enum UnhandledEvent<E> {
  /// An `Event` that can be handled by a `Handleable`.
  Event(E),
  /// A custom event that can contain arbitrary data.
  Custom(Box<dyn Any>),
  /// A request to quit the application has been made.
  Quit,
}

/// A convenience conversion from a single event into an `UnhandledEvent`.
impl<E> From<E> for UnhandledEvent<E> {
  fn from(event: E) -> Self {
    UnhandledEvent::Event(event)
  }
}


/// An event potentially comprising multiple event objects.
#[derive(Debug)]
pub enum ChainEvent<E> {
  /// An arbitrary event.
  Event(E),
  /// A chain of events.
  ///
  /// The events will be processed in the order they are chained.
  Chain(E, Box<ChainEvent<E>>),
}

impl<E> ChainEvent<E> {
  /// Convert this `ChainEvent` into the last event it comprises.
  pub fn into_last(self) -> E {
    match self {
      ChainEvent::Event(event) => event,
      ChainEvent::Chain(_, chain) => chain.into_last(),
    }
  }
}

/// A convenience conversion from a single event into a `ChainEvent`.
impl<E> From<E> for ChainEvent<E> {
  fn from(event: E) -> Self {
    ChainEvent::Event(event)
  }
}


/// An event potentially comprising multiple `UiEvent` objects.
pub type UiEvents<E> = ChainEvent<UiEvent<E>>;

/// A convenience conversion from a single event into a chain of `UiEvent` objects.
impl<E> From<E> for UiEvents<E> {
  fn from(event: E) -> Self {
    ChainEvent::Event(event.into())
  }
}


/// An event potentially comprising multiple `UnhandledEvent` objects.
pub type UnhandledEvents<E> = ChainEvent<UnhandledEvent<E>>;

/// A convenience conversion from a single event into a chain of `UnhandledEvent` objects.
impl<E> From<E> for UnhandledEvents<E> {
  fn from(event: E) -> Self {
    ChainEvent::Event(event.into())
  }
}


/// A trait for chaining of events.
pub trait EventChain<ED> {
  /// Chain together two events.
  ///
  /// The given event will effectively be appended to the current one
  /// and, hence, be handled after the first one got processed.
  fn chain<ES>(self, event: ES) -> ChainEvent<ED>
  where
    ES: Into<ChainEvent<ED>>;

  /// Chain together an event with an optional event.
  ///
  /// This method returns the chain of the first event with the second
  /// one, if present, or otherwise just returns the first one.
  fn chain_opt<ES>(self, event: Option<ES>) -> ChainEvent<ED>
  where
    ES: Into<ChainEvent<ED>>;
}

impl<ES, ED> EventChain<ED> for ES
where
  ES: Into<ChainEvent<ED>>,
{
  fn chain<E>(self, event: E) -> ChainEvent<ED>
  where
    E: Into<ChainEvent<ED>>,
  {
    match self.into() {
      ChainEvent::Event(e) => ChainEvent::Chain(e, Box::new(event.into())),
      ChainEvent::Chain(e, chain) => ChainEvent::Chain(e, Box::new((*chain).chain(event))),
    }
  }

  fn chain_opt<E>(self, event: Option<E>) -> ChainEvent<ED>
  where
    E: Into<ChainEvent<ED>>,
  {
    match event {
      Some(event) => self.chain(event),
      None => self.into(),
    }
  }
}


/// A trait for chaining of optional events.
pub trait OptionChain<ES, ED>
where
  ES: Into<ChainEvent<ED>>,
{
  /// Chain an optional event with another optional event.
  fn chain<E>(self, event: Option<E>) -> Option<ChainEvent<ED>>
  where
    E: Into<ChainEvent<ED>>;

  /// Chain an optional event with the given event.
  fn opt_chain<E>(self, event: E) -> ChainEvent<ED>
  where
    E: Into<ChainEvent<ED>>;
}

impl<ES, ED> OptionChain<ES, ED> for Option<ES>
where
  ES: Into<ChainEvent<ED>>,
{
  fn chain<E>(self, event: Option<E>) -> Option<ChainEvent<ED>>
  where
    E: Into<ChainEvent<ED>>,
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

  fn opt_chain<E>(self, event: E) -> ChainEvent<ED>
  where
    E: Into<ChainEvent<ED>>,
  {
    match self {
      Some(e1) => e1.chain(event),
      None => event.into(),
    }
  }
}

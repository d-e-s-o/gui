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

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

use ui::Id;


/// An object representing a key on the key board.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Key {
  /// The character representing the key.
  Char(char),
  /// The Escape key.
  Esc,
}


/// An event that can be handled by a `Handleable`.
#[derive(Clone, Debug, Eq, PartialEq)]
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
}


/// An event that the `Ui` can process.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiEvent {
  /// An `Event` that can be handled by a `Handleable`.
  Event(Event),
  /// The widget with the given `Id` should be focused.
  Focus(Id),
}

/// A convenience conversion from `Event` to `UiEvent`.
impl From<Event> for UiEvent {
  fn from(event: Event) -> Self {
    UiEvent::Event(event)
  }
}


#[cfg(test)]
mod tests {
  use super::*;


  #[test]
  fn convert_event_into() {
    let event = Event::KeyDown(Key::Char(' '));
    let orig_event = event.clone();
    let ui_event = UiEvent::from(event);

    assert_eq!(ui_event, UiEvent::Event(orig_event));
  }
}
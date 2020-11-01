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


/// An event that the `Ui` can process.
#[derive(Debug, PartialEq)]
pub enum UiEvent<E> {
  /// An `Event` that can be handled by a `Handleable`.
  Event(E),
  /// A request to quit the application has been made.
  Quit,
}

/// A convenience conversion from `Event` to `UiEvent`.
impl<E> From<E> for UiEvent<E> {
  fn from(event: E) -> Self {
    UiEvent::Event(event)
  }
}

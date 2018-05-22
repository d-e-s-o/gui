// handleable.rs

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

use Cap;
use Event;
use UiEvent;


/// A trait representing an object capable of handling events.
pub trait Handleable {
  /// Handle an `Event`.
  ///
  /// The widget has the option to either consume the event and return
  /// nothing, in which no one else will get informed about it, forward
  /// it directly (the default behavior), in which case the its parent
  /// widget will receive it, or return a completely different event.
  #[allow(unused_variables)]
  fn handle(&mut self, event: Event, cap: &mut Cap) -> Option<UiEvent> {
    // By default we just pass through the event, which will cause it to
    // bubble up to the parent.
    Some(event.into())
  }
}

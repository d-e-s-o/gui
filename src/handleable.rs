// handleable.rs

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

use std::fmt::Debug;

use async_trait::async_trait;

use crate::MutCap;
use crate::UiEvent;


/// A trait representing an object capable of handling events.
#[async_trait(?Send)]
pub trait Handleable<E, M>: Debug
where
  E: 'static,
  M: 'static,
{
  /// Handle an `Event`.
  ///
  /// The widget has the option to either consume the event and return
  /// nothing, in which case no one else will get informed about it,
  /// forward it directly (the default behavior), in which case its
  /// parent widget will receive it, or return a completely different
  /// event.
  #[allow(unused_variables)]
  async fn handle(&self, cap: &mut dyn MutCap<E, M>, event: E) -> Option<UiEvent<E>> {
    // By default we just pass through the event, which will cause it to
    // bubble up to the parent.
    Some(event.into())
  }

  /// React to a message.
  ///
  /// This method is the handler for the `MutCap::send` invocation.
  #[allow(unused_variables)]
  async fn react(&self, message: M, cap: &mut dyn MutCap<E, M>) -> Option<M> {
    Some(message)
  }

  /// Respond to a message.
  ///
  /// This is the handler for the `MutCap::call` invocation.
  #[allow(unused_variables)]
  async fn respond(&self, message: &mut M, cap: &mut dyn MutCap<E, M>) -> Option<M> {
    None
  }
}

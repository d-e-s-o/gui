// Copyright (C) 2018-2024 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fmt::Debug;

use async_trait::async_trait;

use crate::MutCap;


/// A trait representing an object capable of handling events.
#[async_trait(?Send)]
pub trait Handleable<E, M>: Debug {
  /// Handle an event.
  ///
  /// The widget has the option to either consume the event and return
  /// nothing, in which case no one else will get informed about it,
  /// forward it directly (the default behavior), in which case its
  /// parent widget will receive it, or return a completely different
  /// event.
  #[allow(unused_variables)]
  async fn handle(&self, cap: &mut dyn MutCap<E, M>, event: E) -> Option<E> {
    // By default we just pass through the event, which will cause it to
    // bubble up to the parent.
    Some(event)
  }

  /// React to a message.
  ///
  /// This method is the handler for the [`MutCap::send`] invocation.
  #[allow(unused_variables)]
  async fn react(&self, message: M, cap: &mut dyn MutCap<E, M>) -> Option<M> {
    Some(message)
  }

  /// Respond to a message.
  ///
  /// This is the handler for the [`MutCap::call`] invocation.
  #[allow(unused_variables)]
  async fn respond(&self, message: &mut M, cap: &mut dyn MutCap<E, M>) -> Option<M> {
    None
  }
}

// Copyright (C) 2018-2024 Daniel Mueller (deso@posteo.net)
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fmt::Debug;

use crate::Id;


/// An `Object` represents a first-class entity in a UI.
pub trait Object: Debug {
  /// Retrieve this object's [`Id`].
  fn id(&self) -> Id;
}

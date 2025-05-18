// Copyright (C) 2020-2024 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later


/// A trait for merging two values.
pub trait Mergeable {
  /// Merge `other` into `self` and return the result.
  fn merge_with(self, other: Self) -> Self;
}

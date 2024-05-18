// Copyright (C) 2018-2024 Daniel Mueller (deso@posteo.net)
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(
  clippy::assertions_on_constants,
  clippy::let_unit_value,
  clippy::redundant_field_names,
)]
#![warn(
  future_incompatible,
  missing_copy_implementations,
  missing_debug_implementations,
  missing_docs,
  rust_2018_compatibility,
  rust_2018_idioms,
  trivial_numeric_casts,
  unreachable_pub,
  unstable_features,
  unused_import_braces,
  unused_qualifications,
  unused_results,
)]

//! A crate containing the basic infrastructure for user interfaces. It
//! strives for being completely agnostic of the underlying system and
//! its rendering machinery as well as event dispatching.

mod handleable;
mod mergeable;
mod object;
mod placeholder;
mod renderable;
mod renderer;
mod ui;
mod widget;

use self::placeholder::Placeholder;

pub use self::handleable::Handleable;
pub use self::mergeable::Mergeable;
pub use self::object::Object;
pub use self::renderable::Renderable;
pub use self::renderer::BBox;
pub use self::renderer::Renderer;
pub use self::ui::Cap;
pub use self::ui::Id;
pub use self::ui::MutCap;
pub use self::ui::Ui;
pub use self::widget::Widget;

/// A module providing custom derive functionality for `gui` related
/// traits.
///
/// The module merely re-reports the procedural macros provided by the
/// `gui_derive` crate.
#[cfg(feature = "derive")]
pub mod derive {
  pub use gui_derive::*;
}

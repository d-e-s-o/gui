// lib.rs

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

#![allow(
  unknown_lints,
  block_in_if_condition_stmt,
  redundant_field_names,
)]
#![deny(
  missing_copy_implementations,
  missing_debug_implementations,
  missing_docs,
  trivial_numeric_casts,
  unstable_features,
  unused_import_braces,
  unused_qualifications,
  unused_results,
)]
#![warn(
  future_incompatible,
  rust_2018_compatibility,
  rust_2018_idioms,
)]

//! A crate containing the basic infrastructure for user interfaces. It
//! strives for being completely agnostic of the underlying system and
//! its rendering machinery as well as event dispatching.

mod event;
mod handleable;
mod object;
mod placeholder;
mod renderable;
mod renderer;
mod ui;
mod widget;

use self::event::CustomEvent;
use self::placeholder::Placeholder;

pub use self::event::ChainEvent;
pub use self::event::Event;
pub use self::event::EventChain;
pub use self::event::Key;
pub use self::event::OptionChain;
pub use self::event::UiEvent;
pub use self::event::UiEvents;
pub use self::event::UnhandledEvent;
pub use self::event::UnhandledEvents;
pub use self::handleable::Handleable;
pub use self::object::Object;
pub use self::renderable::Renderable;
pub use self::renderer::BBox;
pub use self::renderer::Renderer;
pub use self::ui::Cap;
pub use self::ui::Id;
pub use self::ui::Ui;
pub use self::widget::Widget;

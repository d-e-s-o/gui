// renderer.rs

// *************************************************************************
// * Copyright (C) 2018-2019 Daniel Mueller (deso@posteo.net)              *
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

use crate::Cap;
use crate::Renderable;


/// A bounding box representing the area that a widget may occupy. A
/// bounding box always describes a rectangular area. The origin [x=0,
/// y=0] is typically assumed to reside in the upper left corner of the
/// screen, but it is really up to the individual `Renderer` to make do
/// with whatever is provided.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct BBox {
  /// The x-coordinate of the bounding box.
  pub x: u16,
  /// The y-coordinate of the bounding box.
  pub y: u16,
  /// The width of the bounding box.
  pub w: u16,
  /// The height of the bounding box.
  pub h: u16,
}


/// An abstraction for objects used for rendering widgets.
pub trait Renderer {
  /// Retrieve the bounding box of the renderable area (typically the
  /// screen).
  /// Note that the units to be used are not specified. That is, the
  /// result could be in pixels, characters (in case of a terminal), or
  /// just arbitrary numbers (if virtual coordinates are being used), as
  /// long as this `Renderer` knows how to interpret them.
  fn renderable_area(&self) -> BBox;

  /// Perform some pre-render step.
  fn pre_render(&self) {}

  /// Render an object.
  ///
  /// Objects are represented as `Renderable` and need to be cast into
  /// the actual widget type to render by the `Renderer` itself, should
  /// that be necessary. A simplified implementation could look as
  /// follows:
  /// ```rust
  /// # use gui::{BBox, Cap, Id, Renderer, Renderable};
  /// # use gui::derive::{Handleable, Widget};
  /// # #[derive(Debug, Widget, Handleable)]
  /// # #[gui(Event = ())]
  /// # struct ConcreteWidget1 {
  /// #   id: Id,
  /// # }
  /// # #[derive(Debug, Widget, Handleable)]
  /// # #[gui(Event = ())]
  /// # struct ConcreteWidget2 {
  /// #   id: Id,
  /// # }
  /// # #[derive(Debug)]
  /// # struct TestRenderer {}
  /// # impl TestRenderer {
  /// #   fn render_concrete_widget1(&self, widget: &ConcreteWidget1, bbox: BBox) -> BBox {
  /// #     bbox
  /// #   }
  /// #   fn render_concrete_widget2(&self, widget: &ConcreteWidget1, bbox: BBox) -> BBox {
  /// #     bbox
  /// #   }
  /// # }
  /// # impl Renderer for TestRenderer {
  /// #   fn renderable_area(&self) -> BBox {
  /// #     Default::default()
  /// #   }
  /// fn render(&self, widget: &dyn Renderable, bbox: BBox, cap: &dyn Cap) -> BBox {
  ///   if let Some(widget1) = widget.downcast_ref::<ConcreteWidget1>() {
  ///     self.render_concrete_widget1(widget1, bbox)
  ///   } else if let Some(widget2) = widget.downcast_ref::<ConcreteWidget1>() {
  ///     self.render_concrete_widget2(widget2, bbox)
  ///   } else {
  ///     panic!("Renderable {:?} is unknown to the renderer", widget)
  ///   }
  /// }
  /// # }
  /// # fn main() {}
  /// ```
  // TODO: Ideally we would like to have a double dispatch mechanism for
  //       determining the object to render.
  fn render(&self, object: &dyn Renderable, bbox: BBox, cap: &dyn Cap) -> BBox;

  /// Perform some post-render step.
  fn post_render(&self) {}
}

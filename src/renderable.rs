// renderable.rs

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

use BBox;
use Cap;
use Renderer;


/// A trait representing a renderable object.
pub trait Renderable {
  /// Render the renderable object.
  ///
  /// This method just forwards the call to the given `Renderer`,
  /// supplying a trait object of the actual widget. The renderer is
  /// advised to honor the given `BBox` and is free to inquire
  /// additional state using the supplied `Cap`.
  fn render(&self, renderer: &dyn Renderer, bbox: BBox, cap: &dyn Cap) -> BBox;
}

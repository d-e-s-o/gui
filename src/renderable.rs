// renderable.rs

// *************************************************************************
// * Copyright (C) 2018-2021 Daniel Mueller (deso@posteo.net)              *
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

use std::any::TypeId;
use std::fmt::Debug;

use crate::BBox;
use crate::Cap;
use crate::Renderer;


/// A trait representing a renderable object.
pub trait Renderable: 'static + Debug {
  /// Get the [`TypeId`] of `self`.
  fn type_id(&self) -> TypeId;

  /// Render the renderable object.
  ///
  /// This method should just forward the call to the given
  /// [`Renderer`], supplying a trait object of the actual widget. The
  /// renderer is advised to honor the given [`BBox`] and is free to
  /// inquire additional state using the supplied [`Cap`].
  fn render(&self, cap: &dyn Cap, renderer: &dyn Renderer, bbox: BBox) -> BBox;
}

impl dyn Renderable {
  /// Check if the widget is of type `T`.
  pub fn is<T>(&self) -> bool
  where
    T: Renderable,
  {
    let t = TypeId::of::<T>();
    let own_t = self.type_id();

    t == own_t
  }

  /// Downcast the widget reference to type `T`.
  pub fn downcast_ref<T>(&self) -> Option<&T>
  where
    T: Renderable,
  {
    if self.is::<T>() {
      unsafe { Some(&*(self as *const dyn Renderable as *const T)) }
    } else {
      None
    }
  }

  /// Downcast the widget reference to type `T`.
  pub fn downcast_mut<T>(&mut self) -> Option<&mut T>
  where
    T: Renderable,
  {
    if self.is::<T>() {
      unsafe { Some(&mut *(self as *mut dyn Renderable as *mut T)) }
    } else {
      None
    }
  }
}

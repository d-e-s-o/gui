// Copyright (C) 2018-2024 Daniel Mueller (deso@posteo.net)
// SPDX-License-Identifier: GPL-3.0-or-later

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

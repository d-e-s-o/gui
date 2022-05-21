// widget.rs

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

use crate::Cap;
use crate::Handleable;
use crate::MutCap;
use crate::Object;
use crate::Renderable;


/// A widget as used by a [`Ui`][crate::Ui].
///
/// In addition to taking care of [`Id`][crate::Id] management and
/// parent-child relationships, the `Ui` is responsible for dispatching
/// events to widgets and rendering them. Hence, a widget usable for the
/// `Ui` needs to implement [`Handleable`], [`Renderable`], and
/// [`Object`].
pub trait Widget<E, M>: Handleable<E, M> + Renderable + Object + Debug {
  /// Get the [`TypeId`] of `self`.
  fn type_id(&self) -> TypeId;

  /// Retrieve a reference to a widget's data.
  ///
  /// # Panics
  ///
  /// This function will panic if the data associated with the object is
  /// not of type `D`.
  fn data<'c, D>(&self, cap: &'c dyn Cap) -> &'c D
  where
    Self: Sized,
    D: 'static,
  {
    cap.data(self.id()).downcast_ref::<D>().unwrap()
  }

  /// Retrieve a mutable reference to a widget's data.
  ///
  /// # Panics
  ///
  /// This function will panic if the data associated with the object is
  /// not of type `D`.
  fn data_mut<'c, D>(&self, cap: &'c mut dyn MutCap<E, M>) -> &'c mut D
  where
    Self: Sized,
    D: 'static,
  {
    cap.data_mut(self.id()).downcast_mut::<D>().unwrap()
  }
}

impl<E, M> dyn Widget<E, M> {
  /// Check if the widget is of type `T`.
  pub fn is<T: Widget<E, M>>(&self) -> bool {
    let t = TypeId::of::<T>();
    let own_t = Widget::type_id(self);

    t == own_t
  }

  /// Downcast the widget reference to type `T`.
  pub fn downcast_ref<T: Widget<E, M>>(&self) -> Option<&T> {
    if self.is::<T>() {
      unsafe { Some(&*(self as *const dyn Widget<E, M> as *const T)) }
    } else {
      None
    }
  }

  /// Downcast the widget reference to type `T`.
  pub fn downcast_mut<T: Widget<E, M>>(&mut self) -> Option<&mut T> {
    if self.is::<T>() {
      unsafe { Some(&mut *(self as *mut dyn Widget<E, M> as *mut T)) }
    } else {
      None
    }
  }
}

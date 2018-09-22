// widget.rs

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

use std::any::TypeId;
use std::fmt::Debug;

use crate::Handleable;
use crate::Object;
use crate::Renderable;


/// A widget as used by a `Ui`.
///
/// In addition to taking care of `Id` management and parent-child
/// relationships, the `Ui` is responsible for dispatching events to
/// widgets and rendering them. Hence, a widget usable for the `Ui`
/// needs to implement `Handleable`, `Renderable`, and `Object`.
pub trait Widget: Handleable + Renderable + Object + Debug + 'static {
  /// Get the `TypeId` of `self`.
  fn type_id(&self) -> TypeId;
}

impl dyn Widget {
  /// Check if the widget is of type `T`.
  pub fn is<T: Widget>(&self) -> bool {
    let t = TypeId::of::<T>();
    let own_t = self.type_id();

    t == own_t
  }

  /// Downcast the widget reference to type `T`.
  pub fn downcast_ref<T: Widget>(&self) -> Option<&T> {
    if self.is::<T>() {
      unsafe { Some(&*(self as *const dyn Widget as *const T)) }
    } else {
      None
    }
  }

  /// Downcast the widget reference to type `T`.
  pub fn downcast_mut<T: Widget>(&mut self) -> Option<&mut T> {
    if self.is::<T>() {
      unsafe { Some(&mut *(self as *mut dyn Widget as *mut T)) }
    } else {
      None
    }
  }
}

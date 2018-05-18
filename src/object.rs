// object.rs

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

use std::slice::Iter;

use Id;


/// An `Object` represents a first-class entity in a UI.
pub trait Object {
  /// Retrieve this object's `Id`.
  fn id(&self) -> Id;

  /// Retrieve the `Id` for the parent object, if any.
  fn parent_id(&self) -> Option<Id>;

  /// Add a child to an object.
  // TODO: I don't consider it nice to have each `Object` contain this
  //       method. It may be nicer to use a downcast to something like a
  //       container trait but no way has been found to make that
  //       happen.
  fn add_child(&mut self, _id: Id) {
    panic!("Cannot add an object to a non-container")
  }

  /// Retrieve an iterator over the children. Iteration happens in
  /// z-order, from highest to lowest.
  fn iter(&self) -> ChildIter {
    ChildIter::new()
  }
}


/// An iterator over the children of an `Object`.
#[derive(Clone, Debug, Default)]
pub struct ChildIter<'object> {
  iter: Option<Iter<'object, Id>>,
}

impl<'object> ChildIter<'object> {
  /// A child iterator iterating over nothing.
  pub fn new() -> Self {
    ChildIter {
      iter: None,
    }
  }

  /// A child iterator wrapping the given iterator.
  pub fn with_iter(iter: Iter<'object, Id>) -> Self {
    ChildIter {
      iter: Some(iter),
    }
  }
}

impl<'object> Iterator for ChildIter<'object> {
  type Item = &'object Id;

  fn next(&mut self) -> Option<Self::Item> {
    if let Some(ref mut iter) = self.iter {
      iter.next()
    } else {
      None
    }
  }
}

impl<'object> DoubleEndedIterator for ChildIter<'object> {
  fn next_back(&mut self) -> Option<Self::Item> {
    if let Some(ref mut iter) = self.iter {
      iter.next_back()
    } else {
      None
    }
  }
}

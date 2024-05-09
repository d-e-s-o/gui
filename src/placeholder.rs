// Copyright (C) 2018-2024 Daniel Mueller (deso@posteo.net)
// SPDX-License-Identifier: GPL-3.0-or-later

use std::any::TypeId;

use async_trait::async_trait;

use crate::BBox;
use crate::Cap;
use crate::Handleable;
use crate::Id;
use crate::MutCap;
use crate::Object;
use crate::Renderable;
use crate::Renderer;
use crate::Widget;


/// This class is a dummy implementation of a [`Widget`]. Objects of it
/// are used internally in the [`Ui`] class while the actual widget is
/// being created.
#[derive(Debug)]
pub(crate) struct Placeholder;

impl Renderable for Placeholder {
  fn type_id(&self) -> TypeId {
    TypeId::of::<Placeholder>()
  }

  fn render(&self, _cap: &dyn Cap, _renderer: &dyn Renderer, _bbox: BBox) -> BBox {
    unreachable!()
  }
}

impl Object for Placeholder {
  fn id(&self) -> Id {
    unreachable!()
  }
}

#[async_trait(?Send)]
impl<E, M> Handleable<E, M> for Placeholder {
  async fn handle(&self, _cap: &mut dyn MutCap<E, M>, _event: E) -> Option<E> {
    unreachable!()
  }

  async fn react(&self, _message: M, _cap: &mut dyn MutCap<E, M>) -> Option<M> {
    unreachable!()
  }

  async fn respond(&self, _message: &mut M, _cap: &mut dyn MutCap<E, M>) -> Option<M> {
    unreachable!()
  }
}

impl<E, M> Widget<E, M> for Placeholder {
  fn type_id(&self) -> TypeId {
    TypeId::of::<Placeholder>()
  }
}

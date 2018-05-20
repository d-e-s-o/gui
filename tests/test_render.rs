// test_render.rs

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

#![deny(warnings)]

extern crate gui;
#[macro_use]
extern crate gui_derive;

mod common;

use std::any::Any;
use std::cell::Cell;

use gui::Cap;
use gui::Renderer;
use gui::Ui;

use common::TestRootWidget;
use common::TestWidget;


#[derive(Debug)]
struct CountingRenderer {
  pre_render_count: Cell<u64>,
  post_render_count: Cell<u64>,
  widget_render_count: Cell<u64>,
  root_widget_render_count: Cell<u64>,
  total_render_count: Cell<u64>,
}

impl CountingRenderer {
  fn new() -> Self {
    CountingRenderer {
      pre_render_count: Cell::new(0),
      post_render_count: Cell::new(0),
      widget_render_count: Cell::new(0),
      root_widget_render_count: Cell::new(0),
      total_render_count: Cell::new(0),
    }
  }
}

impl Renderer for CountingRenderer {
  fn pre_render(&self) {
    self.pre_render_count.set(self.pre_render_count.get() + 1);
  }

  fn render(&self, object: &Any) {
    if object.downcast_ref::<TestRootWidget>().is_some() {
      self.root_widget_render_count.set(
        self.root_widget_render_count.get() +
          1,
      );
    } else if object.downcast_ref::<TestWidget>().is_some() {
      self.widget_render_count.set(
        self.widget_render_count.get() + 1,
      );
    }
    self.total_render_count.set(
      self.total_render_count.get() + 1,
    );
  }

  fn post_render(&self) {
    self.post_render_count.set(self.post_render_count.get() + 1);
  }
}


#[test]
fn render_is_called_for_each_widget() {
  let renderer = CountingRenderer::new();
  let (mut ui, mut root) = Ui::new(&mut |id, _cap| {
    Box::new(TestRootWidget::new(id))
  });
  let _ = ui.add_widget(&mut root, &mut |parent_id, id, _cap| {
    Box::new(TestWidget::new(parent_id, id))
  });
  let _ = ui.add_widget(&mut root, &mut |parent_id, id, _cap| {
    Box::new(TestWidget::new(parent_id, id))
  });

  ui.render(&renderer);

  assert_eq!(renderer.pre_render_count.get(), 1);
  assert_eq!(renderer.post_render_count.get(), 1);
  assert_eq!(renderer.root_widget_render_count.get(), 1);
  assert_eq!(renderer.widget_render_count.get(), 2);
  assert_eq!(renderer.total_render_count.get(), 3);
}

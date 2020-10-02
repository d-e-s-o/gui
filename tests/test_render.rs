// test_render.rs

// *************************************************************************
// * Copyright (C) 2018-2020 Daniel Mueller (deso@posteo.net)              *
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

mod common;

use std::cell::Cell;

use gui::BBox;
use gui::Cap;
use gui::Id;
use gui::MutCap;
use gui::Object;
use gui::Renderable;
use gui::Renderer;
use gui::Ui;

use crate::common::TestWidget;


#[derive(Debug)]
struct CountingRenderer {
  pre_render_count: Cell<u64>,
  post_render_count: Cell<u64>,
  total_render_count: Cell<u64>,
}

impl CountingRenderer {
  fn new() -> Self {
    Self {
      pre_render_count: Cell::new(0),
      post_render_count: Cell::new(0),
      total_render_count: Cell::new(0),
    }
  }
}

impl Renderer for CountingRenderer {
  fn renderable_area(&self) -> BBox {
    BBox::default()
  }

  fn pre_render(&self) {
    self.pre_render_count.set(self.pre_render_count.get() + 1);
  }

  fn render(&self, object: &dyn Renderable, bbox: BBox, _cap: &dyn Cap) -> BBox {
    assert!(object.downcast_ref::<TestWidget>().is_some());

    self.total_render_count.set(
      self.total_render_count.get() + 1,
    );

    bbox
  }

  fn post_render(&self) {
    self.post_render_count.set(self.post_render_count.get() + 1);
  }
}


#[test]
fn render_is_called_for_each_widget() {
  let renderer = CountingRenderer::new();
  let (mut ui, root) = Ui::new(|id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let _ = ui.add_ui_widget(root, |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let _ = ui.add_ui_widget(root, |id, _cap| {
    Box::new(TestWidget::new(id))
  });

  ui.render(&renderer);

  assert_eq!(renderer.pre_render_count.get(), 1);
  assert_eq!(renderer.post_render_count.get(), 1);
  assert_eq!(renderer.total_render_count.get(), 3);
}

#[test]
fn render_honors_visibility_flag() {
  let renderer = CountingRenderer::new();
  let (mut ui, root) = Ui::new(|id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w1 = ui.add_ui_widget(root, |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w2 = ui.add_ui_widget(root, |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let _ = ui.add_ui_widget(w2, |id, _cap| {
    Box::new(TestWidget::new(id))
  });

  ui.hide(w1);
  ui.render(&renderer);

  assert_eq!(renderer.pre_render_count.get(), 1);
  assert_eq!(renderer.post_render_count.get(), 1);
  assert_eq!(renderer.total_render_count.get(), 3);

  // Hiding `w2` should make two widgets invisible.
  ui.hide(w2);
  ui.render(&renderer);

  assert_eq!(renderer.pre_render_count.get(), 2);
  assert_eq!(renderer.post_render_count.get(), 2);
  assert_eq!(renderer.total_render_count.get(), 4);

  ui.show(w1);
  ui.render(&renderer);

  assert_eq!(renderer.pre_render_count.get(), 3);
  assert_eq!(renderer.post_render_count.get(), 3);
  assert_eq!(renderer.total_render_count.get(), 6);

  // Showing `w2` should make itself and its child visible again.
  ui.show(w2);
  ui.render(&renderer);

  assert_eq!(renderer.pre_render_count.get(), 4);
  assert_eq!(renderer.post_render_count.get(), 4);
  assert_eq!(renderer.total_render_count.get(), 10);
}


static mut ROOT: Option<Id> = None;
static mut CONTAINER: Option<Id> = None;
static mut WIDGET: Option<Id> = None;

#[derive(Debug)]
struct BBoxRenderer {
  valid_bbox_count: Cell<u64>,
}

impl BBoxRenderer {
  fn new() -> Self {
    Self {
      valid_bbox_count: Cell::new(0),
    }
  }
}

impl Renderer for BBoxRenderer {
  fn renderable_area(&self) -> BBox {
    BBox {
      x: 0,
      y: 10,
      w: 100,
      h: 40,
    }
  }

  fn render(&self, object: &dyn Renderable, mut bbox: BBox, _cap: &dyn Cap) -> BBox {
    let mut expected = self.renderable_area();
    let widget = object.downcast_ref::<TestWidget>().unwrap();

    if widget.id() == unsafe { *CONTAINER.as_ref().unwrap() } {
      expected.w -= 10;
    } else if widget.id() == unsafe { *WIDGET.as_ref().unwrap() } {
      expected.w -= 10;
      expected.h -= 10;
    }

    if bbox == expected {
      self.valid_bbox_count.set(self.valid_bbox_count.get() + 1);
    }

    if widget.id() == unsafe { *ROOT.as_ref().unwrap() } {
      bbox.w -= 10
    } else if widget.id() == unsafe { *CONTAINER.as_ref().unwrap() } {
      bbox.h -= 10
    }
    bbox
  }
}


#[test]
fn bounding_box_is_properly_sized() {
  let renderer = BBoxRenderer::new();
  let (mut ui, root) = Ui::new(|id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let cont = ui.add_ui_widget(root, |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let widget = ui.add_ui_widget(cont, |id, _cap| {
    Box::new(TestWidget::new(id))
  });

  unsafe {
    ROOT = Some(root);
    CONTAINER = Some(cont);
    WIDGET = Some(widget);
  }

  ui.render(&renderer);

  assert_eq!(renderer.valid_bbox_count.get(), 3)
}

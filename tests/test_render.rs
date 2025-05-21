// Copyright (C) 2018-2025 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(static_mut_refs)]

mod common;

use std::any::TypeId;
use std::cell::Cell;

use gui::BBox;
use gui::Cap;
use gui::Id;
use gui::MutCap;
use gui::Object;
use gui::Renderable;
use gui::Renderer;
use gui::Ui;
use gui::Widget;
use gui_derive::Handleable;

use crate::common::Event;
use crate::common::Message;
use crate::common::TestWidget;
use crate::common::TestWidgetDataBuilder;


#[derive(Debug)]
struct CountingRenderer {
  pre_render_count: Cell<u64>,
  post_render_count: Cell<u64>,
  total_render_count: Cell<u64>,
  total_render_done_count: Cell<u64>,
}

impl CountingRenderer {
  fn new() -> Self {
    Self {
      pre_render_count: Cell::new(0),
      post_render_count: Cell::new(0),
      total_render_count: Cell::new(0),
      total_render_done_count: Cell::new(0),
    }
  }
}

impl Renderer for CountingRenderer {
  fn renderable_area(&self) -> BBox {
    BBox {
      x: 0,
      y: 0,
      w: 10,
      h: 10,
    }
  }

  fn pre_render(&self) {
    self.pre_render_count.set(self.pre_render_count.get() + 1);
  }

  fn render(&self, _object: &dyn Renderable, _cap: &dyn Cap, bbox: BBox) -> BBox {
    self
      .total_render_count
      .set(self.total_render_count.get() + 1);

    bbox
  }

  fn render_done(&self, _object: &dyn Renderable, _cap: &dyn Cap, _bbox: BBox) {
    assert!(self.total_render_done_count.get() < self.total_render_count.get());

    self
      .total_render_done_count
      .set(self.total_render_done_count.get() + 1);
  }

  fn post_render(&self) {
    self.post_render_count.set(self.post_render_count.get() + 1);
  }
}


#[test]
fn render_is_called_for_each_widget() {
  let renderer = CountingRenderer::new();
  let (mut ui, root) = Ui::new(
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let _ = ui.add_ui_widget(
    root,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let _ = ui.add_ui_widget(
    root,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );

  ui.render(&renderer);

  assert_eq!(renderer.pre_render_count.get(), 1);
  assert_eq!(renderer.post_render_count.get(), 1);
  assert_eq!(renderer.total_render_count.get(), 3);
  assert_eq!(renderer.total_render_done_count.get(), 3);
}

#[test]
fn render_honors_visibility_flag() {
  let renderer = CountingRenderer::new();
  let (mut ui, root) = Ui::new(
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let w1 = ui.add_ui_widget(
    root,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let w2 = ui.add_ui_widget(
    root,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let _ = ui.add_ui_widget(
    w2,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );

  ui.hide(w1);
  ui.render(&renderer);

  assert_eq!(renderer.pre_render_count.get(), 1);
  assert_eq!(renderer.post_render_count.get(), 1);
  assert_eq!(renderer.total_render_count.get(), 3);
  assert_eq!(renderer.total_render_done_count.get(), 3);

  // Hiding `w2` should make two widgets invisible.
  ui.hide(w2);
  ui.render(&renderer);

  assert_eq!(renderer.pre_render_count.get(), 2);
  assert_eq!(renderer.post_render_count.get(), 2);
  assert_eq!(renderer.total_render_count.get(), 4);
  assert_eq!(renderer.total_render_done_count.get(), 4);

  ui.show(w1);
  ui.render(&renderer);

  assert_eq!(renderer.pre_render_count.get(), 3);
  assert_eq!(renderer.post_render_count.get(), 3);
  assert_eq!(renderer.total_render_count.get(), 6);
  assert_eq!(renderer.total_render_done_count.get(), 6);

  // Showing `w2` should make itself and its child visible again.
  ui.show(w2);
  ui.render(&renderer);

  assert_eq!(renderer.pre_render_count.get(), 4);
  assert_eq!(renderer.post_render_count.get(), 4);
  assert_eq!(renderer.total_render_count.get(), 10);
  assert_eq!(renderer.total_render_done_count.get(), 10);
}


#[derive(Debug, Handleable)]
#[gui(Event = Event)]
struct TestNoBBoxWidget {
  id: Id,
}

impl Renderable for TestNoBBoxWidget {
  fn type_id(&self) -> TypeId {
    TypeId::of::<TestNoBBoxWidget>()
  }

  fn render(&self, cap: &dyn Cap, renderer: &dyn Renderer, bbox: BBox) -> BBox {
    let bbox = BBox {
      x: bbox.x,
      y: bbox.y,
      w: 0,
      h: bbox.h,
    };
    renderer.render(self, cap, bbox)
  }

  fn render_done(&self, cap: &dyn Cap, renderer: &dyn Renderer, bbox: BBox) {
    renderer.render_done(self, cap, bbox)
  }
}

impl Object for TestNoBBoxWidget {
  fn id(&self) -> Id {
    self.id
  }
}

impl Widget<Event, Message> for TestNoBBoxWidget {
  fn type_id(&self) -> TypeId {
    TypeId::of::<TestNoBBoxWidget>()
  }
}


/// Check that a widget won't receive a [`Renderable::render`] call if
/// its parent reported an empty `BBox`.
#[test]
fn render_is_omitted_for_empty_bbox() {
  let renderer = CountingRenderer::new();
  let (mut ui, root) = Ui::new(
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let no_bbox = ui.add_ui_widget(
    root,
    || Box::new(()),
    |id, _cap| Box::new(TestNoBBoxWidget { id }),
  );
  let _ = ui.add_ui_widget(
    no_bbox,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );

  ui.render(&renderer);

  assert_eq!(renderer.pre_render_count.get(), 1);
  assert_eq!(renderer.post_render_count.get(), 1);
  assert_eq!(renderer.total_render_count.get(), 2);
  assert_eq!(renderer.total_render_done_count.get(), 2);
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

impl BBoxRenderer {
  fn check_bbox(&self, widget: &TestWidget, bbox: BBox) {
    let mut expected = self.renderable_area();

    if widget.id() == unsafe { *CONTAINER.as_ref().unwrap() } {
      expected.w -= 10;
    } else if widget.id() == unsafe { *WIDGET.as_ref().unwrap() } {
      expected.w -= 10;
      expected.h -= 10;
    }

    if bbox == expected {
      self.valid_bbox_count.set(self.valid_bbox_count.get() + 1);
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

  fn render(&self, object: &dyn Renderable, _cap: &dyn Cap, mut bbox: BBox) -> BBox {
    let widget = object.downcast_ref::<TestWidget>().unwrap();
    let () = self.check_bbox(widget, bbox);

    if widget.id() == unsafe { *ROOT.as_ref().unwrap() } {
      bbox.w -= 10
    } else if widget.id() == unsafe { *CONTAINER.as_ref().unwrap() } {
      bbox.h -= 10
    }
    bbox
  }

  fn render_done(&self, object: &dyn Renderable, _cap: &dyn Cap, bbox: BBox) {
    let widget = object.downcast_ref::<TestWidget>().unwrap();
    let () = self.check_bbox(widget, bbox);
  }
}


#[test]
fn bounding_box_is_properly_sized() {
  let renderer = BBoxRenderer::new();
  let (mut ui, root) = Ui::new(
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let cont = ui.add_ui_widget(
    root,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let widget = ui.add_ui_widget(
    cont,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );

  unsafe {
    ROOT = Some(root);
    CONTAINER = Some(cont);
    WIDGET = Some(widget);
  }

  ui.render(&renderer);

  assert_eq!(renderer.valid_bbox_count.get(), 6)
}

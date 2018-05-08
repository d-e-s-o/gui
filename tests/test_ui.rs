// test_ui.rs

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

use gui::Ui;

use common::TestContainer;
use common::TestRenderer;
use common::TestRootWidget;
use common::TestWidget;


#[test]
fn correct_ids() {
  let mut ui = Ui::<TestRenderer>::new();
  let root = ui.add_root_widget(&|id| {
    Box::new(TestRootWidget::new(id))
  });
  let w1 = ui.add_widget(root, &|parent_id, id| {
    Box::new(TestWidget::new(parent_id, id))
  });
  let w2 = ui.add_widget(root, &|parent_id, id| {
    Box::new(TestWidget::new(parent_id, id))
  });
  // And a container.
  let c1 = ui.add_widget(root, &|parent_id, id| {
    Box::new(TestContainer::new(parent_id, id))
  });
  // And a widget to the container.
  let w3 = ui.add_widget(c1, &|parent_id, id| {
    Box::new(TestWidget::new(parent_id, id))
  });
  // And another container for deeper nesting.
  let c2 = ui.add_widget(c1, &|parent_id, id| {
    Box::new(TestContainer::new(parent_id, id))
  });
  // And the last widget.
  let w4 = ui.add_widget(c2, &|parent_id, id| {
    Box::new(TestWidget::new(parent_id, id))
  });

  assert_eq!(ui.parent_id(root), None);
  assert_eq!(ui.parent_id(w1).unwrap(), root);
  assert_eq!(ui.parent_id(w2).unwrap(), root);
  assert_eq!(ui.parent_id(c1).unwrap(), root);
  assert_eq!(ui.parent_id(w3).unwrap(), c1);
  assert_eq!(ui.parent_id(w4).unwrap(), c2);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "Only one root widget may exist in a Ui")]
fn only_single_root_widget_allowed() {
  let mut ui = Ui::<TestRenderer>::new();
  let _ = ui.add_root_widget(&|id| {
    Box::new(TestRootWidget::new(id))
  });
  let _ = ui.add_root_widget(&|id| {
    Box::new(TestRootWidget::new(id))
  });
}

#[test]
#[should_panic(expected = "Cannot add an object to a non-container")]
fn only_containers_can_have_children() {
  let mut ui = Ui::<TestRenderer>::new();
  let root = ui.add_root_widget(&|id| {
    Box::new(TestRootWidget::new(id))
  });
  let widget = ui.add_widget(root, &|parent_id, id| {
    Box::new(TestWidget::new(parent_id, id))
  });
  let _ = ui.add_widget(widget, &|parent_id, id| {
    Box::new(TestWidget::new(parent_id, id))
  });
}

#[test]
fn initial_focus() {
  let mut ui = Ui::<TestRenderer>::new();
  let root = ui.add_root_widget(&|id| {
    Box::new(TestRootWidget::new(id))
  });
  // The widget created first should receive the focus and stay
  // focused until directed otherwise.
  assert!(ui.is_focused(root));

  let _ = ui.add_widget(root, &|parent_id, id| {
    Box::new(TestWidget::new(parent_id, id))
  });
  assert!(ui.is_focused(root));
}

#[test]
fn focus_widget() {
  let mut ui = Ui::<TestRenderer>::new();
  let root = ui.add_root_widget(&|id| {
    Box::new(TestRootWidget::new(id))
  });
  let widget = ui.add_widget(root, &|parent_id, id| {
    Box::new(TestWidget::new(parent_id, id))
  });

  ui.focus(widget);
  assert!(ui.is_focused(widget));
}

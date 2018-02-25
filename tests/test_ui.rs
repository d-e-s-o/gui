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

mod common;

use gui::Ui;

use common::TestRootWidget;
use common::TestWidget;


#[test]
fn correct_ids() {
  let mut ui = Ui::new();
  let root = ui.add_root_widget(|id| {
    Box::new(TestRootWidget::new(id))
  });
  let w1 = ui.add_widget(root, |parent_id, id| {
    Box::new(TestWidget::new(parent_id, id))
  });

  assert_eq!(ui.parent_id(root), None);
  assert_eq!(ui.parent_id(w1).unwrap(), root);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "Only one root widget may exist in a Ui")]
fn only_single_root_widget_allowed() {
  let mut ui = Ui::new();
  let _ = ui.add_root_widget(|id| {
    Box::new(TestRootWidget::new(id))
  });
  let _ = ui.add_root_widget(|id| {
    Box::new(TestRootWidget::new(id))
  });
}

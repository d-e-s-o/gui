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

use gui::Cap;
use gui::Event;
use gui::Handleable;
use gui::Id;
use gui::Ui;
use gui::UiEvent;

use common::TestContainer;
use common::TestRootWidget;
use common::TestWidget;
use common::unwrap_custom;


#[test]
fn correct_ids() {
  let mut ui = Ui::new();
  let root = ui.add_root_widget(&mut |id, _cap| {
    Box::new(TestRootWidget::new(id))
  });
  let w1 = ui.add_widget(root, &mut |parent_id, id, _cap| {
    Box::new(TestWidget::new(parent_id, id))
  });
  let w2 = ui.add_widget(root, &mut |parent_id, id, _cap| {
    Box::new(TestWidget::new(parent_id, id))
  });
  // And a container.
  let c1 = ui.add_widget(root, &mut |parent_id, id, _cap| {
    Box::new(TestContainer::new(parent_id, id))
  });
  // And a widget to the container.
  let w3 = ui.add_widget(c1, &mut |parent_id, id, _cap| {
    Box::new(TestWidget::new(parent_id, id))
  });
  // And another container for deeper nesting.
  let c2 = ui.add_widget(c1, &mut |parent_id, id, _cap| {
    Box::new(TestContainer::new(parent_id, id))
  });
  // And the last widget.
  let w4 = ui.add_widget(c2, &mut |parent_id, id, _cap| {
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
  let mut ui = Ui::new();
  let _ = ui.add_root_widget(&mut |id, _cap| {
    Box::new(TestRootWidget::new(id))
  });
  let _ = ui.add_root_widget(&mut |id, _cap| {
    Box::new(TestRootWidget::new(id))
  });
}

#[test]
#[should_panic(expected = "Cannot add an object to a non-container")]
fn only_containers_can_have_children() {
  let mut ui = Ui::new();
  let root = ui.add_root_widget(&mut |id, _cap| {
    Box::new(TestRootWidget::new(id))
  });
  let widget = ui.add_widget(root, &mut |parent_id, id, _cap| {
    Box::new(TestWidget::new(parent_id, id))
  });
  let _ = ui.add_widget(widget, &mut |parent_id, id, _cap| {
    Box::new(TestWidget::new(parent_id, id))
  });
}

#[test]
fn initial_focus() {
  let mut ui = Ui::new();
  let root = ui.add_root_widget(&mut |id, _cap| {
    Box::new(TestRootWidget::new(id))
  });
  // The widget created first should receive the focus and stay
  // focused until directed otherwise.
  assert!(ui.is_focused(root));

  let _ = ui.add_widget(root, &mut |parent_id, id, _cap| {
    Box::new(TestWidget::new(parent_id, id))
  });
  assert!(ui.is_focused(root));
}

#[test]
fn focus_widget() {
  let mut ui = Ui::new();
  let root = ui.add_root_widget(&mut |id, _cap| {
    Box::new(TestRootWidget::new(id))
  });
  let widget = ui.add_widget(root, &mut |parent_id, id, _cap| {
    Box::new(TestWidget::new(parent_id, id))
  });

  ui.focus(widget);
  assert!(ui.is_focused(widget));
}


fn counting_handler(event: Event) -> Option<UiEvent> {
  Some(
    match event {
      Event::Custom(e) => {
        let value = *e.downcast::<u64>().unwrap();
        Event::Custom(Box::new(value + 1))
      },
      _ => event,
    }.into(),
  )
}


#[derive(Debug, GuiRootWidget)]
struct CreatingRootWidget {
  id: Id,
  children: Vec<Id>,
}

impl CreatingRootWidget {
  pub fn new(id: Id, cap: &mut Cap) -> Self {
    let _ = cap.add_widget(id, &mut |parent_id, id, cap| {
      Box::new(CreatingContainer::new(parent_id, id, cap))
    });
    CreatingRootWidget {
      id: id,
      children: Vec::new(),
    }
  }
}

impl Handleable for CreatingRootWidget {
  fn handle(&mut self, event: Event) -> Option<UiEvent> {
    counting_handler(event)
  }
}


#[derive(Debug, GuiContainer)]
struct CreatingContainer {
  parent_id: Id,
  id: Id,
  children: Vec<Id>,
}

impl CreatingContainer {
  pub fn new(parent_id: Id, id: Id, cap: &mut Cap) -> Self {
    let _ = cap.add_widget(id, &mut |parent_id, id, cap| {
      Box::new(CreatingWidget::new(parent_id, id, cap))
    });

    CreatingContainer {
      parent_id: parent_id,
      id: id,
      children: Vec::new(),
    }
  }
}

impl Handleable for CreatingContainer {
  fn handle(&mut self, event: Event) -> Option<UiEvent> {
    counting_handler(event)
  }
}


#[derive(Debug, GuiWidget)]
struct CreatingWidget {
  parent_id: Id,
  id: Id,
}

impl CreatingWidget {
  pub fn new(parent_id: Id, id: Id, cap: &mut Cap) -> Self {
    // This widget is not a container and so we add the newly created
    // widget to the parent.
    let child = cap.add_widget(parent_id, &mut |parent_id, id, _cap| {
      Box::new(TestWidget::with_handler(parent_id, id, counting_handler))
    });
    // Focus the "last" widget. Doing so allows us to send an event to
    // all widgets.
    cap.focus(child);

    CreatingWidget {
      parent_id: parent_id,
      id: id,
    }
  }
}

impl Handleable for CreatingWidget {
  fn handle(&mut self, event: Event) -> Option<UiEvent> {
    counting_handler(event)
  }
}


#[test]
fn recursive_widget_creation() {
  let mut ui = Ui::new();
  // We only create the root widget directly but it will take care of
  // recursively creating a bunch of more widgets.
  let _ = ui.add_root_widget(&mut |id, cap| {
    Box::new(CreatingRootWidget::new(id, cap))
  });

  let event = Event::Custom(Box::new(0u64));
  let result = ui.handle(event).unwrap();
  // We expect three increments. Note that we have four widgets in
  // total, but we cannot easily have the event reach all four of them
  // because two are peers sharing a parent.
  assert_eq!(*unwrap_custom::<u64>(result), 3);
}


#[derive(Debug)]
struct Moveable {}

#[derive(Debug, GuiWidget, GuiHandleable)]
struct MovingWidget {
  parent_id: Id,
  id: Id,
  object: Moveable,
}

impl MovingWidget {
  pub fn new(parent_id: Id, id: Id, object: Moveable) -> Self {
    MovingWidget {
      parent_id: parent_id,
      id: id,
      object: object,
    }
  }
}


// This test case illustrates how to create a widget that takes
// ownership of some data during its construction.
#[test]
fn moving_widget_creation() {
  let mut object = Some(Moveable {});
  let mut ui = Ui::new();
  let root = ui.add_root_widget(&mut |id, _cap| {
    Box::new(TestRootWidget::new(id))
  });
  let _ = ui.add_widget(root, &mut |parent_id, id, _cap| {
    let moveable = object.take().unwrap();
    Box::new(MovingWidget::new(parent_id, id, moveable))
  });
}

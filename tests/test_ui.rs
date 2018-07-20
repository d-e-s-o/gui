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
use gui::Key;
use gui::MetaEvent;
use gui::Ui;
#[cfg(debug_assertions)]
use gui::UiEvent;

use common::TestContainer;
use common::TestRootWidget;
use common::TestWidget;
use common::unwrap_custom;


#[test]
fn correct_ids() {
  let (mut ui, root) = Ui::new(&mut |id, _cap| {
    Box::new(TestRootWidget::new(id))
  });
  let w1 = ui.add_widget(root, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w2 = ui.add_widget(root, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  // And a container.
  let c1 = ui.add_widget(root, &mut |id, _cap| {
    Box::new(TestContainer::new(id))
  });
  // And a widget to the container.
  let w3 = ui.add_widget(c1, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  // And another container for deeper nesting.
  let c2 = ui.add_widget(c1, &mut |id, _cap| {
    Box::new(TestContainer::new(id))
  });
  // And the last widget.
  let w4 = ui.add_widget(c2, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
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
#[should_panic(expected = "The given Id belongs to a different Ui")]
fn share_ids_between_ui_objects() {
  let (mut ui1, root) = Ui::new(&mut |id, _cap| {
    Box::new(TestRootWidget::new(id))
  });
  let widget = ui1.add_widget(root, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });

  let (mut ui2, _) = Ui::new(&mut |id, _cap| {
    Box::new(TestRootWidget::new(id))
  });

  // `widget` is registered to `ui1` and so using it in the context of
  // `ui2` is not as intended. On debug builds we have special detection
  // in place to provide a meaningful error, that should trigger here.
  ui2.handle(UiEvent::Custom(widget, Box::new(())));
}

#[test]
fn initial_focus() {
  let (mut ui, root) = Ui::new(&mut |id, _cap| {
    Box::new(TestRootWidget::new(id))
  });

  // The widget created first should receive the focus and stay
  // focused until directed otherwise.
  assert!(ui.is_focused(root));

  let _ = ui.add_widget(root, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  assert!(ui.is_focused(root));
}

#[test]
fn focus_widget() {
  let (mut ui, root) = Ui::new(&mut |id, _cap| {
    Box::new(TestRootWidget::new(id))
  });
  let widget = ui.add_widget(root, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });

  ui.focus(widget);
  assert!(ui.is_focused(widget));
}

#[test]
fn last_focused() {
  let (mut ui, root) = Ui::new(&mut |id, _cap| {
    Box::new(TestRootWidget::new(id))
  });
  let c = ui.add_widget(root, &mut |id, _cap| {
    Box::new(TestContainer::new(id))
  });
  let w = ui.add_widget(c, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });

  assert!(ui.is_focused(root));
  assert!(ui.last_focused().is_none());

  ui.focus(c);
  assert_eq!(ui.last_focused().unwrap(), root);

  ui.focus(w);
  assert_eq!(ui.last_focused().unwrap(), c);
}


fn counting_handler(_widget: Id, event: Event, _cap: &mut Cap) -> Option<MetaEvent> {
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
    let _ = cap.add_widget(id, &mut |id, cap| {
      Box::new(CreatingContainer::new(id, cap))
    });
    CreatingRootWidget {
      id: id,
      children: Vec::new(),
    }
  }
}

impl Handleable for CreatingRootWidget {
  fn handle(&mut self, event: Event, cap: &mut Cap) -> Option<MetaEvent> {
    counting_handler(self.id, event, cap)
  }
}


#[derive(Debug, GuiContainer)]
struct CreatingContainer {
  id: Id,
  children: Vec<Id>,
}

impl CreatingContainer {
  pub fn new(id: Id, cap: &mut Cap) -> Self {
    let _ = cap.add_widget(id, &mut |id, cap| {
      Box::new(CreatingWidget::new(id, cap))
    });

    CreatingContainer {
      id: id,
      children: Vec::new(),
    }
  }
}

impl Handleable for CreatingContainer {
  fn handle(&mut self, event: Event, cap: &mut Cap) -> Option<MetaEvent> {
    counting_handler(self.id, event, cap)
  }
}


#[derive(Debug, GuiWidget)]
struct CreatingWidget {
  id: Id,
}

impl CreatingWidget {
  pub fn new(id: Id, cap: &mut Cap) -> Self {
    let parent = cap.parent_id(id).unwrap();
    // This widget is not a container and so we add the newly created
    // widget to the parent.
    let child = cap.add_widget(parent, &mut |id, _cap| {
      Box::new(TestWidget::with_handler(id, counting_handler))
    });
    // Focus the "last" widget. Doing so allows us to send an event to
    // all widgets.
    cap.focus(child);

    CreatingWidget {
      id: id,
    }
  }
}

impl Handleable for CreatingWidget {
  fn handle(&mut self, event: Event, cap: &mut Cap) -> Option<MetaEvent> {
    counting_handler(self.id, event, cap)
  }
}


#[test]
fn recursive_widget_creation() {
  // We only create the root widget directly but it will take care of
  // recursively creating a bunch of more widgets.
  let (mut ui, _) = Ui::new(&mut |id, cap| {
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
  id: Id,
  object: Moveable,
}

impl MovingWidget {
  pub fn new(id: Id, object: Moveable) -> Self {
    MovingWidget {
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
  let (mut ui, root) = Ui::new(&mut |id, _cap| {
    Box::new(TestRootWidget::new(id))
  });
  let _ = ui.add_widget(root, &mut |id, _cap| {
    let moveable = object.take().unwrap();
    Box::new(MovingWidget::new(id, moveable))
  });
}


fn create_handler(widget: Id, event: Event, cap: &mut Cap) -> Option<MetaEvent> {
  match event {
    Event::KeyDown(key) => {
      match key {
        Key::Char('z') => {
          cap.add_widget(widget, &mut |id, _cap| {
            Box::new(TestWidget::new(id))
          });
          None
        },
        _ => Some(event.into()),
      }
    },
    _ => Some(event.into()),
  }
}


#[test]
fn event_based_widget_creation() {
  let (mut ui, root) = Ui::new(&mut |id, _cap| {
    Box::new(TestRootWidget::with_handler(id, create_handler))
  });

  assert_eq!(ui.children(root).count(), 0);

  let event = Event::KeyDown(Key::Char('z'));
  let result = ui.handle(event);
  assert!(result.is_none());

  // We must have created a widget.
  assert_eq!(ui.children(root).count(), 1);
}


fn recursive_operations_handler(widget: Id, _event: Event, cap: &mut Cap) -> Option<MetaEvent> {
  // Check that we can use the supplied `Cap` object to retrieve our
  // own parent's ID.
  cap.parent_id(widget);
  cap.focus(widget);
  cap.is_focused(widget);
  None
}

#[test]
fn recursive_widget_operations() {
  let (mut ui, _) = Ui::new(&mut |id, _cap| {
    Box::new(TestRootWidget::with_handler(id, recursive_operations_handler))
  });

  ui.handle(Event::Custom(Box::new(())));
}

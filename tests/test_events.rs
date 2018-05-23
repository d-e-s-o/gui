// test_events.rs

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

#![allow(unknown_lints)]
#![deny(warnings)]

extern crate gui;
#[macro_use]
extern crate gui_derive;

mod common;

use std::cell::RefCell;
use std::rc::Rc;

use gui::Cap;
use gui::Event;
use gui::Id;
use gui::Key;
use gui::Ui;
use gui::UiEvent;
use gui::WidgetRef;

use common::clone_event;
use common::compare_ui_events;
use common::TestContainer;
use common::TestRootWidget;
use common::TestWidget;
use common::unwrap_custom;


#[test]
fn convert_event_into() {
  let event = Event::KeyDown(Key::Char(' '));
  let orig_event = clone_event(&event);
  let ui_event = UiEvent::from(event);

  assert!(compare_ui_events(&ui_event, &UiEvent::Event(orig_event)));
}

#[test]
fn events_bubble_up_when_unhandled() {
  let (mut ui, mut r) = Ui::new(&mut |id, _cap| {
    Box::new(TestRootWidget::new(id))
  });
  let mut c1 = ui.add_widget(&mut r, &mut |parent, id, _cap| {
    Box::new(TestContainer::new(parent, id))
  });
  let mut c2 = ui.add_widget(&mut c1, &mut |parent, id, _cap| {
    Box::new(TestContainer::new(parent, id))
  });
  let w1 = ui.add_widget(&mut c2, &mut |parent, id, _cap| {
    Box::new(TestWidget::new(parent, id))
  });

  let event = Event::KeyUp(Key::Char(' '));
  ui.focus(&w1);

  let result = ui.handle(clone_event(&event));
  // An unhandled event should just be returned after every widget
  // forwarded it.
  assert!(compare_ui_events(&result.unwrap(), &event.into()));
}

fn key_handler(event: Event, cap: &mut Cap, to_focus: Option<Id>) -> Option<UiEvent> {
  match event {
    Event::KeyDown(key) => {
      match key {
        Key::Char('a') => {
          if let Some(id) = to_focus {
            cap.focus(&id);
            None
          } else {
            Some(event.into())
          }
        },
        _ => Some(event.into()),
      }
    },
    _ => Some(event.into()),
  }
}

#[test]
fn event_handling_with_focus() {
  let (mut ui, mut r) = Ui::new(&mut |id, _cap| {
    Box::new(TestRootWidget::new(id))
  });
  let w1 = ui.add_widget(&mut r, &mut |parent, id, _cap| {
    Box::new(TestWidget::with_handler(parent, id, |_s, e, c| {
      key_handler(e, c, None)
    }))
  });
  let w2 = ui.add_widget(&mut r, &mut |parent, id, _cap| {
    Box::new(TestWidget::with_handler(parent, id, move |_s, e, c| {
      key_handler(e, c, Some(w1))
    }))
  });

  ui.focus(&w2);
  assert!(ui.is_focused(&w2));

  // Send a key down event, received by `w2`, which it will
  // translate into a focus event for `w1`.
  let event = Event::KeyDown(Key::Char('a'));
  ui.handle(event);

  assert!(ui.is_focused(&w1));
}

fn custom_undirected_response_handler(_: &mut WidgetRef, event: Event, _cap: &mut Cap) -> Option<UiEvent> {
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

#[test]
fn custom_undirected_response_event() {
  let (mut ui, mut r) = Ui::new(&mut |id, _cap| {
    Box::new(TestRootWidget::with_handler(id, custom_undirected_response_handler))
  });
  let mut c1 = ui.add_widget(&mut r, &mut |parent, id, _cap| {
    Box::new(TestContainer::with_handler(parent, id, custom_undirected_response_handler))
  });
  let w1 = ui.add_widget(&mut c1, &mut |parent, id, _cap| {
    Box::new(TestWidget::with_handler(parent, id, custom_undirected_response_handler))
  });

  // We focus the widget we just created, which means that the event
  // will travel through the widget and all its parents.
  ui.focus(&w1);

  let event = Event::Custom(Box::new(42u64));
  let result = ui.handle(event).unwrap();
  // We expect three increments, one from each of the widgets.
  assert_eq!(*unwrap_custom::<u64>(result), 45);
}

fn custom_directed_response_handler(_: &mut WidgetRef, event: Event, _cap: &mut Cap) -> Option<UiEvent> {
  match event {
    Event::Custom(data) => {
      let cell = *data.downcast::<Rc<RefCell<u64>>>().unwrap();
      let value = *cell.borrow();
      cell.replace(value + 1);
      None
    },
    _ => Some(event.into()),
  }
}

#[test]
fn custom_directed_response_event() {
  let (mut ui, mut r) = Ui::new(&mut |id, _cap| {
    Box::new(TestRootWidget::with_handler(id, custom_directed_response_handler))
  });
  let mut c1 = ui.add_widget(&mut r, &mut |parent, id, _cap| {
    Box::new(TestContainer::with_handler(parent, id, custom_directed_response_handler))
  });
  let w1 = ui.add_widget(&mut c1, &mut |parent, id, _cap| {
    Box::new(TestWidget::with_handler(parent, id, custom_directed_response_handler))
  });

  ui.focus(&w1);

  let cell = RefCell::new(1337u64);
  let rc = Rc::new(cell);
  let event = Event::Custom(Box::new(Rc::clone(&rc)));

  let result = ui.handle(event);
  assert!(result.is_none());

  let value = *(*rc).borrow();
  assert_eq!(value, 1338);
}

#[test]
fn direct_custom_event() {
  let (mut ui, mut r) = Ui::new(&mut |id, _cap| {
    Box::new(TestRootWidget::with_handler(id, custom_undirected_response_handler))
  });
  let mut c1 = ui.add_widget(&mut r, &mut |parent_id, id, _cap| {
    Box::new(TestContainer::with_handler(parent_id, id, custom_undirected_response_handler))
  });
  let w1 = ui.add_widget(&mut c1, &mut |parent_id, id, _cap| {
    Box::new(TestWidget::with_handler(parent_id, id, custom_undirected_response_handler))
  });

  ui.focus(&c1);

  let event = UiEvent::Custom(w1, Box::new(10u64));
  let result = ui.handle(event).unwrap();
  assert_eq!(*unwrap_custom::<u64>(result), 13);
}

#[test]
fn quit_event() {
  let (mut ui, mut r) = Ui::new(&mut |id, _cap| {
    Box::new(TestRootWidget::new(id))
  });
  let mut c1 = ui.add_widget(&mut r, &mut |parent, id, _cap| {
    Box::new(TestContainer::new(parent, id))
  });
  let _ = ui.add_widget(&mut c1, &mut |parent, id, _cap| {
    Box::new(TestWidget::new(parent, id))
  });

  let result = ui.handle(UiEvent::Quit);
  assert!(compare_ui_events(&result.unwrap(), &UiEvent::Quit));
}

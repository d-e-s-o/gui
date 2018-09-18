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

use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use gui::Cap;
use gui::ChainEvent;
use gui::Event;
use gui::EventChain;
use gui::Id;
use gui::Key;
use gui::MetaEvent;
use gui::OptionChain;
use gui::Ui;
use gui::UiEvent;
use gui::Widget;

use common::clone_meta_event;
use common::clone_ui_event;
use common::compare_meta_events;
use common::compare_ui_events;
use common::TestWidget;
use common::TestWidgetBuilder;
use common::unwrap_custom;


#[test]
fn convert_event_into() {
  let event = Event::KeyDown(Key::Char(' '));
  let orig_event = event;
  let ui_event = UiEvent::from(event);

  assert!(compare_ui_events(&ui_event, &UiEvent::Event(orig_event)));
}

#[test]
fn chain_meta_event() {
  let event1 = UiEvent::Event(Event::KeyUp(Key::Char('a')));
  let event2 = UiEvent::Quit.into();
  let orig_event1 = clone_ui_event(&event1);
  let orig_event2 = clone_meta_event(&event2);
  let event = event1.chain(event2);
  let expected = ChainEvent::Chain(
    orig_event1,
    Box::new(orig_event2),
  );

  assert!(compare_meta_events(&event, &expected));
}

#[test]
fn chain_meta_event_chain() {
  let event1 = Event::KeyUp(Key::Char('a')).into();
  let orig_event1 = clone_ui_event(&event1);
  let event2 = Event::KeyUp(Key::Char('z')).into();
  let orig_event2 = clone_ui_event(&event2);
  let event3 = UiEvent::Quit.into();
  let orig_event3 = clone_meta_event(&event3);
  let event_chain = ChainEvent::Chain(event1, Box::new(event2.into()));
  let event = event_chain.chain(event3);
  let expected = ChainEvent::Chain(
    orig_event1,
    Box::new(
      ChainEvent::Chain(orig_event2, Box::new(orig_event3))
    ),
  );

  assert!(compare_meta_events(&event, &expected));
}

#[test]
fn event_and_option_chain() {
  let event = Event::KeyDown(Key::Esc).into();
  let orig_event = clone_ui_event(&event).into();
  let result = event.chain_opt(None as Option<Event>);

  assert!(compare_meta_events(&result, &orig_event));

  let event1 = Event::KeyDown(Key::Right).into();
  let orig_event1 = clone_ui_event(&event1).into();
  let event2 = UiEvent::Quit.into();
  let orig_event2 = clone_ui_event(&event2).into();

  let result = event1.chain_opt(Some(event2));
  let expected = ChainEvent::Chain(orig_event1, Box::new(orig_event2));

  assert!(compare_meta_events(&result, &expected));
}

#[test]
fn option_and_option_chain() {
  let result = (None as Option<Event>).chain(None as Option<Event>);
  assert!(result.is_none());

  let event = Event::KeyDown(Key::PageDown).into();
  let orig_event = clone_ui_event(&event).into();
  let result = (None as Option<Event>).chain(Some(event));

  assert!(compare_meta_events(&result.unwrap(), &orig_event));

  let event = Event::KeyDown(Key::Char('u')).into();
  let orig_event = clone_ui_event(&event).into();
  let result = Some(event).chain(None as Option<Event>);

  assert!(compare_meta_events(&result.unwrap(), &orig_event));

  let event1 = Event::KeyDown(Key::End).into();
  let orig_event1 = clone_ui_event(&event1).into();
  let event2 = Event::KeyUp(Key::Char('u')).into();
  let orig_event2 = clone_ui_event(&event2).into();

  let result = Some(event1).chain(Some(event2));
  let expected = ChainEvent::Chain(orig_event1, Box::new(orig_event2));

  assert!(compare_meta_events(&result.unwrap(), &expected));
}

#[test]
fn last_event_in_chain() {
  let event1 = Event::KeyUp(Key::Char('a')).into();
  let event2 = Event::KeyUp(Key::Char('z')).into();
  let orig_event2 = clone_ui_event(&event2);
  let event_chain = ChainEvent::Chain(event1, Box::new(event2.into()));

  assert!(compare_ui_events(&event_chain.into_last(), &orig_event2));
}

#[test]
fn events_bubble_up_when_unhandled() {
  let (mut ui, r) = Ui::new(&mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let c1 = ui.add_widget(r, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let c2 = ui.add_widget(c1, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w1 = ui.add_widget(c2, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });

  let event = Event::KeyUp(Key::Char(' '));
  ui.focus(w1);

  let result = ui.handle(event);
  // An unhandled event should just be returned after every widget
  // forwarded it.
  assert!(compare_meta_events(&result.unwrap(), &event.into()));
}

#[test]
fn targeted_event_returned_on_no_focus() {
  let (mut ui, r) = Ui::new(&mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w = ui.add_widget(r, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });

  let event = Event::KeyUp(Key::Char('y'));
  ui.focus(w);
  ui.hide(w);

  let result = ui.handle(event);
  assert!(compare_meta_events(&result.unwrap(), &event.into()));
}

fn key_handler(event: Event, cap: &mut Cap, to_focus: Option<Id>) -> Option<MetaEvent> {
  match event {
    Event::KeyDown(key) => {
      match key {
        Key::Char('a') => {
          if let Some(id) = to_focus {
            cap.focus(id);
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
  let (mut ui, r) = Ui::new(&mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w1 = ui.add_widget(r, &mut |id, _cap| {
    let widget = TestWidgetBuilder::new()
      .event_handler(|_s, e, c| key_handler(e, c, None))
      .build(id);
    Box::new(widget)
  });
  let w2 = ui.add_widget(r, &mut |id, _cap| {
    let widget = TestWidgetBuilder::new()
      .event_handler(move |_s, e, c| key_handler(e, c, Some(w1)))
      .build(id);
    Box::new(widget)
  });

  ui.focus(w2);
  assert!(ui.is_focused(w2));

  // Send a key down event, received by `w2`, which it will
  // translate into a focus event for `w1`.
  let event = Event::KeyDown(Key::Char('a'));
  ui.handle(event);

  assert!(ui.is_focused(w1));
}

fn custom_undirected_response_handler(_: Id, event: Box<Any>, _cap: &mut Cap) -> Option<MetaEvent> {
  let value = *event.downcast::<u64>().unwrap();
  Some(UiEvent::Custom(Box::new(value + 1)).into())
}

#[test]
fn custom_undirected_response_event() {
  let (mut ui, r) = Ui::new(&mut |id, _cap| {
    let widget = TestWidgetBuilder::new()
      .custom_handler(custom_undirected_response_handler)
      .build(id);
    Box::new(widget)
  });
  let c1 = ui.add_widget(r, &mut |id, _cap| {
    let widget = TestWidgetBuilder::new()
      .custom_handler(custom_undirected_response_handler)
      .build(id);
    Box::new(widget)
  });
  let w1 = ui.add_widget(c1, &mut |id, _cap| {
    let widget = TestWidgetBuilder::new()
      .custom_handler(custom_undirected_response_handler)
      .build(id);
    Box::new(widget)
  });

  // We focus the widget we just created, which means that the event
  // will travel through the widget and all its parents.
  ui.focus(w1);

  let event = UiEvent::Custom(Box::new(42u64));
  let result = ui.handle(event).unwrap();
  // We expect three increments, one from each of the widgets.
  assert_eq!(*unwrap_custom::<u64>(result), 45);
}

fn custom_directed_response_handler(_: Id, event: Box<Any>, _cap: &mut Cap) -> Option<MetaEvent> {
  let cell = *event.downcast::<Rc<RefCell<u64>>>().unwrap();
  let value = *cell.borrow();
  cell.replace(value + 1);
  None
}

#[test]
fn custom_directed_response_event() {
  let (mut ui, r) = Ui::new(&mut |id, _cap| {
    let widget = TestWidgetBuilder::new()
      .custom_handler(custom_directed_response_handler)
      .build(id);
    Box::new(widget)
  });
  let c1 = ui.add_widget(r, &mut |id, _cap| {
    let widget = TestWidgetBuilder::new()
      .custom_handler(custom_directed_response_handler)
      .build(id);
    Box::new(widget)
  });
  let w1 = ui.add_widget(c1, &mut |id, _cap| {
    let widget = TestWidgetBuilder::new()
      .custom_handler(custom_directed_response_handler)
      .build(id);
    Box::new(widget)
  });

  ui.focus(w1);

  let cell = RefCell::new(1337u64);
  let rc = Rc::new(cell);
  let event = UiEvent::Custom(Box::new(Rc::clone(&rc)));

  let result = ui.handle(event);
  assert!(result.is_none());

  let value = *(*rc).borrow();
  assert_eq!(value, 1338);
}

#[test]
fn direct_custom_event() {
  let (mut ui, r) = Ui::new(&mut |id, _cap| {
    let widget = TestWidgetBuilder::new()
      .custom_handler(custom_undirected_response_handler)
      .build(id);
    Box::new(widget)
  });
  let c1 = ui.add_widget(r, &mut |id, _cap| {
    let widget = TestWidgetBuilder::new()
      .custom_handler(custom_undirected_response_handler)
      .build(id);
    Box::new(widget)
  });
  let w1 = ui.add_widget(c1, &mut |id, _cap| {
    let widget = TestWidgetBuilder::new()
      .custom_handler(custom_undirected_response_handler)
      .build(id);
    Box::new(widget)
  });

  ui.focus(c1);

  let event = UiEvent::Directed(w1, Box::new(10u64));
  let result = ui.handle(event).unwrap();
  assert_eq!(*unwrap_custom::<u64>(result), 13);
}

#[test]
fn quit_event() {
  let (mut ui, r) = Ui::new(&mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let c1 = ui.add_widget(r, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let _ = ui.add_widget(c1, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });

  let result = ui.handle(UiEvent::Quit);
  assert!(compare_meta_events(&result.unwrap(), &UiEvent::Quit.into()));
}


static mut ACCUMULATOR: u64 = 0;

fn accumulating_handler(_widget: Id, event: Box<Any>, _cap: &mut Cap) -> Option<MetaEvent> {
  let value = *event.downcast::<u64>().unwrap();

  unsafe {
    ACCUMULATOR += value;
    Some(UiEvent::Custom(Box::new(ACCUMULATOR)).into())
  }
}

fn chaining_handler(_widget: Id, event: Box<Any>, _cap: &mut Cap) -> Option<MetaEvent> {
  let value = event.downcast::<u64>().unwrap();
  let event1 = UiEvent::Custom(Box::new(*value));
  let event2 = UiEvent::Custom(Box::new(*value + 1));
  Some(event1.chain(event2))
}

#[test]
fn chain_event_dispatch() {
  let (mut ui, r) = Ui::new(&mut |id, _cap| {
    let widget = TestWidgetBuilder::new()
      .custom_handler(accumulating_handler)
      .build(id);
    Box::new(widget)
  });
  let c1 = ui.add_widget(r, &mut |id, _cap| {
    let widget = TestWidgetBuilder::new()
      .custom_handler(chaining_handler)
      .build(id);
    Box::new(widget)
  });
  let w1 = ui.add_widget(c1, &mut |id, _cap| {
    let widget = TestWidgetBuilder::new()
      .custom_handler(chaining_handler)
      .build(id);
    Box::new(widget)
  });

  ui.focus(w1);

  let event = UiEvent::Custom(Box::new(1u64));
  let result = ui.handle(event).unwrap().into_last();
  assert_eq!(*unwrap_custom::<u64>(result.into()), 8);
}

static mut HOOK_COUNT: u64 = 0;

fn count_event_hook(_widget: &mut Widget, _event: Event, _cap: &Cap) -> Option<MetaEvent> {
  unsafe {
    HOOK_COUNT += 1;
  }
  None
}

#[test]
fn hook_events_return_value() {
  let (mut ui, r) = Ui::new(&mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w = ui.add_widget(r, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });

  assert!(ui.hook_events(w, None).is_none());
  assert!(ui.hook_events(r, None).is_none());
  assert!(ui.hook_events(w, Some(&count_event_hook)).is_none());
  assert!(ui.hook_events(r, None).is_none());
  assert!(ui.hook_events(r, Some(&count_event_hook)).is_none());
  assert!(ui.hook_events(w, Some(&count_event_hook)).is_some());
  assert!(ui.hook_events(r, Some(&count_event_hook)).is_some());
  assert!(ui.hook_events(w, None).is_some());
}

#[test]
fn hook_events_handler() {
  let (mut ui, r) = Ui::new(&mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let c1 = ui.add_widget(r, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w1 = ui.add_widget(c1, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });

  ui.focus(w1);
  ui.hook_events(c1, Some(&count_event_hook));

  assert_eq!(unsafe { HOOK_COUNT }, 0);

  let event = Event::KeyDown(Key::Char(' '));
  ui.handle(event).unwrap();

  assert_eq!(unsafe { HOOK_COUNT }, 1);

  ui.hook_events(c1, None);

  let event = Event::KeyDown(Key::Char(' '));
  ui.handle(event).unwrap();

  assert_eq!(unsafe { HOOK_COUNT }, 1);
}


fn quit_event_hook(_widget: &mut Widget, _event: Event, _cap: &Cap) -> Option<MetaEvent> {
  Some(UiEvent::Quit.into())
}

fn swallowing_event_handler(_widget: Id, _event: Event, _cap: &mut Cap) -> Option<MetaEvent> {
  None
}


#[test]
fn hook_events_with_return() {
  let (mut ui, r) = Ui::new(&mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w = ui.add_widget(r, &mut |id, _cap| {
    let widget = TestWidgetBuilder::new()
      .event_handler(swallowing_event_handler)
      .build(id);
    Box::new(widget)
  });

  ui.focus(w);
  ui.hook_events(w, Some(&quit_event_hook));

  let event = Event::KeyDown(Key::Char(' '));
  let result = ui.handle(event);

  assert!(compare_meta_events(&result.unwrap(), &UiEvent::Quit.into()));
}


fn returned_event_handler(_widget: Id, event: Box<Any>, _cap: &mut Cap) -> Option<MetaEvent> {
  let value = *event.downcast::<u64>().unwrap();
  Some(UiEvent::Custom(Box::new(value + 1)).into())
}

fn returnable_event_handler(_widget: Id, event: &mut Any, _cap: &mut Cap) -> Option<MetaEvent> {
  match event.downcast_mut::<u64>() {
    Some(value) => *value *= 2,
    None => panic!("encountered unexpected custom event"),
  };
  None
}


#[test]
fn custom_returnable_events() {
  let (mut ui, r) = Ui::new(&mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w1 = ui.add_widget(r, &mut |id, _cap| {
    let widget = TestWidgetBuilder::new()
      .custom_handler(returned_event_handler)
      .build(id);
    Box::new(widget)
  });
  let w2 = ui.add_widget(r, &mut |id, _cap| {
    let widget = TestWidgetBuilder::new()
      .custom_ref_handler(returnable_event_handler)
      .build(id);
    Box::new(widget)
  });

  let src = w1;
  let dst = w2;
  let event = UiEvent::Returnable(src, dst, Box::new(10u64));
  let result = ui.handle(event).unwrap();

  assert_eq!(*unwrap_custom::<u64>(result), 21);
}

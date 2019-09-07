// test_events.rs

// *************************************************************************
// * Copyright (C) 2018-2019 Daniel Mueller (deso@posteo.net)              *
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

use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use gui::Cap;
use gui::ChainEvent;
use gui::EventChain;
use gui::Id;
use gui::MutCap;
use gui::OptionChain;
use gui::Ui;
use gui::UiEvent;
use gui::UnhandledEvent;
use gui::UnhandledEvents;
use gui::Widget;

use crate::common::Event;
use crate::common::TestWidget;
use crate::common::TestWidgetBuilder;
use crate::common::UiEvents;
use crate::common::unwrap_custom;


fn compare_ui_event<E>(event1: &UiEvent<E>, event2: &UiEvent<E>) -> bool
where
  E: PartialEq,
{
  match *event1 {
    UiEvent::Event(ref event1) => {
      match *event2 {
        UiEvent::Event(ref event2) => event1 == event2,
        _ => false,
      }
    },
    UiEvent::Quit => {
      match *event2 {
        UiEvent::Quit => true,
        _ => false,
      }
    },
    UiEvent::Custom(_) |
    UiEvent::Directed(_, _) |
    UiEvent::Returnable(_, _, _) => panic!("Cannot compare custom events"),
  }
}

fn compare_ui_events(event1: &UiEvents, event2: &UiEvents) -> bool {
  match *event1 {
    ChainEvent::Event(ref event1) => {
      match *event2 {
        ChainEvent::Event(ref event2) => compare_ui_event(event1, event2),
        _ => false,
      }
    },
    ChainEvent::Chain(ref event1, ref chain1) => {
      match *event2 {
        ChainEvent::Chain(ref event2, ref chain2) => {
          compare_ui_event(event1, event2) &&
          compare_ui_events(chain1, chain2)
        },
        _ => false,
      }
    },
  }
}

fn compare_unhandled<E>(event1: &UnhandledEvent<E>, event2: &UnhandledEvent<E>) -> bool
where
  E: PartialEq,
{
  match *event1 {
    UnhandledEvent::Event(ref event1) => {
      match *event2 {
        UnhandledEvent::Event(ref event2) => event1 == event2,
        _ => false,
      }
    },
    UnhandledEvent::Quit => {
      match *event2 {
        UnhandledEvent::Quit => true,
        _ => false,
      }
    },
    UnhandledEvent::Custom(_) => panic!("Cannot compare custom events"),
  }
}

fn compare_unhandled_events<E>(events1: &UnhandledEvents<E>, events2: &UnhandledEvents<E>) -> bool
where
  E: PartialEq,
{
  match *events1 {
    ChainEvent::Event(ref event1) => {
      match *events2 {
        ChainEvent::Event(ref event2) => compare_unhandled(event1, event2),
        _ => false,
      }
    },
    ChainEvent::Chain(ref event1, ref unhandled1) => {
      match *events2 {
        ChainEvent::Chain(ref event2, ref unhandled2) => {
          compare_unhandled(event1, event2) &&
          compare_unhandled_events(unhandled1, unhandled2)
        },
        _ => false,
      }
    },
  }
}


#[test]
fn convert_event_into() {
  let event = Event::Key(' ');
  let orig_event = event;
  let ui_event = UiEvent::from(event);

  assert!(compare_ui_event(&ui_event, &UiEvent::Event(orig_event)));
}

#[test]
fn chain_event() {
  let event1 = Event::Key('a');
  let orig_event1 = event1;
  let event2 = UiEvent::Quit;
  let orig_event2 = UiEvent::Quit;

  let event = event1.chain(event2);
  let expected = ChainEvent::Chain(
    orig_event1.into(),
    Box::new(orig_event2.into()),
  );

  assert!(compare_ui_events(&event, &expected));
}

#[test]
fn chain_event_chain() {
  let event1 = Event::Key('a');
  let orig_event1 = event1;
  let event2 = Event::Key('z');
  let orig_event2 = event2;
  let event3 = UiEvent::Quit;
  let orig_event3 = UiEvent::Quit;

  let event_chain = ChainEvent::Chain(event1.into(), Box::new(event2.into()));
  let event = event_chain.chain(event3);
  let expected = ChainEvent::Chain(
    orig_event1.into(),
    Box::new(
      ChainEvent::Chain(
        orig_event2.into(),
        Box::new(orig_event3.into())
      )
    ),
  );

  assert!(compare_ui_events(&event, &expected));
}

#[test]
fn event_and_option_chain() {
  let event = Event::Empty;
  let orig_event = event;
  let result = event.chain_opt(None as Option<Event>);

  assert!(compare_ui_events(&result, &orig_event.into()));

  let event1 = Event::Key('%');
  let orig_event1 = event1;
  let event2 = UiEvent::Quit;
  let orig_event2 = UiEvent::Quit;

  let result = event1.chain_opt(Some(event2));
  let expected = ChainEvent::Chain(
    orig_event1.into(),
    Box::new(orig_event2.into())
  );

  assert!(compare_ui_events(&result, &expected));
}

#[test]
fn option_and_option_chain() {
  let result = OptionChain::<_, UiEvent<Event>>::chain(
    None as Option<Event>,
    None as Option<Event>,
  );
  assert!(result.is_none());

  let event = Event::Key('1');
  let orig_event = event;
  let result = OptionChain::chain(None as Option<Event>, Some(event));

  assert!(compare_ui_events(&result.unwrap(), &orig_event.into()));

  let event = Event::Key('u');
  let orig_event = event;
  let result = OptionChain::chain(Some(event), None as Option<Event>);

  assert!(compare_ui_events(&result.unwrap(), &orig_event.into()));

  let event = Event::Key('2');
  let orig_event = event;
  let result = (None as Option<Event>).opt_chain(event);

  assert!(compare_ui_events(&result, &orig_event.into()));

  let event1 = Event::Key('z');
  let orig_event1 = event1;
  let event2 = Event::Key('u');
  let orig_event2 = event2;

  let result = OptionChain::chain(Some(event1),Some(event2));
  let expected = ChainEvent::Chain(
    orig_event1.into(),
    Box::new(orig_event2.into())
  );

  assert!(compare_ui_events(&result.unwrap(), &expected));
}

#[test]
fn last_event_in_chain() {
  let event1 = Event::Key('a');
  let event2 = Event::Key('z');
  let orig_event2 = event2;

  let event_chain = ChainEvent::Chain(
    event1.into(),
    Box::new(event2.into())
  );

  assert!(compare_ui_event(&event_chain.into_last(), &orig_event2.into()));
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

  let event = Event::Key(' ');
  ui.focus(w1);

  let result = ui.handle(event);
  let expected = UnhandledEvent::Event(event).into();
  // An unhandled event should just be returned after every widget
  // forwarded it.
  assert!(compare_unhandled_events(&result.unwrap(), &expected));
}

#[test]
fn targeted_event_returned_on_no_focus() {
  let (mut ui, r) = Ui::new(&mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w = ui.add_widget(r, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });

  let event = Event::Key('y');
  ui.focus(w);
  ui.hide(w);

  let result = ui.handle(event);
  let expected = UnhandledEvent::Event(event).into();
  assert!(compare_unhandled_events(&result.unwrap(), &expected));
}

fn key_handler(event: Event,
               cap: &mut dyn MutCap<Event>,
               to_focus: Option<Id>) -> Option<UiEvents> {
  match event {
    Event::Key(key) if key == 'a' => {
      if let Some(id) = to_focus {
        cap.focus(id);
        None
      } else {
        Some(event.into())
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
  let event = Event::Key('a');
  ui.handle(event);

  assert!(ui.is_focused(w1));
}

fn custom_undirected_response_handler(_: Id,
                                      event: Box<dyn Any>,
                                      _cap: &mut dyn MutCap<Event>) -> Option<UiEvents> {
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
  assert_eq!(*unwrap_custom::<_, u64>(result), 45);
}

fn custom_directed_response_handler(_: Id,
                                    event: Box<dyn Any>,
                                    _cap: &mut dyn MutCap<Event>) -> Option<UiEvents> {
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
  assert_eq!(*unwrap_custom::<_, u64>(result), 13);
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
  let expected = UnhandledEvent::Quit.into();
  assert!(compare_unhandled_events(&result.unwrap(), &expected));
}


static mut ACCUMULATOR: u64 = 0;

fn accumulating_handler(_: Id,
                        event: Box<dyn Any>,
                        _cap: &mut dyn MutCap<Event>) -> Option<UiEvents> {
  let value = *event.downcast::<u64>().unwrap();

  unsafe {
    ACCUMULATOR += value;
    Some(UiEvent::Custom(Box::new(ACCUMULATOR)).into())
  }
}

fn chaining_handler(_: Id, event: Box<dyn Any>, _cap: &mut dyn MutCap<Event>) -> Option<UiEvents> {
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
  assert_eq!(*unwrap_custom::<Event, u64>(result.into()), 8);
}

static mut HOOK_COUNT: u64 = 0;

#[allow(clippy::trivially_copy_pass_by_ref)]
fn count_event_hook(_: &mut dyn Widget<Event>,
                    _event: &Event,
                    _cap: &dyn Cap) -> Option<UiEvents> {
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

  let event = Event::Key(' ');
  ui.handle(event).unwrap();

  assert_eq!(unsafe { HOOK_COUNT }, 1);

  ui.hook_events(c1, None);

  let event = Event::Key(' ');
  ui.handle(event).unwrap();

  assert_eq!(unsafe { HOOK_COUNT }, 1);
}


#[allow(clippy::trivially_copy_pass_by_ref)]
fn quit_event_hook(_: &mut dyn Widget<Event>, _event: &Event, _cap: &dyn Cap) -> Option<UiEvents> {
  Some(UiEvent::Quit.into())
}

fn swallowing_event_handler(_: Id,
                            _event: Event,
                            _cap: &mut dyn MutCap<Event>) -> Option<UiEvents> {
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

  let event = Event::Key(' ');
  let result = ui.handle(event);
  let expected = UnhandledEvent::Quit.into();

  assert!(compare_unhandled_events(&result.unwrap(), &expected));
}


#[allow(clippy::trivially_copy_pass_by_ref)]
fn emitting_event_hook(_: &mut dyn Widget<Event>,
                       event: &Event,
                       _cap: &dyn Cap) -> Option<UiEvents> {
  assert_eq!(*event, Event::Key('y'));

  Some(Event::Key('z').into())
}

static mut FIRST: bool = true;

fn checking_event_handler(_: Id, event: Event, _cap: &mut dyn MutCap<Event>) -> Option<UiEvents> {
  unsafe {
    if FIRST {
      assert_eq!(event, Event::Key('y'));
    } else {
      assert_eq!(event, Event::Key('z'));
    };

    FIRST = false;
  }

  None
}

#[test]
fn hook_emitted_event_order() {
  let (mut ui, r) = Ui::new(&mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w = ui.add_widget(r, &mut |id, _cap| {
    let widget = TestWidgetBuilder::new()
      .event_handler(checking_event_handler)
      .build(id);
    Box::new(widget)
  });

  ui.focus(w);
  ui.hook_events(w, Some(&emitting_event_hook));

  let event = Event::Key('y');
  let result = ui.handle(event);
  assert!(result.is_none())
}


fn returned_event_handler(_: Id,
                          event: Box<dyn Any>,
                          _cap: &mut dyn MutCap<Event>) -> Option<UiEvents> {
  let value = *event.downcast::<u64>().unwrap();
  Some(UiEvent::Custom(Box::new(value + 1)).into())
}

fn returnable_event_handler(_: Id,
                            event: &mut dyn Any,
                            _cap: &mut dyn MutCap<Event>) -> Option<UiEvents> {
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

  assert_eq!(*unwrap_custom::<_, u64>(result), 21);
}

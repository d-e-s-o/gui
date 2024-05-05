// test_events.rs

// *************************************************************************
// * Copyright (C) 2018-2024 Daniel Mueller (deso@posteo.net)              *
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

use std::future::Future;
use std::pin::Pin;

use gui::Cap;
use gui::Id;
use gui::MutCap;
use gui::Ui;
use gui::Widget;

use crate::common::Event;
use crate::common::Message;
use crate::common::TestWidget;
use crate::common::TestWidgetDataBuilder;

#[tokio::test]
async fn events_bubble_up_when_unhandled() {
  let new_data = || TestWidgetDataBuilder::new().build();
  let (mut ui, r) = Ui::new(new_data, |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let c1 = ui.add_ui_widget(r, new_data, |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let c2 = ui.add_ui_widget(c1, new_data, |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w1 = ui.add_ui_widget(c2, new_data, |id, _cap| {
    Box::new(TestWidget::new(id))
  });

  let event = Event::Key(' ');
  ui.focus(w1);

  let result = ui.handle(event).await;
  // An unhandled event should just be returned after every widget
  // forwarded it.
  assert_eq!(result.unwrap(), event);
}

#[tokio::test]
async fn targeted_event_returned_on_no_focus() {
  let new_data = || TestWidgetDataBuilder::new().build();
  let (mut ui, r) = Ui::new(new_data, |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w = ui.add_ui_widget(r, new_data, |id, _cap| {
    Box::new(TestWidget::new(id))
  });

  let event = Event::Key('y');
  ui.focus(w);
  ui.hide(w);

  let result = ui.handle(event).await;
  assert_eq!(result.unwrap(), event);
}

fn key_handler(
  cap: &mut dyn MutCap<Event, Message>,
  event: Event,
  to_focus: Option<Id>,
) -> Option<Event> {
  match event {
    Event::Key('a') => {
      if let Some(id) = to_focus {
        cap.focus(id);
        None
      } else {
        Some(event)
      }
    },
    _ => Some(event),
  }
}

#[tokio::test]
async fn event_handling_with_focus() {
  let (mut ui, r) = Ui::new(
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let w1 = ui.add_ui_widget(
    r,
    || {
      TestWidgetDataBuilder::new()
        .event_handler(|_s, c, e| key_handler(c, e, None))
        .build()
    },
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let w2 = ui.add_ui_widget(
    r,
    || {
      TestWidgetDataBuilder::new()
        .event_handler(move |_s, c, e| key_handler(c, e, Some(w1)))
        .build()
    },
    |id, _cap| Box::new(TestWidget::new(id)),
  );

  ui.focus(w2);
  assert!(ui.is_focused(w2));

  // Send a key down event, received by `w2`, which it will
  // translate into a focus event for `w1`.
  let event = Event::Key('a');
  ui.handle(event).await;

  assert!(ui.is_focused(w1));
}

fn incrementing_event_handler(
  _: Id,
  _cap: &mut dyn MutCap<Event, Message>,
  event: Event,
) -> Option<Event> {
  let event = match event {
    Event::Empty | Event::Key(..) => unreachable!(),
    Event::Int(value) => Event::Int(value + 1),
  };
  Some(event)
}

/// Check that events are propagated as expected.
#[tokio::test]
async fn event_propagation() {
  let (mut ui, r) = Ui::new(
    || {
      TestWidgetDataBuilder::new()
        .event_handler(incrementing_event_handler)
        .build()
    },
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let c1 = ui.add_ui_widget(
    r,
    || {
      TestWidgetDataBuilder::new()
        .event_handler(incrementing_event_handler)
        .build()
    },
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let w1 = ui.add_ui_widget(
    c1,
    || {
      TestWidgetDataBuilder::new()
        .event_handler(incrementing_event_handler)
        .build()
    },
    |id, _cap| Box::new(TestWidget::new(id)),
  );

  // We focus the widget we just created, which means that the event
  // will travel through the widget and all its parents.
  ui.focus(w1);

  let event = Event::Int(42);
  let result = ui.handle(event).await.unwrap();
  // We expect three increments, one from each of the widgets.
  assert_eq!(result.unwrap_int(), 45);
}

static mut HOOK_COUNT: u64 = 0;

fn count_event_hook<'f>(
  widget: &'f dyn Widget<Event, Message>,
  _cap: &'f mut dyn MutCap<Event, Message>,
  event: Option<&'f Event>,
) -> Pin<Box<dyn Future<Output = Option<Event>> + 'f>> {
  Box::pin(async move {
    assert!(widget.downcast_ref::<TestWidget>().is_some());
    let is_pre_hook = event.is_some();
    // For each event we should always increment our count twice: once for
    // the pre-hook and once for the post-hook. That means that when the
    // count is an integer multiple of two, we should be in a pre-hook.
    assert!((unsafe { HOOK_COUNT } % 2 == 0) == is_pre_hook);

    unsafe {
      HOOK_COUNT += 1;
    }
    None
  })
}

/// Check that `MutCap::hook_events` behaves as expected.
#[test]
fn hook_events_return_value() {
  let (mut ui, r) = Ui::new(
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let w = ui.add_ui_widget(
    r,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );

  assert!(ui.hook_events(w, None).is_none());
  assert!(ui.hook_events(r, None).is_none());
  assert!(ui.hook_events(w, Some(&count_event_hook)).is_none());
  assert!(ui.hook_events(r, None).is_none());
  assert!(ui.hook_events(r, Some(&count_event_hook)).is_none());
  assert!(ui.hook_events(w, Some(&count_event_hook)).is_some());
  assert!(ui.hook_events(r, Some(&count_event_hook)).is_some());
  assert!(ui.hook_events(w, None).is_some());
}

#[tokio::test]
async fn hook_events_handler() {
  let (mut ui, r) = Ui::new(
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let c1 = ui.add_ui_widget(
    r,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let w1 = ui.add_ui_widget(
    c1,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );

  ui.focus(w1);
  ui.hook_events(c1, Some(&count_event_hook));

  assert_eq!(unsafe { HOOK_COUNT }, 0);

  let event = Event::Key(' ');
  ui.handle(event).await.unwrap();

  assert_eq!(unsafe { HOOK_COUNT }, 2);

  ui.hook_events(c1, None);

  let event = Event::Key(' ');
  ui.handle(event).await.unwrap();

  assert_eq!(unsafe { HOOK_COUNT }, 2);
}


fn emitting_event_hook<'f>(
  _: &'f dyn Widget<Event, Message>,
  _cap: &'f mut dyn MutCap<Event, Message>,
  event: Option<&'f Event>,
) -> Pin<Box<dyn Future<Output = Option<Event>> + 'f>> {
  Box::pin(async move {
    if let Some(event) = event {
      assert_eq!(event, &Event::Key('y'));
      Some(Event::Key('z'))
    } else {
      None
    }
  })
}

fn checking_event_handler(
  _: Id,
  _cap: &mut dyn MutCap<Event, Message>,
  event: Event,
) -> Option<Event> {
  assert_eq!(event, Event::Key('y'));
  None
}

/// Test that hook emitted events are not seen by widgets.
#[tokio::test]
async fn hook_emitted_events() {
  let (mut ui, r) = Ui::new(
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let w = ui.add_ui_widget(
    r,
    || {
      TestWidgetDataBuilder::new()
        .event_handler(checking_event_handler)
        .build()
    },
    |id, _cap| Box::new(TestWidget::new(id)),
  );

  ui.focus(w);
  ui.hook_events(w, Some(&emitting_event_hook));

  let event = Event::Key('y');
  let result = ui.handle(event).await;
  assert_eq!(result.unwrap(), Event::Key('z'))
}

fn different_emitting_event_hook<'f>(
  _: &'f dyn Widget<Event, Message>,
  _cap: &'f mut dyn MutCap<Event, Message>,
  _event: Option<&'f Event>,
) -> Pin<Box<dyn Future<Output = Option<Event>> + 'f>> {
  Box::pin(async {
    Some(Event::Key('a'))
  })
}

/// Check that hook emitted events are attempted to be merged.
#[tokio::test]
#[should_panic(expected = "left: 'a'\n right: 'z'")]
async fn hook_event_merging() {
  let (mut ui, r) = Ui::new(
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let w1 = ui.add_ui_widget(
    r,
    || {
      TestWidgetDataBuilder::new()
        .event_handler(checking_event_handler)
        .build()
    },
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let w2 = ui.add_ui_widget(
    r,
    || {
      TestWidgetDataBuilder::new()
        .event_handler(checking_event_handler)
        .build()
    },
    |id, _cap| Box::new(TestWidget::new(id)),
  );

  // We register two event hooks that emit different events that are not
  // actually mergeable by our definition. So we expect a panic.
  ui.hook_events(w1, Some(&emitting_event_hook));
  ui.hook_events(w2, Some(&different_emitting_event_hook));

  let event = Event::Key('y');
  let _ = ui.handle(event).await;
}


fn send_message_hook<'f>(
  _: &'f dyn Widget<Event, Message>,
  cap: &'f mut dyn MutCap<Event, Message>,
  event: Option<&'f Event>,
) -> Pin<Box<dyn Future<Output = Option<Event>> + 'f>> {
  Box::pin(async move {
    if let Some(event) = event {
      let root = cap.root_id();
      cap.send(root, Message::new(event.unwrap_int() * 2)).await;
    }
    None
  })
}

static mut RECEIVED_VALUE: u64 = 42;

/// Check that we can send a message from a hook.
#[tokio::test]
async fn hook_can_send_message() {
  let (mut ui, r) = Ui::new(
    || {
      TestWidgetDataBuilder::new()
        .react_handler(|m, _| {
          unsafe {
            RECEIVED_VALUE = m.value;
          }
          None
        })
        .build()
    },
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let c1 = ui.add_ui_widget(
    r,
    || {
      TestWidgetDataBuilder::new()
        .event_handler(move |_, _, _| None)
        .build()
    },
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let w1 = ui.add_ui_widget(
    c1,
    || {
      TestWidgetDataBuilder::new()
        .event_handler(move |_, _, _| None)
        .build()
    },
    |id, _cap| Box::new(TestWidget::new(id)),
  );

  ui.focus(w1);
  assert_eq!(unsafe { RECEIVED_VALUE }, 42);

  // Without a hook nothing should happen.
  let result = ui.handle(Event::Int(3)).await;
  assert_eq!(result, None);
  assert_eq!(unsafe { RECEIVED_VALUE }, 42);

  ui.hook_events(c1, Some(&send_message_hook));

  let result = ui.handle(Event::Int(3)).await;
  assert_eq!(result, None);
  assert_eq!(unsafe { RECEIVED_VALUE }, 6);
}

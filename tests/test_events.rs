// Copyright (C) 2018-2025 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests for event handling/forwarding functionality.

mod common;

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use gui::Cap;
use gui::EventHookFn;
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
  let (mut ui, r) = Ui::new(new_data, |id, _cap| Box::new(TestWidget::new(id)));
  let c1 = ui.add_ui_widget(r, new_data, |id, _cap| Box::new(TestWidget::new(id)));
  let c2 = ui.add_ui_widget(c1, new_data, |id, _cap| Box::new(TestWidget::new(id)));
  let w1 = ui.add_ui_widget(c2, new_data, |id, _cap| Box::new(TestWidget::new(id)));

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
  let (mut ui, r) = Ui::new(new_data, |id, _cap| Box::new(TestWidget::new(id)));
  let w = ui.add_ui_widget(r, new_data, |id, _cap| Box::new(TestWidget::new(id)));

  let event = Event::Key('y');
  ui.focus(w);
  ui.hide(w);

  let result = ui.handle(event).await;
  assert_eq!(result.unwrap(), event);
}

fn key_handler<'f>(
  cap: &'f mut dyn MutCap<Event, Message>,
  event: Event,
  to_focus: Option<Id>,
) -> Pin<Box<dyn Future<Output = Option<Event>> + 'f>> {
  Box::pin(async move {
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
  })
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
  _widget: Id,
  _cap: &mut dyn MutCap<Event, Message>,
  event: Event,
) -> Pin<Box<dyn Future<Output = Option<Event>> + '_>> {
  Box::pin(async move {
    let event = match event {
      Event::Empty | Event::Key(..) => unreachable!(),
      Event::Int(value) => Event::Int(value + 1),
    };
    Some(event)
  })
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

/// Make sure that we can "rehandle" an event and check propagation.
#[tokio::test]
async fn rehandle_from_event_handler() {
  let (mut ui, r) = Ui::new(
    || {
      TestWidgetDataBuilder::new()
        .event_handler(move |_id, cap, event| {
          Box::pin(async move {
            match event {
              Event::Empty | Event::Key(..) => unreachable!(),
              Event::Int(value) if value < 5 => cap.rehandle(cap.focused().unwrap(), event).await,
              Event::Int(value) => Some(Event::Int(value + 1)),
            }
          })
        })
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

  let () = ui.focus(w1);

  let event = Event::Int(0);
  let result = ui.handle(event).await.unwrap();
  // We expect three increments from each of `w1` and `c1` and another
  // one from the root widget `r`.
  assert_eq!(result.unwrap_int(), 7);
}

/// Test that we can "rehandle" an event from a message handler.
#[tokio::test]
async fn rehandle_from_message_handler() {
  let (mut ui, r) = Ui::new(
    || {
      TestWidgetDataBuilder::new()
        .event_handler(move |widget, cap, event| {
          Box::pin(async move {
            match event {
              Event::Empty | Event::Key(..) => unreachable!(),
              Event::Int(value) if value < 5 => {
                // Send a message to ourselves, which will recursively
                // handle the event but with an increased value.
                let msg = cap.send(widget, Message::new(value)).await.unwrap();
                Some(Event::Int(msg.value))
              },
              Event::Int(..) => Some(event),
            }
          })
        })
        .react_handler(move |widget, msg, cap| {
          Box::pin(async move {
            let event = cap
              .rehandle(widget, Event::Int(msg.value + 1))
              .await
              .unwrap();
            Some(Message {
              value: event.unwrap_int(),
            })
          })
        })
        .build()
    },
    |id, _cap| Box::new(TestWidget::new(id)),
  );

  let () = ui.focus(r);

  let event = Event::Int(0);
  let result = ui.handle(event).await.unwrap();
  assert_eq!(result.unwrap_int(), 5);
}

/// Test that we can "rehandle" an event from an event hook.
#[tokio::test]
async fn rehandle_from_event_hook() {
  let (mut ui, r) = Ui::new(
    || {
      TestWidgetDataBuilder::new()
        .event_handler(move |_id, _cap, event| {
          Box::pin(async move {
            match event {
              Event::Empty => None,
              Event::Key(..) => unreachable!(),
              Event::Int(value) => Some(Event::Int(value + 1)),
            }
          })
        })
        .build()
    },
    |id, _cap| Box::new(TestWidget::new(id)),
  );

  let () = ui.focus(r);
  let _hook = ui.hook_events(
    r,
    Some(&|widget, cap, event| {
      Box::pin(async move {
        if let Some(event) = event {
          match event {
            Event::Empty => cap.rehandle(widget.id(), Event::Int(0)).await,
            Event::Key(..) | Event::Int(..) => unreachable!(),
          }
        } else {
          None
        }
      })
    }),
  );

  let event = Event::Empty;
  let result = ui.handle(event).await.unwrap();
  assert_eq!(result.unwrap_int(), 1);
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
  assert!(ui.hook_events(w, Some(&emitting_event_hook)).is_none());
  assert!(ui.hook_events(r, None).is_none());
  assert!(ui.hook_events(r, Some(&emitting_event_hook)).is_none());
  assert!(ui.hook_events(w, Some(&emitting_event_hook)).is_some());
  assert!(ui.hook_events(r, Some(&emitting_event_hook)).is_some());
  assert!(ui.hook_events(w, None).is_some());
}

#[tokio::test]
async fn hook_events_handler() {
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
) -> Pin<Box<dyn Future<Output = Option<Event>> + '_>> {
  Box::pin(async move {
    assert_eq!(event, Event::Key('y'));
    None
  })
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
  Box::pin(async { Some(Event::Key('a')) })
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


/// Check that we can send a message from a hook.
#[tokio::test]
async fn hook_can_send_message() {
  static mut RECEIVED_VALUE: u64 = 42;

  fn send_message_hook<'f>(
    _widget: &'f dyn Widget<Event, Message>,
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

  let (mut ui, r) = Ui::new(
    || {
      TestWidgetDataBuilder::new()
        .react_handler(|_id, msg, _cap| {
          Box::pin(async move {
            unsafe {
              RECEIVED_VALUE = msg.value;
            }
            None
          })
        })
        .build()
    },
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let c1 = ui.add_ui_widget(
    r,
    || {
      TestWidgetDataBuilder::new()
        .event_handler(move |_, _, _| Box::pin(async { None }))
        .build()
    },
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let w1 = ui.add_ui_widget(
    c1,
    || {
      TestWidgetDataBuilder::new()
        .event_handler(move |_, _, _| Box::pin(async { None }))
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


/// Check that hook installation from an event handler only affects
/// subsequent events, but won't trigger the post-event hook for the
/// currently handled event.
#[tokio::test]
async fn hook_installation_event_handling() {
  static mut COUNTING_HOOK_COUNT: u64 = 0;

  fn counting_event_hook<'f>(
    _widget: &'f dyn Widget<Event, Message>,
    _cap: &'f mut dyn MutCap<Event, Message>,
    _event: Option<&'f Event>,
  ) -> Pin<Box<dyn Future<Output = Option<Event>> + 'f>> {
    unsafe { COUNTING_HOOK_COUNT += 1 };
    Box::pin(async { None })
  }

  fn setup_ui(hook_fn: EventHookFn<Event, Message>) -> Ui<Event, Message> {
    static INSTALLED: AtomicBool = AtomicBool::new(false);

    let (mut ui, root_id) = Ui::new(
      || {
        TestWidgetDataBuilder::new()
          .event_handler(move |id, cap, _| {
            Box::pin(async move {
              if !INSTALLED.load(Ordering::Relaxed) {
                let _prev_hook = cap.hook_events(id, Some(hook_fn));
                INSTALLED.store(true, Ordering::Relaxed);
              }
              None
            })
          })
          .build()
      },
      |id, _cap| Box::new(TestWidget::new(id)),
    );

    let () = ui.focus(root_id);
    ui
  }

  let event = Event::Key(' ');
  let mut ui = setup_ui(&counting_event_hook);

  // The first event will install the hook, but the hook function should
  // not get called on the post-hook path.
  let _result = ui.handle(event).await;
  assert_eq!(unsafe { COUNTING_HOOK_COUNT }, 0);

  // The second event should be seen on the pre- and post-hook path.
  let _result = ui.handle(event).await;
  assert_eq!(unsafe { COUNTING_HOOK_COUNT }, 2);
}

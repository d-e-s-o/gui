// Copyright (C) 2018-2025 Daniel Mueller (deso@posteo.net)
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(
  clippy::cognitive_complexity,
  clippy::needless_pass_by_value,
  clippy::redundant_field_names,
)]

mod common;

use std::fmt::Write;

use async_trait::async_trait;

use gui::Cap;
use gui::derive::Handleable;
use gui::derive::Widget;
use gui::Handleable;
use gui::Id;
use gui::MutCap;
use gui::Ui;

use crate::common::Event;
use crate::common::Message;
use crate::common::TestWidget;
use crate::common::TestWidgetDataBuilder;


#[test]
fn correct_ids() {
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
  // And a container.
  let c1 = ui.add_ui_widget(
    root,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  // And a widget to the container.
  let w3 = ui.add_ui_widget(
    c1,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  // And another container for deeper nesting.
  let c2 = ui.add_ui_widget(
    c1,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  // And the last widget.
  let w4 = ui.add_ui_widget(
    c2,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );

  assert_eq!(ui.parent_id(root), None);
  assert_eq!(ui.parent_id(w1).unwrap(), root);
  assert_eq!(ui.parent_id(w2).unwrap(), root);
  assert_eq!(ui.parent_id(c1).unwrap(), root);
  assert_eq!(ui.parent_id(w3).unwrap(), c1);
  assert_eq!(ui.parent_id(w4).unwrap(), c2);
}

#[test]
fn debug_format() {
  let (ui, _) = Ui::new(
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );

  let mut string = String::new();
  write!(&mut string, "{:?}", ui).unwrap();

  #[cfg(debug_assertions)]
  {
    assert!(string.starts_with("Ui { "), "{}", string);
    assert!(string.ends_with(" }"));
  }
  #[cfg(not(debug_assertions))]
  {
    assert_eq!(string, "Ui");
  }
}

#[test]
fn creation_order_is_child_order() {
  let (mut ui, root) = Ui::new(
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let w1 = ui.add_ui_widget(
    root,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let w11 = ui.add_ui_widget(
    w1,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let w12 = ui.add_ui_widget(
    w1,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let w2 = ui.add_ui_widget(
    root,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let w21 = ui.add_ui_widget(
    w2,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );

  let mut it = ui.children(root);
  assert_eq!(*it.next().unwrap(), w1);
  assert_eq!(*it.next().unwrap(), w2);
  assert!(it.next().is_none());

  let mut it = ui.children(w1);
  assert_eq!(*it.next().unwrap(), w11);
  assert_eq!(*it.next().unwrap(), w12);
  assert!(it.next().is_none());

  let mut it = ui.children(w11);
  assert!(it.next().is_none());

  let mut it = ui.children(w2);
  assert_eq!(*it.next().unwrap(), w21);
  assert!(it.next().is_none());
}

/// Check that wrong `Id` usage is flagged when assertions are enabled.
#[tokio::test]
#[cfg(debug_assertions)]
#[should_panic(expected = "Created widget does not have provided Id")]
async fn incorrect_widget_id() {
  let (mut ui, root) = Ui::new(
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let _ = ui.add_ui_widget(
    root,
    || TestWidgetDataBuilder::new().build(),
    // Initialize the widget with the ID of the root widget.
    |_id, _cap| Box::new(TestWidget::new(root)),
  );
}

#[tokio::test]
#[cfg(debug_assertions)]
#[should_panic(expected = "The given Id belongs to a different Ui")]
async fn share_ids_between_ui_objects() {
  let (mut ui1, root) = Ui::new(
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let widget = ui1.add_ui_widget(
    root,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );

  let (mut ui2, _) = Ui::new(
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );

  // `widget` is registered to `ui1` and so using it in the context of
  // `ui2` is not as intended. On debug builds we have special detection
  // in place to provide a meaningful error, that should trigger here.
  let message = Message::new(0);
  ui2.send(widget, message).await;
}

#[test]
fn visibility_fun() {
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
  let w3 = ui.add_ui_widget(
    w2,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );

  assert!(ui.is_visible(root));
  assert!(ui.is_visible(w1));
  assert!(ui.is_visible(w2));
  assert!(ui.is_visible(w3));
  assert!(ui.is_displayed(root));
  assert!(ui.is_displayed(w1));
  assert!(ui.is_displayed(w2));
  assert!(ui.is_displayed(w3));

  ui.hide(root);
  assert!(!ui.is_visible(root));
  assert!(ui.is_visible(w1));
  assert!(ui.is_visible(w2));
  assert!(ui.is_visible(w3));
  assert!(!ui.is_displayed(root));
  assert!(!ui.is_displayed(w1));
  assert!(!ui.is_displayed(w2));
  assert!(!ui.is_displayed(w3));

  ui.hide(w2);
  assert!(!ui.is_visible(root));
  assert!(ui.is_visible(w1));
  assert!(!ui.is_visible(w2));
  assert!(ui.is_visible(w3));
  assert!(!ui.is_displayed(root));
  assert!(!ui.is_displayed(w1));
  assert!(!ui.is_displayed(w2));
  assert!(!ui.is_displayed(w3));

  ui.hide(w1);
  assert!(!ui.is_visible(root));
  assert!(!ui.is_visible(w1));
  assert!(!ui.is_visible(w2));
  assert!(ui.is_visible(w3));
  assert!(!ui.is_displayed(root));
  assert!(!ui.is_displayed(w1));
  assert!(!ui.is_displayed(w2));
  assert!(!ui.is_displayed(w3));

  ui.show(w3);
  assert!(ui.is_visible(root));
  assert!(!ui.is_visible(w1));
  assert!(ui.is_visible(w2));
  assert!(ui.is_visible(w3));
  assert!(ui.is_displayed(root));
  assert!(!ui.is_displayed(w1));
  assert!(ui.is_displayed(w2));
  assert!(ui.is_displayed(w3));
}

#[test]
fn no_initial_focus() {
  let (mut ui, root) = Ui::new(
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );

  // No widget has the input focus by default.
  assert!(ui.focused().is_none());

  let _ = ui.add_ui_widget(
    root,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  assert!(ui.focused().is_none());
}

#[test]
fn focus_widget() {
  let (mut ui, root) = Ui::new(
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let widget = ui.add_ui_widget(
    root,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );

  ui.focus(widget);
  assert!(ui.is_focused(widget));
}

#[test]
fn focus_makes_widget_visible() {
  let (mut ui, root) = Ui::new(
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let widget = ui.add_ui_widget(
    root,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );

  ui.hide(root);
  ui.hide(widget);

  assert!(!ui.is_visible(root));
  assert!(!ui.is_visible(widget));

  // Focusing the widget should make it and all its parents visible
  // again.
  ui.focus(widget);
  assert!(ui.is_displayed(widget));
}

#[test]
fn focus_changes_child_order() {
  let (mut ui, root) = Ui::new(
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let w1 = ui.add_ui_widget(
    root,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let w11 = ui.add_ui_widget(
    w1,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let w2 = ui.add_ui_widget(
    root,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let w21 = ui.add_ui_widget(
    w2,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let w22 = ui.add_ui_widget(
    w2,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let w23 = ui.add_ui_widget(
    w2,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );

  ui.focus(w1);
  {
    let mut it = ui.children(root);
    assert_eq!(*it.next().unwrap(), w1);
    assert_eq!(*it.next().unwrap(), w2);
    assert!(it.next().is_none());
  }

  ui.focus(w22);
  {
    let mut it = ui.children(root);
    assert_eq!(*it.next().unwrap(), w2);
    assert_eq!(*it.next().unwrap(), w1);
    assert!(it.next().is_none());

    let mut it = ui.children(w2);
    assert_eq!(*it.next().unwrap(), w22);
    assert_eq!(*it.next().unwrap(), w21);
    assert_eq!(*it.next().unwrap(), w23);
    assert!(it.next().is_none());
  }

  ui.focus(w11);
  {
    let mut it = ui.children(root);
    assert_eq!(*it.next().unwrap(), w1);
    assert_eq!(*it.next().unwrap(), w2);
    assert!(it.next().is_none());

    let mut it = ui.children(w1);
    assert_eq!(*it.next().unwrap(), w11);
    assert!(it.next().is_none());
  }
}

/// Check that hiding as well as showing a widget preserves its order in
/// the parent's array of children.
#[test]
fn hide_and_show_preserve_order() {
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
  let w3 = ui.add_ui_widget(
    root,
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );

  let before = ui.children(root).cloned().collect::<Vec<_>>();

  // By default all widgets are visible. Make sure that issuing a show
  // does not change the order of children. It should be a no-op.
  ui.show(w2);
  let after = ui.children(root).cloned().collect::<Vec<_>>();
  assert_eq!(before, after);

  ui.show(w3);
  let after = ui.children(root).cloned().collect::<Vec<_>>();
  assert_eq!(before, after);

  ui.hide(w2);
  let after = ui.children(root).cloned().collect::<Vec<_>>();
  assert_eq!(before, after);

  ui.hide(w2);
  let after = ui.children(root).cloned().collect::<Vec<_>>();
  assert_eq!(before, after);

  ui.hide(w1);
  let after = ui.children(root).cloned().collect::<Vec<_>>();
  assert_eq!(before, after);
}


fn counting_handler(
  _widget: Id,
  _cap: &mut dyn MutCap<Event, Message>,
  event: Event,
) -> Option<Event> {
  let event = match event {
    Event::Empty | Event::Key(..) => unreachable!(),
    Event::Int(value) => Event::Int(value + 1),
  };
  Some(event)
}


/// Check if we need to create another `CreatingWidget`.
fn need_more(id: Id, cap: &mut dyn MutCap<Event, Message>) -> bool {
  cap.parent_id(id).is_none()
}

#[derive(Debug, Widget)]
#[gui(Event = Event, Message = Message)]
struct CreatingWidget {
  id: Id,
}

impl CreatingWidget {
  pub fn new(id: Id, cap: &mut dyn MutCap<Event, Message>) -> Self {
    let child = cap.add_widget(
      id,
      Box::new(|| {
        TestWidgetDataBuilder::new()
          .event_handler(counting_handler)
          .build()
      }),
      Box::new(|id, _cap| Box::new(TestWidget::new(id))),
    );
    // Focus the "last" widget. Doing so allows us to send an event to
    // all widgets.
    cap.focus(child);

    if need_more(id, cap) {
      let _ = cap.add_widget(
        id,
        Box::new(|| Box::new(())),
        Box::new(|id, cap| Box::new(CreatingWidget::new(id, cap))),
      );
    }

    Self { id }
  }
}

#[async_trait(?Send)]
impl Handleable<Event, Message> for CreatingWidget {
  async fn handle(&self, cap: &mut dyn MutCap<Event, Message>, event: Event) -> Option<Event> {
    counting_handler(self.id, cap, event)
  }
}


/// Test that we can recursively create a widget from a widget's
/// constructor.
#[tokio::test]
async fn recursive_widget_creation() {
  // We only create the root widget directly but it will take care of
  // recursively creating a bunch of more widgets.
  let (mut ui, _) = Ui::new(
    || Box::new(()),
    |id, cap| Box::new(CreatingWidget::new(id, cap)),
  );

  let event = Event::Int(0u64);
  let result = ui.handle(event).await.unwrap();
  // We expect three increments. Note that we have four widgets in
  // total, but we cannot easily have the event reach all four of them
  // because two are peers sharing a parent.
  assert_eq!(result.unwrap_int(), 3);
}


#[derive(Debug, Widget, Handleable)]
#[gui(Event = Event)]
struct MovingWidget {
  id: Id,
}

impl MovingWidget {
  pub fn new(id: Id) -> Self {
    Self { id }
  }
}


// This test case illustrates how to create a widget that takes
// ownership of some data during its construction.
#[test]
fn moving_widget_creation() {
  let (mut ui, root) = Ui::new(
    || TestWidgetDataBuilder::new().build(),
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  let _ = ui.add_ui_widget(
    root,
    || Box::new(()),
    |id, _cap| Box::new(MovingWidget::new(id)),
  );
}


fn create_handler(widget: Id, cap: &mut dyn MutCap<Event, Message>, event: Event) -> Option<Event> {
  match event {
    Event::Key('z') => {
      cap.add_widget(
        widget,
        Box::new(|| Box::new(())),
        Box::new(|id, _cap| Box::new(TestWidget::new(id))),
      );
      None
    },
    _ => Some(event),
  }
}


/// Test dynamic creation of a widget based on an event.
#[tokio::test]
async fn event_based_widget_creation() {
  let (mut ui, root) = Ui::new(
    || {
      TestWidgetDataBuilder::new()
        .event_handler(create_handler)
        .build()
    },
    |id, _cap| Box::new(TestWidget::new(id)),
  );
  ui.focus(root);

  assert_eq!(ui.children(root).count(), 0);

  let event = Event::Key('z');
  let result = ui.handle(event).await;
  assert!(result.is_none());

  // We must have created a widget.
  assert_eq!(ui.children(root).count(), 1);
}


fn recursive_operations_handler(
  widget: Id,
  cap: &mut dyn MutCap<Event, Message>,
  _event: Event,
) -> Option<Event> {
  // Check that we can use the supplied `MutCap` object to retrieve our
  // own parent's ID.
  cap.parent_id(widget);
  cap.focus(widget);
  cap.is_focused(widget);
  Some(Event::Int(42))
}

/// Check that widget operations on the own widget work properly.
#[tokio::test]
async fn recursive_widget_operations() {
  let (mut ui, root) = Ui::new(
    || {
      TestWidgetDataBuilder::new()
        .event_handler(recursive_operations_handler)
        .build()
    },
    |id, _cap| Box::new(TestWidget::new(id)),
  );

  ui.focus(root);

  let result = ui.handle(Event::Empty).await.unwrap();
  assert_eq!(result.unwrap_int(), 42);
}

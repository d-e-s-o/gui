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

use common::TestWidget;
use common::unwrap_custom;


#[test]
fn correct_ids() {
  let (mut ui, root) = Ui::new(&mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w1 = ui.add_widget(root, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w2 = ui.add_widget(root, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  // And a container.
  let c1 = ui.add_widget(root, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  // And a widget to the container.
  let w3 = ui.add_widget(c1, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  // And another container for deeper nesting.
  let c2 = ui.add_widget(c1, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
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
fn creation_order_is_child_order() {
  let (mut ui, root) = Ui::new(&mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w1 = ui.add_widget(root, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w11 = ui.add_widget(w1, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w12 = ui.add_widget(w1, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w2 = ui.add_widget(root, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w21 = ui.add_widget(w2, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });

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

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "The given Id belongs to a different Ui")]
fn share_ids_between_ui_objects() {
  let (mut ui1, root) = Ui::new(&mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let widget = ui1.add_widget(root, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });

  let (mut ui2, _) = Ui::new(&mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });

  // `widget` is registered to `ui1` and so using it in the context of
  // `ui2` is not as intended. On debug builds we have special detection
  // in place to provide a meaningful error, that should trigger here.
  ui2.handle(UiEvent::Custom(widget, Box::new(())));
}

#[test]
fn visibility_fun() {
  let (mut ui, root) = Ui::new(&mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w1 = ui.add_widget(root, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w2 = ui.add_widget(root, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w3 = ui.add_widget(w2, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });

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
  let (mut ui, root) = Ui::new(&mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });

  // No widget has the input focus by default.
  assert!(ui.focused().is_none());

  let _ = ui.add_widget(root, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  assert!(ui.focused().is_none());
}

#[test]
fn focus_widget() {
  let (mut ui, root) = Ui::new(&mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let widget = ui.add_widget(root, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });

  ui.focus(widget);
  assert!(ui.is_focused(widget));
}

#[test]
fn focus_makes_widget_visible() {
  let (mut ui, root) = Ui::new(&mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let widget = ui.add_widget(root, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });

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
  let (mut ui, root) = Ui::new(&mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w1 = ui.add_widget(root, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w11 = ui.add_widget(w1, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w2 = ui.add_widget(root, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w21 = ui.add_widget(w2, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w22 = ui.add_widget(w2, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w23 = ui.add_widget(w2, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });

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

#[test]
fn repeated_show_preserves_order() {
  let (mut ui, root) = Ui::new(&mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w1 = ui.add_widget(root, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w11 = ui.add_widget(w1, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w2 = ui.add_widget(root, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w21 = ui.add_widget(w2, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });

  // By default all widgets are visible. Make sure that issuing a show
  // does not change the order of children. It should be a no-op.
  let before = ui.children(root).cloned().collect::<Vec<_>>();

  ui.show(w11);
  let after = ui.children(root).cloned().collect::<Vec<_>>();
  assert_eq!(before, after);

  ui.show(w21);
  let after = ui.children(root).cloned().collect::<Vec<_>>();
  assert_eq!(before, after);
}

#[test]
fn hide_changes_child_order() {
  let (mut ui, root) = Ui::new(&mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w1 = ui.add_widget(root, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w2 = ui.add_widget(root, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w3 = ui.add_widget(root, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w4 = ui.add_widget(root, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });

  ui.hide(w2);
  {
    let mut it = ui.children(root);
    assert_eq!(*it.next().unwrap(), w1);
    assert_eq!(*it.next().unwrap(), w3);
    assert_eq!(*it.next().unwrap(), w4);
    assert_eq!(*it.next().unwrap(), w2);
    assert!(it.next().is_none());
  }

  ui.hide(w3);
  {
    let mut it = ui.children(root);
    assert_eq!(*it.next().unwrap(), w1);
    assert_eq!(*it.next().unwrap(), w4);
    assert_eq!(*it.next().unwrap(), w3);
    assert_eq!(*it.next().unwrap(), w2);
    assert!(it.next().is_none());
  }
}

#[test]
fn repeated_hide_preserves_order() {
  let (mut ui, root) = Ui::new(&mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let _ = ui.add_widget(root, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w2 = ui.add_widget(root, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });
  let w3 = ui.add_widget(root, &mut |id, _cap| {
    Box::new(TestWidget::new(id))
  });

  ui.hide(w2);
  ui.hide(w3);

  let before = ui.children(root).cloned().collect::<Vec<_>>();
  ui.hide(w2);

  let after = ui.children(root).cloned().collect::<Vec<_>>();
  assert_eq!(before, after);
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


/// Check if we need to create another `CreatingWidget`.
fn need_more(id: Id, cap: &Cap) -> bool {
  cap.parent_id(id).is_none()
}

#[derive(Debug, GuiWidget)]
struct CreatingWidget {
  id: Id,
}

impl CreatingWidget {
  pub fn new(id: Id, cap: &mut Cap) -> Self {
    let child = cap.add_widget(id, &mut |id, _cap| {
      Box::new(TestWidget::with_handler(id, counting_handler))
    });
    // Focus the "last" widget. Doing so allows us to send an event to
    // all widgets.
    cap.focus(child);

    if need_more(id, cap) {
      let _ = cap.add_widget(id, &mut |id, cap| {
        Box::new(CreatingWidget::new(id, cap))
      });
    }

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
    Box::new(CreatingWidget::new(id, cap))
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
    Box::new(TestWidget::new(id))
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
    Box::new(TestWidget::with_handler(id, create_handler))
  });
  ui.focus(root);

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
    Box::new(TestWidget::with_handler(id, recursive_operations_handler))
  });

  ui.handle(Event::Custom(Box::new(())));
}

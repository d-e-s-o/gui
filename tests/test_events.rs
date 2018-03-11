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

use gui::Event;
use gui::Handleable;
use gui::Id;
use gui::Key;
use gui::Object;
use gui::Renderable;
use gui::Renderer;
use gui::Ui;
use gui::UiEvent;
use gui::Widget;

use common::TestContainer;
use common::TestRenderer;
use common::TestRootWidget;
use common::TestWidget;


#[derive(Debug)]
struct TestEventWidget {
  id: Id,
  parent_id: Id,
  to_focus: Option<Id>,
}

impl TestEventWidget {
  fn new(parent_id: Id, id: Id, to_focus: Option<Id>) -> Self {
    TestEventWidget {
      id: id,
      parent_id: parent_id,
      to_focus: to_focus,
    }
  }
}

impl<R> Renderable<R> for TestEventWidget
where
  R: Renderer,
{
  fn render(&self, renderer: &R) {
    renderer.render(self)
  }
}

impl Handleable for TestEventWidget {
  fn handle(&mut self, event: Event) -> Option<UiEvent> {
    Some(match event {
      Event::KeyDown(key) => {
        match key {
          Key::Char('a') => {
            if let Some(id) = self.to_focus {
              UiEvent::Focus(id)
            } else {
              event.into()
            }
          },
          _ => event.into(),
        }
      },
      _ => event.into(),
    })
  }
}

impl Object for TestEventWidget {
  fn id(&self) -> Id {
    self.id
  }

  fn parent_id(&self) -> Option<Id> {
    Some(self.parent_id)
  }
}

impl<R> Widget<R> for TestEventWidget
where
  R: Renderer,
{
}


#[test]
fn events_bubble_up_when_unhandled() {
  let mut ui = Ui::<TestRenderer>::new();
  let r = ui.add_root_widget(|id| {
    Box::new(TestRootWidget::new(id))
  });
  let c1 = ui.add_widget(r, |parent_id, id| {
    Box::new(TestContainer::new(parent_id, id))
  });
  let c2 = ui.add_widget(c1, |parent_id, id| {
    Box::new(TestContainer::new(parent_id, id))
  });
  let w1 = ui.add_widget(c2, |parent_id, id| {
    Box::new(TestWidget::new(parent_id, id))
  });

  let event = Event::KeyUp(Key::Char(' '));
  ui.focus(w1);

  let result = ui.handle(event.clone());
  // An unhandled event should just be returned after every widget
  // forwarded it.
  assert_eq!(result.unwrap(), event.into());
}

#[test]
fn event_handling_with_focus() {
  let mut ui = Ui::<TestRenderer>::new();
  let r = ui.add_root_widget(|id| {
    Box::new(TestRootWidget::new(id))
  });
  let w1 = ui.add_widget(r, |parent_id, id| {
    Box::new(TestEventWidget::new(parent_id, id, None))
  });
  let w2 = ui.add_widget(r, |parent_id, id| {
    Box::new(TestEventWidget::new(parent_id, id, Some(w1)))
  });

  ui.focus(w2);
  assert!(ui.is_focused(w2));

  // Send a key down event, received by `w2`, which it will
  // translate into a focus event for `w1`.
  let event = Event::KeyDown(Key::Char('a'));
  ui.handle(event);

  assert!(ui.is_focused(w1));
}

#[test]
fn quit_event() {
  let mut ui = Ui::<TestRenderer>::new();
  let r = ui.add_root_widget(|id| {
    Box::new(TestRootWidget::new(id))
  });
  let c1 = ui.add_widget(r, |parent_id, id| {
    Box::new(TestContainer::new(parent_id, id))
  });
  let _ = ui.add_widget(c1, |parent_id, id| {
    Box::new(TestWidget::new(parent_id, id))
  });

  let result = ui.handle(UiEvent::Quit);
  assert_eq!(result.unwrap(), UiEvent::Quit);
}

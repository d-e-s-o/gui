Unreleased
----------
- Moved `Custom` event variant from `gui::Event` into `gui::UiEvent` and
  renamed former `gui::UiEvent::Custom` into `gui::UiEvent::Directed`
- Adjusted signature of event hook functions to take event by value, not
  reference (made possible because `gui::Event` is now copyable)


0.1.1
-----
- Added link to `docs.rs` based documentation to README


0.1.0
-----
- Initial release

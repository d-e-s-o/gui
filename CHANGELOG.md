Unreleased
----------
- Adjusted crate to use Rust Edition 2018
- Removed `#![deny(warnings)]` attribute and demoted lints prone to
  future changes from `deny` to `warn`
- Enabled CI pipeline comprising building, testing, and linting of the
  project
  - Added badge indicating pipeline status
- Added categories to `Cargo.toml`


0.2.1
-----
- Usage of event hooks no longer induces an unnecessary clone of a
  `HashSet` every time an event is handled
- Hook emitted events are now delivered to the destination widget after
  the source event was delivered
  - Order was left unspecified beforehand, but was happening in reverse
    (i.e., hook emitted events arrived before the source event did)
- Enabled Rust 2018 edition lints
- Enabled `unused-results` lint


0.2.0
-----
- Moved `Custom` event variant from `gui::Event` into `gui::UiEvent` and
  renamed former `gui::UiEvent::Custom` into `gui::UiEvent::Directed`
- Adjusted signature of event hook functions to take event by value, not
  reference (made possible because `gui::Event` is now copyable)
- Added support for "returnable" events, i.e., a variant of a custom
  event that is guaranteed to be returned to the sending widget (after
  potential modification by the destination widget)
  - Handling of custom events changed to using two new methods in the
    `Handleable` trait: `handle_custom` and `handle_custom_ref`
- Introduced new event type for unhandled events: `UnhandledEvent`
  - Changed return type of `Ui::handle` from `Option<MetaEvent>` to
    `Option<UnhandledEvent>`
- Renamed `MetaEvent` to `UiEvents`


0.1.1
-----
- Added link to `docs.rs` based documentation to README


0.1.0
-----
- Initial release

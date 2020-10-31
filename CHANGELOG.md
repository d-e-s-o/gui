Unreleased
----------
- Removed custom event variants & infrastructure
- Adjusted event handlers to return a single event only
  - Removed event chaining logic


0.6.0-alpha.2
-------------
- Converted event hooks to async functions
- Adjusted event hooks to return a single event
  - Introduced `Mergeable` trait to combine events emitted from an event
    hook
- Introduced post-hook support


0.6.0-alpha.1
-------------
- Introduced asynchronous message passing mechanism
  - Added additional generic `Message` type to `Ui`, `Widget`, and
    more structs & traits
  - Added `MutCap::send` & `MutCap::call` methods
- Converted all event handlers to async functions
  - Added `async-trait` dependency in version `0.1`


0.6.0-alpha.0
-------------
- Separated widgets from their mutable data
  - Added data creation function alongside widget creation one
  - Added `data` and `data_mut` accessors to `Cap` and `MutCap`
  - Changed all `Handleable` event handling methods to target `&self`
  - Changed event hooks to target non-mutable `&dyn Widget`
- Reordered `Cap` & `MutCap` parameter in various function signatures
- Changed widget creation function from `FnMut` to `FnOnce`
- Adjusted event hooks to accept `MutCap` instead of `Cap`


0.5.0
-----
- Added code coverage collection and reporting to CI pipeline
- Bumped minimum required Rust version to `1.42.0`


0.4.0
-----
- Bumped minimum required Rust version to `1.36.0`
- Downgraded `deny` crate-level lints to `warn`


0.3.0
-----
- Made `Handleable` trait generic over the event type to use
  - Made `UiEvent` and `UnhandledEvent` generic over the underlying event
- Split `Cap` trait into `Cap` and `MutCap` with all methods requiring
  a mutable self ending up in `MutCap`
- Require `Debug` implementation for `Cap`, `MutCap`, `Handleable`, and
  `Object`
- Added `TypeId` functionality to `Renderable`
- Adjusted event hook function signature to take event to use by
  reference
- Removed `Event` and `Key` types
- Introduced 'derive' feature pulling in and re-exporting the custom
  derive functionality provided by `gui-derive`


0.2.2
-----
- Adjusted crate to use Rust Edition 2018
- Removed `#![deny(warnings)]` attribute and demoted lints prone to
  future changes from `deny` to `warn`
- Enabled CI pipeline comprising building, testing, and linting of the
  project
- Added badges indicating pipeline status, current `crates.io` published
  version of the crate, current `docs.rs` published version of the
  documentation, and minimum version of `rustc` required
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

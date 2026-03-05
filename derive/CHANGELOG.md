0.6.7
-----
- Bumped `syn` dependency to `2.0`
- Bumped minimum supported Rust version to `1.64`


0.6.3
-----
- Switched to using Rust edition 2021


0.6.1
-----
- Emit `Renderable::render_done` implementation


0.6.0
-----
- Integrated with version `0.6.0` of `gui`


0.6.0-alpha.1
-------------
- Added support for generic `Message` type for `Widget` and `Handleable`
  derive macros


0.5.0
-----
- Fixed build breakage from using private `__rt` member from `quote`
- Bumped minimum required Rust version to `1.42.0`


0.4.0
-----
- Made `Event = ...` attribute support actual event type and not just
  string representation of it
- Bumped `syn` dependency to `1.0`
- Bumped `quote` dependency to `1.0`
- Bumped `proc-macro` dependency to `1.0`
- Bumped minimum required Rust version to `1.36.0`


0.3.0
-----
- Added support for `gui(Event = ...)` attribute
- Dropped "Gui" prefix from custom derive macros
- Downgraded dependency to `gui` to a dev-dependency


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
- Bumped `syn` dependency to `0.15`


0.2.1
-----
- Enabled Rust 2018 edition lints
- Enabled `unused-results` lint


0.1.0
-----
- Initial release

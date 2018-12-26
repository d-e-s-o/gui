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

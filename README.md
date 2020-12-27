[![pipeline](https://gitlab.com/d-e-s-o/gui/badges/master/pipeline.svg)](https://gitlab.com/d-e-s-o/gui/commits/master)
[![coverage](https://gitlab.com/d-e-s-o/gui/badges/master/coverage.svg)](https://gitlab.com/d-e-s-o/gui/-/jobs/artifacts/master/file/kcov/kcov-merged/index.html?job=coverage:kcov)
[![crates.io](https://img.shields.io/crates/v/gui.svg)](https://crates.io/crates/gui)
[![Docs](https://docs.rs/gui/badge.svg)](https://docs.rs/gui)
[![rustc](https://img.shields.io/badge/rustc-1.43+-blue.svg)](https://blog.rust-lang.org/2020/04/23/Rust-1.43.0.html)

gui
===

- [Documentation][docs-rs]
- [Changelog](CHANGELOG.md)

**gui** (short for **g**eneric **u**ser **i**nterface) is a crate
providing basic user interface functionality. It strives to be
as independent as possible of the underlying system architecture. That
is, it is compatible with windowing as well as terminal based systems
and it does not rely on the specifics of any GUI toolkit.

Part of this story means that it is not an out-of-the-box replacement
for your favorite user interface toolkit (think: [GTK+][gtk], [Qt][qt],
[wxWidgets][wxwidgets] to name popular ones), but that it should be seen
more as a building block, providing certain hooks and basic
functionality typically seen in a UI. The infrastructure for dispatching
events to widgets is an example. To make proper use of it, the
functionality provided by this crate needs to be glued with the
underlying system.

**gui** is used for exploring parts of the design space for user
interface architecture using [Rust][rust-lang]. Design of UI systems in
Rust is generally considered hard and to a large degree an unsolved
problem, although there are various promising designs out there.

The crate uses Rust's `async`/`await` for ergonomic event handling and
message passing between widgets and to the best of the author's knowledge,
is the first doing so.


Features
--------
- completely independent of underlying architecture
  - generic over events and messages used
  - compatible with any rendering library
- `async`/`await` based event handling and message passing support
- no dependencies other than [`async-trait`][async-trait] to work around
  current short comings in Rust


Status
------

The crate is under active development, and while its core has been
reasonably stable for a while, changes should be anticipated in the
future.

Given this current state, changes in API design are to be expected.


Example Usage
-------------

The [`notnow`][notnow] program is relying on the **gui** crate for the
creation of its terminal based UI. The basic workings can be seen there.

[async-trait]: https://crates.io/crates/async-trait
[docs-rs]: https://docs.rs/crate/gui
[gtk]: https://www.gtk.org
[qt]: https://www.qt.io
[wxwidgets]: https://wxwidgets.org
[rust-lang]: https://www.rust-lang.org
[notnow]: https://crates.io/crates/notnow

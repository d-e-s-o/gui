[![pipeline](https://gitlab.com/d-e-s-o/gui/badges/master/pipeline.svg)](https://gitlab.com/d-e-s-o/gui/commits/master)
[![coverage](https://gitlab.com/d-e-s-o/gui/badges/master/coverage.svg)](https://gitlab.com/d-e-s-o/gui/commits/master)
[![crates.io](https://img.shields.io/crates/v/gui.svg)](https://crates.io/crates/gui)
[![Docs](https://docs.rs/gui/badge.svg)](https://docs.rs/gui)
[![rustc](https://img.shields.io/badge/rustc-1.36+-blue.svg)](https://blog.rust-lang.org/2019/07/04/Rust-1.36.0.html)

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
[wxWidgets][wxwidgets] to name popular ones, although these are
naturally much more complex and provide way more functionality -- they
are just among the most well known), but that it should be seen more as
a building block, providing certain hooks and basic functionality
typically seen in a UI. The infrastructure for dispatching events to
widgets is an example. To make proper use of it, the functionality
provided by this crate needs to be glued with the underlying system.

**gui** is used for exploring parts of the design space for user
interface architecture using [Rust][rust-lang]. Design of UI systems in
Rust is generally considered hard and to a large degree an unsolved
problem, although there are various promising designs out there.


Status
------

The crate is under active development, and while its core has been
reasonably stable for a while, changes should be anticipated in the
future. In its current state mouse support of any kind is missing.

Given this current state, changes in API design are to be expected.

The **gui** crate typically compiles with the most recent version of
stable Rust. On compile errors please try upgrading to a more recent
version first.


Example Usage
-------------

The [notnow][notnow] program is relying on the **gui** crate for the
creation of its terminal based UI. The basic workings can be seen there.

[docs-rs]: https://docs.rs/crate/gui
[gtk]: https://www.gtk.org
[qt]: https://www.qt.io
[wxwidgets]: https://wxwidgets.org
[rust-lang]: https://www.rust-lang.org
[notnow]: https://crates.io/crates/notnow

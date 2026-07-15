# Wakey vt100 patch

This fork is based on `vt100` version `0.16.2`. Wakey tracks it as a Git
submodule and uses the root workspace's `[patch.crates-io]` entry to select it
without changing the dependency's name or version.

Wakey adds `Screen::snapshot_formatted` and one private grid serializer. The
extension reads the retained primary grid even while the alternate grid is
active, allowing a reconnecting terminal to restore both shell scrollback and
the current full-screen application without mutating the parser.

Keep changes outside that extension synchronized with upstream
<https://github.com/doy/vt100-rust/tree/v0.16.2>.

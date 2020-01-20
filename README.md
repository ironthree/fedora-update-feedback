# fedora-update-feedback

[![crates.io](https://img.shields.io/crates/v/fedora-update-feedback.svg)](https://crates.io/crates/fedora-update-feedback/)
[![crates.io](https://img.shields.io/crates/d/fedora-update-feedback.svg)](https://crates.io/crates/fedora-update-feedback/)
[![crates.io](https://img.shields.io/crates/l/fedora-update-feedback.svg)](https://crates.io/crates/fedora-update-feedback/)
[![docs.rs](https://docs.rs/fedora-update-feedback/badge.svg)](https://docs.rs/fedora-update-feedback/)

This project is inspired by [fedora-easy-karma][f-e-k], but with more features.

[f-e-k]: https://pagure.io/fedora-easy-karma

It allows submitting feedback for bugs and test cases in addition to providing a
comment and karma (providing bug and test case feedback is a work in progress
and is blocked by a [bodhi server issue][bodhi-issue]).

[bodhi-issue]: https://github.com/fedora-infra/bodhi/issues/3888

By default, all updates in `testing` or `pending` state that the user has not
submitted themselves or has already commented on are presented, sorted by
ascending submission date (so, oldest to most recent update).

### requirements

The program assumes that the `dnf` and `rpm` binaries are present on the system
(which is probably a reasonable assumption for a CLI tool targeted at fedora
users).

It also expects a config file at `~/.config/fedora.toml`, with at least the
following contents:

```toml
[FAS]
username = "USERNAME"
```

This value is used to filter out updates that the user themselves has submitted,
or has already commented on.


### installation

To compile the program, first install `cargo` (the build tool, also pulls in
the Rust compiler) and `openssl-devel` (used by the OpenSSL rust bindings).

To download, build, and install the latest version from <https://crates.io>,
just run `cargo install fedora-update-feedback`.

To build from the sources provided on GitHub, download the sources
(recommended: tarball of the latest release from GitHub), and easily build
and install the binary for yourself by running `cargo install --path .` in
the source directory.

Either way, `cargo` will install the binary into `~/.cargo/bin` by default.

To make it available in `$PATH`, either copy it into `$HOME/.local/bin`, or add
`~/.cargo/bin` to your `$PATH` (probably by editing `~/.bash_profile`).


### TODO

- As mentioned above, submitting bug and testcase feedback is implemented (but
  hidden behind a compilation flag), but blocked by a bodhi server issue (or me
  being stupid).

- I'd like to improve the "visual quality" of the terminal output and
  pretty-printed data, which should be easy.

- It would be great to add additional switches and arguments to the binary (for
  example, sorting updates by a different value than submission date).

- Ignoring certain updates or packages is not yet implemented.


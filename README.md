# fedora-update-feedback

[![crates.io](https://img.shields.io/crates/v/fedora-update-feedback.svg)](https://crates.io/crates/fedora-update-feedback/)
[![crates.io](https://img.shields.io/crates/d/fedora-update-feedback.svg)](https://crates.io/crates/fedora-update-feedback/)
[![crates.io](https://img.shields.io/crates/l/fedora-update-feedback.svg)](https://crates.io/crates/fedora-update-feedback/)
[![docs.rs](https://docs.rs/fedora-update-feedback/badge.svg)](https://docs.rs/fedora-update-feedback/)

This project is inspired by [fedora-easy-karma][f-e-k], but with more features.

[f-e-k]: https://pagure.io/fedora-easy-karma

It allows submitting feedback for bugs and test cases in addition to providing a
comment and karma.

[bodhi-issue]: https://github.com/fedora-infra/bodhi/issues/3888

By default, all updates in the `testing` state that the user has not submitted
themselves or has already commented on are presented, sorted by ascending
submission date (so, oldest to most recent update).


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

If this file is not present, the legacy `~/.fedora.upn` file is used as a
fallback mechanism.

If both files are not present, the username has to be specified with the
`--username USERNAME` CLI switch.

The username is used to authenticate with bodhi, and to filter out updates that
the user themselves has submitted, or has already commented on.


### features

By default, `fedora-update-feedback` queries bodhi for updates for the current
release that are in the `testing` state.

Some additional options can be set either on the command line, or in a
`[fedora-update-feedback]` section in the `~/.config/fedora.toml` configuration
file.

With the `--check-pending` CLI switch or the `check-pending = true`
configuration option, updates in the `pending` state are also queried (for
example, if the user has manually installed builds from koji and wants to give
bodhi feedback for those as well). 

Additionally, with the `--check-obsoleted` and `--check-unpushed` flags (or
the `check-obsoleted = true` and `check-unpushed` configuration options),
`fedora-update-feedback` will check if any lingering builds from unpushed
or obsoleted updates are still installed locally.

With the `save-password = true` configuration option, `fedora-update-feedback`
will attempt to securely save the FAS password in the login keyring, so it
does not have to be entered every time. To ignore or overwrite a stored
password, use the `--ignore-keyring` CLI switch. 

This information is also printed when running `fedora-update-feedback --help`.


### installation

> RPM packages are now available on COPR:
> 
> <https://copr.fedorainfracloud.org/coprs/decathorpe/fedora-update-feedback/>

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

- I'd like to improve the "visual quality" of the terminal output and
  pretty-printed data, which should be easy.

- It would be great to add additional switches and arguments to the binary (for
  example, sorting updates by a different value than submission date).


# Release 2.1.4

- Port from deprecated the deprecated term_size crate to terminal_size.
- Update the env_logger dependency from 0.10 to 0.11.

# Release 2.1.3

- Update various dependencies.

# Release 2.1.2 "Werk Around" (April 10, 2023)

- Apply a workaround to fix repository queries with DNF 4.15.

# Release 2.1.1 "Simple" (March 14, 2023)

- Simplify code for printing server messages.
- Revert back to using rpassword v5 for now.

# Release 2.1.0 "Pretty" (March 10, 2023)

This release makes some console output prettier and updates some dependencies
to the latest versions.

# Release 2.0.2 "Invert" (July 1, 2022)

This release only includes a fix for a minor logic error: Previously, any
server messages that were returned when posting a comment to bodhi were
always ignored, but lists of empty server messages were printed. This release
inverts the behaviour to the correct one (server messages are printed if - and
only if - the server actually returned messages).

# Release 2.0.1 "Optimize" (May 30, 2022)

This release includes some minor changes, including:

- bumped `bodhi` dependency to `2.0.1` to improve compatibility with future
  releases of the bodhi server (including 6.0.0)
- improved heuristics for determining whether an update is ready for feedback
  (again) after karma was reset
- skip printing update details that only contain wrong / garbage data

# Release 2.0.0 "Finally" (February 01, 2022)

This release contains only minor code changes compared to the previous beta:

- slightly improved progress reporting
- use custom User-Agent header for HTTP requests

Additionally, some crate dependencies were updated to match the versions that
are available from Fedora repositories (at the time of publishing).

For a complete list of changes since `v1.1.0`, read the release notes for the
last three beta releases.

# Release 2.0.0-beta.3 "Fix the Fix" (January 28, 2022)

This beta release only changes the way bash completions are generated to
un-break `cargo package` and `cargo publish`.

# Release 2.0.0-beta.2 "Spring Cleaning" (January 28, 2022)

This beta release mostly consists of minor code cleanups, and small improvements
for error messages and output formatting.

One notable change is the introduction of a build script that generates bash
completions for the fedora-update-feedback CLI during the build process.

# Release 2.0.0-beta.1 "Modern Times" (January 23, 2022)

New features:

- add possibility to permanently ignore updates for certain packages
  (`--add-ignored-package` CLI flag, `[b]lock` action)
- add `--print-ignored` flag to print ignored updates and packages
- print a warning if the `updates-testing` repository is not enabled

# Release 1.1.0 "Cookie Monster" (September 24, 2021)

Improvements:

- add "abort" action to stop providing update feedback
- don't print administrative comments by bodhi
- show progress through list of updates

Internal changes:

- add debug logging infrastructure
- refactored a lot of code from "main" into separate functions

# Release 1.0.4 "Face Lift" (August 09, 2021)

Minor changes:

- fix output formatting issues caused by upgrading to `textwrap` version `0.14`

# Release 1.0.3 "Moar Upgrade" (July 30, 2021)

Internal changes:

- update `textwrap` to `0.14`

# Release 1.0.2 "The Fix" (June 03, 2021)

Internal changes:

- fix code style for 2021 edition changes (`panic!` macro changes)
- update `rustyline` to `8`

# Release 1.0.1 "Upgrade" (January 06, 2021)

Internal changes:

- update `secret-service` to `2.0` (with the new `zbus`backend!)
- update `textwrap` to `0.13`

# Release 1.0.0 "Stability" (November 30, 2020)

Internal changes:

- update `bodhi` to `1.0`
- update `dirs` to `3.0.1`
- update `rpassword` to `5.0.0`

# Release 0.6.0 "Lib Drop" (October 31, 2020)

Breaking Changes:

- refactored code into a binary-only crate
- dropped unused `fedora_update_feedback` library component

# Release 0.5.5 "TL;DR" (June 22, 2020)

Improvements:

- include package summaries when printing updates

# Release 0.5.4 "Dep Bump" (June 22, 2020)

Improvements:

- bump bodhi dependency to ^0.6 for bodhi 5.4.0 server support

# Release 0.5.3 "Off Set" (Apr. 08, 2020)

Bugfixes:

- fix the wrong offset for "installtime" calculation (dnf returns UTC, not the
  local time)

# Release 0.5.2 "Storage Area" (Apr. 06, 2020)

Incremental improvements:

- show how long updates have been installed locally, in addition to the dates
  when they were submitted and pushed in bodhi

# Release 0.5.1 "Innit Nice" (Mar. 28, 2020)

Incremental improvements:

- improve some error messages for password / keyring handling
- cleaner and nicer "UI" when printing updates and asking for feedback
- show already existing comments when printing updates (including the author's
  username, submission date, and associated karma)
- show the date & time an update was pushed to updates-testing

# Release 0.5.0 "Forget-me-not" (Mar. 08, 2020)

Incremental improvements:

- optionally store FAS password in the session keyring (using `libsecret` / the
  `SecretService` D-Bus API)
- this feature can be enabled by setting the `save-password = true` setting in
  the `fedora.toml` config file
- to ignore a previously stored password, use the `--ignore-keyring` CLI switch 

# Release 0.4.2 "Print me by my name" (Feb. 28, 2020)

Incremental improvements:

- nice update banner when pretty-printing the update when asking for feedback
- also print human-readable update title in addition to the update alias

# Release 0.4.1 "Show me what you've got" (Feb. 28, 2020)

Incremental improvements:

- only list actually installed binary packages when asking for feedback, instead
  of only printing the corresponding source packages

# Release 0.4.0 "Moar Options" (Feb. 24, 2020)

Incremental improvements:

- optionally query for obsolete and unpushed updates, and warn if builds from
  any of them are installed locally
- this behaviour can be controlled by the `fedora.toml` config file and with
  CLI switches

# Release 0.3.2 "A Dep Bump Is Not A DB Dump" (Feb. 17, 2020)

Incremental improvements:

- updated dependencies
- use more nice `structopt` features (colored error messages, etc.)

# Release 0.3.1 "Weak Link" (Jan. 26, 2020)

Incremental improvements:

- only list actually installed builds for multi-build updates
- print URLs for bugs (→ Red Hat BugZilla) and test cases (→ fedora wiki)

# Release 0.3.0 "Ignore me senpai" (Jan. 26, 2020)

Incremental improvements:

- allow ignoring certain updates permanently
- don't ask for feedback for previously ignored updates
- automatically garbage-collect ignore-list
- add `--clear-ignored` CLI switch to manually clear the list

# Release 0.2.2 "Dependency Bump" (Jan. 26, 2020)

Fix a typo in the `term_size` dependency version.

# Release 0.2.1 "Let's Wrap" (Jan. 24, 2020)

Incremental improvements:

- wrap update text to terminal width, so things are not cut off mid-word
- warn about any unpushed updates that are installed locally

# Release 0.2.0 "Listen to me complain" (Jan. 23, 2020)

Incremental improvements:

- bug and testcase feedback are now enabled by default (and work)
- the FAS username can be supplied via CLI argument as well
- add a CLI flag to also check updates in `pending` state
- bump `bodhi` dependency to 0.5.1

# Release 0.1.2 "Prettify" (Jan. 21, 2020)

Incremental improvements:

- nicer progress bar while fetching bodhi updates
- read legacy `~/.fedora.upn` file for FAS username fallback value
- internal refactoring and code cleanup
- added documentation for all "public" items
- implement clippy advice

# Release 0.1.1 "Nothing to see here" (Jan. 19, 2020)

Bump `bodhi` dependency to 0.5.0.

# Release 0.1.0 "Housekeeping" (Jan. 19, 2020)

Incremental improvements:

- bump `bodhi` dependency to 0.4.0
- adapt to minor API changes
- use new parsing features

# Release 0.0.2 "Feedback Feature Flag" (Jan. 12, 2020)

Incremental improvements:

- hide bug and testcase feedback behind a feature flag until it works
- display progress bar forgetting update list from bodhi

# Release 0.0.1 "Morgenstemning" (Jan. 09, 2020)

Initial release, working implementation.

Providing bug- and test case feedback with comments is not yet done.


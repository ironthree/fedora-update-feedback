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
- additionally show already existing comments when printing updates (including
  the author's username, submission date, and associated karma)
- additionally show the date & time an update was pushed to updates-testing

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
- add CLI flag to also check updates in `pending` state
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


# fedora-update-feedback

This project is inspired by [fedora-easy-karma][f-e-k], but with more features.

It allows submitting bug and testcase feedback in addition to providing a comment and karma.

Like `fedora-update-notifier`, it expects a config file at `~/.config/fedora.toml`, with at least the following
contents:

```toml
[FAS]
username = "USERNAME"
```

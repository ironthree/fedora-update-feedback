# fedora-update-feedback

This project is inspired by [fedora-easy-karma][f-e-k], but with more features.

[f-e-k]: https://pagure.io/fedora-easy-karma

It allows submitting feedback for bugs and test cases in addition to providing a
comment and karma (providing test case feedback is a work in progress and is
blocked by a [bodhi server issue][bodhi-issue]).

[bodhi-issue]: https://github.com/fedora-infra/bodhi/issues/3888

Like `fedora-update-notifier`, it expects a config file at
`~/.config/fedora.toml`, with at least the following contents:

```toml
[FAS]
username = "USERNAME"
```


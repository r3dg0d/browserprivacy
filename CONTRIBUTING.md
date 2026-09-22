# Contributing to browserprivacy

Thanks for your interest in contributing!

## Development

```bash
cargo build
cargo test
cargo build --release
cargo run -- --help
```

## Guidelines

- Prefer clear error messages over silent failures
- Keep privacy/security claims honest and conservative
- Add tests for parsers and non-destructive logic
- Do not commit secrets, credentials, or personal profile data
- Follow Rust 2021 edition idioms; `clippy` should be clean on new code
- Document user-facing behavior in the README

## Pull requests

1. Fork and create a feature branch
2. Add tests where practical
3. Update CHANGELOG.md under an `Unreleased` section
4. Ensure `cargo test` and `cargo clippy` pass

Owner: **r3dg0d**

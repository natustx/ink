# Contributing to ink

Thanks for your interest in contributing. Here's how to get started.

## Getting started

1. Fork the repo and clone it
2. Make sure you have Rust installed (`rustup` recommended)
3. Run `cargo build` to verify everything compiles
4. Run `cargo test` to make sure tests pass

## Making changes

- Create a branch from `main`
- Keep your changes focused — one feature or fix per PR
- Add tests if you're adding new functionality
- Run `cargo test` and `cargo clippy` before submitting

## Code style

- Follow existing patterns in the codebase
- Keep functions small and focused
- Handle errors gracefully — ink should never crash on bad input
- If a terminal doesn't support a feature, fall back silently

## Reporting bugs

Open an issue with:
- What you expected to happen
- What actually happened
- Your terminal emulator and OS
- The markdown content that triggered the bug (if applicable)

## Feature requests

Open an issue describing what you'd like and why. Keep it concrete — "support X syntax" is better than "make rendering better."

## License

By contributing, you agree that your work will be licensed under the same terms as the project. See [LICENSE](LICENSE).

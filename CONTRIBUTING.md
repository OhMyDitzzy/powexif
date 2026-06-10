# Contributing to powexif

Thank you for your interest in contributing! Here's everything you need to know.

---

## Getting Started

1. **Fork** the repository and clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/powexif.git
   cd powexif
   ```

2. Make sure you have a recent stable Rust toolchain installed:
   ```bash
   rustup update stable
   ```

3. Build the project:
   ```bash
   cargo build
   ```

4. Run the tests:
   ```bash
   cargo test
   ```

---

## Reporting Bugs

Please open an issue at https://github.com/OhMyDitzzy/powexif/issues and include:

- A minimal reproducible example (ideally a small test JPEG/TIFF file or hex dump)
- The output of `powexif --version`
- Your operating system and Rust version (`rustc --version`)
- The exact command or code that triggered the bug

---

## Suggesting Features

Open an issue and describe:

- The use case you're trying to solve
- The API or CLI behavior you'd expect
- Any edge cases you've thought of

---

## Submitting a Pull Request

1. Create a branch for your change:
   ```bash
   git checkout -b fix/some-bug
   # or
   git checkout -b feat/new-feature
   ```

2. Make your changes. A few guidelines:
   - Write tests for new behavior in `src/` or in a `tests/` directory
   - Keep public API additions documented with `///` doc comments
   - Run `cargo fmt` before committing
   - Run `cargo clippy -- -D warnings` and fix any lints

3. Commit with a clear message:
   ```
   fix: handle zero-length EXIF APP1 segment gracefully

   Previously a zero-length segment caused a panic in the offset
   calculation. Now we return ExifError::NoExifSegment instead.
   ```

4. Open a pull request against `main`. Describe what you changed and why.

---

## Code Style

- Follow standard Rust idioms; `cargo fmt` enforces formatting automatically
- Prefer `?` over explicit `match` on `Result` where possible
- Avoid `unwrap()` and `expect()` in library code; use `?` or return an `ExifError`
- New public types and functions must have at least a one-line `///` doc comment

---

## Commit Message Convention

We loosely follow [Conventional Commits](https://www.conventionalcommits.org/):

| Prefix | Use for |
|--------|---------|
| `feat:` | New feature or new public API |
| `fix:` | Bug fix |
| `docs:` | Documentation only |
| `refactor:` | Code change that neither fixes a bug nor adds a feature |
| `test:` | Adding or fixing tests |
| `chore:` | Build scripts, CI, dependency bumps |

---

## License

By contributing, you agree that your contributions will be licensed under the
[Apache License, Version 2.0](LICENSE), the same license as the project.
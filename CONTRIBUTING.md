# Contributing to awesome-rust-auth

Thank you for taking the time to contribute! 🎉

The following is a set of guidelines for contributing to `awesome-rust-auth`.
These are mostly guidelines, not rules. Use your best judgment, and feel free to
propose changes to this document in a pull request.

## Table of contents

1. [Code of Conduct](#code-of-conduct)
2. [How can I contribute?](#how-can-i-contribute)
3. [Development setup](#development-setup)
4. [Pull request process](#pull-request-process)
5. [Style guide](#style-guide)
6. [Parity roadmap](#parity-roadmap)

---

## Code of Conduct

This project and everyone participating in it is governed by the
[Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to
uphold this code. Please report unacceptable behaviour to the maintainers.

---

## How can I contribute?

### Reporting bugs

1. Search existing [issues](../../issues) to see if the bug has already been reported.
2. If not, open a new issue using the **Bug report** template.
3. Include a minimal reproducer in a ````rust` code block.

### Suggesting features

1. Check the [parity snapshot table](README.md#parity-snapshot-vs-awesome-node-auth)
   to see whether the feature is already tracked.
2. Open a new issue using the **Feature request** template.

### Submitting code

- For small fixes (typos, single-function bugs), open a PR directly.
- For larger changes, open an issue first so the design can be discussed before
  significant work is invested.

---

## Development setup

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) stable (≥ 1.86.0)

### Building

```bash
# Default (no framework adapters)
cargo build

# With all adapters
cargo build --all-features

# Only axum adapter
cargo build --features axum
```

### Testing

```bash
cargo test
cargo test --all-features
```

### Linting

```bash
cargo clippy -- -D warnings
cargo fmt --check
```

---

## Pull request process

1. Fork the repository and create your branch from `main`.
2. Make your changes following the [style guide](#style-guide) below.
3. Add or update tests for any new behaviour.
4. Ensure `cargo test` and `cargo clippy` pass with **no new warnings**.
5. Update the parity table in `README.md` if your change affects a row's status.
6. Fill in the pull request template completely.
7. Request a review from the maintainers.

PRs that break existing tests or add `unsafe` code will not be merged.

---

## Style guide

- Follow standard Rust conventions (`rustfmt`, `clippy`).
- **No `unsafe` code.** The crate has `#![deny(unsafe_code)]`.
- Public types must have doc comments.
- New traits in `src/traits.rs` must document every method.
- New service structs should be generic over their store traits (see
  `AuthService`, `MagicLinkService`, etc. as reference).
- Error variants belong in `src/error.rs` as `AuthError` variants.
- Avoid adding new crate dependencies unless strictly necessary; open an issue to
  discuss first.

---

## Parity roadmap

The [parity snapshot table](README.md#parity-snapshot-vs-awesome-node-auth)
tracks which capabilities of `awesome-node-auth` are implemented, partial, or
missing. Contributions that advance any row from ❌ / ➖ Scaffolded toward ✅
are especially welcome.

When a row reaches full parity, update its status to `✅ Implemented` and
remove the `Notes` caveat.

# Copilot Instructions for tinyassist

## Project Philosophy

- Keep it simple. Prefer clarity over cleverness.
- Don't use emojis in code or documentation
- Write comments in English when necessary, regardless of prompt language
- Practice autonomous learning; when you create a task list, include a final todo: "update .github/copilot-instructions.md and relevant docs if necessary"

## Autonomous Learning

- Update this file and relevant docs after tasks or when issues arise
- Keep filenames short and integrate content into existing files rather than creating new ones.
- Capture new patterns, errors, fixes, and better practices
- Record mistakes and their solutions to avoid repeats
- After implementing a feature, always reconsider if the implementation can be simplified further

## Code Quality Standards

- Remove self-explanatory comments; keep only complex logic notes
- Reuse existing functions/components before adding new ones
- Verify that abstractions provide real value rather than just added complexity
- If example code is provided, ensure example code works without modification
- Run `just check` before finalizing work

## Rust Language Patterns

### Import Management

- **Aggressively use `use` declarations** at module level: `use std::collections::HashMap;` instead of full paths
- **Avoid glob imports** (`use foo::*;`) unless necessary
- Common imports: `Future`, `Pin`, `HashMap`, `Instant`, `Ordering`

### Idiomatic Rust

- Use `?` operator for error propagation, not pattern matching
- Use iterator chains over loops
- Prefer borrowing: `&str` over `String`, `&[T]` over `Vec<T>`
- Use `Into`/`From` traits and `str::parse()` for conversions
- Use safe crate APIs, never `unsafe` bindings

Check https://docs.rs for accurate documentation and feature flags

Especially if a method isn't found, it may be behind an optional feature

## Error Handling

- Use `thiserror` with the `Error` enum defined in `src/error.rs` or `src/module_name/error.rs`
- Return descriptive error messages using the `Error` enum variants
- Use the `?` operator for error propagation
- For contextual errors, use `Error::Context(format!("..."))` variant

## Code Organization & Refactoring

### Module Structure

- Add `mod` declarations to `lib.rs` when creating new modules
- Use `pub mod` only in `lib.rs`; in other files use `mod` and `pub use` for re-exports
- One responsibility per function; break functions >30 lines into helpers

### Multi-line Text

- Use the `indoc` crate for multi-line strings to improve readability

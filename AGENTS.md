# tinyassist

## Philosophy

- Keep it simple. Prefer clarity over cleverness.
- Don't use emojis in code or documentation.
- Write comments in English, regardless of prompt language.
- Remove self-explanatory comments; keep only complex logic notes.
- Run `just check` before finalizing work.

## Error Handling

- Do not hide errors or unexpected conditions; propagate them to the caller.
- Use `thiserror` with the `Error` enum in `src/error.rs` or `src/<module>/error.rs`.
- Return descriptive error messages using the enum variants.
- Use `?` for propagation and `Error::Context(format!("..."))` for context.

## Code Organization

- Register modules with `mod` in `src/main.rs`.
- Use `pub mod` only at the crate root; elsewhere use `mod` plus `pub use`.
- One responsibility per function; split functions longer than ~30 lines.
- Use the `indoc` crate for multi-line strings.

## Build & Release

- Cross builds run from the Nix devShell: `just build-musl`, `just build-windows`.
- `.github/workflows/release.yml` builds both targets on `v*` tags.

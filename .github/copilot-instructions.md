## Project Philosophy

- **Simplicity first**: Keep the project as simple as possible, remove unnecessary complexity
- **Deno ecosystem**: Use `deno` for package management (avoid `npm` or `npx`)
- **English comments**: Write comments in English when necessary, regardless of prompt language
- **Autonomous learning**: When creating a task list, make sure to add the task to "update this file if necessary" at the end

## Autonomous Learning

- Update this file after completing tasks or encountering issues
- Document new patterns, errors, and solutions
- Record mistakes and their solutions to prevent repetition
- Update best practices when discovering better approaches

## Development Workflow

### Essential Commands

**TypeScript/Deno (packages/tinyassist-deno/):**

```bash
deno task setup    # Setup monorepo (install lefthook, configure git hooks)
deno task fmt      # Format code across all packages
deno task lint     # Lint code across all packages
deno task check    # Type-check across all packages
deno task test     # Run tests across all packages
deno task dev      # Start the Deno application in watch mode
deno task start    # Start the Deno application
deno task build    # Build the Deno application
deno task lefthook # Run lefthook commands
```

**Rust (packages/tinyassist/):**

```bash
cargo fmt      # Format code
cargo clippy   # Lint code
cargo check    # Type-check
cargo test     # Run tests
cargo run      # Start the application
cargo build --release  # Build optimized binary
cargo build --bin tinyassist  # Build specific binary
```

- Run `deno task setup` first to configure the monorepo, then `deno task fmt && deno task lint && deno task check` to verify (TypeScript/Deno)
- Run `cargo fmt` first, then `cargo clippy && cargo check` to verify (Rust)

### Package Management

- Add packages: `deno add --allow-scripts npm:package@version`
- Clean up duplicate imports in `deno.json` after adding packages
- Focus type checking on `src/` to avoid external dependency errors

## Code Organization

### File Structure

- Use `kebab-case` for file names and directories
- Place reusable utilities in `src/utils/`
- Avoid creating redundant files with duplicate functionality

### TypeScript Guidelines

- Use relative imports only within the same directory
- Prefer `undefined` over `null` (except for database `NULL` values)
- Use `??` operator for default values instead of `||`
- Rely on TypeScript's type inference, avoid unnecessary explicit types

## Library Standards

### CLI Development

- **Parser**: Use `@stricli/core` for type-safe CLI parsing with automatic help generation
- **Colors**: Use `picocolors` for terminal colors (requires `--allow-env` flag)
- **Commands**: Use `buildCommand` for single commands, `buildApplication` for applications

### Error Handling

- Keep error handling simple for CLI tools - let errors bubble up naturally
- Avoid over-engineering try/catch chains
- Verify API parameter names with official documentation when updating dependencies

### Testing Strategy

- Focus tests on business logic, not trivial functionality
- Simplify tests when removing core dependencies
- Avoid complex integration tests for basic CLI tools

## Common Mistakes to Avoid

### Code Duplication

- Check for existing functions/components before creating new ones
- Audit codebase regularly for redundant files and functionality
- Replace hardcoded ANSI escape codes with consistent `picocolors` usage

### Dependency Management

- Remove unused dependencies from `deno.json`
- Clean up duplicate entries after adding packages
- Remove unrelated projects/files from workspace

### Over-Engineering

- Avoid unnecessary abstraction layers - use libraries directly when appropriate
- Don't over-complicate error handling in CLI tools
- Remove obvious comments and unnecessary complexity

## Maintenance Guidelines

### Backward Compatibility

- Preserve existing function signatures when refactoring utilities
- Maintain API compatibility during library updates

## Historical Learnings

This section captures lessons learned during development sessions that are specific to this project's context and requirements.

**Important**: Avoid adding universal claims about technologies, languages, or tools. Focus on specific learnings from user prompts, your mistakes and solutions, or best practices discovered.

### Library Migration Best Practices (2025-09-30)

- **Research first**: Always examine official examples and documentation before implementing new library patterns
- **Abstraction evaluation**: Question whether wrapper functions add value - sometimes direct library usage is clearer

### Code Quality Improvement Patterns (2025-10-01)

- **Comment evaluation**: Remove obvious comments that don't add insight beyond what code structure provides
- **Workspace hygiene**: Check for unrelated projects or files that accumulated over time
- **Dependency cleanup**: Remove unused packages from dependency manifests after refactoring

### GenAI Crate Migration (2025-10-02)

- **Version selection**: Use RC versions when latest stable lacks required features (`genai = "0.2.0-rc.2"`)
- **Environment variables**: GenAI crate uses environment variables for authentication - set them before creating client
- **API simplification**: GenAI provides unified API across providers, eliminating need for provider-specific request/response structs
- **Error handling**: Map GenAI errors to application-specific error types for consistent error propagation
- **Model mapping**: GenAI automatically detects provider based on model name patterns (gpt* → OpenAI, claude* → Anthropic)

### Clippy Warning Resolution and Error Handling Simplification (2025-10-02)

- **Anyhow adoption**: Replace custom error enums with `anyhow::Result<T>` for simpler error handling in CLI tools
- **Error context**: Use `anyhow::bail!` for cleaner error propagation
- **Module simplification**: When file becomes simple enough, eliminate dedicated modules entirely

### GenAI Crate Major Simplification (2025-10-03)

- **API key management**: GenAI crate handles environment variables automatically - no manual key management needed
- **Code reduction**: ~200+ lines of provider-specific code reduced to ~50 lines with unified GenAI API

### Rust File Consolidation Best Practices (2025-10-03)

- **Module granularity**: For small projects, prefer fewer files with related functionality grouped together over excessive module separation
- **File consolidation strategy**: Merge modules when they contain only simple structs or single functions that don't warrant separate files

### Unified Packages Monorepo Structure (2025-10-03)

- **Packages directory**: Use `packages/` directory to organize all packages regardless of language (Rust, TypeScript/Deno, etc.)
- **Language-specific structure**:
  - Rust packages: `packages/package-name/` with standard Rust crate structure
  - Deno packages: `packages/package-name-deno/` with deno.json and src/ directory
- **Workspace configuration**: Configure Rust workspace to include specific packages under `packages/` in Cargo.toml members
- **Build commands**: Use `cargo build --bin <name>` for Rust binaries from workspace root
- **Cross-language development**: Keep language-specific config files (deno.json, Cargo.toml) within individual packages

### Deno Monorepo and Lefthook Setup (2025-10-03)

- **Deno workspace**: Use root `deno.json` with `workspace` field to manage multiple Deno packages
- **Lefthook integration**: Install lefthook at root level with `deno task setup` for git hooks across the monorepo
- **Monorepo tasks**: Create tasks in root `deno.json` that operate across all packages (fmt, lint, check, test)
- **Package delegation**: Use `deno task --cwd packages/package-name task-name` to run package-specific tasks from root
- **NodeModulesDir warning**: Only specify `nodeModulesDir` in root deno.json, not in package-level configs
- **Lefthook access**: Provide `deno task lefthook` for easy access to lefthook commands without PATH dependencies

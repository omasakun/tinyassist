## Overview

- Write comments in English if necessary. (even if the prompt is in another language)
- Use `deno` for package management. (don't use `npm` or `npx`)
- Be as concise as possible. Project should be as simple as possible

## Essential Commands

```bash
deno fmt    # Format code
deno lint   # Lint code
deno check  # Type-check code
deno test   # Run tests
```

- Run `deno fmt` first, then `deno lint && deno check` to verify

## File Organization

- Use `kebab-case` for file names and directories

## TypeScript Best Practices

- Use relative imports only when importing from the same directory
- Use `undefined` instead of `null` unless absolutely necessary (use `null` only to set database values to `NULL`)
- Use `??` operator for default values instead of `||`
- Rely on TypeScript's type inference where possible - avoid explicit type annotations when types can be automatically
  inferred

## Testing Setup

- **Focus on business logic**, not trivial code

## Common Pitfalls to Avoid

- **Check if there are any existing functions/components/utilities** before creating new ones.
- **Don't duplicate utility functions** - place reusable utility functions in `src/utils.ts`
- **Be as concise as possible. Project should be as simple as possible** - remove unnecessary complexity and obvious
  comments

## Update Instructions

- Always update this file with new learnings and best practices after completing tasks or encountering issues.
- **Knowledge accumulation** - When encountering new patterns, errors, or solutions, automatically add learnings to
  relevant sections
- **Document mistakes** - Record common mistakes and their solutions in appropriate sections to prevent repetition
- **Update best practices** - When discovering better approaches through official documentation or testing, update
  existing guidelines

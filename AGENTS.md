# AGENTS.md

These defaults apply across the repository unless a task or language-specific section overrides them.  
If a task explicitly asks for something else, follow the task and note the deviation.

## Global Rules

### Always
- Preserve existing behavior unless the task explicitly requires a change.
- Make the smallest correct change possible unless the task explicitly requires more.
- Run the relevant linter and type checker on changed files before finishing.

### Ask First
- Adding new third-party dependencies.
- Changes touching multiple files or public APIs.
- Any modification to `AGENTS.md`.

### Never
- Modify unrelated files.
- Rewrite/reformat code outside explicit scope.
- Update dependencies unless required by the task.
- Add tests unless explicitly asked.
- Swallow exceptions or add silent error handling.

## Core Principles

- Prefer composition and explicit dependency injection.
- Keep code readable and maintainable for humans and future agents.
- Flatten control flow; avoid deep nesting.
- Separate pure logic from I/O and side effects.

## Workflow & Verification

- Prefer single-file checks when sufficient and faster.
- Run the full test/lint suite only when explicitly requested or before marking the task complete.
- If tests already exist, run the relevant suite before coding to establish a baseline and after coding to catch regressions.
- Ensure code passes all currently present tests.

## Logic & Control Flow

### Do
- Flatten control flow: prefer at most 2 levels of nesting, 3 absolute max. Do not count function/method declarations or equivalent structural blocks toward nesting depth.
- Use guard clauses and fail fast.
- Invert conditions when it improves readability.
- Evaluate specific cases first; leave default behavior at the bottom.
- Extract small helpers when they meaningfully flatten the main path.

### Don’t
- Bury logic in deep conditional trees.
- Use trailing `else` branches when an early return/continue/break keeps code flatter.
- Trade readability for cleverness.

## Architecture & Design

### Do
- Prefer the smallest correct change.
- Avoid long parameter lists; if a function/method needs more than 3–4 arguments, group related inputs into logical structures.
- Prefer explicit domain models, typed structures, or well-defined data containers over massive unstructured maps/objects.
- Use composition + explicit constructor or parameter injection, especially in orchestration/coordinating layers.
- Use interfaces, abstract base types, or equivalent language-native contract mechanisms when components need to be swappable.
- Split files, classes, modules, or functions before they violate single responsibility.
- Keep modules/files tightly scoped and cohesive.
- Keep pure logic separate from network, file I/O, database access, and other side effects.
- Extract and reorganize instead of blindly appending to growing structures.

### Don’t
- Add abstraction layers unless they clearly reduce real complexity.
- Build deep inheritance hierarchies.
- Create god classes, god files, god modules, or god functions.
- Introduce DI containers or service locators without a clear, justified need.

## Types, Contracts & Documentation

### Do
- Prefer the language’s modern native type/contract features where available.
- Use generic/advanced typing features only when they add clear value.
- Model complex state with explicit, well-defined structures instead of anonymous bags of data.
- Add precise type annotations, interface definitions, or equivalent contracts on public APIs and non-trivial values where the language/tooling supports them.
- Write concise docstrings/comments/docs for public APIs when they clarify intent, side effects, or invariants.
- Add comments explaining why for complex or non-obvious decisions.

### Don’t
- Overuse generics, indirection, or type machinery without clear payoff.
- Use weakly structured catch-all types for data that has a real schema.

## Reliability, I/O & Errors

### Do
- Validate inputs at boundaries.
- Make failure modes explicit and fail fast.
- Use explicit timeouts on network/external calls.
- Clean up resources deterministically.
- Add contextual logging at key boundaries and failure points; never log secrets, keys, or PII.
- Handle transient failures gracefully with retries only when appropriate.

### Don’t
- Swallow exceptions/errors silently.
- Block async runtimes, event loops, or latency-sensitive execution paths with avoidable sync I/O or heavy CPU work.

## Dependencies & Environment

### Do
- Inject configuration and secrets via environment variables or config files; never hardcode them.
- Prefer standard library or built-in solutions when sufficient.
- Use established ecosystem standards when they clearly reduce boilerplate or improve maintainability.
- Add third-party dependencies only when they provide clear, ongoing value in maintainability or functionality.

### Don’t
- Add dependencies just because they are popular or convenient.

## Testing & Maintenance

### Do
- When explicitly asked to write tests, target only complex, high-risk logic.
- Plan tests as a distinct, separate step.
- Mock external dependencies and I/O in tests.
- Test pure logic where possible.

### Don’t
- Write new tests unless explicitly asked.
- Pester the user about missing tests or coverage.
- Practice implicit TDD or bundle haphazard tests with feature code.
- Refactor purely for aesthetics.
- Change behavior silently.
- Modify files outside the explicit scope of the task.

## Python

### Setup & Commands
- Use `uv` exclusively for dependency and environment management.
- Manage configuration strictly via `pyproject.toml`.
- Always work inside `.venv`; create it with `uv venv` if missing.
- Sync dependencies with `uv sync`.
- Run lint/auto-fix with `uv run ruff check --fix`.
- Run formatter with `uv run ruff format`.
- Type-check with `uv run ty check`.
- Use `pytest` for all test suites.

### Syntax & Patterns
- Rely on modern Python 3.12+ features.
- Prefer native annotations (`list[str]`, `dict[str, int]`, `X | Y`) over legacy `typing` forms when possible.
- Prefer `ABC` and `@abstractmethod` for swappable components and explicit contracts.
- Use `@dataclass` or Pydantic to encapsulate related data/execution contexts instead of functions with massive signatures.

### Don’t
- Use legacy `typing` forms when native annotations are sufficient.
- Prefer `Protocol` over `ABC` unless structural typing is explicitly needed.

## Rust
To be added.

## Frontend
To be added.

# AGENTS.md

This file provides guidance to AI coding agents working with code in this repository.

## What This Is

Alfrusco is a Rust library (crate) for building [Alfred](https://www.alfredapp.com/) macOS workflow script filters. It
provides type-safe abstractions over Alfred's JSON protocol, supporting both sync and async workflows.

## Common Commands

```bash
make test                    # Run all tests (uses cargo-nextest, auto-installed)
make build                   # Debug build
make lint                    # Format + clippy
make ci                      # Full CI pipeline: fmt-check, lint, build, doc-check, test
make run-example-<name>      # Run an example (e.g. make run-example-sleep)
```

**Run a single test file:**

```bash
cargo nextest run --test clipboard_tests --test-threads 1
```

**Run tests matching a pattern:**

```bash
cargo nextest run --filter-expr "test(filter_and_sort)" --test-threads 1
```

All tests must run with `--test-threads 1` because clipboard tests use a shared system resource.

## Architecture

**Execution flow:** `execute(provider, runnable, writer)` sets up a `Workflow` from a `ConfigProvider`, runs the user's
`Runnable`/`AsyncRunnable` implementation, then serializes the `Response` (containing `Item`s) as JSON to the writer. If
the workflow has a filter keyword set, items are fuzzy-matched and sorted before output.

**Key traits:**

- `Runnable` / `AsyncRunnable` — User implements one of these. Has an associated `Error: WorkflowError` type. The `run`/
  `run_async` method receives a `&mut Workflow` to populate with items.
- `WorkflowError` — Errors that can convert themselves into an Alfred `Item` for display. Automatically shows errors in
  Alfred's UI.
- `ConfigProvider` — Strategy for resolving workflow config (bundle ID, directories). `AlfredEnvProvider` reads Alfred
  env vars; `TestingProvider` uses temp directories for tests.

**Key types:**

- `Item` — Builder pattern. Represents one row in Alfred's results. Supports `subtitle()`, `arg()`, `icon_*()`,
  `modifier()`, `matches()`, `boost()`, `sticky()`, etc.
- `URLItem` — Specialized `Item` for URLs. Automatically adds Cmd (Markdown) and Alt (rich text) modifier keys for
  clipboard operations. Converts to `Item` via `From`.
- `Workflow` — Holds config and the response being built. Methods: `append_item()`, `prepend_item()`,
  `set_filter_keyword()`, `cache()`, `rerun()`, `data_dir()`, `cache_dir()`.
- `Response` — The complete Alfred JSON output. Contains items plus control fields (rerun interval, cache settings).
- `BackgroundJob` — Runs shell commands without blocking Alfred's UI. Tracks execution status (
  Fresh/Stale/Running/Failed) with automatic retry.

**Sorting/filtering:** When a filter keyword is set on the `Workflow`, `filter_and_sort_items()` uses `fuzzy-matcher`'s
`SkimMatcherV2`. Items with `sticky()` always appear first. The `boost()` value is added to match scores to influence
ranking.

**Module visibility:** Most modules are private and re-exported through `lib.rs`. Three modules are public: `clipboard`,
`config`, `internal_handlers`.

## Testing

- Integration tests live in `tests/`. Unit tests are inline in source files.
- Use `config::TestingProvider` (with `tempfile`) to create test workflows without Alfred environment variables.
- Tests verify behavior by serializing to JSON and checking the output structure.

## CI

Runs on macOS only (required for clipboard and Alfred integration). Pipeline: fmt-check, clippy, build, doc-check, test,
coverage upload to Codecov.

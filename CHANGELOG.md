# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **`alfrusco::ui`, typed builders for Alfred UI `ui.json` files** (Alfred 6). It covers
  the top level (`bundleid`, `window`, `variables`, `variablesconfig`, `templates`,
  `colours`, `pulse`, `log`, `debug`, `iconpath`), `Window` and its toolbar, `View` with
  the stack, border, grid, inline, form and flex layouts and component `Constraints`,
  and every documented component: label, button, checkbox, picker, textfield,
  passwordfield, textarea, markdownarea, progress, imageview, dataview (cell, rows,
  filter and `modify` edits), webkitview, template and file. Events and triggers,
  `confirm` dialogs, actions (`log`, `script`, `json`, `function`, with `args`,
  `variables`, `flattenarrayargs` and `async`) and every JSON action command are
  modelled too, along with `Colour` (hex, `rgb()`, `hsv()`, named colours, light/dark
  pairs and `null`), `Font`, `Image`, menus, shortcuts and `Placeholder` helpers such as
  `Placeholder::value("search", "text")` for `{value:search.text}`.
- Every `ui` struct is `#[non_exhaustive]` and has a `set(key, value)` escape hatch that
  replaces the typed key of the same name, and `Component::raw` takes any JSON, so
  properties Alfred adds or renames can be used before the crate models them.
- Action and `finish` arguments and `variablesconfig` defaults take any JSON value, as
  Alfred 6.0.3 does. Doc comments record what Alfred 6.0.3 was observed to do where it
  differs from Alfred's docs: every `variablesconfig` key reaches scripts whatever
  `scriptenv` says, and `finish` passes output on only when the UI opens inside Alfred.

## [0.5.0] - 2026-11-26

### Breaking
- **`WorkflowConfig` has a new public field, `theme_window_width`** (read from
  `alfred_theme_window_width`). Code that builds `WorkflowConfig` with a struct literal must
  add the field, for example `theme_window_width: None`. Code that gets its config from a
  `ConfigProvider` such as `AlfredEnvProvider` needs no change.

### Added
- **Alfred 6 Script Filter keys.** `Response::strict_mods` / `Workflow::strict_mods` write
  `strictmods`, so modifier combinations an item doesn't define act as `valid: false`.
  `Response::icon_resize` / `Workflow::icon_resize` write `iconresize`, and `Icon::resize`
  writes the per-icon `resize`, both taking the new `IconResize` enum (`Fit`, `Fill`,
  `Stretch`, `Centre`).
- **Item `type`.** `Item::item_type` with the new `ItemType` enum (`Default`, `File`,
  `FileSkipCheck`) marks an `arg` as a file path, so Universal Actions, the File Buffer and
  Quick Look work on it.
- **Universal Actions.** `Item::action` sets values whose type Alfred infers;
  `Item::universal_action` takes the new `UniversalAction` builder with explicit `text`,
  `url`, `file` and `auto` values.
- **Session variables.** `Response::var` / `Response::unset_var` / `Workflow::var` write
  the top-level `variables` object. The simulator follows them too: a Conditional that
  tests `{var:name}` reads the session variable when the actioned item doesn't set `name`.
- **Grid View `preselect`.** `Response::preselect` / `Workflow::preselect`.
- **Icon constructors.** `Icon::from_image`, `Icon::for_file` (`fileicon`) and
  `Icon::for_filetype`, plus `icon_for_file` on `Item`, `Modifier` and `URLItem`.
- **`TextView`** for scripts that populate Alfred's Text View: `response`, `footer`,
  `actionoutput`, `behaviour` (`TextViewMode`, `ScrollMode`, `InputFieldMode`), `rerun` and
  session `variables`.
- **`JsonConfig`** for the `alfredworkflow` JSON a Run Script action or JSON Utility writes:
  `arg`, `config` overrides and `variables`.
- **`WorkflowConfig::alfred_major_version()`** and the `theme_window_width` field
  (`alfred_theme_window_width`).
- `Arg` now converts from `&str`, `String`, `Vec<_>` and arrays.

### Changed
- Builds cleanly under clippy on Rust 1.99 and 1.100: `AsyncRunnable` allows the
  `double_must_use` lint that `#[async_trait]` triggers, and two simulator expressions are
  rewritten for the `map_unwrap_or` lint. No behavior change.

### Fixed
- The simulator uses the selected modifier's argument, variables and validity when routing a
  result. An undefined combination with `strictmods` set behaves like `valid: false`.
- Cache TTLs are written as whole seconds. Fractional rerun intervals retain sub-millisecond
  precision in Script Filter and Text View output.
- Missing workflow directories no longer hide the running Alfred version, build or theme width.

## [0.4.5] - 2026-11-01

### Changed
- **Background job checks look up only the job's own process.** `run_in_background`
  previously built `System::new_all()` and refreshed every process on the host, twice per
  call, even when no PID file existed. It now skips the lookup entirely when there is no
  PID file, refreshes only the job's PID with no optional process data, and does it once
  per call. On a Linux host with about 950 processes (fresh process per run, debug build),
  a first call with no PID file went from 846-955 ms to under 1 ms. Running-job detection,
  zombie handling, retry of exited jobs and deduplication are unchanged.

### Fixed
- **Fast-exiting background jobs no longer stay stuck at `running`.** The `running` status
  was written after spawning the job, so a job that finished first had its final status
  overwritten. The status is now written before the spawn; a spawn failure still records
  `failed`.
- Background job tests poll for the expected status instead of sleeping a fixed 500 ms,
  and a retry test no longer depends on file-timestamp clock resolution on Linux.
- Test assertions updated for new clippy lints on Rust 1.99. No behavior change.

## [0.4.4] - 2026-07-19

### Added
- **`URLItem::link_action_items()`** — generates the standard set of link-action rows
  for a URL: an Open row; Markdown + Rich Text copy rows for each unique link text
  (short title, title, `"{short} — {title}"` combined form, explicit long title —
  deduplicated); and a plain Copy URL row (`copytext`). Rows carry item-level
  command variables and stable uids. Use to build link submenus when modifier keys
  aren't enough.
- **Stable auto-uids under `skip_knowledge`** — responses flagged `skip_knowledge(true)`
  assign a deterministic, title-derived uid to any item lacking one at write time, so
  the user's selection survives `rerun` cycles (e.g. while a background job re-renders
  results every second). Only done under `skipknowledge`, where uids cannot affect
  ranking; explicit uids are never modified.

### Fixed
- **Fuzzy filtering is now case-insensitive.** The matcher's default "smart case"
  became case-sensitive whenever the query contained an uppercase letter, so
  mixed-case queries (e.g. `Nasa Hq`) failed against all-caps candidates
  (e.g. `NASA HQ`).

## [0.4.3] - 2026-07-19

### Added
- **`Item::pin_to_bottom(bool)`** — pin an item to the bottom of results, exempt from
  fuzzy filtering (the counterpart to `sticky`, which pins to the top). Sorting now
  assembles three zones: sticky (top), filtered results (middle), bottom-pinned (last).

### Fixed
- **Background job status rows no longer disturb list navigation.** The item emitted by
  `run_in_background` previously had no `uid` and participated in fuzzy filtering — its
  "running for Ns" subtitle changed its match score every rerun, so it flickered in and
  out of results at a score-dependent position, reshuffling the list under the user's
  selection. It is now bottom-pinned with a stable `uid` (`background-job:{id}`), so it
  stays visible with live progress while Alfred preserves the selection across reruns.
- Clippy `useless_borrows_in_formatting` errors on Rust ≥1.97 (redundant `&` in two
  `format!` args in `url_item.rs`) that broke CI. No behavior change.

## [0.4.2] - 2026-07-18

### Fixed
- **External Trigger resolution used the wrong config key** — `resolve_external_trigger` and
  `reachable_kinds` looked up `triggerid` on `CallExternalTrigger` nodes, but real Alfred plists
  use `externaltriggerid` on the *output* side (`triggerid` belongs only to the `ExternalTrigger`
  *input* node). Every External-Trigger drill-in in a real workflow therefore resolved to
  `DeadEnd`/unreachable. Undetected because the test fixture used the wrong key on both sides;
  the fixture and its assertion are corrected too.

### Added
- **`copytext` clipboard command** — `ALFRUSCO_COMMAND=copytext` with a `TEXT` env var copies
  arbitrary plain text to the clipboard, alongside the existing `markdown`/`richtext` link
  commands. Backed by the public `copy_text_to_clipboard()` helper.

## [0.4.1] - 2026-06-30

### Changed
- **Faithful routing in dynamic audit** — the audit now models Alfred's actual routing semantics:
  - Evaluates Conditional nodes' `matchstring`/`matchmode`/`matchcasesensitive` against the actioned item's `arg` (instead of heuristically assuming "carries variables ⇒ navigation").
  - Resolves External Trigger re-entry (`CallExternalTrigger` → matching `ExternalTrigger` input → Script Filter) as a legitimate drill-in.
  - `RanScript` and `OpenedUrl` outcomes are **no longer flagged** — they are intentional act-and-exit terminals.
  - **Only `DeadEnd`** (a matched conditional branch whose output connects to a non-existent object or has no connection) is an error.

### Added
- `ObjectKind::CallExternalTrigger` and `ObjectKind::ExternalTrigger` — new graph node kinds.
- `WorkflowGraph::external_trigger_uid()` — resolve a trigger ID to its input node UID.
- `MatchMode` enum with `evaluate()` — faithful matchmode evaluation for all 7 Alfred conditional modes.
- `Condition` struct — parsed conditional branch configuration.

## [0.4.0] - 2026-06-28

### Added
- **Headless workflow simulator** (`alfrusco::simulator`) for deterministic, UI-less testing of workflow navigation:
  - `WorkflowGraph` — parse an `info.plist` and analyze its object/connection graph (reachability, keyword lookup, navigation audit).
  - `Simulator` — invoke a workflow from a directory, in-process (`run_in_process`) or as a subprocess (`invoke` / `invoke_script_filter`), with builder overrides for cache/data/binary.
  - `Screen` / `ScreenItem` / `ActionResult` — inspect rendered items and assert action routing (`assert_drills_in`, `assert_opens_url`, …).
  - Static and dynamic navigation audits that detect drill-in items misrouted to a Run Script / Open URL instead of another Script Filter.
- **`alfrusco-simulator` CLI** — `audit` and `walk` subcommands for auditing a workflow's navigation graph.
- **Self-locating configuration** — `AlfredEnvProvider` now resolves config in three tiers (env vars → infer from the binary's location + adjacent `info.plist` → ephemeral temp directories with an STDERR warning), so a workflow binary no longer panics when run without Alfred's environment variables (terminal, cron, CI, tests).
- `examples/menu.rs` — a generic hierarchical-menu workflow demonstrating the drill-in pattern.

### Changed
- `AlfredEnvProvider` no longer exits on missing Alfred environment variables; it falls back to inferred or temporary configuration (see Self-locating configuration).

### Dependencies
- Added `plist` for parsing Alfred `info.plist` files.

## [0.3.0] - 2026-01-08

### Added
- Boost feature for controlling item ranking in filtered results
- `Item::boost(value)` method to adjust fuzzy match scores
- Boost constants for common priority levels:
  - `BOOST_SLIGHT` (25) - Subtle preference
  - `BOOST_LOW` (50) - Minor preference
  - `BOOST_MODERATE` (75) - Noticeable preference
  - `BOOST_HIGH` (100) - Strong preference
  - `BOOST_HIGHER` (150) - Very strong preference
  - `BOOST_HIGHEST` (200) - Effectively guarantees top ranking

## [0.2.1] - 2025-09-30

### Added
- `.var()` method support for URLItem - allows setting custom workflow variables on URL items

## [0.2.0] - 2025-09-30

### Added
- Custom `.arg()` support for URLItem - allows URLItems to be both web links and Alfred workflow navigation items
- Complete API consistency: `cache()` and `rerun()` methods now available directly on `Workflow`
- New example demonstrating URLItem with custom args for workflow navigation

### Changed
- Improved API consistency - all response methods now available on `Workflow` directly
- Enhanced documentation with unified API patterns

## [0.1.9] - 2025-06-10

### Fixed
- Clipboard test contention issues by serializing test runs
- Various clippy warnings and code cleanup
- Improved clipboard test reliability

## [0.1.8] - 2025-06-09

### Changed
- Replaced `rust-clipboard` dependency with `arboard` for better clipboard support
- Improved clipboard functionality reliability

## [0.1.7] - 2025-06-09

### Added
- Enhanced background job tracking with detailed status messages
- Background job failure tracking and automatic retry logic
- Workflow command system for internal operations
- Item sticky concept to prevent sorting of specific items
- Data and cache directory helper methods
- Ability to unset workflow variables
- URLItem subtitle customization support

### Changed
- Enhanced background job status reporting with timestamps and duration
- Improved error reporting for missing environment variables
- Better logging initialization error handling

### Fixed
- Background job status tracking and display
- Log initialization failures that were previously swallowed
- Compatibility with newer sysinfo versions
- Duration serialization in responses

## [0.1.2] - 2024-09-15

### Added
- Initial stable release with core Alfred workflow functionality
- Basic Item and URLItem support
- Response caching and rerun capabilities
- Background job execution
- Clipboard integration
- Fuzzy filtering and sorting
- Comprehensive error handling

[Unreleased]: https://github.com/adlio/alfrusco/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/adlio/alfrusco/compare/v0.4.5...v0.5.0
[0.4.5]: https://github.com/adlio/alfrusco/compare/v0.4.4...v0.4.5
[0.4.4]: https://github.com/adlio/alfrusco/compare/v0.4.3...v0.4.4
[0.4.3]: https://github.com/adlio/alfrusco/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/adlio/alfrusco/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/adlio/alfrusco/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/adlio/alfrusco/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/adlio/alfrusco/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/adlio/alfrusco/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/adlio/alfrusco/compare/v0.1.9...v0.2.0
[0.1.9]: https://github.com/adlio/alfrusco/compare/v0.1.8...v0.1.9
[0.1.8]: https://github.com/adlio/alfrusco/compare/v0.1.7...v0.1.8
[0.1.7]: https://github.com/adlio/alfrusco/compare/v0.1.2...v0.1.7
[0.1.2]: https://github.com/adlio/alfrusco/releases/tag/v0.1.2

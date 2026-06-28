# PROGRESS — alfrusco simulator v0.5.0: faithful routing precision

Branch: `feat/simulator-routing-precision` (off `main` @ v0.4.0). Do NOT merge.
Evidence-required: check a box ONLY after pasting the milestone's behavioural evidence below.

## Milestones

- [x] **R1 — Matchstring-faithful conditional routing.** Evaluate `conditions[*]`
      (matchstring + matchmode + matchcasesensitive) against the actioned item's `arg`/`{query}`;
      follow the matched branch, else the `else` output. Document the matchmode mapping table.
      Evidence: arg→branch routing unit tests on a generic conditional fixture.
- [x] **R2 — External Trigger drill-in resolution.** `callexternaltrigger` → matching
      `trigger.external` input (by trigger id) → Script Filter == drill-in. Generic
      `menu_external_trigger` fixture. Evidence: reachability/action report `DrilledIn`; audit clean.
- [ ] **R3 — Terminal classification + dead-end-only error.** DrilledIn / OpenedUrl / RanScript /
      TypedAutocomplete = OK; DeadEnd (dangling matched-branch output) = the ONLY hard error. Remove
      "nav→RunScript ⇒ error". Evidence: 3-fixture matrix — flags ONLY the dangling-loopback fixture.
- [ ] **R4 — Acceptance matrix + README routing-model docs + polish.** README "How the audit models
      routing" section; `make ci` green; IP-clean grep empty. Evidence: ci + grep + audit matrix.

## Validation gates (DX Reviewer / Code Simplifier / Test Coverage, per milestone)

- [ ] R1 gate — DX:[ ] Simplify:[ ] Coverage:[ ]
- [ ] R2 gate — DX:[ ] Simplify:[ ] Coverage:[ ]
- [ ] R3 gate — DX:[ ] Simplify:[ ] Coverage:[ ]
- [ ] R4 gate — DX:[ ] Simplify:[ ] Coverage:[ ]

## Context notes (read before starting)

- **Why this loop exists:** v0.4.0's dynamic audit over-flags. It assumes any item carrying
  `variables`/`autocomplete` that doesn't reach a Script Filter is broken navigation. That false-flags
  (a) legitimate **act-and-exit** items (route to a Run Script / Open URL on purpose) and (b) legitimate
  **External-Trigger drill-ins** (re-enter a Script Filter via `callexternaltrigger`, not a direct
  connection). The ONLY real bug is a **dead-end** (the matched conditional branch's output is
  unconnected → actioning the item silently does nothing).
- **Intent is in the plist:** conditionals route on `arg`/`{query}` via `matchstring`+`matchmode`. A
  `loopback` branch = drill-in; a `run` branch = act-and-exit. Read them; don't guess.
- **Fit the existing `ActionResult` enum** (DrilledIn/OpenedUrl/RanScript/TypedAutocomplete/DeadEnd).
- **IP-clean:** generic menu fixtures only; never commit a real internal workflow plist.

## R1 Evidence

### MatchMode mapping table (implemented in `src/simulator/graph.rs`)

| Value | Mode           | Description                                            |
|-------|----------------|--------------------------------------------------------|
| 0     | Is             | Exact equality (empty pattern → "is empty" semantics)  |
| 1     | IsNot          | Not equal (empty pattern → "is not empty" semantics)   |
| 2     | Contains       | Substring match                                        |
| 3     | DoesNotContain | No substring match                                     |
| 4     | StartsWith     | Prefix match                                           |
| 5     | EndsWith       | Suffix match                                           |
| 6     | MatchesRegex   | Regular expression match                               |

### Test output (16 conditional routing tests, all pass)

```
running 16 tests
test conditional_case_insensitive_match ... ok
test conditional_routes_loopback_arg_to_script_filter ... ok
test conditional_routes_unmatched_arg_to_else_branch ... ok
test conditional_routes_url_arg_to_open_url ... ok
test conditional_starts_with_partial_url ... ok
test matchmode_contains ... ok
test matchmode_does_not_contain ... ok
test matchmode_ends_with ... ok
test matchmode_from_integer_roundtrip ... ok
test matchmode_is_empty_check ... ok
test matchmode_is_exact ... ok
test matchmode_is_not ... ok
test matchmode_regex ... ok
test matchmode_starts_with ... ok
test parses_conditions_from_conditional_node ... ok
test parses_source_output_uid_from_connections ... ok
test result: ok. 16 passed; 0 failed; 0 ignored
```

### Key routing tests proving correct branch selection

- `arg="loopback"` → condition[0] matches (matchmode=Is) → routes to SF-SUB-001 → **DrilledIn**
- `arg="https://example.com"` → condition[1] matches (matchmode=StartsWith "http") → routes to ACTION-URL-001 → **OpenedUrl**
- `arg="run"` → no condition matches → else branch → routes to RUN-SCRIPT-001 → **RanScript**
- `arg="LOOPBACK"` → case-insensitive Is match → routes to SF-SUB-001 → **DrilledIn**

### `make ci` status: ✅ GREEN (303 tests pass)

## R2 Evidence

### External Trigger fixture graph

```
SF-MAIN-001 → COND-001 →(loopback)→ CALL-TRIGGER-001 [callexternaltrigger, triggerid="sub-menu-trigger"]
                                          ↓ (resolved by trigger ID)
                                     EXT-TRIGGER-001 [trigger.external, triggerid="sub-menu-trigger"]
                                          ↓
                                     SF-SUB-001 → ACTION-URL-001
                    →(else)→ ACTION-URL-001
```

### Test output (8 external trigger tests, all pass)

```
running 8 tests
test audit_navigation_clean_for_external_trigger_workflow ... ok
test external_trigger_drill_in_reports_drilled_in ... ok
test external_trigger_else_branch_opens_url ... ok
test external_trigger_uid_resolution ... ok
test parses_call_external_trigger_node ... ok
test parses_external_trigger_node ... ok
test reachable_kinds_traverses_external_trigger ... ok
test reaches_script_filter_via_external_trigger ... ok
test result: ok. 8 passed; 0 failed; 0 ignored
```

### Key routing tests proving External Trigger drill-in

- `arg="loopback"` → COND-001 condition[0] matches → CALL-TRIGGER-001 → resolves triggerid "sub-menu-trigger" → EXT-TRIGGER-001 → SF-SUB-001 → **DrilledIn**
- `arg="https://example.com"` → COND-001 no match → else branch → ACTION-URL-001 → **OpenedUrl**
- `reachable_kinds("SF-MAIN-001")` includes `ScriptFilter` (via external trigger chain) ✅
- `reaches_script_filter("SF-MAIN-001")` = true ✅
- `audit_navigation(&[])` returns 0 errors ✅

### `make ci` status: ✅ GREEN (311 tests pass)

## Iteration log

- 2026-06-28: R1 implemented — MatchMode enum, Condition struct, sourceoutputuid parsing,
  conditional evaluation in resolve_action. 16 new tests, all pass. `make ci` green (303 tests).
  IP-clean grep empty. Commit: `74600e8`.
- 2026-06-28: R2 implemented — CallExternalTrigger + ExternalTrigger ObjectKind variants,
  external_trigger_uid() lookup, resolve_external_trigger() in action routing,
  reachable_kinds() follows trigger boundaries. Generic fixture + 8 new tests, all pass.
  `make ci` green (311 tests). IP-clean grep empty. Commit: `f600031`.

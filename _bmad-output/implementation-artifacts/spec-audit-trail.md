---
title: 'Add audit trail'
type: 'feature'
created: '2026-08-22'
status: 'in-review'
baseline_commit: 'a0692497d9013aff58115ccbeb8810b609c84bec'
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The system currently has no durable record of who performed important actions, so administrators cannot review when records were added, changed, deleted, exported, or backed up.

**Approach:** Add an append-only SQLite audit log, record successful authenticated mutations and important operational actions, expose the entries through a protected command/API, and add an Audit Trail section in the application navigation.

## Boundaries & Constraints

**Always:** Record only successful actions; use UTC timestamps; identify the single current administrator as `Admin`; include action type, entity, optional record ID, concise human-readable summary, and timestamp; never store passwords, session tokens, or full sensitive payloads; keep audit entries newest-first and cap the returned list at 500 entries; make the audit log available in both native and demo modes.

**Ask First:** None.

**Never:** Do not add audit-entry editing, deletion, or clearing controls; do not audit read-only list/detail/report refreshes; do not change existing record behavior or expose secrets in the UI.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Successful mutation | Authenticated create, update, or delete command | Data change and one matching audit entry are committed | If the transaction fails, neither the data change nor audit entry is recorded |
| Audit listing | Authenticated request | Newest audit entries are returned in a stable, capped order | Invalid/expired session returns the existing unauthorized error |
| Empty history | Fresh database or demo state with no events | Audit Trail shows a clear empty state | No error is shown |

</frozen-after-approval>

## Code Map

- `src-tauri/src/db.rs` -- schema migration for the append-only audit table and indexes.
- `src-tauri/src/models.rs` -- serialized audit event model.
- `src-tauri/src/commands.rs` -- audit query command and event recording across successful mutations and operational actions.
- `src-tauri/src/lib.rs` -- register the new Tauri command.
- `src/lib/types.ts` -- frontend audit event type.
- `src/lib/api.ts` -- authenticated audit-list API wrapper.
- `src/lib/demo-api.ts` -- demo audit data and mutation logging behavior.
- `src/components/views/AuditTrailView.vue` -- protected audit history section with empty/loading/error states.
- `src/components/MainApp.vue` -- navigation tab and view routing.

## Tasks & Acceptance

**Execution:**
- [x] Add the audit schema, model, and authenticated list command.
- [x] Instrument successful create/update/delete and important operational commands without recording secrets.
- [x] Add native and demo API support for retrieving and recording events.
- [x] Add the Audit Trail navigation item and newest-first history table.

**Acceptance Criteria:**
- Given an authenticated administrator creates, updates, or deletes a donor, project, documentation record, contribution, expense, category, or documentation expense, when the operation succeeds, then exactly one corresponding audit event is visible in Audit Trail.
- Given an authenticated administrator creates an export or backup, when it succeeds, then the operation appears as an audit event with a concise summary.
- Given a user opens Audit Trail, when audit entries exist, then the page shows timestamp, action, entity, record reference, and summary newest first.
- Given no audit entries exist, when Audit Trail loads, then an empty-state message is displayed.
- Given an expired session requests Audit Trail, when the command runs, then access is rejected using the existing unauthorized flow.

## Verification

**Commands:**
- `npm run build` -- expected: Vue type-check and production build succeed.
- `cargo check --manifest-path src-tauri/Cargo.toml` -- expected: Rust backend compiles with the audit migration and commands.

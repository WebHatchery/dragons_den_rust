# TODO — Dragon's Den

Only repository-level implementation work remains here. Shipped UI polish and
work that depends on an external asset pack are intentionally not tracked in
this backlog.

## Persistence

- Define the save-version history and the legacy shapes that must remain
  readable in `src/save.rs`.
  - Record the current schema version and each supported legacy version.
  - Document fields added, renamed, or removed between supported versions.
  - Add fixtures for each legacy shape before changing the migration code.
- Implement a versioned migration dispatcher in `src/save.rs`.
  - Route each supported version to a dedicated migration step.
  - Fill defaults for fields introduced after that version.
  - Preserve run and persistent progression data while normalizing the save
    to the current `SaveData` shape.
  - Return source-versioned errors for unknown or malformed payloads.
- Add focused migration tests in `src/save/tests.rs`.
  - Verify current-version saves still round-trip unchanged.
  - Verify every supported legacy fixture migrates to the current shape.
  - Verify unknown versions and malformed payloads fail clearly.
- Rewrite a successfully migrated save in the current format and verify that a
  second load no longer invokes a legacy migration.

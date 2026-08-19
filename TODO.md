# TODO — Dragon's Den

Only repository-level implementation work remains here. Shipped UI polish and
work that depends on an external asset pack are intentionally not tracked in
this backlog.

## Persistence

Legacy save migration is out of scope before the first public release; there
are no existing player saves to preserve.

- Verify the current `SaveData` shape saves and loads correctly through the
  toolkit persistence layer.
  - Cover a fresh save round-trip with run and persistent progression intact.
  - Cover offline earnings calculated from the saved timestamp.
  - Cover autosave and explicit save paths using the same current shape.
- Add focused current-save tests in `src/save/tests.rs`.
  - Verify valid current saves round-trip unchanged.
  - Verify malformed or incompatible saves fail clearly.
  - Verify no legacy fixture or migration path is required.
- Make invalid-save recovery explicit in the menu flow.
  - Tell the player the save could not be loaded and ask them to start a new
    game.
  - Keep the existing `New Game` action available after a load failure.

# UAT-7B-04 — Successor upgrade

Publish a successor for the referenced Component. Verify the old Dashboard
reference remains pinned and only that declared, active, published, same-owner
successor is offered by Upgrade. Execute Upgrade once, retain the action receipt,
and verify placement rebinding and finding closure are atomic. Replay the same
idempotency key and verify the stored result is returned without another write.
Restore the reference composition.

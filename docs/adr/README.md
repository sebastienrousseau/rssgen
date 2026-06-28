# Architecture Decision Records

Each numbered file in this directory captures a single architectural
decision: the context that forced the call, the alternatives weighed,
the choice made, and the resulting trade-offs. ADRs are append-only —
new decisions get a new file rather than rewriting existing history.

| # | Title | Status |
| :--- | :--- | :--- |
| [0001](0001-structured-validation-errors.md) | Structured `ValidationError` payload | Accepted (v0.0.6) |
| [0002](0002-time-only-date-stack.md) | Date stack consolidated on `time` (drop `dtt`) | Accepted (v0.0.6) |
| [0003](0003-atom-as-sibling-module.md) | Atom 1.0 as a sibling module, not an RSS subtype | Accepted (v0.0.6) |

New ADRs use the [MADR](https://adr.github.io/madr/) template (a
shortened variant of Michael Nygard's original format).

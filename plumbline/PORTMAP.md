# Port map: the building model against BIMCraft

Every public item of the building model (`crates/world/src/building*.rs` and anything
under `crates/world/src/building/`) is listed here once, as **ported** from BIMCraft or as
**ours**. `tools/portmap.sh` checks it and cannot be passed from memory: a ported row's
line has to hold the symbol it names in BIMCraft's source at the pinned commit, and an
"ours" row has to say what was searched for and where.

BIMCraft: https://github.com/IPedrax/bimcraft at
`f202abd85d1cf165325e2f7b250da0b5e51f5b71` (MIT OR Apache-2.0; used here under MIT).

| item | how | BIMCraft symbol | file:line | searched |
|---|---|---|---|---|

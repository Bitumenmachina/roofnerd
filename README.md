# plumbline (working name)

A 3D estimating platform for high-end residential work. The job is one
entity-component world: every building element is an entity carrying geometry,
condition, cost, schedule, status and provenance. The 3D view, the estimate
sheet, the reports and the schedule are views over that one world. It is being
proven on the building envelope first.

Where things stand is in [STATUS.md](STATUS.md). The model is in [SPEC.md](SPEC.md).

| File | What it holds |
|---|---|
| STATUS.md | Each deliverable: what was run, what it showed, what remains |
| SPEC.md | The world data model and the saved-job format |
| DECISIONS.md | Every build-on, borrow, or write call, with its evidence |
| LICENSES.md | Every linked crate and its license; every candidate's license |
| QUESTIONS.md | Open questions for the owner and the assumption in use meanwhile |
| PROPOSALS.md | Work outside a deliverable. Not built until accepted. |

## Run it

```
cargo test                 # the world-model checks
tools/landscape.sh         # re-run the D0 landscape checks (needs network)
demos/d1.sh                # the D1 demonstration
demos/units.sh             # the units demonstration: 30 in, 25' 6 1/2", stretch-out
demos/d3.sh                # the D3 demonstration: a parapet's four lines, sheets and pounds
tools/licenses.sh          # every linked crate and its license; fails on a non-permissive one
cargo run -p plumbline -- sample job.json
cargo run -p plumbline -- show job.json
cargo run -p plumbline -- set job.json parapet-0024 height "30 in" --status Known --source "detail 3/A-501"
```

A length is typed the way the drawing or the tape gives it (`2.5 ft`, `30 in`, `19 1/2"`,
`25' 6 1/2"`) and is kept in that unit. A length with no unit is refused, never read as feet.

`refs/` is for the owner's reference estimates. Git ignores it. Nothing in it is ever committed.

Licensed under either of Apache License 2.0 or the MIT license, at your option.

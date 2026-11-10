# Proposals

Work outside a deliverable. Nothing here is built until the owner accepts it.

1. **A constraint solver (z3) for cross-axis rules.** Derivation today is direct
   recomputation, which is enough for geometry-to-quantity. A solver earns its place only
   if rules must be solved in both directions at once across cost, schedule and geometry.
   It adds a C++ dependency. Not needed by D1 through D6 as written.
2. **License checking in a pre-commit hook.** `tools/licenses.sh` is run by hand now.
3. **More geometric consistency checks.** D1 checks each plane and each line against the
   planes it names. It does not check planes against each other: two outlines that
   overlap, or a gap between planes that should meet, pass unnoticed.
4. **UUID paths.** Element paths are short readable strings (`plane-0003`). IFC5 examples
   use UUIDs. If jobs are ever merged from several machines, the counter-based paths will
   collide and UUIDs would be needed.
5. **Metric.** The owner ruled that US measurements stay and did not ask for metric. A
   value in mm, cm or m is refused with a message that says it is metric. Taking metric
   would be a fourth and fifth length unit in `units.rs` and a decision about what derived
   quantities are shown in.
6. **Pitch in degrees or percent.** A pitch is entered as rise in 12 (`6:12`, `1/4:12`).
   Degrees and percent are refused by name. Some low-slope drawings give percent and some
   manufacturer data gives degrees; nobody has asked for either.

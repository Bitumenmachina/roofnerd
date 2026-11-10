# Proposals

Work outside a deliverable. Nothing here is built until the owner accepts it.

1. **A constraint solver (z3) for cross-axis rules.** Derivation today is direct
   recomputation, which is enough for geometry-to-quantity. A solver earns its place only
   if rules must be solved in both directions at once across cost, schedule and geometry.
   It adds a C++ dependency. Not needed by D1 through D6 as written.
2. **License checking in a pre-commit hook.** `tools/licenses.sh` is run by hand now.
3. **More geometric consistency checks.** D1 flags a shared line whose planes disagree on
   its height. Outlines that overlap, planes with a clockwise outline, and lines that do
   not touch the plane they name are not checked.
4. **UUID paths.** Element paths are short readable strings (`plane-0003`). IFC5 examples
   use UUIDs. If jobs are ever merged from several machines, the counter-based paths will
   collide and UUIDs would be needed.

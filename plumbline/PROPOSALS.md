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
7. **What BIMCraft's building model has that was not taken in the re-base** (DECISIONS.md
   call 29). Each is a field or a part of its model that was read and left, with the
   reason it would be wanted: curved walls (`arc`); a wall's top following the roof or
   floor above it (`attach_top`, `attach_base`), which a gable-end wall needs; openings in
   a floor (a stair well comes out of a floor's area); a floor's slope and shaped top;
   a room's height, for wall finish and volume; room separation lines; families, with
   their forms, types and formulas, in place of `what` and `size`; a door's swing and a
   window's operation; an opening that rises past its wall's top, cut as a notch;
   the mitred plan outline of runs that join; project parameters.
8. **Three or more run ends meeting at a point with no fitting.** BIMCraft's network says
   nothing where runs meet end to end, however many. Three pipe ends at one point is a tee
   that was not entered, the same as a branch on a run's length. Saying so would be a
   fifth warning, ours.
9. **A fitting's size from its connectors.** A fitting's takeoff line is its kind and
   what it is said to be. Its connectors carry a size; grouping fittings by connector
   size, and a transition by its two sizes, would make the count orderable by size.
10. **Putting the fittings in where a run meets another along its length.** The network
    finds each place. A command could split the run there and place a tee, for the
    estimator to accept.
11. **A cable or a duct that ends at equipment joining through it.** Equipment does not
    join what meets it, as BIMCraft has it, so the two supply trunks off one air handler
    are two networks. Whether a unit's supply connectors are one network is a rule this
    market's estimators would have a view on.

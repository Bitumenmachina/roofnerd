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
   quantities are shown in. A laser set to metres types `7.123m` and is refused the
   same way (QUESTIONS.md Q19).
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

12. **Class adders and bond.** The earlier estimating program in this repository adds
    sales tax and escalation on material, burden on labor and on supervision (each at its
    own rate), general liability on subcontract, and bond on the contract amount
    (`packages/engine/src/recap.ts`). The brief's D5 asks for overhead and profit, and
    those are what is built. A real price needs the rest; none has a default there, and
    none would here.
13. **Production by the crew-day.** A foreman says forty squares a day with a crew of
    six. The earlier program takes a rate that way (`model.ts`, `perManHour`); it needs
    the crew and the hours in a day entered. Here a rate is so many to the man-hour, or
    hours to each.
14. **Pricing the job at a chosen supplier.** The sheet shows the generic price and each
    supplier's side by side; the recap is at the generic price. Choosing one supplier (or
    the best quote for each material) to price the job by is not asked for.
15. **A second material on one element.** An element is made of one material, and its
    lines are either made of that or carry a price of their own. Underlayment on a copper
    roof carries its own price and so is not compared across suppliers.
16. **Equipment and Other as classes of cost.** The earlier program has them; the brief
    names material, labor, fabrication, supervision, subcontract and general conditions.
17. **A Bosch GLM laser over Bluetooth** (Patrick, 2026-11-11: to proposals). A laser in
    keyboard mode types its reading at the cursor and needs nothing from the program. A
    Bosch GLM speaks over Bluetooth and would need the program to listen for it.
18. **Finding lines in a scanned plan or an aerial image**, so a tracing snaps to a
    picture as it does to a vector PDF's own lines (QUESTIONS.md Q18).

19. **Squaring up a scan that lies crooked on its page.** A drawing has a scale and a
    place on the plan and no turn. USDA Plan No. 7213's floor plan lies about a quarter
    of a degree off square on its scan: its 41 ft 3 in dimension line climbs 3 pixels in
    619. Walls traced square by their typed lengths are right; the sheet under them
    drifts by about 2 1/2 in from one end of the house to the other. Two points picked on
    a line known to be level would give the turn.

20. **Foundation, framing, waterproofing and interior woodwork drawn in the model**,
    each with a switch of its own (from Patrick's request of 2026-11-12 for a snapshot
    of each trade: DECISIONS.md call 69, QUESTIONS.md Q23). Today they are estimate
    lines with nothing in the model behind them. A footing and foundation wall under
    each exterior wall; plates, studs and headers in each framed wall; the weather
    barrier under the finish and flashing at openings; base along each room and casing
    at each opening. Each estimate line would take its quantity from what is drawn.


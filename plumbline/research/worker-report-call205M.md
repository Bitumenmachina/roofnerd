# W-205M report (contract 205M), worker on Sonnet 5.5

## Before any code, part 1

### 1. BIMCraft: how a family's form is added, changed, taken away, and how a family is duplicated
Read `crates/core/src/commands.rs` `fn family` (10588), `patch_part` (10879), `take_part` (10906), `part_at`, `family_changes` (10852), `family.rs` `problems`.
- One command, `family`, addresses a family by name. `forms` replaces the whole list (`keep_parts` keeps ids); `form` {id|guid, ...fields} merges the fields over that form (a null takes a field away; its place, id and GUID stay); `delete_form` [ids] takes forms away by id; a form with no id is added by the `forms` list.
- All of it is worked on a CLONE (`let mut f = was.clone()`); then `f.problems(..)` is asked of the whole result; any error refuses and the stored family is untouched (`return Err`), warnings come with the reply. `dry_run` runs the same and returns `changes` (parameters/types added, removed, changed; forms replaced/patched/deleted) without storing. So: check the whole after the change, refuse leaving it as it was, say what changed.
- Rename is refused onto a name another family has (`there's a family X already`), and carries the elements and the library along.
- There is NO duplicate-a-family command. A family is copied by reading it (`save` to a file, or reading it back) and sending it again under another `name` (a `name` with no family of that name starts from `Family::default()`); `like` exists only for text/dimension/etc. types.
- Taken (the pattern, not the code): change a copy, check the whole, refuse and keep the old, return a record of what changed (`family_changes`). Block is addressed by its number (the contract and call 205 say so; BIMCraft's id/GUID has no counterpart in a shape, which has no ids). Not taken: ids/guids, `keep_parts`, JSON merge, dry_run.
- PORTMAP: each new public item gets a row, `ours`, with the search that finds nothing (`grep -rn 'fn keep_shape_as' crates/core/src`: nothing and so on), and the pattern said in the row.

### 2. What the model does today when a job's shape has a starter's name
Run with my own build (`plumbline import shapes`, a one-block "base cabinet" into a job with the starter chest):

    bc.csv (CSV): 1 row under its headings / set: 1 / base cabinet: added: 1 blocks
    plumbline items j.json "Base cabinet, 36 in"  ->  drawn as a base cabinet / its shape is 1 block (block 1, timber: across -18 in to 18 in; back 0 in to 24 in; up 0 in to 34.5 in)
    plumbline export shapes -> the base cabinet's rows are the job's one row, not the starter's three

`shape::find(job, name)` looks in the job first (`shape.rs`), so in the model (`items`, `export shapes`) the job's shape takes the starter's place; the window's `forms::blocks` -> `starter_blocks` ignores the job's, so the window draws the starter's: two meanings for one name (as call 205 point 3 says). `tables::shapes_in` lets it in (`None =>` branch only skips it when its blocks equal the starter's: counted "same").

Tests and demonstrations that rely on it: **none**. Searched every caller of `shapes_in` / `import shapes` (`crates/world/tests/shapes_job.rs`, `crates/cli`, `demos/shapes.sh`) and every starter name (base cabinet, bathtub, bed, chair, desk, lavatory, outdoor unit, sink, sofa, table, tank, wardrobe, water closet): the shapes brought in by tests are `wall-hung closet`, `ok shape`, `area shape`, `stuff shape`, `paint shape`, `sum shape`, `ok`, `bare`, `wide`, `tall`, `flip`, `gate`, `x`, `bad shape`, none a starter's. Two tests bring in a starter's OWN rows (`shapes_job.rs` line 100: `STARTER_SHAPES`, "same 13"; line 123: the vanity's written-out rows, "same 1") and demos/shapes.sh step 5 does too (`library/shapes.csv`, "the same as the job already has: 13"): those say the SAME blocks, which stays allowed and counted as the same, carrying nothing. So the refusal is only of a starter's name with different blocks. A saved JOB FILE that already holds such a shape is not refused on opening (the saved job's format and every old job opening as it did are the contract's; held for a ruling, below).

### 3. The changes as I will add them (a new file `crates/world/src/editing/shapes.rs`, `pub use`d from `editing`)
Each is all-or-nothing (`all_or_nothing`, as `item_size`), reads the shape as it will be, checks it as on the way in (`Shape::compile`: formulas, lengths, inside the box at the three sizes), and gives back a `ShapeChanged` (`said()` is the line the window and the command line both say), with what follows: the items drawn as the shape (names) and the things set down from them (paths), as `ItemSizeDone` has `followed`.
- `keep_shape_as(job, item_path, new_name) -> ShapeKept` {item, from (the shape's name it was drawn as), name, blocks, followers: items drawn as the copy (just the one), set_down}.
- `shape_block_add(job, shape, made: Option<Made>) -> ShapeChanged` (a box the item's whole size, of the shape's first block's material, or the one named).
- `shape_block_set(job, shape, block (1..), what: BlockSet, text) -> ShapeChanged`, `BlockSet` = `Figure(0..6)` | `Round(bool)` | `Made(Made)` | `When(Option<String>)`.
- `shape_block_take(job, shape, block) -> ShapeChanged`.
- `shape_own(job, name) -> Result<usize, String>`: the one place that says "starter shape: not changed; keep it as a new shape first" and "no shape of that name".
- `check_shape_name(job, name)`: the one place that says a name is taken (job or starter): used by Keep and by the table.
- Command line: one new verb, `plumbline shape <file> keep <item> "<name>"`, `shape <file> add <shape> [--made timber]`, `shape <file> set <shape> <block> "<across from|across to|back from|back to|up from|up to|form|made of|there when>" <value>`, `shape <file> take <shape> <block>`; `plumbline shape <file> <shape>` lists a shape's blocks and who is drawn as it. No existing verb fits (`items` lists items, `set` is for a value with a status). `shape` with no more words lists the job's own shapes.
- History: `History::record` compares the saved text before and after; the saved job carries `shapes`, so each change is one step and undo/redo restore the shapes with the job (tested).

## Built (final names differ slightly from the plan above)
Part 1: `editing::shapes`: `keep_shape_as`, `shape_block_add`, `shape_block_set` (`BlockSet::{Figure,Round,Made,When}`), `shape_block_take`, `shape_name_free`, `ShapeChanged{said,follows}`, `block_said`, `inches_said`; `Piece::whole`, `Shape::worked_block`; table refusal of a starter's name with different blocks (identical blocks still "same"); reach message now says "(1 in above the box)". CLI verb `plumbline shape` (list, keep, add, set, take). Red run: 0 passed 16 failed (stub), then 16 passed. demos/shapes.sh step 7.
Part 2: `StoreyTotals::set_down: SetDown{kinds: Vec<SetDownKind{kind,count,names,paths,status,not_priced,not_drawn}>}`, `.count() .not_priced() .not_drawn() .said()`; `view::nothing_priced_on`. Red run: 0 passed 6 failed, then 6 passed. `takeoff` prints "Set down on this level:" with the lines of `said()` under every level.
Window call: `storey_totals(job)[i].set_down` instead of grouping `snap.drawn`; `said()` for the sentences.
Divergences from window's rule: kinds are equipment, fixture, furnishing only (window's `is_placed` includes fittings); things with no size are counted and said not drawn (window leaves them out). HELD for lead: fittings in or out.
Expectation moved: demos/lists.sh step 4 (two lines under the level). No test moved. Duplicate arithmetic: `pricing()` still has its own copy of the nothing-priced rule (tests tie the two).
Run: world, cli, ifc, plans, sheets, roofing tests all pass; 52 CLI demos, none FAIL; gates, licenses, portmap pass.

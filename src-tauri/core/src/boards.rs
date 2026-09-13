// ── The board grid ─────────────────────────────────────────────────────────
// Nobody buys a surface. A crew buys 4 ft boards off a printed table, stacked to
// a thickness, cut at the region's edges and at the roof's, counted, and laid in
// courses. This turns C1's planes into that.
//
// THE GRID. One cell per board, 4 ft square (`panel_size_ft` in the bulletin),
// square to the region's own axes — the field's longest side and square to it,
// which is the string line an estimator sets. The top layer's joints run straight
// both ways (NRCA p.208, Fig. 5-9), which is what a grid IS; the layers under it
// are offset half a board, the trade's usual and the only stagger the box states
// (Versico Manual PDF p.31). The stagger changes no count — a course is a course
// however it is shifted — so it travels as a stated figure for the section to
// draw rather than as a second grid.
//
// WHERE THE GRID SITS. The drain is at the middle of a cell, not at a corner. Two
// reasons, both about the roof and not about the arithmetic: a sump is a square
// centred on the drain and bought as whole panels on the same 4 ft module, so a
// drain at a cell centre puts the sump on whole board positions instead of
// quartering four of them; and the valleys leave the drain at 45 degrees, so from
// a cell centre they run corner to corner of the cells they cross, which is how
// NRCA says to cut them — "the boards easily can be cut at 45 degrees from one
// edge or corner to the opposing corner" (p.166).
//
// Each region carries its own grid off its own drain, so two grids meet at a
// ridge and their joints do not line up. That is not a defect to tidy: the ridge
// is where two patterns meet, which is the case NRCA p.166 answers with an area
// divider, and C1 already reports how far the two disagree in depth.
//
// THE CUT. A cell is kept where the ground under it is — region, less the roof's
// own outline, less the sumps. A cut board still counts one, because one is what
// gets bought, and it reports the square feet the cut took off. That is a
// different number from the item's waste percentage on the estimate line: this is
// board bought and thrown away at a known place, and that is an allowance for
// damage and trim. Adding them would charge the cut twice.
//
// THE STACK. The thickness is read at the board's own middle, off whichever of
// the region's planes governs there — `Plane::rise_at`, the one expression the
// core and the engine are pinned to. Then the fill boards and the letter come off
// the table in `profile.rs`.

use std::collections::BTreeMap;

use crate::geom::{common_with, rectangle_of, take_out_all, Plane, Point, Shape};
use crate::model::{Board, BoardBill};
use crate::profile::{empty_rows, stack_at, LetterRow, ProfileSet, Schedule};
use crate::taper::read_at;

/// Half a board. Every course under the top one is offset by this, which is the
/// stagger the manual draws and the only one the box states.
pub const STAGGER_OF_A_BOARD: f64 = 0.5;

/// A kept piece smaller than this is not a board, it is a rounding artefact.
///
/// `geo`'s booleans run through a fixed-point grid, so a cut corner lands within
/// about three ten-millionths of a foot of where the arithmetic puts it. A cell
/// that only touches the ground along an edge comes back as a sliver of a
/// ten-thousandth of a square foot, and a sliver is not something anybody orders.
const SLIVER_SQUARE_FEET: f64 = 1e-4;

/// The ground the boards are laid on: the region, less whatever is cut out of it.
pub fn ground_for(shapes: &[Shape], cut_outs: &[Shape]) -> Vec<Shape> {
    if cut_outs.is_empty() {
        return shapes.to_vec();
    }
    take_out_all(shapes, cut_outs)
}

/// Lay the grid on one region's ground and read every board off the planes.
///
/// `drain` is where the grid is anchored — at a cell's middle. `planes`, `start`
/// and `cap` are the region's, so the thickness at a board's middle is the solved
/// surface and not a second opinion about it.
pub fn boards_over(
    ground: &[Shape],
    drain: Point,
    along: Point,
    across: Point,
    planes: &[Plane],
    start_inches: f64,
    cap_inches: Option<f64>,
    schedule: &Schedule,
) -> Vec<Board> {
    let side = schedule.panel_feet;
    if side <= 0.0 || ground.is_empty() {
        return Vec::new();
    }
    let half = side / 2.0;

    // How far the ground reaches along each axis from the drain, so the grid is
    // laid over the ground and not over the whole sheet.
    let mut least_along = f64::INFINITY;
    let mut most_along = f64::NEG_INFINITY;
    let mut least_across = f64::INFINITY;
    let mut most_across = f64::NEG_INFINITY;
    for shape in ground {
        for p in &shape.ring {
            let a = (p.x - drain.x) * along.x + (p.y - drain.y) * along.y;
            let b = (p.x - drain.x) * across.x + (p.y - drain.y) * across.y;
            least_along = least_along.min(a);
            most_along = most_along.max(a);
            least_across = least_across.min(b);
            most_across = most_across.max(b);
        }
    }
    if !least_along.is_finite() || !least_across.is_finite() {
        return Vec::new();
    }

    let first = |least: f64| ((least - half) / side).floor() as i64;
    let last = |most: f64| ((most + half) / side).ceil() as i64;

    let mut out = Vec::new();
    for i in first(least_along)..=last(most_along) {
        for j in first(least_across)..=last(most_across) {
            let centre = Point::new(
                drain.x + along.x * (i as f64 * side) + across.x * (j as f64 * side),
                drain.y + along.y * (i as f64 * side) + across.y * (j as f64 * side),
            );
            let cell = rectangle_of(centre, along, across, half, half);
            let kept = common_with(&cell, ground);
            let laid: f64 = kept.iter().map(Shape::square_feet).sum();
            if laid <= SLIVER_SQUARE_FEET {
                continue;
            }
            let whole = side * side;
            // A board is cut when the ground takes a bite out of it. The tolerance
            // is the boolean's own: a whole board comes back a few hundred
            // thousandths of a square foot light.
            let cut = laid < whole - 0.01;
            let Some(reading) = read_at(centre, planes, start_inches, cap_inches) else {
                continue;
            };
            let stack = stack_at(schedule, reading.thickness_inches, start_inches);
            out.push(Board {
                centre,
                shapes: kept,
                square_feet: laid,
                board_square_feet: whole,
                cut,
                waste_square_feet: (whole - laid).max(0.0),
                thickness_inches: reading.thickness_inches,
                flat: reading.flat,
                letter: stack.letter,
                panel_inches: stack.panel_inches,
                fill_inches: stack.fill_inches,
                fill_boards: stack.fill_boards,
                courses: stack.courses,
            });
        }
    }
    out
}

/// Add a run of boards up into the bill an estimator reads.
pub fn bill_of(boards: &[Board], schedule: &Schedule, start_inches: f64) -> BoardBill {
    let mut by_letter: BTreeMap<String, LetterRow> = empty_rows(schedule, start_inches);
    let mut bill = BoardBill {
        manufacturer: schedule.manufacturer.clone(),
        product: schedule.product.clone(),
        document_id: schedule.document_id.clone(),
        profile_set: schedule.set,
        slope_inches_per_foot: schedule.slope_inches_per_foot,
        fill_step_inches: schedule.fill_step_inches,
        panel_feet: schedule.panel_feet,
        stagger_feet: schedule.panel_feet * STAGGER_OF_A_BOARD,
        by_letter: BTreeMap::new(),
        boards: 0,
        cut_boards: 0,
        square_feet: 0.0,
        waste_square_feet: 0.0,
        laid_square_feet: 0.0,
        fill_boards: 0,
        fill_square_feet: 0.0,
        courses_max: 0,
        unlettered: 0,
    };

    for board in boards {
        bill.boards += 1;
        if board.cut {
            bill.cut_boards += 1;
        }
        bill.square_feet += board.board_square_feet;
        bill.laid_square_feet += board.square_feet;
        bill.waste_square_feet += board.waste_square_feet;
        bill.fill_boards += board.fill_boards;
        bill.fill_square_feet += board.board_square_feet * board.fill_boards as f64;
        bill.courses_max = bill.courses_max.max(board.courses);
        match &board.letter {
            None => bill.unlettered += 1,
            Some(letter) => {
                // A letter outside what the layout can reach still gets a row, so a
                // board that got bought is in the bill under the name printed on it.
                let row = by_letter.entry(letter.clone()).or_default();
                row.boards += 1;
                row.square_feet += board.board_square_feet;
            }
        }
    }

    bill.by_letter = by_letter;
    bill
}

/// Two bills added: a field's is the sum of its regions'.
pub fn merge(into: &mut BoardBill, other: &BoardBill) {
    into.boards += other.boards;
    into.cut_boards += other.cut_boards;
    into.square_feet += other.square_feet;
    into.waste_square_feet += other.waste_square_feet;
    into.laid_square_feet += other.laid_square_feet;
    into.fill_boards += other.fill_boards;
    into.fill_square_feet += other.fill_square_feet;
    into.courses_max = into.courses_max.max(other.courses_max);
    into.unlettered += other.unlettered;
    for (letter, row) in &other.by_letter {
        let mine = into.by_letter.entry(letter.clone()).or_default();
        mine.boards += row.boards;
        mine.square_feet += row.square_feet;
    }
}

/// An empty bill in the shape of one schedule: the letters, each at nothing.
pub fn empty_bill(schedule: &Schedule, start_inches: f64) -> BoardBill {
    bill_of(&[], schedule, start_inches)
}

/// Why a field has no bill, in words, or nothing because it has one.
///
/// A slope the bulletin prints no panel chart for is the case: the layout is still
/// the layout, and there is no lettered board to buy it out of. Saying that beats
/// a bill of nought boards, which reads as a roof that needs no insulation.
pub fn no_schedule_because(slope: f64, set: ProfileSet) -> String {
    format!(
        "the table prints no {} panel chart at {} in per foot, so there is no lettered board to bill",
        set.word(),
        slope,
    )
}

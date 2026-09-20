// ── Help ───────────────────────────────────────────────────────────────────
// Every area has a "?" and every "?" opens a page of plain words about that
// area. The paragraph that used to sit under a title on every open lives here
// now: an explanation belongs where somebody has asked for it.

let open: HTMLElement | null = null;

export function openHelp(title: string, body: string): void {
  closeHelp();

  const sheet = document.createElement('div');
  sheet.className = 'help-sheet';

  const header = document.createElement('header');
  const heading = document.createElement('h2');
  heading.textContent = title;
  const close = document.createElement('button');
  close.type = 'button';
  close.className = 'help-close';
  close.textContent = 'Close';
  close.addEventListener('click', closeHelp);
  header.append(heading, close);

  const text = document.createElement('div');
  text.className = 'help-body';
  for (const paragraph of body.split('\n\n')) {
    const p = document.createElement('p');
    p.textContent = paragraph.trim();
    text.append(p);
  }

  sheet.append(header, text);
  document.body.append(sheet);
  open = sheet;

  const escape = (e: KeyboardEvent) => { if (e.key === 'Escape') closeHelp(); };
  document.addEventListener('keydown', escape, { once: true });
}

export function closeHelp(): void {
  open?.remove();
  open = null;
}

/**
 * What the "?" says, keyed by what the window is showing.
 *
 * The names are the names on screen now: the surface has two views, the cost
 * panel has three tabs, and the book is reached from the Condition tab. There
 * is no "Estimate Sheet" and no "Reports" because there are no such editors —
 * a help page for a thing nobody can open is a help page nobody can use.
 */
export const HELP: Readonly<Record<string, string>> = {
  Plan: `This is the drawing. Add one, tell the program how big it is, then trace what you are pricing.

Set the scale first: pick Scale off the rail, click two points a known distance apart, and type that distance the way the drawing writes it — 12'-6", 6", or 12.5 all read the same. Until a sheet is scaled, anything measured on it shows a dash rather than a number, because a wrong number is worse than none.

The rail is the acts: Trace an area outlines a surface, Run a parapet runs along one, Drop a drain marks each one, Cricket lays a ridge between drains. Click to place corners, press Enter or double-click to finish, Escape to throw it away, Backspace to take back a corner. Hold Shift to keep a segment square. Corners snap to corners already on the sheet, which is what stops a roof leaking area at every junction.

Instead of clicking you can type: 40x30 lays a box off the corner already down, and 25 carries a run on in the direction it is already going.

Press 1 to 9 to pick a condition — the number is printed on its row. Whatever is picked is what the next shape goes into, and the pill on the drawing says which one while you are tracing.

One traced thing gives you square feet, linear feet and a count at the same time, because a parapet run is all three.

Getting around a big sheet: scroll to move, hold Ctrl and scroll to zoom where the pointer is, or drag with the middle mouse button. Zoom and Fit are at the top right of the drawing. On a full-size drawing, zoom in before you trace — fitted to the window, one screen pixel can be half a foot of roof, and a click lands where the pixel is, not where you meant.`,

  Model: `The roof, stood up. Everything here comes off the same traces and properties as the estimate, so it cannot disagree with what you are pricing.

An area lies at its elevation. A run with a height stands up off its base. Where a field has a taper and the roof has drains, it is laid out the way a tapered package is laid out: the roof is split between the drains, each one gets its own ground, and the boards fall from it four ways along the walls. The thickness at a point is what you put at the drain plus the slope times the run — and the run is how far the boards have come in the direction they are laid, not how far the drain is in a straight line. Where two of those falls meet you get a valley, at 45° to the walls, which is the miter a board is cut on. Halfway between two drains you get a ridge.

The field is lighter where it is thicker. Amber is where the boards run out: the taper has built as high as it can, the roof goes flat, and flat is where water stays. Walk that area before you bid it.

Move the pointer over the roof and the strip at the bottom of the window says what it is doing there — the slope, how thick the insulation is at that point, which way the water runs and how far it has to go to reach a drain.

Clicking picks a condition and picks it everywhere else too. Nothing in this window changes the job — there is no editing in 3D.`,

  Section: `The roof, cut open at a drain, so you can see what you are buying.

A tapered package is bought as boards, and nobody can see a board in a plan or in a shaded surface. This cuts through one drain along the direction the boards are laid and shows the stack: the deck it lands on, the flat board by the course, the lettered panel with its own letter on it, the sump cut down to the bowl, and the wall it stops against with the flashing, the coping, the cleat and the bar.

There is one cut for every drain on the roof, and the chips at the top right switch between them. A cut runs out to the wall where there is one, because a section detail is where you prove the flashing; where a drain's ground reaches no wall, it runs to the ridge it shares with the next drain and says so.

The horizontal is 1" = 4'-0". The vertical is stretched four times, and the drawing says so on itself: a board is half an inch over a run of twenty feet, and at true scale the whole build-up would be a line. Never scale a thickness off it — the dimensions are the numbers.

Beside the drawing the same cut in three dimensions, with everything in front of the plane taken away, so you can see where on the roof it was taken from.

While a cut is up, the Estimate names the lines the plane goes through: the field it cut, the drain it cut down to, and the wall it ended against.`,

  Estimate: `Every line here is one thing a condition consumes, grouped under the condition it comes off.

The line is short on purpose: what it is, how much of it, in what unit, and what it costs. A diamond after the unit means the program worked the quantity out from what you traced; a dotted underline means you typed it. There is no third kind.

Click a line and it opens where it is, to the whole of it: the formula, and every step from the traced measure to the money — quantity, waste, what you buy, what the price is quoted against, the unit cost — and then the arithmetic written out so you can check the number without doing it yourself.

The formula is edited there, on the line. A library you cannot see into is a library nobody trusts.

A line carries three units, because a supply house uses three: what you estimated in, what you buy in, and what the price is quoted against. Membrane is estimated in squares, bought by the roll and priced by the square foot.

A line with no price says so. A line off a sheet nobody has scaled says so. Neither is ever counted as nothing, because a zero looks like a finished line for free.`,

  Condition: `A condition is one thing you traced, and this is everything the drawing cannot tell you about it.

The list at the top is what has been traced. Pick one — here, on the drawing, on the roof, or with the number printed on its row — and what it measures and what it costs read underneath.

The numbers are what it measures right now. Everything below sets what the detail calls for: how tall the flashing runs, how wide the coping is, what the slope is, how much flat metal a formed profile eats.

Anything else you need, name it yourself with "Add a property". Every formula on this condition's lines can then use it by that name.

"Load an assembly" opens the book: saved sets of items, and the profiles the sheet-metal ones are formed from. Loading one copies its items onto the condition; it does not link them. Change a price on this bid and you have changed this bid, which is how it should be.`,

  Recap: `Where the money actually sits, by class.

A cost lands in a class and a class carries its own adders. Fifteen thousand dollars of "material" that is really material plus a dumpster plus a permit is the lie this structure exists to make impossible.

If the total left something out it says so underneath, by name. A total taken over a line nobody could price is a total that lies.`,

  Library: `The book: saved sets of items, and the profiles the sheet-metal ones are formed from.

It is optional. A job traces, measures, prices and recaps with no library open at all — this is where you stop retyping the same build-up on every bid.

Pick a condition, then load an assembly onto it. That copies the items; it does not link them. Change a price on this bid and you have changed this bid, which is how it should be — a library that reached back into finished jobs would be one nobody dared edit.

A profile is the flat legs a piece of metal is formed from. The stretch-out — the specification's word is girth — is their sum plus what the hems eat, and it belongs to the detail rather than the material: 19 1/2 inches at a coping, 8 at a gravel stop, 26 at an equipment curb cap, all off the same coil. That is why you enter it once per detail and use it everywhere that detail runs.`,

  Lens: `One estimate, filtered for whoever is receiving it.

Pick who it is for in the document bar. What you get is the same estimate with their columns and nothing else — a lens leaves things out, and that is all it does. Nothing here changes a number.

The supply house sees what to send, in the units they sell, with no cost and no waste on it. A subcontractor sees the measurements without your internal assumptions. The Consolidated lens is the only one that shows how a quantity was worked out, and it says on its face that it is internal.

A cost a lens leaves out is not on the page at all — not greyed, not blurred, not there. A number that is in the document and merely hidden is one look at the source away from the wrong person.`,
};

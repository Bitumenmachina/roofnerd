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

/** What each editor's "?" says. Plain words, no software in them. */
export const HELP: Readonly<Record<string, string>> = {
  Plan: `This is the drawing. Add one, tell the program how big it is, then trace what you are pricing.

Set the scale first: click Scale, click two points a known distance apart, and type that distance the way the drawing writes it — 12'-6", 6", or 12.5 all read the same. Until a sheet is scaled, anything measured on it shows a dash rather than a number, because a wrong number is worse than none.

Area outlines a surface. Line runs along something. Count drops a marker on each one. Click to place points, press Enter or double-click to finish, Escape to throw it away, Backspace to take back a corner. Hold Shift to keep a segment square. Corners snap to corners already on the sheet, which is what stops a roof leaking area at every junction.

One traced thing gives you square feet, linear feet and a count at the same time, because a parapet run is all three.

Getting around a big sheet: scroll to move, hold Ctrl and scroll to zoom where the pointer is, or drag with the middle mouse button. The zoom buttons and Fit are at the right end of the toolbar. On a full-size drawing, zoom in before you trace — fitted to the window, one screen pixel can be half a foot of roof, and a click lands where the pixel is, not where you meant.`,

  'Estimate Sheet': `Every line here is one thing a condition consumes, grouped under the condition it comes off.

The formula on each line is the arithmetic that turned what you traced into how much of that item you need. It is printed on the line and you can change it there. A library you cannot see into is a library nobody trusts.

A line carries three units, because a supply house uses three: what you estimated in, what you buy in, and what the price is quoted against. Membrane is estimated in squares, bought by the roll and priced by the square foot. Each step shows its own quantity.

A line with no price says so. A line off a sheet nobody has scaled says so. Neither is ever counted as nothing, because a zero looks like a finished line for free.`,

  Model: `The roof, stood up. Everything here comes off the same traces and properties as the estimate, so it cannot disagree with what you are pricing.

An area lies at its elevation. A run with a height stands up off its base. Where a field has a taper and the roof has drains, the thickness at any point is what you put at the drain plus the slope times how far that point is from the nearest drain — the same arithmetic you could write on a line by hand.

The field is lighter where it is thicker. Amber is where the boards run out: the taper has built as high as it can, the roof goes flat, and flat is where water stays. Walk that area before you bid it.

Clicking picks a condition and picks it everywhere else too. Nothing in this window changes the job — there is no editing in 3D.`,

  Condition: `A condition is one thing you traced, and this is everything the drawing cannot tell you about it.

The numbers at the top are what it measures right now. Everything below sets what the detail calls for: how tall the flashing runs, how wide the coping is, what the slope is, how much flat metal a formed profile eats.

Anything else you need, name it yourself with "+ property". Every formula on this condition's lines can then use it by that name.`,
};

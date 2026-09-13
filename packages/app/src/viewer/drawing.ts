// ── The drawing renderer ───────────────────────────────────────────────────
// One renderer, three drawings (D155).
//
// The tapered layout, the fastening plan and the section through a drain are
// all `Drawing` — polygons, lines, words and dimensions in feet, solved in
// `src-tauri/core/src/drawing.rs`. This file turns feet into SVG and paints. It
// decides nothing about the roof: no thickness, no board, no letter, no length.
// Every number that reaches a screen through here was written in the core, in
// the words a roofer writes it in, and this puts it where the core said.
//
// That is the same rule the Model view follows and it is here for the same
// reason: two arithmetics for one number disagree eventually, and the one that
// disagrees silently is the drawing.
//
// WHAT IT DOES DECIDE. How big a letter is drawn, how far a dimension line sits
// off the thing it measures, what a valley's dashes look like — the typography
// of a drawing, which is a drawing's own business and not the roof's. Colour is
// the stylesheet's: a role travels as a word and `app.css` paints it, so there
// is one palette and the payload carries no hex.
//
// TWO COORDINATE SYSTEMS, ONE CODE PATH. The Plan's overlays are drawn into the
// sheet's own SVG, in PAGE units, so a foot is `1 / feetPerUnit` of one; the
// Section has an SVG of its own drawn at a stated scale, so a foot is so many
// pixels and the vertical is stretched by the exaggeration the drawing states.
// Both arrive here as a pair of numbers and nothing else changes.

import type { Drawing, DrawnText, Spot } from '@roofnerd/engine';
import { svg } from './surface.js';

const SVG_NS = 'http://www.w3.org/2000/svg';

/**
 * How the feet on a drawing land in the coordinates being drawn in.
 *
 * `perFootY` is the one that carries the vertical exaggeration on a section. It
 * is applied per point rather than as a transform on purpose: a transform would
 * stretch the lettering with the geometry, and a stretched word is a word nobody
 * can read on a drawing that exists to be read.
 */
export interface DrawingView {
  readonly perFootX: number;
  readonly perFootY: number;
  /** How big an ordinary word is, in the coordinates being drawn in. */
  readonly wordSize: number;
  /** How big the letter on a board is, in the same. */
  readonly letterSize: number;
}

/** What the renderer hands back for the window to say in its own words. */
export interface Drawn {
  /** What the drawing is drawn at, for the corner of the sheet. */
  readonly note: string;
  /** Why there is nothing on it, where there is nothing. */
  readonly nothingYet: string | null;
  /** How many marks went down, so a window can say the drawing arrived. */
  readonly marks: number;
}

const at = (p: Spot, view: DrawingView) => ({ x: p.x * view.perFootX, y: p.y * view.perFootY });

const points = (all: readonly Spot[], view: DrawingView): string =>
  all.map((p) => { const q = at(p, view); return `${q.x},${q.y}`; }).join(' ');

/** A polygon with its holes, as one path, so a hole reads as a hole. */
function pathOf(ring: readonly Spot[], holes: readonly (readonly Spot[])[], view: DrawingView): string {
  const loop = (all: readonly Spot[]) => {
    if (all.length < 3) return '';
    const first = at(all[0]!, view);
    return `M ${first.x} ${first.y} `
      + all.slice(1).map((p) => { const q = at(p, view); return `L ${q.x} ${q.y}`; }).join(' ')
      + ' Z ';
  };
  return loop(ring) + holes.map(loop).join('');
}

/**
 * Draw a drawing into a group. The group is emptied first: half of one drawing
 * beside half of another is worse than neither.
 */
export function renderDrawing(into: SVGGElement, drawing: Drawing, view: DrawingView): Drawn {
  into.replaceChildren();
  into.classList.add('drawing-marks');
  into.setAttribute('data-drawing', drawing.id);

  let marks = 0;
  const add = (element: SVGElement) => { into.append(element); marks += 1; };

  // The filled ground first, then the lines over it, then the words over those.
  // That order is the drawing's: a board is under its own letter.
  for (const shape of drawing.shapes) {
    const d = pathOf(shape.shape.ring, shape.shape.holes, view);
    if (!d) continue;
    const element = svg('path', { d, class: `mark ${shape.role}`, 'data-role': shape.role });
    element.setAttribute('fill-rule', 'evenodd');
    if (shape.letter) element.setAttribute('data-letter', shape.letter);
    if (shape.layer) element.setAttribute('data-layer', shape.layer);
    add(element);
  }

  for (const line of drawing.lines) {
    if (line.points.length < 2) continue;
    const element = svg(line.closed ? 'polygon' : 'polyline', {
      points: points(line.points, view),
      class: `mark line ${line.role}${line.dashed ? ' dashed' : ''}`,
      'data-role': line.role,
    });
    if (line.layer) element.setAttribute('data-layer', line.layer);
    add(element);
  }

  for (const word of drawing.texts) add(wordOf(word, view));

  for (const dimension of drawing.dimensions) {
    const a = at(dimension.a, view);
    const b = at(dimension.b, view);
    const run = Math.hypot(b.x - a.x, b.y - a.y) || 1;
    // Square to the line, and by a length measured on the HORIZONTAL scale
    // whichever way it points — otherwise a dimension under a section stands
    // four times further off the drawing than one beside it, because the
    // vertical is stretched four times and the offset would stretch with it.
    const nx = -((b.y - a.y) / run) * dimension.offset * view.perFootX;
    const ny = ((b.x - a.x) / run) * dimension.offset * view.perFootX;
    const from = { x: a.x + nx, y: a.y + ny };
    const to = { x: b.x + nx, y: b.y + ny };
    const group = document.createElementNS(SVG_NS, 'g');
    group.setAttribute('class', `mark dimension${dimension.derived ? ' derived' : ''}`);
    group.setAttribute('data-role', 'dimension');
    group.append(
      svg('line', { x1: a.x, y1: a.y, x2: from.x, y2: from.y, class: 'witness' }),
      svg('line', { x1: b.x, y1: b.y, x2: to.x, y2: to.y, class: 'witness' }),
      svg('line', { x1: from.x, y1: from.y, x2: to.x, y2: to.y, class: 'dim-line' }),
    );
    const text = svg('text', {
      x: (from.x + to.x) / 2,
      y: (from.y + to.y) / 2,
      class: 'dim-text',
      'text-anchor': 'middle',
      'font-size': view.wordSize,
    });
    // Clear of its own line, on the side the line is offset to.
    text.setAttribute('dy', String(dimension.offset >= 0 ? view.wordSize : -view.wordSize * 0.4));
    text.textContent = dimension.text;
    group.append(text);
    add(group);
  }

  // The layer key, stacked where the core put it, with a leader to the stack it
  // is a key TO — which is what makes it a key and not a caption.
  if (drawing.key.length > 0 && drawing.keyAt) {
    const anchor = at(drawing.keyAt, view);
    if (drawing.keyFrom) {
      const from = at(drawing.keyFrom, view);
      add(svg('polyline', {
        points: `${from.x},${from.y} ${anchor.x},${anchor.y}`,
        class: 'mark line leader',
        'data-role': 'leader',
      }));
    }
    drawing.key.forEach((row, i) => {
      const text = svg('text', {
        x: anchor.x,
        y: anchor.y + i * view.wordSize * 1.4,
        class: 'key-row',
        'text-anchor': 'start',
        'font-size': view.wordSize,
        'data-role': 'key',
      });
      text.textContent = row;
      add(text);
    });
  }

  return { note: drawing.note, nothingYet: drawing.nothingYet, marks };
}

function wordOf(word: DrawnText, view: DrawingView): SVGElement {
  const p = at(word.at, view);
  const size = word.role === 'letter'
    ? view.letterSize
    : word.role === 'thickness' ? view.letterSize * 1.3 : view.wordSize;
  const text = svg('text', {
    x: p.x,
    y: p.y,
    class: `mark ${word.role === 'letter' ? 'board-letter' : word.role}`,
    'text-anchor': word.anchor,
    'font-size': size,
    'data-role': word.role,
  });
  // A letter sits in the middle of its board rather than on the line under it.
  text.setAttribute('dy', String(size * 0.35));
  text.textContent = word.text;
  return text;
}

/** How big what was drawn actually came out, or null where nothing did. */
export function extentOf(marks: SVGGElement): DOMRect | null {
  try {
    const box = marks.getBBox();
    return box.width < 0.01 || box.height < 0.01 ? null : box;
  } catch {
    return null;
  }
}

/**
 * Move the words on a drawing off each other.
 *
 * A label on a section is placed where the thing it names is, and the things it
 * names are stacked in half-inch courses along a twenty-foot run — so the words
 * land on top of one another however carefully the core places them. On the
 * demo, "existing deck 1 1/2 in · the datum" was drawn through the wall's own
 * dimension, "the ridge this ground shares with the next drain" ran into the
 * layer key, and the drain's note landed on the sheet's footer (C8 §8).
 *
 * What moves: labels, and a dimension's own figure. What does not: the letter
 * on a board, and the key. A letter belongs to the board under it and there is
 * nowhere else for it to be; a dimension's figure belongs ABOVE its line, which
 * is where lifting it keeps it — that is how a crowded dimension is drawn by
 * hand, and "6 in at the wall" was sitting on a board letter until it was.
 *
 * A word that has to move goes UP, in steps of its own height, because a
 * section is drawn with the sky above it and the build-up below: up is where
 * there is room and down is the drawing. The x never moves, so a word stays
 * over the thing it names. A dimension's figure is held to a short lift — far
 * enough to clear a letter, not far enough to look like it belongs to
 * something else.
 */
export function spreadWords(marks: SVGGElement, gap = 2): void {
  const box = (el: SVGGraphicsElement) => {
    try { return el.getBBox(); } catch { return null; }
  };
  type Taken = { x: number; y: number; w: number; h: number };
  type Word = { el: SVGTextElement; at: Taken; limit: number };
  const clash = (a: Taken, b: Taken) => a.x < b.x + b.w + gap && b.x < a.x + a.w + gap
    && a.y < b.y + b.h + gap && b.y < a.y + a.h + gap;

  const taken: Taken[] = [];
  const movable: Word[] = [];
  for (const el of Array.from(marks.querySelectorAll<SVGTextElement>('text'))) {
    const b = box(el);
    if (!b || b.width < 0.01) continue;
    const at = { x: b.x, y: b.y, w: b.width, h: b.height };
    const role = el.getAttribute('data-role');
    const free = role === 'label' || role === 'dimension'
      || el.classList.contains('dim-text');
    if (free) movable.push({ el, at, limit: role === 'label' ? Infinity : b.height * 2.5 });
    else taken.push(at);
  }

  // Highest first, so a stack of labels settles from the top down rather than
  // pushing the same one up twice.
  movable.sort((a, b) => a.at.y - b.at.y);
  for (const word of movable) {
    let lifted = 0;
    // Enough tries to climb clear of a full stack of layer labels and no more:
    // a label that cannot find room in eight steps is on a drawing with a
    // different problem, and it stays where the core put it.
    for (let pass = 0; pass < 8; pass += 1) {
      const over = taken.find((t) => clash(word.at, t));
      if (!over) break;
      const step = over.y + over.h - word.at.y + gap;
      if (lifted + step > word.limit) { lifted = 0; break; }
      word.at.y -= step;
      lifted += step;
    }
    if (lifted > 0) {
      const y = Number(word.el.getAttribute('y') ?? 0);
      word.el.setAttribute('y', String(y - lifted));
    } else {
      // It could not find room inside its own limit, so it stays where the core
      // put it and is not treated as taking room it does not hold.
      const b = box(word.el);
      if (b) { word.at.x = b.x; word.at.y = b.y; word.at.w = b.width; word.at.h = b.height; }
    }
    taken.push(word.at);
  }
}

/**
 * Fit an SVG round what was drawn into it, at the scale the drawing states.
 *
 * The scale is the point of a section — 1" = 4'-0", which on a screen is 96 CSS
 * pixels to the inch and so 24 to the foot — and it is kept where the window has
 * the room. Where it has not, the drawing comes down uniformly and SAYS SO,
 * because a drawing that states a scale it is not drawn at is worse than one
 * that states none.
 *
 * The box is measured off what actually got drawn rather than off the geometry's
 * own extent, because the words are half the drawing and a label runs well past
 * the thing it names.
 */
export function fitDrawing(
  surface: SVGSVGElement, marks: SVGGElement, pad: number, footer = 0,
): { drawnToFit: boolean } {
  let box: DOMRect;
  try {
    box = marks.getBBox();
  } catch {
    // A group with nothing in it, or one that has not been laid out. Neither is
    // an error and neither is a drawing.
    return { drawnToFit: false };
  }
  if (box.width < 1 || box.height < 1) return { drawnToFit: false };

  const width = box.width + pad * 2;
  const height = box.height + pad * 2;
  surface.setAttribute('viewBox', `${box.x - pad} ${box.y - pad} ${width} ${height}`);

  // The footer note is drawn over the bottom of the pane, so the room a drawing
  // has is the pane less that band — otherwise the lowest label on the sheet is
  // laid across the sentence that says what the sheet is.
  const room = surface.parentElement?.getBoundingClientRect();
  const fits = !!room && width <= room.width && height <= room.height - footer;
  if (fits) {
    // One drawn unit to one pixel, which is the stated scale.
    surface.setAttribute('width', String(Math.round(width)));
    surface.setAttribute('height', String(Math.round(height)));
  } else {
    surface.removeAttribute('width');
    surface.removeAttribute('height');
  }
  surface.classList.toggle('to-fit', !fits);
  return { drawnToFit: !fits };
}

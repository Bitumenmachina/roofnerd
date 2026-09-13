// ── The verbs ──────────────────────────────────────────────────────────────
// The acts in the handoff's tool rail that do something TO the roof, rather than
// tracing something new onto the sheet: set the high side and the low, box out a
// curb, measure between two places, cut a section.
//
// One file for all four, and both views feed it. The Plan picks points off the
// sheet through its own tools and the Model picks them off the roof with a ray —
// two ways of putting a finger on a place — and everything after that is the
// same act: how many picks it wants, what it asks the core, what the strip says,
// and what it writes. Two copies of that would be two programs, and the one that
// disagreed would be whichever view was edited second.
//
// THE SHAPE OF AN ACT, AND WHY IT IS THIS SHAPE:
//
//   pick      the view hands over a place on the roof, in FEET. Feet, not page
//             units, because the core answers in feet and because a pick on the
//             Model has no page under it at all
//   ask       the core says what those picks would mean — the run along the
//             boards, the slope they imply, the ruler's own reading, where the
//             cricket behind a curb goes. Nothing has moved yet
//   say       the strip shows the answer, with the derived mark on what the
//             solve worked out and a plain field on what was typed. A refusal is
//             a sentence in that strip and not a silence
//   write     the act writes through `set`, the shell solves the whole model
//             again from the document, and every window hears about it
//
// Nothing in here computes geometry. The run between two picks, the slope that
// falls out of it, the face a cricket stands against — every one of those is the
// core's answer, because a second arithmetic in the window is the defect this
// program has already paid for once.

import {
  formatFeetInches, parseFeet,
  type Condition, type Item, type Page, type Spot,
} from '@roofnerd/engine';
import type { BetweenAnswer, CricketBehindAnswer, HighAndLowAnswer } from '@roofnerd/engine';
import {
  askBetween, askCricketBehind, askHighAndLow, askSectionAt, at, doc, set, type Doc,
} from './doc.js';
import { derivedField, stripButton, stripField } from './chrome.js';
import { wantCut } from './cut.js';
import { inches } from './labels.js';
import { select } from './selection.js';
import { hueFor } from './icons.js';

/** The four acts that work on the roof rather than on the sheet. */
export type VerbName = 'high-low' | 'curb' | 'measure' | 'section';

/** What each one is called in the rail, in the handoff's own words. */
export const VERB_LABELS: Readonly<Record<VerbName, string>> = {
  'high-low': 'Set high and low',
  curb: 'Box out a curb',
  measure: 'Measure',
  section: 'Cut a section',
};

/** How many picks each one takes before it has anything to ask. */
const PICKS: Readonly<Record<VerbName, number>> = {
  'high-low': 2,
  curb: 2,
  measure: 2,
  section: 1,
};

/**
 * A mark an act leaves on the roof, in feet.
 *
 * The view draws it: the Plan as shapes on the sheet, the Model as lines and
 * words over the canvas. What it IS — where the picks are, which way the ruler
 * runs, where the cut goes — belongs here, because it is a fact about the act
 * and not about either drawing.
 */
export interface VerbMark {
  readonly kind: 'high' | 'low' | 'corner' | 'ruler' | 'cut' | 'box';
  readonly at: Spot;
  /** The other end, on a mark that is a line or a box. */
  readonly to?: Spot;
  /** A ring, on a mark that is a shape. */
  readonly ring?: readonly Spot[];
  readonly text?: string;
}

/** The wall thickness a curb gets where nothing on the job says otherwise. */
const CURB_WALL_INCHES = 6;

let name: VerbName | null = null;
let picks: Spot[] = [];
/** The core's answers, one per act. Null until the picks are in. */
let highLow: HighAndLowAnswer | null = null;
let ruler: BetweenAnswer | null = null;
let cricket: CricketBehindAnswer | null = null;
let cutLine: { at: Spot; to: Spot; index: number } | null = null;
/** What the act said last — a refusal, or what it just did. */
let said: string | null = null;
/** Typed in the strip. */
let highInches: number | null = null;
let lowInches: number | null = null;
let sumpFeet: number | null = null;
let curbHeightFeet: number | null = null;
/** An ask or a write in flight, so the strip can say the act is working. */
let busy = 0;

const watchers = new Set<() => void>();
const changed = () => { for (const w of watchers) w(); };

export function watchVerb(fn: () => void): () => void {
  watchers.add(fn);
  return () => watchers.delete(fn);
}

export const verbNow = (): VerbName | null => name;

/** Is an act in the rail one of these? The views ask before they route a pick. */
export const isVerb = (id: string): id is VerbName => id in VERB_LABELS;

/**
 * Start an act, or put one down.
 *
 * The picks go with it. "Until the next act" is what the handoff says about the
 * marks on the roof, and the next act is exactly this being called again.
 */
export function useVerb(next: VerbName | null): void {
  name = next;
  clearPicks();
  curbHeightFeet = null;
  changed();
}

function clearPicks(): void {
  picks = [];
  highLow = null;
  ruler = null;
  cricket = null;
  cutLine = null;
  said = null;
  highInches = null;
  lowInches = null;
  sumpFeet = null;
}

/** Escape: drop what is picked and stay in the act. */
export function clearVerbPicks(): void {
  clearPicks();
  changed();
}

/**
 * A place on the roof, in feet.
 *
 * Past the number of picks the act wants, the next one starts again — which is
 * what a hand expects and what stops an act from being stuck after a misclick.
 */
export function verbPick(p: Spot): void {
  if (!name) return;
  if (picks.length >= PICKS[name]) clearPicks();
  picks.push(p);
  changed();
  if (picks.length === PICKS[name]) void askNow();
}

export const verbPicks = (): readonly Spot[] => picks;

/** What the act has left on the roof. */
export function verbMarks(): readonly VerbMark[] {
  if (!name) return [];
  const out: VerbMark[] = [];
  if (name === 'high-low') {
    if (picks[0]) {
      out.push({
        kind: 'high',
        at: picks[0],
        text: highInches === null ? 'High' : `High ${inches(highInches)}`,
      });
    }
    // The low mark stands on the DRAIN the pick took, not on the pixel that was
    // clicked: the strip says which drain it is and the roof should agree with
    // the strip. Until the core has answered, the pick itself is the honest mark.
    const low = highLow?.drainAt ?? picks[1] ?? null;
    if (low) {
      out.push({
        kind: 'low',
        at: low,
        text: lowInches === null
          ? 'Low'
          : `Low ${inches(lowInches)}${highLow?.drainNamed ? ` at ${drainWords()}` : ''}`,
      });
    }
    if (picks[0] && low) out.push({ kind: 'ruler', at: low, to: picks[0] });
  }
  if (name === 'measure') {
    for (const p of picks) out.push({ kind: 'corner', at: p });
    if (picks[0] && picks[1]) {
      out.push({
        kind: 'ruler',
        at: picks[0],
        to: picks[1],
        text: ruler ? formatFeetInches(ruler.feet) : undefined,
      });
    }
  }
  if (name === 'curb') {
    for (const p of picks) out.push({ kind: 'corner', at: p });
    const ring = boxRing();
    if (ring) out.push({ kind: 'box', at: ring[0]!, ring });
    if (cricket?.ridge) {
      out.push({ kind: 'ruler', at: cricket.ridge[0], to: cricket.ridge[1], text: 'cricket' });
    }
  }
  if (name === 'section' && cutLine) {
    out.push({ kind: 'cut', at: cutLine.at, to: cutLine.to, text: `cut ${letterOf(cutLine.index)}` });
  }
  return out;
}

/** What a cut is called on the drawing: A–A for the first, B–B for the second. */
const letterOf = (index: number): string => {
  const letter = String.fromCharCode(65 + (index % 26));
  return `${letter}–${letter}`;
};

/**
 * Which drain the low pick took, in the words the roof itself writes on it.
 *
 * "Drain 2", the same as the label standing on it in the Model and the same as
 * the section's own title. Not the condition's name: one count condition carries
 * every drain on this roof, so its name is "Roof Drains" and calling the pick
 * that would be a second word for the thing the estimator is looking at.
 */
function drainWords(): string {
  const number = highLow?.drainNumber ?? null;
  return number === null ? '' : `Drain ${number}`;
}

// ── asking ─────────────────────────────────────────────────────────────────

/**
 * Ask the core what the picks mean.
 *
 * Every answer is kept whole rather than picked apart: the strip reads it, the
 * marks read it, and the write reads it, so all three are looking at one answer
 * from one solve. A token guards against an older answer landing after a newer
 * one — two picks in quick succession are two asks in flight.
 */
let asking = 0;

async function askNow(): Promise<void> {
  const token = asking + 1;
  asking = token;
  busy += 1;
  changed();
  try {
    if (name === 'high-low' && picks[0] && picks[1]) {
      const answer = await askHighAndLow({
        high: picks[0], low: picks[1], highInches, lowInches,
      });
      if (token !== asking) return;
      highLow = answer;
      said = answer.refused ?? answer.note;
      // Seed the two fields from what the roof already IS at the picks, so the
      // estimator edits a real number rather than an empty field — and so the
      // seeding is a reading of the solve rather than arithmetic in here.
      if (highInches === null && answer.highNowInches !== null) {
        highInches = answer.highNowInches;
      }
      if (lowInches === null && answer.lowNowInches !== null) lowInches = answer.lowNowInches;
      if (sumpFeet === null && answer.sumpFeet !== null) sumpFeet = answer.sumpFeet;
      // Now that the two thicknesses are stated, the slope they imply.
      if (!answer.refused && answer.taperInchesPerFoot === null) {
        const again = await askHighAndLow({
          high: picks[0], low: picks[1], highInches, lowInches,
        });
        if (token !== asking) return;
        highLow = again;
        said = again.refused ?? again.note;
      }
    }
    if (name === 'measure' && picks[0] && picks[1]) {
      const answer = await askBetween(picks[0], picks[1]);
      if (token !== asking) return;
      ruler = answer;
    }
    if (name === 'curb') {
      const ring = boxRing();
      if (ring) {
        const answer = await askCricketBehind(ring);
        if (token !== asking) return;
        cricket = answer;
        if (curbHeightFeet === null) curbHeightFeet = lastCurbHeight();
      }
    }
    if (name === 'section' && picks[0]) {
      const answer = await askSectionAt(picks[0]);
      if (token !== asking) return;
      if (answer.index === null || !answer.at || !answer.along) {
        cutLine = null;
        said = 'there is no section through that place — it is on no drainage area';
      } else {
        cutLine = {
          at: answer.at,
          to: {
            x: answer.at.x + answer.along.x * answer.runFeet,
            y: answer.at.y + answer.along.y * answer.runFeet,
          },
          index: answer.index,
        };
        said = null;
        wantCut(answer.index);
      }
    }
  } catch (e) {
    if (token === asking) said = message(e);
  } finally {
    busy -= 1;
    changed();
  }
}

const message = (e: unknown) =>
  (typeof e === 'string' ? e : e instanceof Error ? e.message : String(e));

// ── the strip ──────────────────────────────────────────────────────────────

/**
 * Everything the act needs and nothing else, for the window's tool settings
 * strip.
 *
 * The view calls this and hands it to `chrome.strip.show`. It is built here
 * rather than in either view for the same reason the act is: a strip that said
 * one thing on the Plan and another on the Model would be two acts wearing one
 * word.
 */
export function verbStrip(): { name: string; fields: Node[] } | null {
  if (!name) return null;
  const fields: Node[] = [];
  if (name === 'high-low') highLowStrip(fields);
  if (name === 'curb') curbStrip(fields);
  if (name === 'measure') measureStrip(fields);
  if (name === 'section') sectionStrip(fields);
  if (busy > 0) fields.push(stripField('', 'working it out…'));
  if (said) fields.push(sentence(said));
  return { name: VERB_LABELS[name], fields };
}

/** A sentence in the strip: a refusal, or a thing worth saying about the roof. */
function sentence(text: string): HTMLElement {
  const span = document.createElement('span');
  span.className = 'verb-said';
  span.textContent = text;
  return span;
}

/** A field in the strip somebody types into, with what it is called beside it. */
function typedField(
  label: string, value: string, hint: string, onDone: (text: string) => void,
): HTMLElement {
  const wrap = document.createElement('span');
  wrap.className = 'strip-field verb-typed';
  const k = document.createElement('span');
  k.textContent = label;
  const input = document.createElement('input');
  input.type = 'text';
  input.className = 'keyed';
  input.value = value;
  input.title = hint;
  input.setAttribute('aria-label', label);
  input.addEventListener('change', () => onDone(input.value));
  input.addEventListener('keydown', (e) => {
    if (e.key !== 'Enter') return;
    e.preventDefault();
    onDone(input.value);
  });
  wrap.append(k, input);
  return wrap;
}

/**
 * Frame 2's strip, as an act rather than a readout.
 *
 * High, Low, Slope ◈, Boards ◈, Sump width, Sump depth ◈ — the same six the
 * handoff draws, in the same order. Two of them are typed and the rest are the
 * solve's, which is exactly the split the frame draws with its one cyan mark on
 * the slope.
 */
function highLowStrip(fields: Node[]): void {
  if (picks.length === 0) {
    fields.push(stripField('First pick', 'click the high side of the roof'));
    return;
  }
  if (picks.length === 1) {
    fields.push(stripField('Second pick', 'click the drain the water goes to'));
    return;
  }

  fields.push(typedField('High', highInches === null ? '' : inches(highInches),
    'inches — how thick the package is at the first pick', (text) => {
      const read = parseInches(text);
      if (read !== null) highInches = read;
      void askNow();
    }));
  fields.push(typedField('Low', lowInches === null ? '' : inches(lowInches),
    'inches — how thick it is at the drain', (text) => {
      const read = parseInches(text);
      if (read !== null) lowInches = read;
      void askNow();
    }));
  if (highLow?.drainNamed) fields.push(stripField('At', drainWords()));

  const slope = highLow?.taperInchesPerFoot ?? null;
  if (slope !== null) {
    fields.push(derivedField('Slope', `${inches(slope)} in 12`,
      `The fall between the two picks over the run along the boards between them:`
      + ` ${formatFeetInches(highLow?.runFeet ?? 0)}.`));
  }
  const needed = highLow?.boardsNeeded ?? null;
  if (needed !== null) {
    const allowed = highLow?.boardsAllowed ?? null;
    fields.push(derivedField('Boards',
      allowed === null ? `${needed} courses` : `${needed} courses against ${allowed} allowed`,
      'How many layers of board the deepest position takes, the tapered panel'
      + ' included, beside the ceiling the job typed.'));
  }
  fields.push(typedField('Sump width', sumpFeet === null ? '' : formatFeetInches(sumpFeet),
    'feet — the square cut down to the drain', (text) => {
      const read = parseFeet(text);
      if (read !== null) sumpFeet = read;
      changed();
    }));
  const depth = highLow?.sumpDepthInches ?? null;
  if (depth !== null) {
    fields.push(derivedField('Sump depth', inches(depth),
      'The roof at the sump’s own rim, less the thickness at the drain.'));
  }

  const ready = highLow !== null && !highLow.refused && highLow.taperInchesPerFoot !== null;
  const act = stripButton('Set high and low', () => { void applyHighAndLow(); });
  act.disabled = !ready;
  fields.push(act);
}

/** Inches typed the way a roofer types them: 4 1/2, 4.5, 4 1/2". */
function parseInches(text: string): number | null {
  const feet = parseFeet(`${String(text).trim().replace(/"$/, '')}"`);
  return feet === null ? null : feet * 12;
}

function curbStrip(fields: Node[]): void {
  fields.push(typedField('Height', curbHeightFeet === null ? '' : formatFeetInches(curbHeightFeet),
    'how far the curb stands above the roof', (text) => {
      const read = parseFeet(text);
      if (read !== null) curbHeightFeet = read;
      changed();
    }));
  fields.push(stripField('Wall thickness', `${wallInches()}"`,
    'from the last curb on this job'));
  if (picks.length < 2) {
    fields.push(stripField('Corners',
      picks.length === 0 ? 'click one corner of the curb' : 'click the corner across from it'));
    return;
  }
  const ring = boxRing();
  if (ring) {
    fields.push(derivedField('Size', boxWords(ring),
      'The box the two corners make, square to the sheet.'));
  }
  if (cricket?.ridge) {
    fields.push(derivedField('Cricket', `${formatFeetInches(cricket.faceFeet)} of face`,
      'A cricket on the upslope side, as wide each side as half the face the water'
      + ' comes at (NRCA Fig. 8-1).'));
  } else if (cricket?.refused) {
    fields.push(stripField('Cricket', 'none'));
  }
  const act = stripButton('Box it out', () => { void applyCurb(); });
  act.disabled = curbHeightFeet === null || !(curbHeightFeet > 0);
  fields.push(act);
}

function measureStrip(fields: Node[]): void {
  if (picks.length < 2) {
    fields.push(stripField('Measure',
      picks.length === 0 ? 'click one end' : 'click the other end'));
    fields.push(stripField('', 'Escape clears it · nothing is written'));
    return;
  }
  if (!ruler) return;
  fields.push(derivedField('Distance', formatFeetInches(ruler.feet),
    'The straight line between the two picks, which is what a tape reads.'));
  if (ruler.riseInches !== null) {
    fields.push(derivedField('Rise', risen(ruler.riseInches),
      'How far the finished roof climbs from the first pick to the second, off the'
      + ' solved planes.'));
  }
  fields.push(endField('At the first pick', ruler.aInches));
  fields.push(endField('At the second pick', ruler.bInches));
  fields.push(stripField('', 'Escape clears it · nothing is written'));
}

/** A rise, signed and said: a ruler dragged downhill reads a fall. */
function risen(value: number): string {
  if (Math.abs(value) < 1 / 16) return 'level';
  return value > 0 ? `${inches(value)} up` : `${inches(-value)} down`;
}

/** The package's depth at one end of the ruler, or why there is none. */
function endField(label: string, value: number | null): HTMLElement {
  if (value === null) {
    const field = stripField(label, '—', undefined, true);
    field.title = 'That end is on no tapered area, so there is no thickness to read.';
    return field;
  }
  return derivedField(label, inches(value), 'Read off the solved planes at that place.');
}

function sectionStrip(fields: Node[]): void {
  if (!cutLine) {
    fields.push(stripField('Cut a section', 'click a drain, or anywhere on the roof'));
    return;
  }
  fields.push(stripField('Showing', `cut ${letterOf(cutLine.index)}`));
  fields.push(derivedField('Run length',
    formatFeetInches(Math.hypot(cutLine.to.x - cutLine.at.x, cutLine.to.y - cutLine.at.y)),
    'How far the cut runs, along the boards, from the drain to what it ends against.'));
}

// ── the box two corners make ───────────────────────────────────────────────

/**
 * A rectangle from two opposite corners, square to the sheet, closed.
 *
 * Closed — the first corner repeated at the end — because a ring whose last edge
 * is missing is a curb with three faces, and the face the cricket wants may be
 * the fourth. `cricket.rs` reads a run's edges as consecutive pairs, so an open
 * ring quietly hides one of them.
 */
function boxRing(): readonly Spot[] | null {
  const [a, b] = picks;
  if (!a || !b) return null;
  if (Math.abs(b.x - a.x) < 1e-9 || Math.abs(b.y - a.y) < 1e-9) return null;
  return [
    { x: a.x, y: a.y },
    { x: b.x, y: a.y },
    { x: b.x, y: b.y },
    { x: a.x, y: b.y },
    { x: a.x, y: a.y },
  ];
}

const boxWords = (ring: readonly Spot[]): string =>
  `${formatFeetInches(Math.abs(ring[1]!.x - ring[0]!.x))}`
  + ` × ${formatFeetInches(Math.abs(ring[2]!.y - ring[1]!.y))}`;

// ── what the acts write ────────────────────────────────────────────────────

const conditions = (d: Doc = doc()): Condition[] => (at('/conditions', d) as Condition[]) ?? [];
const pages = (d: Doc = doc()): Page[] => (at('/pages', d) as Page[]) ?? [];

/**
 * Set high and low: the field's taper and thickness, and the drain's sump.
 *
 * Both numbers are the core's. Nothing here works out a slope — `TAPER` is what
 * the answer said it would be, and writing anything else would make the strip a
 * liar about the act it just carried out.
 */
async function applyHighAndLow(): Promise<void> {
  const answer = highLow;
  if (!answer || answer.refused || answer.taperInchesPerFoot === null) return;
  const taper = answer.taperInchesPerFoot;
  const thickness = answer.thicknessInches;
  const all = conditions();
  busy += 1;
  changed();
  try {
    const next = all.map((c) => {
      if (c.id === answer.conditionId) {
        return {
          ...c,
          properties: { ...(c.properties ?? {}), TAPER: taper, ...(thickness === null ? {} : { T: thickness }) },
        };
      }
      if (c.id === answer.drainConditionId && sumpFeet !== null && sumpFeet > 0) {
        return { ...c, properties: { ...(c.properties ?? {}), SUMP: sumpFeet } };
      }
      return c;
    });
    await set('/conditions', next);
    said = `The roof falls ${inches(taper)} in 12 to ${drainWords()}.`;
    // Ask again on the same picks, so the strip's derived rows are this solve's
    // and not the one before it.
    await askNow();
  } catch (e) {
    said = message(e);
  } finally {
    busy -= 1;
    changed();
  }
}

/** The wall thickness a new curb takes: the last curb's, or the trade's minimum. */
function wallInches(): number {
  const last = [...conditions()].reverse().find((c) => c.role === 'curb');
  const wall = last?.properties?.['WALL'];
  return typeof wall === 'number' && wall > 0 ? wall : CURB_WALL_INCHES;
}

/** The last curb's own height, so the field opens at what this job already uses. */
function lastCurbHeight(): number | null {
  const last = [...conditions()].reverse().find((c) => c.role === 'curb');
  const h = last?.properties?.['H'];
  return typeof h === 'number' && h > 0 ? h : null;
}

/**
 * Box out a curb: a curb condition, and the cricket behind it.
 *
 * Two conditions in one write, because they are one act — a curb on a tapered
 * roof has a cricket behind it, and the estimator asked for the curb. Where the
 * core says there is no cricket (a curb too narrow across the fall for the water
 * to back up behind), the curb is written on its own and the strip says so.
 *
 * The lines come off the last curb on the job, which is this job's own standard
 * curb detail — the flashing, the cap and the labour, with their formulas, their
 * waste and their prices. A curb with no lines on it is a curb that does not
 * price, and the handoff's whole argument is that geometry and money move
 * together.
 */
async function applyCurb(): Promise<void> {
  const ring = boxRing();
  if (!ring || curbHeightFeet === null) return;
  const all = conditions();
  const page = pageForFeet();
  if (!page) {
    said = 'this drawing has no scale yet, so a curb on it would have no size';
    changed();
    return;
  }
  busy += 1;
  changed();
  try {
    const stamp = Date.now().toString(36);
    const curbId = `condition-${stamp}`;
    const template = [...all].reverse().find((c) => c.role === 'curb');
    const named = `Curb ${all.filter((c) => c.role === 'curb').length + 1}`;

    const properties: Record<string, number> = {};
    for (const [key, value] of Object.entries(template?.properties ?? {})) {
      // The height is this curb's own, and how far the membrane is detailed to
      // run up it is a decision nobody has made about a curb nobody has drawn
      // yet. Everything else — the cap's width, its girth, the datum — is the
      // detail this job already uses.
      if (key === 'H' || key === 'FLASHUP') continue;
      if (typeof value === 'number') properties[key] = value;
    }
    properties['H'] = curbHeightFeet;
    properties['WALL'] = wallInches();

    const curb: Condition = {
      id: curbId,
      name: named,
      kind: 'line',
      role: 'curb',
      traces: [{ id: `trace-${stamp}`, pageId: page.id, points: toPage(ring, page) }],
      properties,
      items: copyLines(template?.items ?? [], stamp),
      color: hueFor(all.length),
    };

    const written: Condition[] = [curb];
    if (cricket?.ridge) {
      const ridgeTemplate = [...all].reverse().find((c) => c.role === 'ridge');
      written.push({
        id: `condition-${stamp}-cricket`,
        name: `Cricket behind ${named}`,
        kind: 'line',
        role: 'ridge',
        between: [curbId],
        traces: [{
          id: `trace-${stamp}-cricket`,
          pageId: page.id,
          points: toPage(cricket.ridge, page),
        }],
        // No taper typed: a cricket is cut at twice the field beside it, and the
        // solve knows what that field is. Typing one here would be this window
        // deciding a slope.
        properties: {},
        items: copyLines(ridgeTemplate?.items ?? [], `${stamp}-cricket`),
        color: hueFor(all.length + 1),
      });
    }

    await set('/conditions', [...all, ...written]);
    select({ kind: 'condition', id: curbId });
    said = cricket?.ridge
      ? `${named} is on the roof, with its cricket on the upslope side.`
      : `${named} is on the roof. ${cricket?.refused ?? 'It has no cricket.'}`;
    clearPicksKeepingTheAct();
  } catch (e) {
    said = message(e);
  } finally {
    busy -= 1;
    changed();
  }
}

/** After a curb lands, the act stays up for the next one and the marks go. */
function clearPicksKeepingTheAct(): void {
  const keep = said;
  picks = [];
  cricket = null;
  said = keep;
}

/** Fresh ids on copied lines: two conditions naming one line open as one row. */
function copyLines(items: readonly Item[], stamp: string): Item[] {
  return items.map((item) => ({ ...item, id: `${item.id}-${stamp}` }));
}

/** The sheet a new trace goes on: the one the roof is already traced on. */
function pageForFeet(): (Page & { feetPerUnit: number }) | null {
  const all = pages();
  const scaled = all.filter((p): p is Page & { feetPerUnit: number } => !!p.feetPerUnit);
  const field = conditions().find((c) => c.kind === 'area' && c.properties?.['TAPER'] !== undefined);
  const on = field?.traces?.[0]?.pageId;
  return scaled.find((p) => p.id === on) ?? scaled[0] ?? null;
}

/** Feet back into the page units a trace is stored in. */
const toPage = (ring: readonly Spot[], page: Page & { feetPerUnit: number }) =>
  ring.map((p) => ({ x: p.x / page.feetPerUnit, y: p.y / page.feetPerUnit }));

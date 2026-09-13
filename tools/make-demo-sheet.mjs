// ── The demo job's drawing ─────────────────────────────────────────────────
// The sheet an estimator traces the Northgate demo on.
//
// It was the probe harness's blank test sheet — a grey rectangle headed "ROOF
// PLAN — test sheet" — so the one thing a stranger opens to find out what this
// program is showed a trace floating on a swatch, with no building under it
// (C8 §2). The whole promise of the Plan view is that you trace over the
// architect's drawing, and the demo had no drawing.
//
// So this draws one: the outline, the parapet, the three drains with their
// sumps, the RTU curb, a north arrow, a graphic scale and a title block. Every
// line of it is invented. There is no client's work in this repository and this
// is what keeps it that way — the same reason `make-demo-prices.mjs` exists,
// and the sheet says so on its own face.
//
//   node tools/make-demo-sheet.mjs
//
// ImageMagick, which is what made the test sheet before it (its own metadata
// says so), through MVG — ImageMagick's drawing language — so the sheet is text
// in this repository and not a binary somebody has to take on trust.
//
// THE COORDINATES ARE THE JOB'S. The page is 612 x 792 points and the demo's
// traces are in those points, so the building has to be drawn where the trace
// already is or the trace floats again. They are read out of the job rather
// than typed here: retrace the demo and this sheet follows it.

import { execFile } from 'node:child_process';
import { readFile, writeFile, rm } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { promisify } from 'node:util';
import { tmpdir } from 'node:os';

const run = promisify(execFile);
const ROOT = resolve(import.meta.dirname, '..');
const OUT = join(ROOT, 'jobs/demo-job/pages/roof-plan.pdf');

/** A US letter sheet, in points, which is what the page's own units are. */
const SHEET = { w: 612, h: 792 };

/**
 * How the sheet is rendered and then paged.
 *
 * MVG is read as CSS pixels at 96 to the inch, so rendering at 288 gives three
 * pixels to the unit; writing that raster back at 216 puts 612 x 792 POINTS on
 * the page, which is the size the page's own coordinates are in. Get this pair
 * wrong and every trace on the demo lands somewhere else on the drawing.
 */
const RENDER_DPI = 288;
const PAGE_DPI = 216;

const job = JSON.parse(await readFile(join(ROOT, 'jobs/demo-job/conditions.json'), 'utf8'));
const pages = JSON.parse(await readFile(join(ROOT, 'jobs/demo-job/pages/pages.json'), 'utf8'));
const page = pages[0];
const feetPerUnit = page.feetPerUnit;
/** Points to the foot on this sheet. */
const perFoot = 1 / feetPerUnit;

const traceOf = (id) => (job.find((c) => c.id === id)?.traces ?? [])[0]?.points ?? [];
const field = traceOf('c-field');
const drains = traceOf('c-drains');
const curb = traceOf('c-curb');
if (field.length < 3) throw new Error('the demo has no field traced, so there is nothing to draw under it');

const sumpFeet = job.find((c) => c.id === 'c-drains')?.properties?.SUMP ?? 4;
const wallInches = job.find((c) => c.id === 'c-parapet')?.properties?.WALL ?? 8;

// ── the drawing ────────────────────────────────────────────────────────────
const mvg = [];
const say = (...lines) => mvg.push(...lines);
const n = (v) => Number(v.toFixed(2));
const poly = (pts) => pts.map((p) => `${n(p.x)},${n(p.y)}`).join(' ');

/** Everything on an architect's sheet is drawn in one of four weights. */
const HAIR = 0.4;
const THIN = 0.7;
const MEDIUM = 1.2;
const HEAVY = 2.2;

const FACE = 'DejaVu Sans';
/**
 * An inch mark inside an MVG string ends the string.
 *
 * Which is a problem on a drawing, because every length on one carries a foot
 * mark or an inch mark: the first run of this wrote `text 306,625 "108'-0""`
 * and ImageMagick refused the whole sheet.
 */
const said = (text) => String(text).replace(/"/g, '\\"');
const words = (x, y, text, size = 7, anchor = 'start', weight = 'normal') => say(
  'push graphic-context',
  `font-family "${FACE}"`,
  `font-size ${size}`,
  `font-weight ${weight}`,
  'stroke none',
  'fill "#1a1a1a"',
  `text-align ${anchor === 'middle' ? 'center' : anchor === 'end' ? 'right' : 'left'}`,
  `text ${n(x)},${n(y)} "${said(text)}"`,
  'pop graphic-context',
);

say('viewbox 0 0 612 792', 'push graphic-context');
say('fill "#ffffff"', 'stroke none', `rectangle 0,0 ${SHEET.w},${SHEET.h}`);
say('fill none', 'stroke "#1a1a1a"');

// The sheet's own border, the way a drawing is bound: a wide left margin.
say(`stroke-width ${MEDIUM}`, 'rectangle 36,18 594,774');
say(`stroke-width ${HAIR}`, 'rectangle 40,22 590,770');

// ── the building ───────────────────────────────────────────────────────────
// The parapet is drawn as two lines because that is what it is: a wall with a
// thickness, and the roof stops at its inside face. The trace follows the
// outer one, which is where an estimator measures a parapet from.
const inset = (wallInches / 12) * perFoot;
const middle = field.reduce((all, p) => ({ x: all.x + p.x / field.length, y: all.y + p.y / field.length }),
  { x: 0, y: 0 });
const inward = field.map((p) => {
  const dx = middle.x - p.x;
  const dy = middle.y - p.y;
  const run = Math.hypot(dx, dy) || 1;
  return { x: p.x + (dx / run) * inset, y: p.y + (dy / run) * inset };
});
say(`stroke-width ${HEAVY}`, `polygon ${poly(field)}`);
say(`stroke-width ${THIN}`, `polygon ${poly(inward)}`);

// The deck, hatched lightly along its span, so the sheet reads as a roof and
// not as an empty box. Every twelve feet, which is a purlin spacing nobody has
// to believe: it is a drawing, and this one says it is a drawing.
const box = field.reduce((all, p) => ({
  minX: Math.min(all.minX, p.x), maxX: Math.max(all.maxX, p.x),
  minY: Math.min(all.minY, p.y), maxY: Math.max(all.maxY, p.y),
}), { minX: Infinity, maxX: -Infinity, minY: Infinity, maxY: -Infinity });
//
// Cut to the outline, not to its bounding box: the first run drew the deck
// straight across the bump, so eight feet of hatching stood outside the
// building on both sides of it.
const spansAt = (x) => {
  const cuts = [];
  for (let i = 0; i < field.length; i += 1) {
    const a = field[i];
    const b = field[(i + 1) % field.length];
    if ((a.x <= x && b.x > x) || (b.x <= x && a.x > x)) {
      cuts.push(a.y + ((x - a.x) / (b.x - a.x)) * (b.y - a.y));
    }
  }
  cuts.sort((p, q) => p - q);
  const spans = [];
  for (let i = 0; i + 1 < cuts.length; i += 2) spans.push([cuts[i], cuts[i + 1]]);
  return spans;
};
say('push graphic-context', 'stroke "#c8ccce"', `stroke-width ${HAIR}`);
for (let x = box.minX + perFoot * 12; x < box.maxX; x += perFoot * 12) {
  for (const [from, to] of spansAt(x)) say(`line ${n(x)},${n(from)} ${n(x)},${n(to)}`);
}
say('pop graphic-context');

// ── what is on the roof ────────────────────────────────────────────────────
const sump = (sumpFeet * perFoot) / 2;
drains.forEach((d, i) => {
  say('push graphic-context', 'stroke "#1a1a1a"', `stroke-width ${THIN}`, 'fill none');
  say(`rectangle ${n(d.x - sump)},${n(d.y - sump)} ${n(d.x + sump)},${n(d.y + sump)}`);
  say(`circle ${n(d.x)},${n(d.y)} ${n(d.x + 4)},${n(d.y)}`);
  say(`line ${n(d.x - 6)},${n(d.y)} ${n(d.x + 6)},${n(d.y)}`);
  say(`line ${n(d.x)},${n(d.y - 6)} ${n(d.x)},${n(d.y + 6)}`);
  say('pop graphic-context');
  words(d.x, d.y + sump + 11, `RD-${i + 1}`, 6.5, 'middle');
});

if (curb.length >= 4) {
  const cb = curb.reduce((all, p) => ({
    minX: Math.min(all.minX, p.x), maxX: Math.max(all.maxX, p.x),
    minY: Math.min(all.minY, p.y), maxY: Math.max(all.maxY, p.y),
  }), { minX: Infinity, maxX: -Infinity, minY: Infinity, maxY: -Infinity });
  say('push graphic-context', 'stroke "#1a1a1a"', `stroke-width ${THIN}`, 'fill "#eef0f1"');
  say(`rectangle ${n(cb.minX)},${n(cb.minY)} ${n(cb.maxX)},${n(cb.maxY)}`);
  say('pop graphic-context');
  words(cb.maxX + 6, cb.minY + (cb.maxY - cb.minY) / 2 + 2, 'RTU-1', 6.5);
}

// ── the dimensions along two sides ─────────────────────────────────────────
const feetInches = (feet) => {
  const whole = Math.floor(feet + 1e-6);
  const inches = Math.round((feet - whole) * 12);
  return inches === 12 ? `${whole + 1}'-0"` : `${whole}'-${inches}"`;
};
const dim = (ax, ay, bx, by, text, at) => {
  say('push graphic-context', 'stroke "#1a1a1a"', `stroke-width ${HAIR}`, 'fill none');
  say(`line ${n(ax)},${n(ay)} ${n(bx)},${n(by)}`);
  const tick = 3;
  say(`line ${n(ax - tick)},${n(ay - tick)} ${n(ax + tick)},${n(ay + tick)}`);
  say(`line ${n(bx - tick)},${n(by - tick)} ${n(bx + tick)},${n(by + tick)}`);
  say('pop graphic-context');
  words(at.x, at.y, text, 6.5, at.anchor ?? 'middle');
};
const across = box.maxX - box.minX;
const down = box.maxY - box.minY;
dim(box.minX, box.maxY + 22, box.maxX, box.maxY + 22, feetInches(across * feetPerUnit),
  { x: (box.minX + box.maxX) / 2, y: box.maxY + 31 });
// The one down the side runs INSIDE the building, because the sheet's bound
// edge leaves no gutter to put it in: at x - 22 the figure was laid across the
// border and the last character of it was off the page.
dim(box.minX + 16, box.minY + 40, box.minX + 16, box.maxY - 40, feetInches(down * feetPerUnit),
  { x: box.minX + 21, y: (box.minY + box.maxY) / 2, anchor: 'start' });

// ── north, and a scale somebody can put a rule on ──────────────────────────
const north = { x: 556, y: 92 };
say('push graphic-context', 'stroke "#1a1a1a"', `stroke-width ${THIN}`, 'fill "#1a1a1a"');
say(`path "M ${north.x},${north.y - 18} L ${north.x + 6},${north.y + 6} L ${north.x},${north.y + 1} `
  + `L ${north.x - 6},${north.y + 6} Z"`);
say('pop graphic-context');
words(north.x, north.y + 18, 'N', 8, 'middle', 'bold');

const bar = { x: 60, y: 700, feet: 40 };
say('push graphic-context', 'stroke "#1a1a1a"', `stroke-width ${HAIR}`, 'fill "#1a1a1a"');
for (let f = 0; f < bar.feet; f += 10) {
  const filled = (f / 10) % 2 === 0;
  say(filled ? 'fill "#1a1a1a"' : 'fill "#ffffff"');
  say(`rectangle ${n(bar.x + f * perFoot)},${n(bar.y)} `
    + `${n(bar.x + (f + 10) * perFoot)},${n(bar.y + 4)}`);
}
say('pop graphic-context');
for (const f of [0, 20, 40]) words(bar.x + f * perFoot, bar.y + 13, `${f}`, 6, 'middle');
words(bar.x + bar.feet * perFoot + 10, bar.y + 13, 'FEET', 6);

// ── the notes, and the title block ─────────────────────────────────────────
words(60, 92, 'GENERAL NOTES', 7.5, 'start', 'bold');
[
  ' 1.  REMOVE EXISTING ROOF SYSTEM TO THE STRUCTURAL DECK.',
  ' 2.  EXISTING STEEL DECK TO REMAIN. VERIFY FASTENER PULL-OUT BEFORE ORDERING.',
  ' 3.  NEW TAPERED INSULATION TO DRAIN AS SHOWN. POSITIVE DRAINAGE TO ALL DRAINS.',
  ' 4.  PARAPET HEIGHTS AND CURB HEIGHTS TO BE FIELD VERIFIED.',
].forEach((line, i) => words(60, 106 + i * 11, line, 6.5));

const block = { x: 372, y: 666, w: 222, h: 108 };
say('push graphic-context', 'stroke "#1a1a1a"', `stroke-width ${MEDIUM}`, 'fill none');
say(`rectangle ${block.x},${block.y} ${block.x + block.w},${block.y + block.h}`);
say(`stroke-width ${HAIR}`);
for (const at of [30, 52, 74]) {
  say(`line ${block.x},${block.y + at} ${block.x + block.w},${block.y + at}`);
}
say('pop graphic-context');
words(block.x + 10, block.y + 20, 'NORTHGATE RETAIL CENTER', 9.5, 'start', 'bold');
words(block.x + 10, block.y + 44, 'ROOF PLAN', 8.5, 'start', 'bold');
words(block.x + 10, block.y + 66, `SCALE  ${page.scaleNote ?? ''}`, 7);
words(block.x + block.w - 10, block.y + 66, 'SHEET  R-101', 7, 'end');
words(block.x + 10, block.y + 90, 'DEMONSTRATION DRAWING', 7, 'start', 'bold');
words(block.x + 10, block.y + 101, 'INVENTED THROUGHOUT — NOT FOR CONSTRUCTION', 6.5);

say('pop graphic-context');

// ── and out ────────────────────────────────────────────────────────────────
const scratch = join(tmpdir(), `roofnerd-sheet-${process.pid}.mvg`);
await writeFile(scratch, `${mvg.join('\n')}\n`);
try {
  await run('magick', [
    '-density', String(RENDER_DPI), `mvg:${scratch}`,
    '-colorspace', 'gray', '-depth', '8',
    '-units', 'PixelsPerInch', '-density', String(PAGE_DPI),
    '-compress', 'zip', OUT,
  ]);
} finally {
  await rm(scratch, { force: true });
}

const bytes = (await readFile(OUT)).length;
console.log(`${mvg.length} drawing commands → ${OUT}`);
console.log(`${(bytes / 1024).toFixed(1)} kB, ${SHEET.w} x ${SHEET.h} points, `
  + `${perFoot.toFixed(2)} points to the foot — the scale the demo's traces are in.`);
console.log('Every line of it invented. No client drawing is in this repository.');

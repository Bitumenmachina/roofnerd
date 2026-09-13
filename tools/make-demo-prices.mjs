// ── Synthetic prices ───────────────────────────────────────────────────────
// Every screenshot, every probe's dressing and every example in the docs draws
// its prices from here.
//
// The reason is not tidiness. Three times a real figure reached the public tree
// because it was the number in front of me while I was writing, and each time
// the fix was to invent one afterwards. A safe source that is always closer
// than the real one removes the temptation rather than relying on me to resist
// it. The client-data hook stays on regardless — belt as well as braces.
//
// Seeded, so the same run gives the same prices and a screenshot does not churn
// on every regeneration. Regenerate with:
//
//   node tools/make-demo-prices.mjs
//
// Plausible ranges, invented values. Nothing here came off anybody's bid.
//
// It writes TWICE, and the second write is the point. `jobs/demo-job/prices.json`
// is the price book; the demo's own lines carry their own unit cost, because a
// line with a price typed on it is what an estimator actually has in front of
// them. Those two used to be kept in step by hand, which is a thing that works
// until it does not: a regenerated book beside a stale set of lines is a demo
// whose recap disagrees with its own library and nothing says so. So the
// catalogue below is the one source and both files are written from it.

import { readFile, writeFile } from 'node:fs/promises';
import { readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';

const { readJob, writeJob } = await import('../packages/engine/dist/index.js');

const ROOT = resolve(import.meta.dirname, '..');
const SEED = 20260912;

const readJson = async (rel) => JSON.parse(await readFile(join(ROOT, rel), 'utf8'));
const conditionsPath = join(ROOT, 'jobs/demo-job/conditions.json');
const conditions = JSON.parse(await readFile(conditionsPath, 'utf8'));

/** A small deterministic generator: same seed, same prices, every time. */
function generator(seed) {
  let state = seed >>> 0;
  return () => {
    state = (state * 1664525 + 1013904223) >>> 0;
    return state / 0x100000000;
  };
}

/** FNV-1a over a string, for a seed that belongs to one item and not to a place. */
function hashOf(text) {
  let h = 0x811c9dc5;
  for (let i = 0; i < text.length; i += 1) {
    h ^= text.charCodeAt(i);
    h = Math.imul(h, 0x01000193) >>> 0;
  }
  return h;
}

/**
 * A price somewhere in a plausible band, to the cent — seeded on the item's OWN
 * id rather than on its place in the list.
 *
 * It used to be one stream drawn in order, which is deterministic and also
 * fragile in a way that only shows up later: retire one line and every price
 * after it in the catalogue moves, so a batch that adds a fastener line quietly
 * re-prices the coping. C2 retires the single tapered line and adds six, and C3
 * and C4 add more; seeding per id means each item keeps its own price for good and
 * the catalogue can be edited anywhere. Every price moves once, here, and never
 * again for a reason that has nothing to do with the item.
 */
const between = (id, low, high) => {
  const random = generator(hashOf(id) ^ SEED);
  // One draw thrown away: the first value out of a linear congruential generator
  // is the seed run through one step, so close seeds give close first values and
  // `iso-x` and `iso-y` would price nearly the same.
  random();
  return Math.round((low + random() * (high - low)) * 100) / 100;
};

/**
 * Product names are a public matter — a manufacturer's catalogue is not
 * anybody's bid. What they cost on a particular job is, so the prices below
 * are invented and the names are not.
 *
 * `band` is in the unit the LAST step of the line is counted in, which is not
 * always the unit the line is estimated in: membrane is estimated in squares,
 * ordered in rolls and priced by the square foot, so its band is dollars a
 * square foot. The steps are on the entry beside it, so the two can never
 * drift apart.
 *
 * `labor` is units per MAN-hour, and the money beside it is dollars a man-hour.
 * Crew days come out as hours over the crew size over eight, which is the
 * arithmetic in `line.ts` — so a rate written as though it were units per
 * crew-hour would put every crew day on this job out by a factor of six. The
 * rates below are stated in the form a roofer says them, in the notes on each
 * one, and converted here.
 */

/** Units per man-hour, from the way the trade states a rate: units a crew-day. */
const perDay = (units, crew) => Math.round((units / (crew * 8)) * 100000) / 100000;

const CATALOGUE = [
  // ── the field ─────────────────────────────────────────────────────────
  { id: 'tpo-60', description: 'TPO membrane, 60 mil', costCode: '07-100-100', unit: 'SQ',
    order: { name: 'ROLL', per: 10, rule: 'ceil' }, price: { name: 'SF', contains: 1000, rule: 'exact' },
    band: [1.05, 1.25], waste: 10 },
  // The tapered package is not one line any more — see THE PACKAGE, BY LETTER
  // below. It used to be `SQ` off the traced area at a band picked for an
  // average thickness somebody guessed, which is the money seam D146 named.
  { id: 'coverboard', description: 'High density cover board, 1/2 in', costCode: '07-100-100', unit: 'SQ',
    band: [88, 104], waste: 8 },
  { id: 'plates', description: 'Fasteners and plates', costCode: '07-100-170', unit: 'EA',
    order: { name: 'BOX', per: 1000, rule: 'exact' }, band: [420, 500] },
  // ── the parapet ───────────────────────────────────────────────────────
  { id: 'wallflash', description: 'TPO wall flashing, 60 mil', costCode: '07-100-100', unit: 'SF',
    band: [1.02, 1.18], waste: 10 },
  // Linear feet of profile become pounds through a stretch-out that belongs to
  // the detail, so this one is priced by the pound and the conversion lives on
  // the line. 24 ga galvanized is 1.156 lb a square foot.
  { id: 'coping', description: 'Coping cap, 24 ga galvanized', costCode: '07-100-150', unit: 'SF',
    order: { name: 'LB', contains: 1.156, rule: 'exact' }, band: [3.35, 3.7], waste: 10 },
  { id: 'cleat', description: 'Continuous cleat, 20 ga', costCode: '07-100-150', unit: 'LF',
    band: [1.95, 2.32], waste: 5 },
  { id: 'mitre', description: 'Shop-formed corner, mitre welded', costCode: '07-100-150', unit: 'EA',
    band: [78, 100] },
  { id: 'termbar', description: 'Termination bar and sealant', costCode: '07-100-160', unit: 'LF',
    band: [3.1, 3.7] },
  { id: 'metal-fasteners', description: 'Fasteners, cleat and termination bar', costCode: '07-100-170', unit: 'EA',
    order: { name: 'BOX', per: 1000, rule: 'exact' }, band: [340, 420] },
  // ── the drains and the curb ───────────────────────────────────────────
  { id: 'drain', description: 'Retrofit drain, 4 in', costCode: '07-100-100', unit: 'EA',
    band: [420, 520] },
  { id: 'sump-panel', description: 'Tapered sump panel, 4 ft square', costCode: '07-100-100', unit: 'EA',
    band: [160, 210] },
  { id: 'curbflash', description: 'TPO curb flashing, 60 mil', costCode: '07-100-100', unit: 'SF',
    band: [1.02, 1.18], waste: 10 },
  { id: 'curbcap', description: 'Curb cap, 24 ga galvanized', costCode: '07-100-150', unit: 'SF',
    order: { name: 'LB', contains: 1.156, rule: 'exact' }, band: [3.4, 3.8], waste: 10 },
  // ── the crews ─────────────────────────────────────────────────────────
  // Stated the way a roofer states them, and converted above. A six-man crew
  // lays this system at roughly twenty-two squares a day all in — the three
  // install lines below add up to that — and tears the old one off faster than
  // it puts the new one on.
  { id: 'tearoff', description: 'Tear off existing roof', costCode: '07-100-210', unit: 'SQ',
    labor: { productionRate: perDay(40, 6), crewSize: 6 }, band: [48, 58] },
  { id: 'set-insulation', description: 'Lay the tapered package', costCode: '07-100-230', unit: 'SQ',
    labor: { productionRate: perDay(60, 6), crewSize: 6 }, band: [48, 58] },
  { id: 'set-coverboard', description: 'Lay cover board', costCode: '07-100-230', unit: 'SQ',
    labor: { productionRate: perDay(80, 6), crewSize: 6 }, band: [48, 58] },
  { id: 'install-membrane', description: 'Weld the membrane', costCode: '07-100-230', unit: 'SQ',
    labor: { productionRate: perDay(60, 6), crewSize: 6 }, band: [48, 58] },
  { id: 'install-flashing', description: 'Flash the wall', costCode: '07-100-240', unit: 'LF',
    labor: { productionRate: perDay(120, 2), crewSize: 2 }, band: [48, 58] },
  { id: 'set-drain', description: 'Set the drains', costCode: '07-100-240', unit: 'EA',
    labor: { productionRate: perDay(2.7, 2), crewSize: 2 }, band: [48, 58] },
  { id: 'set-curb', description: 'Flash the curb', costCode: '07-100-240', unit: 'LF',
    labor: { productionRate: perDay(48, 2), crewSize: 2 }, band: [48, 58] },
  // ── bought in ─────────────────────────────────────────────────────────
  { id: 'set-metal', description: 'Form and set the metal', costCode: '07-100-900', unit: 'LF',
    band: [7.6, 9.0] },
  { id: 'lift', description: 'Telehandler rental, weekly', costCode: '07-100-940', unit: 'EA',
    order: { name: 'EA', per: 1, rule: 'ceil' }, band: [1900, 2300] },
  { id: 'dumpster', description: 'Dumpster, 30 yard', costCode: '07-100-950', unit: 'EA',
    order: { name: 'EA', per: 1, rule: 'ceil' }, band: [720, 900] },
  { id: 'permit', description: 'Permit and inspections', costCode: '07-100-950', unit: 'EA',
    band: [980, 1300] },
  { id: 'warranty', description: 'Manufacturer warranty, 20 year NDL', costCode: '07-100-920', unit: 'SQ',
    band: [52, 68] },
  { id: 'supervision', description: 'Supervision, foreman day', costCode: '07-100-990', unit: 'EA',
    band: [460, 540] },
];

// ── the package, by letter ─────────────────────────────────────────────────
// One entry per lettered panel the layout can actually reach, generated from the
// manufacturer's own chart rather than typed here — so the catalogue follows the
// chart and cannot drift from what the solve bills against.
//
// Polyiso is quoted by the square foot per inch of thickness, which is why a
// band per letter is the honest shape: an X panel averages an inch across its
// 4 ft and a ZZ averages four, so a ZZ costs about four times an X and a single
// band for "tapered polyiso" cannot say either. The flat fill board under them is
// the cheaper article — no taper to cut — and its band is per inch too.
//
// Invented rates, plausible ranges. Nothing here came off anybody's bid.
const PER_INCH_TAPERED = [48, 58];
const PER_INCH_FLAT = [40, 48];

const profile = await readJson('data/profiles/versico-versicore-mp-h.json');
const field = conditions.find(
  (c) => c.kind === 'area' && !c.hidden && c.properties?.TAPER !== undefined,
);
const atDrain = field.properties.T;
const charted = profile.slopes.find(
  (s) => Math.abs(s.slope_in_per_ft - field.properties.TAPER) < 1e-9,
);
if (!charted?.extended) {
  throw new Error(`the chart draws no extended set at ${field.properties.TAPER} in per foot`);
}
const fillStep = Number.parseFloat(charted.extended.fill_in);
/** The panels a layout off this drain thickness can reach: `[T, T + step)`. */
const letters = charted.extended.panels.filter(
  (p) => p.thick_edge_in > atDrain + 1e-9 && p.thin_edge_in < atDrain + fillStep - 1e-9,
);

/** As a roofer writes it: 1/2", 2 1/4", 4". */
const inchLabel = (v) => {
  const whole = Math.floor(v);
  const eighths = Math.round((v - whole) * 8);
  const fraction = eighths === 0 ? '' : `${eighths / 8}`.replace('0.', '').length > 0
    ? ` ${[0, '1/8', '1/4', '3/8', '1/2', '5/8', '3/4', '7/8'][eighths]}` : '';
  return `${whole || (fraction ? '' : 0)}${fraction}"`.trim();
};

for (const panel of letters) {
  const average = (panel.thin_edge_in + panel.thick_edge_in) / 2;
  CATALOGUE.push({
    id: `iso-${panel.designation.toLowerCase()}`,
    // THE LETTER FIRST. The panel is 364 px wide and the line's name ellipsizes
    // in it: "Tapered polyisocyanurate X, 1/4 : 12 — …" cut to "Tapered
    // polyisocyan…" on all four of them, so the sheet showed four lines that
    // read alike and differed only in their money. What an estimator needs to
    // tell them apart is the letter, so the letter is the third word.
    description: `Tapered ${panel.designation} · ${inchLabel(panel.thin_edge_in)}`
      + `–${inchLabel(panel.thick_edge_in)}`,
    costCode: '07-100-100', unit: 'SQ',
    band: [PER_INCH_TAPERED[0] * average, PER_INCH_TAPERED[1] * average],
    waste: 8,
  });
}
CATALOGUE.push({
  id: 'iso-fill',
  description: `Flat polyiso fill · ${inchLabel(fillStep)}`,
  costCode: '07-100-100', unit: 'SQ',
  band: [PER_INCH_FLAT[0] * fillStep, PER_INCH_FLAT[1] * fillStep],
  waste: 8,
});
CATALOGUE.push({
  // Cricket stock is its own lettered series — Carlisle prints two charts side by
  // side, headed Cricket and Primary Slope — and the bulletin on disk carries only
  // the primary one. So this is priced by the square foot of footprint and not by
  // letter, and that gap is on the record rather than papered over with the
  // field's own table.
  id: 'cricket-stock', description: 'Cricket stock, cut to the saddle',
  costCode: '07-100-100', unit: 'SF', band: [2.15, 2.6], waste: 10,
});

/**
 * The lines the MODEL answers, and the name each one is worth.
 *
 * A formula of derived names alone renders ◈ and is not editable in place; the
 * estimator overrides it by typing a formula, which then reads as typed. That is
 * the existing mechanism and nothing about it changes here.
 */
const DERIVED = [
  ...letters.map((panel) => ({
    condition: 'c-field',
    id: `iso-${panel.designation.toLowerCase()}`,
    formula: `TAPER_${panel.designation}_SF / 100`,
  })),
  { condition: 'c-field', id: 'iso-fill', formula: 'FILL_SF / 100' },
  { condition: 'c-drains', id: 'sump-panel', formula: 'SUMP_PANELS_EA' },
  { condition: 'c-cricket', id: 'cricket-stock', formula: 'CRICKET_SF' },
  // C3: the wall and curb lines move off LF * H and off SEGMENTS, onto the
  // names the wall solver carries. wallflash/curbflash were the wall's own
  // height times its run, which is every inch of the wall whether or not the
  // detail is drawn to flash that far; FLASH_SF is the exact area under the
  // ceiling FLASHUP and the roof beneath it actually leave. coping/curbcap
  // and mitre moved off LF and SEGMENTS the same way C2's own notes asked for
  // COPING_LF to exist at all: a formula should never have to spell
  // LF * (STRETCHOUT / 12) again, and a corner count read off the trace's own
  // turns is exact where SEGMENTS (every straight piece, corner or not) was
  // only ever a stand-in. cleat keeps its LF * 2 but reads it off CLEAT_LF —
  // the same figure, now a named scope instead of an expression repeated on
  // two conditions. metal-fasteners is the one line that changes shape:
  // CLEAT_LF * (12 / FASTEN_OC) is one fastener a foot at the 12 in spacing
  // the demo types, which is the same count the retired LF * 2 gave — a
  // fastener spacing other than 12 in now actually moves this line, which
  // LF * 2 could never do.
  { condition: 'c-parapet', id: 'wallflash', formula: 'FLASH_SF' },
  { condition: 'c-parapet', id: 'coping', formula: 'COPING_LF * (STRETCHOUT / 12)' },
  {
    condition: 'c-parapet', id: 'mitre', formula: 'COPING_PCS_EA',
    notes: 'Every coping piece the run is cut into, corners and the straight '
      + 'sections between them alike — this fixture prices one shop-formed line '
      + 'for both rather than carrying a second for plain lengths. The corners '
      + "are the least of it: six outside, two inside on this roof's own trace.",
  },
  { condition: 'c-parapet', id: 'cleat', formula: 'CLEAT_LF' },
  { condition: 'c-parapet', id: 'metal-fasteners', formula: 'CLEAT_LF * (12 / FASTEN_OC)' },
  { condition: 'c-curb', id: 'curbflash', formula: 'FLASH_SF' },
  { condition: 'c-curb', id: 'curbcap', formula: 'COPING_LF * (STRETCHOUT / 12)' },
];

/**
 * The cleat and termination-bar spacing C3's coping is priced against.
 *
 * A typed default rather than an invented one: nothing in the box states how
 * many of a term bar's own 6 in punches get a fastener (`coping.rs`'s own
 * GAP), so the demo states the spacing an estimator would type absent a
 * detail calling for tighter — 12 in, one to the foot — on the record here
 * rather than silently assumed by a formula that could not otherwise reach
 * `FASTEN_OC` at all.
 */
const FASTEN_OC_DEMO_IN = 12;

/**
 * What the demo's wind zones are set out and rated from.
 *
 * TYPED, never computed. The zone geometry follows a published rule (NRCA
 * *Membrane Roof Systems* 2019 Appendix 1, p.553) and the program works it; the
 * PRESSURE on a building is an engineered figure for that building, and a
 * program that invented one would be inventing the whole fastening plan under it.
 *
 * Where these four came from:
 *
 * - `ROOF_HEIGHT_FT` — the mean roof height the handoff gives this building.
 * - the three pressures — the coastal envelope brief in the box
 *   (`KNOWLEDGE/GOVERNANCE`), which tabulates ASCE 7 component-and-cladding roof
 *   pressures for this coast at 130 mph: the field at -42 to -71, the perimeter
 *   strip at -86 to -96 and the corners at -107 to -122 pounds a square foot.
 *   The upper end of each range is taken, because the most restrictive
 *   requirement governs and because the upper end of the corner range is the one
 *   computed at the same GCp the appendix itself prints for a corner (3.2, Fig
 *   1-2 p.546) — so the three upper figures are one consistent set rather than
 *   three separate picks.
 *
 * The names say the basis: `WIND_PSF_*` is strength design, which is what ASCE
 * 7's own maps produce (Appendix 1 p.546), and the solver reduces it by 0.6 to
 * working load. A job holding figures already at working load types
 * `WIND_ASD_FIELD` and so on instead, and nothing is reduced twice.
 *
 * GAP, on the record: the brief's own roof table is captioned at Exposure C and
 * this building is Exposure D, which is a higher pressure at the same speed. The
 * brief names -122 at the corner for Exposure D in its own text, so the corner
 * figure is right for this building and the other two are a floor. The ASCE
 * 7-22 figure itself is not on this machine; when it is, these three are the
 * three numbers to change, and everything under them re-solves.
 */
const WIND_DEMO = {
  ROOF_HEIGHT_FT: 22,
  WIND_PSF_FIELD: 71,
  WIND_PSF_PERIMETER: 96,
  WIND_PSF_CORNER: 122,
};

/** Lines a solved bill replaces. Left on the job they would charge it twice. */
const RETIRED = ['iso-tapered'];

const library = CATALOGUE.map(({ band, labor, ...item }) => ({
  ...item,
  ...(labor ?? {}),
  unitCost: between(item.id, band[0], band[1]),
}));

const out = {
  note: 'Generated by tools/make-demo-prices.mjs. Invented values in plausible ranges. '
    + 'Nothing here came off a real bid. Regenerate rather than hand-editing.',
  seed: SEED,
  items: library,
};

const pricesPath = join(ROOT, 'jobs/demo-job/prices.json');
await writeFile(pricesPath, `${JSON.stringify(out, null, 2)}\n`);

// ── and the demo's own lines, from the same catalogue ──────────────────────
// The money and the crew move. Formulas, units, waste and every word on a line
// that the estimator owns belong to the job and are left exactly where they are.
//
// The exception is the lines the MODEL answers. Those are put here, with the
// derived-name formula on them, because the name and the catalogue entry are one
// fact: a line priced per square of X panel and a formula naming `TAPER_Y_SF`
// would look entirely plausible on the sheet. The retired line goes in the same
// pass — a solved bill beside the one line it replaces would charge the tapered
// package twice.
const byId = new Map(library.map((i) => [i.id, i]));
let moved = 0;
let derived = 0;
let retired = 0;
const missing = [];

// The one property this pass types rather than prices: metal-fasteners is the
// only line on the job that reaches FASTEN_OC directly, and a name a formula
// cannot resolve leaves the line pending rather than silently priced at some
// assumed spacing.
const parapet = conditions.find((c) => c.id === 'c-parapet');
if (parapet && parapet.properties?.FASTEN_OC === undefined) {
  parapet.properties = { ...(parapet.properties ?? {}), FASTEN_OC: FASTEN_OC_DEMO_IN };
}

// And the wind, for the same reason: the zones are set out from a height and
// rated from a pressure, and neither is anything a drawing can be asked for.
const roof = conditions.find((c) => c.id === 'c-field');
if (roof) {
  for (const [name, value] of Object.entries(WIND_DEMO)) {
    if (roof.properties?.[name] === undefined) {
      roof.properties = { ...(roof.properties ?? {}), [name]: value };
    }
  }
}

for (const condition of conditions) {
  const items = condition.items ?? (condition.items = []);
  // Where the retired line stood, so the lines that replace it stand there too and
  // the sheet reads in the order a crew builds in.
  let at = items.findIndex((i) => RETIRED.includes(i.id));
  if (at < 0) at = items.length;
  const before = items.length;
  condition.items = items.filter((i) => !RETIRED.includes(i.id));
  retired += before - condition.items.length;

  for (const line of DERIVED) {
    if (line.condition !== condition.id) continue;
    const priced = byId.get(line.id);
    if (!priced) { missing.push(`${condition.name} → ${line.id}`); continue; }
    let item = condition.items.find((i) => i.id === line.id);
    if (!item) {
      item = { id: line.id };
      condition.items.splice(Math.min(at, condition.items.length), 0, item);
      at += 1;
    }
    item.description = priced.description;
    item.costCode = priced.costCode;
    item.unit = priced.unit;
    item.formula = line.formula;
    if (priced.waste !== undefined) item.waste = priced.waste;
    if (line.notes !== undefined) item.notes = line.notes;
    derived += 1;
  }
}

for (const condition of conditions) {
  for (const item of condition.items ?? []) {
    const priced = byId.get(item.id);
    if (!priced) { missing.push(`${condition.name} → ${item.id}`); continue; }
    item.unitCost = priced.unitCost;
    if (priced.productionRate !== undefined) item.productionRate = priced.productionRate;
    if (priced.crewSize !== undefined) item.crewSize = priced.crewSize;
    moved += 1;
  }
}
// Written through the engine's own writer, not restringified here. The shell and
// the command line agree on one canonical form — sorted keys, two spaces, a
// trailing newline — and `cargo test` holds the demo job to it. A line added with
// its keys in the order a person would type them is not in that form, and the
// round-trip test caught exactly that. One writer, one form.
const folder = join(ROOT, 'jobs/demo-job');
const job = readJob((rel) => {
  try { return readFileSync(join(folder, rel), 'utf8'); }
  catch (e) { if (e.code === 'ENOENT') return null; throw e; }
});
job.conditions = conditions;
writeJob(job, (rel, contents) => writeFileSync(join(folder, rel), contents));

console.log(`${library.length} synthetic prices → jobs/demo-job/prices.json`);
console.log(`${moved} lines on the demo job priced from the same catalogue`);
console.log(`${derived} of them answered by the model, by name; ${retired} retired`);
if (missing.length) {
  // Named, never silently left at whatever it was. A line the book does not
  // carry is a line whose price nobody regenerated.
  console.log('not in the catalogue, so left alone:');
  for (const m of missing) console.log(`  ${m}`);
}
console.log('Seeded, so the same run gives the same prices and screenshots do not churn.');

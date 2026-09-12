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
import { join, resolve } from 'node:path';

const ROOT = resolve(import.meta.dirname, '..');
const SEED = 20260912;

/** A small deterministic generator: same seed, same prices, every time. */
function generator(seed) {
  let state = seed >>> 0;
  return () => {
    state = (state * 1664525 + 1013904223) >>> 0;
    return state / 0x100000000;
  };
}
const random = generator(SEED);

/** A price somewhere in a plausible band, to the cent. */
const between = (low, high) => Math.round((low + random() * (high - low)) * 100) / 100;

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
  // A quarter in twelve off a half inch at the drain reaches fourteen inches at
  // the far corner of this roof, so the package averages seven. Tapered polyiso
  // is bought by the square foot per inch of average thickness, which is why
  // this band is where it is rather than where a two-inch flat board's is.
  { id: 'iso-tapered', description: 'Tapered polyisocyanurate, 1/4 : 12', costCode: '07-100-100', unit: 'SQ',
    band: [400, 486], waste: 8 },
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

const library = CATALOGUE.map(({ band, labor, ...item }) => ({
  ...item,
  ...(labor ?? {}),
  unitCost: between(band[0], band[1]),
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
// Only the money and the crew move. Formulas, units, waste and every word on
// the line belong to the job and are left exactly where they are.
const byId = new Map(library.map((i) => [i.id, i]));
const conditionsPath = join(ROOT, 'jobs/demo-job/conditions.json');
const conditions = JSON.parse(await readFile(conditionsPath, 'utf8'));
let moved = 0;
const missing = [];
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
await writeFile(conditionsPath, `${JSON.stringify(conditions, null, 2)}\n`);

console.log(`${library.length} synthetic prices → jobs/demo-job/prices.json`);
console.log(`${moved} lines on the demo job priced from the same catalogue`);
if (missing.length) {
  // Named, never silently left at whatever it was. A line the book does not
  // carry is a line whose price nobody regenerated.
  console.log('not in the catalogue, so left alone:');
  for (const m of missing) console.log(`  ${m}`);
}
console.log('Seeded, so the same run gives the same prices and screenshots do not churn.');

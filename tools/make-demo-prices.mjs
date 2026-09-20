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
  // A supply house quotes to the quarter dollar; so does this book.
  return Math.round((low + random() * (high - low)) * 4) / 4;
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
 * `production` is the rate SAID the way the trade says it — how many units, per
 * what, and the crew it assumes — and the money beside it is dollars a man-hour.
 * It used to be converted to units per man-hour here, by a helper with the crew
 * size sitting beside it, which meant the crew that divided the rate and the
 * crew printed on the line were two facts kept in step by hand. The engine does
 * the one conversion now (`perManHour`), off the one place it is written.
 *
 * `quotedBy` is whose number a subcontract line is. A sub's figure with no name
 * on it cannot be chased and cannot be defended in a buyout meeting. The names
 * below are invented, like every figure in this file.
 *
 * `off` takes a line off the bid and leaves it on the sheet, with the reason.
 */

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
  // ── what holds the base layer down, by zone ───────────────────────────
  // One fastener is one fastener wherever it goes in, so the three zone lines
  // share ONE price draw (`priceAs`) rather than three: a sheet that charged
  // more for a corner fastener than a field one would be saying something about
  // the fastener that is not true. What differs by zone is how many of them.
  //
  // The retired `plates` line was `SQ * 10` — ten fasteners-and-plates to the
  // square, everywhere, off the traced area. It knew nothing about a corner.
  { id: 'base-fasteners-field', description: 'Base layer fasteners, field',
    costCode: '07-100-170', unit: 'EA', layer: 'fastening', priceAs: 'base-fastener',
    order: { name: 'BOX', per: 1000, rule: 'ceil' }, band: [240, 290] },
  { id: 'base-fasteners-perimeter', description: 'Base layer fasteners, perimeter',
    costCode: '07-100-170', unit: 'EA', layer: 'fastening', priceAs: 'base-fastener',
    order: { name: 'BOX', per: 1000, rule: 'ceil' }, band: [240, 290] },
  { id: 'base-fasteners-corner', description: 'Base layer fasteners, corner',
    costCode: '07-100-170', unit: 'EA', layer: 'fastening', priceAs: 'base-fastener',
    order: { name: 'BOX', per: 1000, rule: 'ceil' }, band: [240, 290] },
  { id: 'base-plates', description: 'Barbed plates, 3 in', costCode: '07-100-170', unit: 'EA',
    layer: 'fastening', order: { name: 'BOX', per: 1000, rule: 'ceil' }, band: [180, 220] },
  // ── and what sticks the layers over it down ───────────────────────────
  // Also one price for three lines, and for the same reason: a set of tanks is
  // a set of tanks. The zones differ in area and in how many courses stand on
  // them, not in what the adhesive costs.
  { id: 'iso-adhesive-field', description: 'Insulation adhesive, dual tank set, field',
    costCode: '07-100-100', unit: 'EA', layer: 'adhesive', priceAs: 'iso-adhesive', band: [1050, 1320] },
  { id: 'iso-adhesive-perimeter', description: 'Insulation adhesive, dual tank set, perimeter',
    costCode: '07-100-100', unit: 'EA', layer: 'adhesive', priceAs: 'iso-adhesive', band: [1050, 1320] },
  { id: 'iso-adhesive-corner', description: 'Insulation adhesive, dual tank set, corner',
    costCode: '07-100-100', unit: 'EA', layer: 'adhesive', priceAs: 'iso-adhesive', band: [1050, 1320] },
  // Bought by the pail, which is why the line divides: the solve answers in
  // gallons because that is the unit the coverage rate is printed in.
  { id: 'bonding-adhesive', description: 'Bonding adhesive, 5 gallon pail',
    costCode: '07-100-100', unit: 'EA', layer: 'adhesive', band: [205, 255] },
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
  // A corner is cut, folded and welded; a straight length is run off the brake
  // and cut square. They are two prices, and pricing every piece at the
  // corner's rate put about three and a half thousand dollars of shop work on
  // forty straight lengths of this roof's parapet (C8 §3a).
  { id: 'mitre', description: 'Shop-formed corner, mitre welded', costCode: '07-100-150', unit: 'EA',
    band: [78, 100] },
  { id: 'coping-length', description: 'Coping length, shop-formed', costCode: '07-100-150', unit: 'EA',
    layer: 'edge metal', band: [18, 26] },
  { id: 'termbar', description: 'Termination bar and sealant', costCode: '07-100-160', unit: 'LF',
    band: [3.1, 3.7] },
  { id: 'metal-fasteners', description: 'Fasteners, cleat and termination bar', costCode: '07-100-170', unit: 'EA',
    order: { name: 'BOX', per: 1000, rule: 'ceil' }, band: [340, 420] },
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
  //
  // Stated the way a foreman states them: how many go up, and what they get
  // through in a day. Two of these moved, and the reason is on the sheet rather
  // than in anybody's judgement — the supervision line below carries fifteen
  // foreman days, and at the rates this file used to hold, the whole roof went
  // on in about four and a half. A production rate that cannot be reconciled
  // with the days somebody is being paid to stand there is the first thing an
  // estimator catches on somebody else's bid.
  //
  // What moved: the tapered package (a four course, ribbon adhered package on a
  // solved layout is not a single mechanically attached board, and sixty
  // squares a day by six men was the rate for the second of those), and the
  // membrane (adhered and welded, not laid loose). Tear off, the cover board,
  // the wall and the curb are where they were.
  //
  // A six-man crew now gets through about ten squares of this roof a day all
  // in, tear-off included, which is a fortnight on ninety-six squares and
  // reconciles with the fifteen days of supervision beside it.
  { id: 'tearoff', description: 'Tear off existing roof', costCode: '07-100-210', unit: 'SQ',
    production: { rate: 40, per: 'crew-day', crew: 6 }, band: [48, 58] },
  { id: 'set-insulation', description: 'Lay the tapered package', costCode: '07-100-230', unit: 'SQ',
    production: { rate: 25, per: 'crew-day', crew: 6 }, band: [48, 58] },
  { id: 'set-coverboard', description: 'Lay cover board', costCode: '07-100-230', unit: 'SQ',
    production: { rate: 50, per: 'crew-day', crew: 6 }, band: [48, 58] },
  { id: 'install-membrane', description: 'Weld the membrane', costCode: '07-100-230', unit: 'SQ',
    production: { rate: 40, per: 'crew-day', crew: 6 }, band: [48, 58] },
  { id: 'install-flashing', description: 'Flash the wall', costCode: '07-100-240', unit: 'LF',
    production: { rate: 120, per: 'crew-day', crew: 2 }, band: [48, 58] },
  { id: 'set-drain', description: 'Set the drains', costCode: '07-100-240', unit: 'EA',
    production: { rate: 3, per: 'crew-day', crew: 2 }, band: [48, 58] },
  { id: 'set-curb', description: 'Flash the curb', costCode: '07-100-240', unit: 'LF',
    production: { rate: 48, per: 'crew-day', crew: 2 }, band: [48, 58] },
  // ── the shop, and then the roof ───────────────────────────────────────
  //
  // Two lines for every piece of formed metal, which is how a sheet-metal
  // roofer prices it and how an Edge report prints it: fabricate it at the
  // brake, then carry it up and set it. They are different buildings, different
  // people and different weeks — a cap fabricated on Tuesday is set on Thursday
  // by somebody else — so one line covering both hides which half is slipping
  // when the job runs late.
  //
  // A shop hour is one pair of hands at the brake. Coping is a five-bend piece
  // with a hem each side and it runs slower than a cleat, which is two bends
  // and a hem.
  { id: 'fab-coping', description: 'Fabricate coping cap, 24 ga', costCode: '07-100-320', unit: 'LF',
    layer: 'edge metal', production: { rate: 70, per: 'shop-hour' }, band: [44, 54] },
  { id: 'fab-cleat', description: 'Fabricate continuous cleat, 20 ga', costCode: '07-100-320', unit: 'LF',
    layer: 'edge metal', production: { rate: 200, per: 'shop-hour' }, band: [44, 54] },
  { id: 'install-coping', description: 'Install coping cap, 24 ga', costCode: '07-100-300', unit: 'LF',
    layer: 'edge metal', production: { rate: 160, per: 'crew-day', crew: 2 }, band: [48, 58] },
  { id: 'install-cleat', description: 'Install continuous cleat, 20 ga', costCode: '07-100-300', unit: 'LF',
    layer: 'edge metal', production: { rate: 300, per: 'crew-day', crew: 2 }, band: [48, 58] },
  // ── bought in ─────────────────────────────────────────────────────────
  //
  // The metal sub's own number for the same work the two pairs above do in
  // house. It stays on the sheet and out of the total: it is the alternate, and
  // an alternate that gets deleted is one nobody can take when the shop is
  // three weeks out. Struck, with the reason, and the recap still says what
  // taking it would add back.
  { id: 'set-metal', description: 'Form and set the metal', costCode: '07-100-900', unit: 'LF',
    quotedBy: 'Bayside Sheet Metal', off: { reason: 'alternate — the shop is fabricating this' },
    band: [7.6, 9.0] },
  // Somebody else's crane and somebody else's mechanical crew. A curb that has
  // to be raised means the unit comes off it and goes back on it, and that is a
  // rigging day with a name on it and no hours of ours against it.
  { id: 'crane', description: 'Crane and curb rework', costCode: '07-100-900', unit: 'EA',
    quotedBy: 'Ridgeline Rigging', band: [2850, 3400] },
  { id: 'lift', description: 'Telehandler rental, weekly', costCode: '07-100-940', unit: 'EA',
    order: { name: 'EA', per: 1, rule: 'ceil' }, band: [1900, 2300] },
  { id: 'dumpster', description: 'Dumpster, 30 yard', costCode: '07-100-950', unit: 'EA',
    order: { name: 'EA', per: 1, rule: 'ceil' }, band: [720, 900] },
  { id: 'permit', description: 'Permit and inspections', costCode: '07-100-950', unit: 'EA',
    band: [980, 1300] },
  { id: 'warranty', description: 'Manufacturer warranty, 20 year NDL', costCode: '07-100-920', unit: 'SQ',
    band: [52, 68] },
  // A foreman day is a crew of one for a day. Stating it that way puts its
  // hours in the same column as everybody else's, which is where the Hours/SQ
  // figure needs them: a job supervised for fifteen days and built in four is
  // not a job anybody priced correctly, and the recap can only say so if the
  // supervision hours are in it.
  { id: 'supervision', description: 'Supervision, foreman day', costCode: '07-100-990', unit: 'EA',
    production: { rate: 1, per: 'crew-day', crew: 1 }, band: [58, 72] },
];

// ── the package, by letter ─────────────────────────────────────────────────
// One entry per lettered panel the manufacturer's charts NAME — every letter in
// every slope's set, not only the four this roof's current slope reaches.
//
// The reason is the whole of C9's second half. A quarter in twelve buys X, Y, Z
// and ZZ; an eighth buys AA, A, B, C and more; a half buys Q and QQ. Those are
// different series and not the same panels with moved edges, so a price book
// that carried only today's four would have nothing to sell the moment somebody
// set the high and low points and the solve re-lettered the roof. A supply
// house prices the panel, not this job's opinion of it.
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
void atDrain;

/**
 * Every lettered panel the whole document names, by designation.
 *
 * Across both sets of every slope, because which of them a roof buys is the
 * solve's answer and can change under the sheet.
 *
 * THE EXTENDED SETS FIRST, and the reason is a real collision in the document
 * rather than a preference: at a half inch in twelve the standard chart's Q1 is
 * a 2 1/2 to 4 1/2 panel and the extended chart's Q1 is a 4 1/2 to 6 1/2 one.
 * A designation is unique inside a set and not across the two of them, and the
 * name the solve puts in the scope (`TAPER_Q1_SF`) carries the letter and not
 * the set — so a book keyed on the letter alone is right for one of the two.
 * Extended, because that is the set this program defaults to (D151), and the
 * collision is printed at the end of this run rather than papered over.
 */
const panels = new Map();
const collisions = [];
for (const set of ['extended', 'standard']) {
  for (const slope of profile.slopes) {
    for (const panel of slope[set]?.panels ?? []) {
      const held = panels.get(panel.designation);
      if (held === undefined) { panels.set(panel.designation, panel); continue; }
      if (held.thin_edge_in !== panel.thin_edge_in || held.thick_edge_in !== panel.thick_edge_in) {
        collisions.push(panel.designation);
      }
    }
  }
}

/** As a roofer writes it: 1/2", 2 1/4", 4". */
const inchLabel = (v) => {
  const whole = Math.floor(v);
  const eighths = Math.round((v - whole) * 8);
  const fraction = eighths === 0 ? '' : `${eighths / 8}`.replace('0.', '').length > 0
    ? ` ${[0, '1/8', '1/4', '3/8', '1/2', '5/8', '3/4', '7/8'][eighths]}` : '';
  return `${whole || (fraction ? '' : 0)}${fraction}"`.trim();
};

/** What the supply house quotes each lettered panel at, by the panel's own id. */
const byLetter = [];
for (const [designation, panel] of panels) {
  const average = (panel.thin_edge_in + panel.thick_edge_in) / 2;
  byLetter.push({
    id: `iso-${designation.toLowerCase()}`,
    // THE LETTER FIRST. The panel is 364 px wide and the line's name ellipsizes
    // in it: "Tapered polyisocyanurate X, 1/4 : 12 — …" cut to "Tapered
    // polyisocyan…" on all four of them, so the sheet showed four lines that
    // read alike and differed only in their money. What an estimator needs to
    // tell them apart is the letter, and it is the second word.
    description: `Tapered ${designation}`,
    thickness: `${inchLabel(panel.thin_edge_in)}–${inchLabel(panel.thick_edge_in)}`,
    costCode: '07-100-100', unit: 'SQ',
    band: [PER_INCH_TAPERED[0] * average, PER_INCH_TAPERED[1] * average],
    waste: 8,
  });
}
CATALOGUE.push(...byLetter);
// The one line an estimator writes for the whole tapered package. The sheet
// shows one per letter the layout actually reached — the same way the fastener
// lines are three because a roof has three wind zones, except that how many
// letters there are is a fact about the SLOPE and changes under the sheet.
// Nothing about a letter is typed into the job file.
CATALOGUE.push({
  id: 'iso',
  description: 'Tapered *',
  eachOf: 'TAPER_*_SF',
  costCode: '07-100-100', unit: 'SQ',
  waste: 8,
  notes: 'One line per letter the layout bought. The letter on each course is under review '
    + 'against the manufacturer\u2019s chart: the panels read half an inch off the surface '
    + 'they are drawn under in the Section, and which side of the tie the trade takes is '
    + 'not settled.',
});
CATALOGUE.push({
  id: 'iso-fill',
  // No thickness in the name. The flat board under the package is 4 in on the
  // extended charts and 2 in on the standard ones, and which of those a roof
  // takes is decided by the solve — so a figure typed here is one that goes
  // quietly wrong the first time anybody moves the slope.
  description: 'Flat polyiso fill board',
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
  {
    condition: 'c-field', id: 'iso', formula: 'TAPER_*_SF / 100',
    notes: 'One line for the whole tapered package. The sheet draws one per '
      + 'lettered panel the layout reached, and each takes its price from the '
      + "book by its own name — so moving the slope into another chart's series "
      + 'buys the panels that chart prints rather than pricing nothing.',
  },
  { condition: 'c-field', id: 'iso-fill', formula: 'FILL_SF / 100' },
  { condition: 'c-drains', id: 'sump-panel', formula: 'SUMP_PANELS_EA' },
  {
    condition: 'c-cricket', id: 'cricket-stock', formula: 'CRICKET_SF',
    notes: 'Priced by the square foot of footprint off the field\u2019s own chart. Cricket '
      + 'stock is its own lettered series and the chart on disk carries only the primary '
      + 'slope, so this is an approximation until that chart is in hand.',
  },
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
    condition: 'c-parapet', id: 'mitre', formula: 'COPING_CORNERS_EA',
    notes: "Every turn the run makes: six outside and two inside on this roof's "
      + 'own trace, and both kinds are mitred. The straight lengths between them '
      + 'are the line under this one.',
  },
  {
    condition: 'c-parapet', id: 'coping-length', formula: 'COPING_PCS_EA - COPING_CORNERS_EA',
    notes: 'The pieces the run is cut into that are not corners — run off the '
      + 'brake, cut square, no weld. The section length sets how many there are.',
  },
  { condition: 'c-parapet', id: 'cleat', formula: 'CLEAT_LF' },
  { condition: 'c-parapet', id: 'metal-fasteners', formula: 'CLEAT_LF * (12 / FASTEN_OC)' },
  // C9: the shop and then the roof, for each of the two formed pieces, off the
  // same solved run the material lines are priced against. Fabricating and
  // setting were one bought-in line before this, which could not say which half
  // of it was ours.
  { condition: 'c-parapet', id: 'fab-coping', formula: 'COPING_LF' },
  { condition: 'c-parapet', id: 'install-coping', formula: 'COPING_LF' },
  { condition: 'c-parapet', id: 'fab-cleat', formula: 'CLEAT_LF' },
  { condition: 'c-parapet', id: 'install-cleat', formula: 'CLEAT_LF' },
  { condition: 'c-curb', id: 'curbflash', formula: 'FLASH_SF' },
  { condition: 'c-curb', id: 'curbcap', formula: 'COPING_LF * (STRETCHOUT / 12)' },
  {
    condition: 'c-curb', id: 'crane', formula: '1',
    notes: 'One rigging day: the unit comes off the curb and goes back on it so '
      + 'the curb can be raised and reflashed. A typed one, because it is a '
      + "quoted day and not a thing the drawing can be asked how many of.",
  },
  // C4: the base layer, by zone, and the adhesive over it. Three fastener lines
  // rather than one, because the zone is the whole point — a sheet with one
  // fastener line on it cannot be read to find out whether the corners were
  // counted, and this roof's corners take two and a half times what its field
  // does, board for board.
  {
    condition: 'c-field', id: 'base-fasteners-field', formula: 'FASTENERS_Z1_EA',
    notes: 'One fastened board to a board position, and the pattern for the field '
      + "zone off the manufacturer's own sheet at the uplift rating the field "
      + 'pressure asks for. Everything above the base layer is adhered.',
  },
  { condition: 'c-field', id: 'base-fasteners-perimeter', formula: 'FASTENERS_Z2_EA' },
  { condition: 'c-field', id: 'base-fasteners-corner', formula: 'FASTENERS_Z3_EA' },
  {
    condition: 'c-field', id: 'base-plates', formula: 'PLATES_EA',
    notes: 'One plate to a fastener, which is how the sheet prints them: '
      + '"Insulation Fasteners & Plates".',
  },
  { condition: 'c-field', id: 'iso-adhesive-field', formula: 'ADHESIVE_SETS_Z1_EA',
    notes: "The manufacturer's ribbon-spacing table stops at 120 mph and this roof is designed for 130, so the 120 mph row is used until the evaluation report is in hand. Priced at the spacing that row prints." },
  { condition: 'c-field', id: 'iso-adhesive-perimeter', formula: 'ADHESIVE_SETS_Z2_EA',
    notes: "The manufacturer's ribbon-spacing table stops at 120 mph and this roof is designed for 130, so the 120 mph row is used until the evaluation report is in hand. Priced at the spacing that row prints." },
  {
    condition: 'c-field', id: 'iso-adhesive-corner', formula: 'ADHESIVE_SETS_Z3_EA',
    notes: 'Every board course above the fastened one, and the cover board over the '
      + "lot, at the ribbon spacing the manufacturer's warranty table gives. The table "
      + 'stops at 120 mph and this roof is designed for 130, so the 120 mph row is used '
      + 'until the evaluation report is in hand.',
  },
  // C9: the two lines that are not work on the roof move off the roof field and
  // onto the group a sheet keeps them in. They were diluting every per-square
  // figure the field carried — a permit is not a cost of laying a square, and
  // reading it as one is exactly what the classes exist to stop.
  {
    condition: 'c-general', id: 'supervision', formula: '15',
    notes: 'Fifteen foreman days: a crew of one for a day apiece, so the hours '
      + 'land in the same column as everybody else\u2019s and the recap can be '
      + 'read against them. Typed, because how long somebody stands on a roof is '
      + 'a judgement and not a thing the drawing can be asked.',
  },
  { condition: 'c-general', id: 'permit', formula: '1' },
  {
    condition: 'c-field', id: 'bonding-adhesive', formula: 'BONDING_GAL / 5',
    notes: 'Gallons into five gallon pails. The coverage rate the manual prints '
      + 'already carries both faces, so it is not doubled here.',
  },
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

/**
 * Lines a solved bill replaces. Left on the job they would charge it twice.
 *
 * The four lettered ones join the list in C9: they were the panels a quarter in
 * twelve buys, typed into the job as four items, and the one line above stands
 * for all of them however many the slope turns out to want.
 */
const RETIRED = ['iso-tapered', 'plates', 'iso-x', 'iso-y', 'iso-z', 'iso-zz'];

/**
 * When the quotes on this job were good.
 *
 * A price with no date is a price nobody can defend — it is the first question
 * asked of a sub's number six weeks after it was given. Invented, like every
 * other figure here.
 */
const QUOTED_ON = '2026-09-02';

// `priceAs` is how two or three lines for the SAME product carry the same
// price. The draw is seeded on the id so that adding a line never re-prices the
// rest; a product sold once and billed on three lines needs one seed for all
// three, and saying which is better than three bands that happen to coincide.
const library = CATALOGUE.map(({ band, priceAs, quotedBy, ...item }) => ({
  ...item,
  // A family line has no price of its own: each letter the sheet draws from it
  // takes the book's price for that letter.
  ...(band === undefined ? {} : {}),
  // Whose number it is, where somebody quoted it. `priceSource` is where a
  // price says where it came from and how firm it is, and a subcontract line
  // is the case that has to carry a name.
  ...(quotedBy === undefined
    ? {}
    : { priceSource: { from: quotedBy, firmness: 'firm', on: QUOTED_ON } }),
  ...(band === undefined ? {} : { unitCost: between(priceAs ?? item.id, band[0], band[1]) }),
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
/**
 * General Conditions, as its own group on the sheet.
 *
 * Every commercial estimate this program is modelled on carries one: a
 * condition with nothing traced on it, holding what the job costs that is not
 * work on any particular part of the roof. Supervision and the permit sat on
 * the roof field before this, where they quietly went into every dollar a
 * square the field reported.
 *
 * Created here rather than hand-written into the job, so the file stays
 * regenerable from one source, and last in the list because the sheet reads in
 * the order a crew builds in.
 */
if (!conditions.some((c) => c.id === 'c-general')) {
  conditions.push({
    id: 'c-general',
    name: 'General Conditions — Labor & Misc',
    kind: 'count',
    traces: [],
    properties: {},
    items: [],
    color: '#7a7a82',
  });
}
// And off the roof field, where they were.
for (const condition of conditions) {
  if (condition.id !== 'c-field') continue;
  condition.items = (condition.items ?? []).filter(
    (i) => !['supervision', 'permit'].includes(i.id),
  );
}

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
    // The steps between the quantity and the money come with the product, the
    // same as its unit does. They were hand-authored on the demo's own lines
    // before this pass existed, which worked while every line with a step on it
    // was older than the catalogue — and would have priced the first derived
    // line that needed one as though the supplier sold single fasteners.
    if (priced.order !== undefined) item.order = priced.order;
    if (priced.price !== undefined) item.price = priced.price;
    // What the line IS in the roof, where the catalogue says. A line with no
    // thickness draws nothing in a section, and it is still true.
    if (priced.layer !== undefined) item.layer = priced.layer;
    if (priced.eachOf !== undefined) item.eachOf = priced.eachOf;
    else delete item.eachOf;
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
    if (priced.unitCost === undefined) delete item.unitCost;
    else item.unitCost = priced.unitCost;
    // The rate, the name on the quote and whether the line is off the bid all
    // come off the catalogue, and all three are REMOVED where it no longer
    // carries them. A rate left behind on a line the catalogue has stopped
    // calling labor is a line with hours nobody can account for.
    for (const field of ['production', 'priceSource', 'off']) {
      if (priced[field] !== undefined) item[field] = priced[field];
      else delete item[field];
    }
    // The two fields the rate used to be written in. Left behind they are a
    // second, stale statement of the same fact — which is the shape of defect
    // this whole pass exists to remove.
    delete item.productionRate;
    delete item.crewSize;
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

// ── and the letters, into the price set where a book belongs ───────────────
//
// A scenario IS a set of prices for the job, and this is the one thing on the
// demo that has to be priced from a book rather than on a line: the lettered
// panels are drawn from one item, so there is no line to type a price on, and
// which letters get drawn is the solve's answer rather than the estimator's.
//
// Every letter of every chart goes in, not only the four this slope reaches —
// so moving the high and low points buys the panels the new chart prints
// instead of a sheet of lines with no price against them.
//
// Into every price set, because this fixture's two differ in their adders and
// not in what the supply house charges. A second house that really quoted
// differently would take a second draw here.
const letterPrices = Object.fromEntries(byLetter.map((e) => [e.id, byId.get(e.id).unitCost]));
for (const scenario of job.job.scenarios) {
  scenario.prices = { ...(scenario.prices ?? {}), ...letterPrices };
}

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
if (collisions.length) {
  // Said out loud every run. A letter that means two panels is a fact about the
  // manufacturer's document, and the price book can only be right about one of
  // them until the solve says which set a roof is on.
  console.log('one designation is drawn at two thicknesses across the sets, '
    + `and the extended one is priced: ${[...new Set(collisions)].join(', ')}`);
}
console.log('Seeded, so the same run gives the same prices and screenshots do not churn.');

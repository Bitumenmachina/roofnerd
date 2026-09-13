// ── C4: the fastening plan, checked the way a roofer reads one ─────────────
// A crew holding this drawing has to be able to walk the roof and know, at any
// board, how many fasteners go in it. So the claim is not "the program computed
// something": it is that the overlay draws three named zones, says how wide the
// zone band is, says what uplift class each zone is fastened to, prints the
// standard it was drawn to, and that every count on it is the same count the
// Estimate buys.
//
// Front door throughout — the demo button, the view switch, the overlay chips,
// the Estimate tab, the condition tree. Nothing here calls `doc_open`, and the
// only command it invokes is the one the app itself pulls on connect.
//
// NOTHING IN IT IS TYPED. The zone width, the working pressure, the uplift class
// and the fastener count per board are all worked out here, from the job's own
// outline and its own typed pressures plus the manufacturer's table on disk —
// a second arithmetic against the core's. Where the two disagree, one of them is
// wrong and the row says which numbers they were.
//
// RED AT C3'S HEAD. There is no `wind.rs`, no `fastening.rs`, no zone on any
// drawing and no `FASTENERS_Z1_EA` anywhere in the scope; the Plan's `fastening`
// chip draws nothing and says so in a sentence. Every row below fails, which is
// the point of writing it first.
//
//   pnpm build:app && node tools/probe/section-fastening.mjs

import { mkdir, writeFile, readFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import {
  centreOf, errorsSoFar, launch, openDemoJob, showTab, showView, until, wait, watchErrors,
  commitStamp,
} from './tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '../..');
const EVIDENCE = join(ROOT, 'evidence');
const COMMIT = commitStamp();

const results = [];
const check = (name, fn) => {
  try { fn(); results.push('PASS'); console.log(`  PASS  ${name}`); }
  catch (e) { results.push('FAIL'); console.log(`  FAIL  ${name}\n        ${e.message}`); }
};

const shot = async (session, name) =>
  writeFile(join(EVIDENCE, `${name}-${COMMIT}.png`), Buffer.from(await session.screenshot(), 'base64'));

// ── what the job says ──────────────────────────────────────────────────────

const read = async (rel) => JSON.parse(await readFile(join(ROOT, rel), 'utf8'));

const conditions = await read('jobs/demo-job/conditions.json');
const pages = await read('jobs/demo-job/pages/pages.json');
const patterns = await read('data/fastening/dr-05-2024-07.json');

const scaleOf = (pageId) => (pages.find((p) => p.id === pageId) || {}).feetPerUnit ?? null;

/** A trace in feet, or nothing where its sheet has no scale. */
const feet = (trace) => {
  const s = scaleOf(trace.pageId);
  return s === null ? [] : trace.points.map((p) => ({ x: p.x * s, y: p.y * s }));
};

const fieldJob = conditions.find((c) => c.id === 'c-field');
assert.ok(fieldJob, 'the demo carries a roof field');
const ring = feet(fieldJob.traces[0]);
assert.ok(ring.length >= 3, 'the field is traced on a scaled sheet');

const xs = ring.map((p) => p.x);
const ys = ring.map((p) => p.y);
const leastSide = Math.min(Math.max(...xs) - Math.min(...xs), Math.max(...ys) - Math.min(...ys));

/** The polygon's own area, off the shoelace — the roof this is all divided into. */
const areaOf = (poly) => {
  let twice = 0;
  for (let i = 0; i < poly.length; i += 1) {
    const a = poly[i];
    const b = poly[(i + 1) % poly.length];
    twice += a.x * b.y - b.x * a.y;
  }
  return Math.abs(twice) / 2;
};
const fieldSf = areaOf(ring);

/** Inside the ring, by ray casting. Good enough for a lattice count. */
const inRing = (p, poly) => {
  let hit = false;
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i, i += 1) {
    const a = poly[i];
    const b = poly[j];
    if ((a.y > p.y) !== (b.y > p.y)
      && p.x < ((b.x - a.x) * (p.y - a.y)) / (b.y - a.y) + a.x) hit = !hit;
  }
  return hit;
};

const distanceToEdge = (p, a, b) => {
  const vx = b.x - a.x;
  const vy = b.y - a.y;
  const run = vx * vx + vy * vy;
  const t = run === 0 ? 0 : Math.max(0, Math.min(1, ((p.x - a.x) * vx + (p.y - a.y) * vy) / run));
  return Math.hypot(p.x - (a.x + vx * t), p.y - (a.y + vy * t));
};
const toBoundary = (p, poly) => {
  let least = Infinity;
  for (let i = 0; i < poly.length; i += 1) {
    least = Math.min(least, distanceToEdge(p, poly[i], poly[(i + 1) % poly.length]));
  }
  return least;
};

/** The corners that turn OUTWARD, which are the only ones with a corner zone. */
const outsideCorners = (poly) => {
  let twice = 0;
  for (let i = 0; i < poly.length; i += 1) {
    const a = poly[i];
    const b = poly[(i + 1) % poly.length];
    twice += a.x * b.y - b.x * a.y;
  }
  const out = [];
  for (let i = 0; i < poly.length; i += 1) {
    const before = poly[(i - 1 + poly.length) % poly.length];
    const here = poly[i];
    const after = poly[(i + 1) % poly.length];
    const ux = here.x - before.x;
    const uy = here.y - before.y;
    const wx = after.x - here.x;
    const wy = after.y - here.y;
    if ((ux * wy - uy * wx) * twice > 0) out.push({ before, here, after });
  }
  return out;
};

/** The square a corner zone is, `a` back along one edge and `a` on along the other. */
const cornerSquare = (corner, a) => {
  const ux = corner.here.x - corner.before.x;
  const uy = corner.here.y - corner.before.y;
  const wx = corner.after.x - corner.here.x;
  const wy = corner.after.y - corner.here.y;
  const lu = Math.hypot(ux, uy) || 1;
  const lw = Math.hypot(wx, wy) || 1;
  const u = { x: (ux / lu) * a, y: (uy / lu) * a };
  const w = { x: (wx / lw) * a, y: (wy / lw) * a };
  return [
    { x: corner.here.x, y: corner.here.y },
    { x: corner.here.x - u.x, y: corner.here.y - u.y },
    { x: corner.here.x - u.x + w.x, y: corner.here.y - u.y + w.y },
    { x: corner.here.x + w.x, y: corner.here.y + w.y },
  ];
};

/**
 * The three zones by area, counted on a lattice.
 *
 * A count on a lattice is not the solve's exact boolean and is not meant to be:
 * it is an independent reading, good to a fraction of a per cent on a roof this
 * size, and that is exactly enough to catch a band built the wrong width or a
 * corner square laid on the wrong side of its own corner.
 */
const zonesByLattice = (poly, a, step) => {
  const corners = outsideCorners(poly).map((c) => cornerSquare(c, a));
  let field = 0;
  let perimeter = 0;
  let corner = 0;
  const cell = step * step;
  for (let x = Math.min(...poly.map((p) => p.x)) + step / 2; x < Math.max(...poly.map((p) => p.x)); x += step) {
    for (let y = Math.min(...poly.map((p) => p.y)) + step / 2; y < Math.max(...poly.map((p) => p.y)); y += step) {
      const p = { x, y };
      if (!inRing(p, poly)) continue;
      if (corners.some((square) => inRing(p, square))) corner += cell;
      else if (toBoundary(p, poly) <= a) perimeter += cell;
      else field += cell;
    }
  }
  return { field, perimeter, corner };
};

// ── the wind chain, worked out here ────────────────────────────────────────
//
// NRCA *Membrane Roof Systems* 2019 Appendix 1: the `a` rule (p.553), the load
// reduction to working load (p.547), the safety factor (p.554) and the uplift
// rating ladder in fifteens (p.565).

const ASD_FACTOR = 0.6;
const SAFETY_FACTOR = 2.0;
const RATING_STEP = 15;
const LEAST_RATING = 60;

const dimensionA = (least, height) => {
  const bound = Math.min(least / 10, height * 0.4);
  return Math.max(bound, Math.max(least * 0.04, 3));
};

const ratingFor = (requiredPsf) =>
  Math.max(LEAST_RATING, Math.ceil(requiredPsf / RATING_STEP) * RATING_STEP);

/** The thickness a product name states, in inches: `2"`, `1-1/2"`, `5/8"`. */
const thicknessOf = (product) => {
  const m = /^(\d+)?(?:-)?(?:(\d+)\/(\d+))?\s*"/.exec(String(product).trim());
  if (m === null) return null;
  const whole = m[1] ? Number(m[1]) : 0;
  const part = m[2] ? Number(m[2]) / Number(m[3]) : 0;
  return whole + part;
};

/**
 * The pattern DR-05 prints for a board of this size, in this product, at or
 * above this rating — the fewest fasteners that reach it, off the thickest
 * minimum the board on the roof actually meets.
 */
const patternFor = (boardSize, rating, membrane, thickInches) => {
  let best = null;
  for (const pattern of patterns.patterns) {
    if (pattern.board !== boardSize) continue;
    for (const rated of pattern.ratings ?? []) {
      const psf = Number(String(rated.fm_class).replace(/^1-/, ''));
      if (!Number.isFinite(psf) || psf < rating) continue;
      for (const product of rated.products) {
        const name = String(product).toLowerCase();
        if (!name.includes('polyiso') && !name.includes('poyiso')) continue;
        if (!name.includes('insulbase')) continue;
        if (name.includes('recover')) continue;
        const membranes = /\(([^)]*)\)/.exec(name);
        if (membranes && /epdm|tpo|pvc|fleeceback/.test(membranes[1])
          && !membranes[1].includes(membrane.toLowerCase())) continue;
        const minimum = thicknessOf(product);
        if (minimum === null || minimum > thickInches) continue;
        const candidate = { psf, fasteners: pattern.fasteners, minimum };
        if (best === null
          || candidate.psf < best.psf
          || (candidate.psf === best.psf && candidate.minimum > best.minimum)) best = candidate;
      }
    }
  }
  return best;
};

// What the job types. Absent at C3's head, which is what makes the rows below red.
const heightFt = fieldJob.properties.ROOF_HEIGHT_FT ?? null;
const typedPsf = {
  field: fieldJob.properties.WIND_PSF_FIELD ?? null,
  perimeter: fieldJob.properties.WIND_PSF_PERIMETER ?? null,
  corner: fieldJob.properties.WIND_PSF_CORNER ?? null,
};

const aFeet = heightFt === null ? null : dimensionA(leastSide, heightFt);
const chain = (psf) => {
  if (psf === null) return null;
  const asd = psf * ASD_FACTOR;
  const required = asd * SAFETY_FACTOR;
  return { asd, required, rating: ratingFor(required) };
};
const chains = {
  field: chain(typedPsf.field),
  perimeter: chain(typedPsf.perimeter),
  corner: chain(typedPsf.corner),
};

console.log(`  REPORT  the roof is ${leastSide} ft on its least side and ${fieldSf} SF in all`);
console.log(`  REPORT  the job types a mean roof height of ${heightFt ?? 'nothing'}`
  + `, so the zone band is ${aFeet === null ? 'unknown' : aFeet.toFixed(2)} ft wide`);
for (const zone of ['field', 'perimeter', 'corner']) {
  const c = chains[zone];
  console.log(`  REPORT  ${zone}: ${typedPsf[zone] ?? 'nothing'} psf typed`
    + `${c === null ? '' : ` · ${c.asd.toFixed(1)} at working load · ${c.required.toFixed(1)} wanted · FM 1-${c.rating}`}`);
}

// ── what the window says ───────────────────────────────────────────────────

/** Press one of the Plan's overlay chips by the word on it. */
const pressChip = async (session, label) => {
  const found = await session.execute(function (want) {
    const b = [...document.querySelectorAll('.view-chip')]
      .find(function (e) { return (e.textContent || '').trim() === String(want); });
    if (!b) return false;
    b.click();
    return true;
  }, label);
  await wait(900);
  return found;
};

/** Everything drawn on an overlay, and the sentence under it. */
const drawingNow = (session, where) => session.execute(function (sel) {
  const svg = document.querySelector(sel);
  if (!svg) return JSON.stringify({ there: false });
  const marks = [...svg.querySelectorAll('[data-role]')];
  const note = document.querySelector('.drawing-note');
  return JSON.stringify({
    there: true,
    note: note && !note.hidden ? (note.textContent || '').replace(/\s+/g, ' ').trim() : '',
    marks: marks.length,
    roles: marks.map(function (e) { return e.getAttribute('data-role'); }),
    dimensions: [...svg.querySelectorAll('.dim-text')].map(function (e) {
      return (e.textContent || '').replace(/\s+/g, ' ').trim();
    }),
    texts: [...svg.querySelectorAll('text')].map(function (e) {
      return (e.textContent || '').replace(/\s+/g, ' ').trim();
    }),
  });
}, where);

/** Every line on the Estimate tab, as the estimator sees it. */
const linesNow = (session) => session.execute(function () {
  return JSON.stringify([...document.querySelectorAll('.cost-line')].map(function (r) {
    const text = function (sel) {
      const el = r.querySelector(sel);
      return el ? (el.textContent || '').trim() : null;
    };
    return {
      description: text('.line-item'),
      condition: text('.line-basis > span'),
      formula: text('.line-formula'),
      quantity: text('.line-qty'),
      unit: (text('.line-unit') || '').replace('◈', '').trim(),
      derived: !!r.querySelector('.line-unit .derived'),
    };
  }));
});

/** The derived names the Condition panel lists, with the words beside each. */
const panelNow = (session) => session.execute(function () {
  const panel = document.querySelector('.condition-panel');
  if (!panel) return JSON.stringify({ there: false });
  const rows = [...panel.querySelectorAll('.panel-derived .derived-row')].map(function (r) {
    const text = function (sel) {
      const el = r.querySelector(sel);
      return el ? (el.textContent || '').trim() : null;
    };
    return { name: text('.derived-name'), label: text('.derived-label'), value: text('.derived-value') };
  });
  return JSON.stringify({ there: true, rows });
});

/** Pick a condition in the tree beside the panel, the way a hand picks it. */
const pickCondition = async (session, name) => {
  const found = await session.execute(function (want) {
    const row = [...document.querySelectorAll('.cost-condition .condition-list .condition')]
      .find(function (e) { return (e.textContent || '').indexOf(want) >= 0; });
    if (!row) return false;
    row.click();
    return true;
  }, name);
  await wait(800);
  return found;
};

/** The named numbers the shell solved, pulled the way the app pulls them. */
const scopesNow = (session) => session.execute(function () {
  return window.__TAURI_INTERNALS__.invoke('solve_get', { part: 'scopes' })
    .then(function (s) { return JSON.stringify(s); })
    .catch(function (e) { return JSON.stringify({ error: String(e) }); });
});

/** The solved zones themselves, the same pull the Plan makes for the overlay. */
const fasteningNow = (session) => session.execute(function () {
  return window.__TAURI_INTERNALS__.invoke('solve_get', { part: 'fastening' })
    .then(function (s) { return JSON.stringify(s); })
    .catch(function (e) { return JSON.stringify({ error: String(e) }); });
});

const number = (text) => {
  const m = /-?[\d,]+(\.\d+)?/.exec(String(text ?? ''));
  return m === null ? null : Number.parseFloat(m[0].replace(/,/g, ''));
};

await mkdir(EVIDENCE, { recursive: true });
const app = await launch();
const { session } = app;

try {
  await until(session, () => document.querySelector('#editor')?.children.length > 0, { what: 'the window' });
  await watchErrors(session);

  const demoAt = await centreOf(session, '.start-demo');
  assert.ok(demoAt, 'the demo button is on the start screen');
  await openDemoJob(session);
  await until(session, () => document.querySelector('.view-switch'), { what: 'the job to open' });

  // ── the solve ────────────────────────────────────────────────────────────

  const solved = JSON.parse(await fasteningNow(session));
  const scopes = JSON.parse(await scopesNow(session));
  const fieldScope = scopes[fieldJob.id] ?? {};

  check('the zones are solved rather than answered "not solved yet"', () => {
    assert.ok(!solved.notSolvedYet, 'the shell still answers that fastening is not solved yet');
    assert.ok(Array.isArray(solved.fields) && solved.fields.length > 0,
      'the solve carries no wind field at all');
  });

  const wind = (solved.fields ?? [])[0] ?? {};
  const zones = wind.zones ?? [];
  const zoneOf = (name) => zones.find((z) => z.zone === name) ?? null;

  check('the zone band is as wide as the rule makes it, not as wide as anybody typed', () => {
    assert.ok(heightFt !== null, 'the job types no mean roof height for the zones to be set out from');
    assert.ok(typeof wind.aFeet === 'number', 'the solve reports no zone width');
    assert.ok(Math.abs(wind.aFeet - aFeet) < 0.01,
      `the solve set the band at ${wind.aFeet} ft where the rule gives ${aFeet}`);
  });

  check('the three zones tile the roof: nothing is in two of them and nothing is in none', () => {
    assert.equal(zones.length, 3, `the solve reports ${zones.length} zones`);
    const summed = zones.reduce((all, z) => all + z.squareFeet, 0);
    assert.ok(Math.abs(summed - fieldSf) < 0.01,
      `the zones come to ${summed.toFixed(2)} SF against the roof's own ${fieldSf} SF`);
  });

  check('each zone is the size a second reading of the same rule makes it', () => {
    assert.ok(aFeet !== null, 'there is no zone width to lay a lattice against');
    const mine = zonesByLattice(ring, aFeet, 0.5);
    for (const [name, want] of [['1', mine.field], ['2', mine.perimeter], ['3', mine.corner]]) {
      const z = zoneOf(name);
      assert.ok(z, `there is no zone ${name} on the solve`);
      const off = Math.abs(z.squareFeet - want) / Math.max(want, 1);
      assert.ok(off < 0.02,
        `zone ${name} is ${z.squareFeet.toFixed(1)} SF against a lattice's ${want.toFixed(1)} SF`);
    }
  });

  check('every zone carries the uplift class the pressure on it asks for', () => {
    for (const [name, key] of [['1', 'field'], ['2', 'perimeter'], ['3', 'corner']]) {
      const z = zoneOf(name);
      const c = chains[key];
      assert.ok(z, `there is no zone ${name} on the solve`);
      assert.ok(c, `the job types no ${key} pressure`);
      assert.ok(Math.abs(z.asdPsf - c.asd) < 0.01,
        `zone ${name} works at ${z.asdPsf} psf where ${c.asd} is the reduced load`);
      assert.equal(z.fmClass, c.rating,
        `zone ${name} is fastened to FM 1-${z.fmClass} where FM 1-${c.rating} is what it needs`);
    }
  });

  check('the pattern in each zone is the one the manufacturer prints for that class', () => {
    const thick = fieldScope.BASE_BOARD_IN ?? null;
    assert.ok(thick !== null, 'the solve does not say how thick the base board is');
    for (const [name, key] of [['1', 'field'], ['2', 'perimeter'], ['3', 'corner']]) {
      const z = zoneOf(name);
      const want = patternFor('4x4', chains[key].rating, 'TPO', thick);
      assert.ok(want, `the table prints no pattern for FM 1-${chains[key].rating} on a 4x4`);
      assert.equal(z.fastenersPerBoard, want.fasteners,
        `zone ${name} puts ${z.fastenersPerBoard} in a board where the table prints ${want.fasteners}`);
    }
  });

  check("NRCA's own worked example is in the table this reads", () => {
    // p.207: "an FM 1-90-rated roof assembly may have six fasteners per 4- by
    // 8-foot insulation board in the field zones, 10 in the perimeter zones and
    // 16 in the corner zones."
    for (const count of [6, 10, 16]) {
      const there = patterns.patterns.some((p) => p.board === '4x8' && p.fasteners === count
        && (p.ratings ?? []).some((r) => r.fm_class === '1-90'));
      assert.ok(there, `the table prints no 4x8 pattern of ${count} at FM 1-90`);
    }
  });

  check('the fastener count is the boards times the pattern, zone by zone', () => {
    let all = 0;
    for (const [name, scope] of [['1', 'FASTENERS_Z1_EA'], ['2', 'FASTENERS_Z2_EA'], ['3', 'FASTENERS_Z3_EA']]) {
      const z = zoneOf(name);
      assert.ok(z, `there is no zone ${name} on the solve`);
      const want = z.boards * z.fastenersPerBoard;
      assert.equal(fieldScope[scope], want,
        `${scope} is ${fieldScope[scope]} where ${z.boards} boards at ${z.fastenersPerBoard} give ${want}`);
      all += want;
    }
    assert.equal(fieldScope.PLATES_EA, all,
      `there are ${fieldScope.PLATES_EA} plates against ${all} fasteners, and they go in one to one`);
  });

  check('every base board is in exactly one zone, and the straddlers are counted', () => {
    const laid = zones.reduce((all, z) => all + z.boards, 0);
    assert.ok(typeof wind.baseBoards === 'number', 'the solve does not say how many base boards there are');
    assert.equal(laid, wind.baseBoards,
      `the zones hold ${laid} boards against ${wind.baseBoards} on the roof`);
    assert.ok(typeof wind.straddling === 'number',
      'the solve does not say how many boards lie across a zone boundary');
  });

  // ── the drawing ──────────────────────────────────────────────────────────

  await showView(session, 'Plan');
  await wait(600);
  const pressed = await pressChip(session, 'fastening');
  const plan = JSON.parse(await drawingNow(session, '.surface-overlay'));

  check('the Plan draws a fastening plan rather than saying there is none', () => {
    assert.ok(pressed, 'the Plan offers no fastening overlay');
    assert.ok(plan.there, 'there is no overlay on the sheet');
    assert.ok(plan.marks > 0, 'the fastening overlay is empty');
  });

  check('three bands are on it and each one is named in the trade\'s words', () => {
    const roles = new Set(plan.roles ?? []);
    for (const role of ['zone-field', 'zone-perimeter', 'zone-corner']) {
      assert.ok(roles.has(role), `nothing on the drawing is a ${role}`);
    }
    const said = (plan.texts ?? []).join(' ').toLowerCase();
    for (const word of ['field', 'perimeter', 'corner']) {
      assert.ok(said.includes(word), `the drawing never says "${word}"`);
    }
  });

  check('each band says what goes in a board there, and to what class', () => {
    const said = (plan.texts ?? []).join(' ');
    for (const name of ['1', '2', '3']) {
      const z = zoneOf(name);
      assert.ok(said.includes(`1-${z.fmClass}`),
        `the drawing never names FM 1-${z.fmClass}, which zone ${name} is fastened to`);
      assert.ok(said.includes(String(z.fastenersPerBoard)),
        `the drawing never says ${z.fastenersPerBoard}, which is what goes in a board in zone ${name}`);
    }
  });

  check('the drawing names the standard it was set out to, and the basis of the pressure', () => {
    const said = `${plan.note} ${(plan.texts ?? []).join(' ')}`;
    assert.ok(/ASCE 7-22/.test(said), 'the drawing never names the standard');
    assert.ok(/strength|working load/i.test(said), 'the drawing never says what the pressure is');
  });

  check('the zone width is dimensioned on the drawing at the width it was solved at', () => {
    const dims = plan.dimensions ?? [];
    assert.ok(dims.length > 0, 'nothing on the fastening plan is dimensioned');
    const feetOn = dims.map((d) => number(d)).filter((v) => v !== null);
    assert.ok(feetOn.some((v) => Math.abs(v - Math.floor(aFeet)) < 1.01),
      `no dimension on it reads near ${aFeet.toFixed(2)} ft — they read ${dims.join(', ')}`);
  });

  await shot(session, 'section-fastening-plan');

  // ── the sheet ────────────────────────────────────────────────────────────

  await showTab(session, 'Estimate');
  await wait(700);
  const lines = JSON.parse(await linesNow(session));

  check('the base layer is bought as three zone lines, each derived', () => {
    const zoneLines = lines.filter((l) => /FASTENERS_Z[123]_EA/.test(l.formula ?? ''));
    assert.equal(zoneLines.length, 3,
      `the Estimate carries ${zoneLines.length} zone fastener lines`);
    for (const line of zoneLines) {
      assert.ok(line.derived, `"${line.description}" does not read as derived`);
    }
    const summed = zoneLines.reduce((all, l) => all + (number(l.quantity) ?? 0), 0);
    const want = ['FASTENERS_Z1_EA', 'FASTENERS_Z2_EA', 'FASTENERS_Z3_EA']
      .reduce((all, n) => all + (fieldScope[n] ?? 0), 0);
    assert.ok(Math.abs(summed - want) < 1,
      `the three lines buy ${summed} fasteners against the solve's ${want}`);
  });

  check('no line on the roof still prices the fastening by the square', () => {
    const bySquare = lines.filter((l) => /fasten|plate/i.test(l.description ?? '')
      && /\bSQ\b/.test(l.formula ?? ''));
    assert.equal(bySquare.length, 0,
      `${bySquare.map((l) => l.description).join(', ')} still prices fastening by the square`);
  });

  check('the adhesive is bought by zone as well, and by the surface for the membrane', () => {
    const sets = lines.filter((l) => /ADHESIVE_SETS_Z[123]_EA/.test(l.formula ?? ''));
    assert.equal(sets.length, 3, `the Estimate carries ${sets.length} adhesive lines by zone`);
    for (const line of sets) assert.ok(line.derived, `"${line.description}" does not read as derived`);
    const bonding = lines.find((l) => /BONDING_GAL/.test(l.formula ?? ''));
    assert.ok(bonding, 'nothing on the Estimate buys the membrane its bonding adhesive');
    assert.ok(bonding.derived, 'the bonding adhesive does not read as derived');
  });

  await shot(session, 'section-fastening-estimate');

  // ── the panel ────────────────────────────────────────────────────────────

  await showTab(session, 'Condition');
  await wait(500);
  const picked = await pickCondition(session, 'Main Roof Field');
  const panel = JSON.parse(await panelNow(session));

  check('the panel lists the new names, each with a word an estimator reads', () => {
    assert.ok(picked, 'the roof field cannot be picked in the tree');
    assert.ok(panel.there, 'there is no Condition panel');
    const byName = new Map((panel.rows ?? []).map((r) => [r.name, r]));
    for (const name of [
      'FASTENERS_Z1_EA', 'FASTENERS_Z2_EA', 'FASTENERS_Z3_EA', 'PLATES_EA',
      'ZONE_A_FT', 'FM_CLASS_Z1', 'FM_CLASS_Z2', 'FM_CLASS_Z3', 'WIND_ASD_Z3_PSF',
      'ADHESIVE_SETS_Z1_EA', 'BONDING_GAL',
    ]) {
      const row = byName.get(name);
      assert.ok(row, `the panel does not list ${name}`);
      assert.ok(row.label && row.label !== name && !/_/.test(row.label),
        `${name} is listed as "${row.label}", which is its identifier and not a word`);
    }
  });
} catch (e) {
  console.log(`  FAIL  the probe itself: ${e.stack || e.message}`);
  results.push('FAIL');
} finally {
  const errors = await errorsSoFar(session).catch(() => []);
  if (errors.length) {
    console.log(`  REPORT  the console carried ${errors.length} of its own errors during the run:`);
    for (const e of errors) console.log(`    ${e}`);
  }
  await app.close().catch(() => undefined);
}

const pass = results.filter((r) => r === 'PASS').length;
console.log(`\n${pass}/${results.length} passed`);
process.exit(results.every((r) => r === 'PASS') ? 0 : 1);

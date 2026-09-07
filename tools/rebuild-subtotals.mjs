// ── Rebuilding a class subtotal from its item lines ────────────────────────
// Section 3's check as the handoff writes it: both recaps reproduced FROM THEIR
// QUANTITIES AND PRICES. Reproducing the ladder from printed class subtotals
// proved the recap arithmetic; this proves the item→class chain underneath it.
//
// Every item line goes through the engine's own three-step chain — estimating
// quantity, order quantity, price quantity, money — using the conversions the
// report itself states. The rebuilt cost-code subtotals are compared with the
// printed ones, then rolled into classes and compared with those.
//
// Reads fixtures/, which is local only. Prints the full table by default and a
// figureless shape with --shape.

import { readFile, readdir } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { CLASS_NAMES, priceLine } from '../packages/engine/dist/index.js';

const ROOT = resolve(import.meta.dirname, '..');
const shapeOnly = process.argv.includes('--shape');
const TOLERANCE = 0.0001;

/** Which class a cost code belongs to, by the last group of its number. */
function classOfCode(code) {
  const tail = Number(code.split('-')[2]);
  if (tail === 990) return 'Supervision';
  if (tail === 940) return 'Equipment';
  if (tail === 920 || tail === 950) return 'Other';
  if (tail === 900) return 'Sub';
  if (tail >= 200 && tail < 900) return 'Labor';
  return 'Material';
}

/**
 * Price one reported line THROUGH THE ENGINE.
 *
 * This is the point of the exercise. The report states an order quantity, a
 * price unit and a unit price; the engine is handed those as an item and asked
 * what it costs, and the answer is compared with what the report printed.
 *
 * Two kinds of line come out of a real report:
 *
 *   VERIFIED   the price unit is the order unit, so the whole chain is stated
 *              and the engine either reproduces the money or it does not.
 *
 *   DERIVED    the price unit differs from the order unit — membrane bought by
 *              the roll and quoted by the square foot — and the report does not
 *              print how many square feet are in a roll. The conversion is
 *              recovered from the money, which means the line cannot also be
 *              evidence that the money is right. It is counted separately and
 *              never allowed to look like a pass.
 */
function priceThroughEngine(line) {
  const { orderQuantity, orderUnit, unitPrice, priceUnit, quantity, unit } = line;
  const stated = priceUnit === orderUnit;

  // The report's order quantity is taken as given: what is under test here is
  // the step from there to money, which is where the third unit lives.
  // The report's order quantity is taken as given, so the order step is an
  // identity — what is under test is the step from there to money.
  const item = {
    id: 'x',
    description: line.description,
    costCode: line.costCode,
    unit,
    formula: 'Q',
    order: { name: orderUnit, contains: 1, rule: 'exact' },
    unitCost: unitPrice,
    ...(stated ? {} : {
      price: {
        name: priceUnit,
        // Recovered from the printed money — see DERIVED above.
        contains: orderQuantity && unitPrice ? line.netCost / (orderQuantity * unitPrice) : 1,
        rule: 'exact',
      },
    }),
  };

  const result = priceLine(item, { Q: orderQuantity });
  return { cost: result.extended, verified: stated, derivedFactor: stated ? null : item.price.contains };
}

// Whatever jobs are in the local tree. Naming them here would put a client's
// bid names in a public file, which is the thing this whole apparatus is for.
let jobs;
try {
  jobs = (await readdir(join(ROOT, 'fixtures/items')))
    .filter((f) => f.endsWith('.json'))
    .map((f) => f.replace(/\.json$/, ''))
    .sort();
} catch {
  console.error('No fixtures/items — they are a client\'s figures and are not in the repository.');
  process.exit(2);
}
if (jobs.length === 0) { console.error('No parsed jobs in fixtures/items.'); process.exit(2); }

let allWithin = true;

for (const name of jobs) {
  const parsed = JSON.parse(await readFile(join(ROOT, 'fixtures/items', `${name}.json`), 'utf8'));
  const recap = JSON.parse(await readFile(join(ROOT, 'fixtures/recaps', `${name}.json`), 'utf8'));

  // ── cost code subtotals, from the lines under them ──────────────────────
  const byCode = new Map();
  let verified = 0;
  let derived = 0;
  let lineMismatches = 0;
  const mismatched = [];

  for (const line of parsed.items) {
    if (!line.costCode) continue;
    const { cost, verified: isVerified } = priceThroughEngine(line);
    if (cost === null) { lineMismatches++; continue; }

    // Every line is checked against its own printed money, not just the totals.
    // A subtotal can come out right with two lines wrong in opposite directions.
    //
    // The tolerance is derived rather than flat. A report DISPLAYS an order
    // quantity to two decimal places and computes the money on the unrounded
    // one, so a line reading "N.NN boxes" was really N.NNNN boxes and its
    // printed money says so. Feeding the displayed figure back in can therefore
    // be out by up to half a hundredth of a unit's worth of money, and that is
    // the report's rounding showing through, not a disagreement about the
    // arithmetic. It is the rule the addendum states for hours, and it turns
    // out not to be only hours: it applies wherever a quantity is displayed.
    const displayRounding = 0.005 * Math.abs(line.unitPrice) + 0.01;
    const gap = Math.abs(cost - line.netCost);
    const relative = line.netCost === 0 ? (cost === 0 ? 0 : Infinity) : gap / Math.abs(line.netCost);
    if (gap > displayRounding && relative > TOLERANCE) {
      lineMismatches++;
      mismatched.push(line.description);
    }

    if (isVerified) verified++; else derived++;
    byCode.set(line.costCode, (byCode.get(line.costCode) ?? 0) + cost);
  }

  const codeRows = [];
  for (const [code, printed] of Object.entries(parsed.subtotals)) {
    const rebuilt = byCode.get(code) ?? 0;
    const off = printed === 0 ? (rebuilt === 0 ? 0 : Infinity) : Math.abs(rebuilt - printed) / Math.abs(printed);
    codeRows.push({ code, name: parsed.codeNames[code] ?? '', printed, rebuilt, off, within: off <= TOLERANCE });
  }

  // ── class subtotals, from the cost codes ────────────────────────────────
  const byClass = new Map(CLASS_NAMES.map((c) => [c, 0]));
  for (const [code, total] of byCode) {
    const cls = classOfCode(code);
    byClass.set(cls, (byClass.get(cls) ?? 0) + total);
  }

  const classRows = CLASS_NAMES.map((cls) => {
    const printed = recap.subtotals[cls] ?? 0;
    const rebuilt = byClass.get(cls) ?? 0;
    const off = printed === 0 ? (rebuilt === 0 ? 0 : Infinity) : Math.abs(rebuilt - printed) / Math.abs(printed);
    return { cls, printed, rebuilt, off, within: off <= TOLERANCE };
  });

  const ok = codeRows.every((r) => r.within) && classRows.every((r) => r.within) && lineMismatches === 0;
  allWithin &&= ok;

  console.log(`\n${'='.repeat(70)}`);
  console.log(`${name} — class subtotals rebuilt from ${parsed.items.length} item lines`);
  console.log('='.repeat(70));
  console.log(`${verified} line(s) priced from a fully stated chain; ` +
    `${derived} with a conversion the report does not print, recovered from its money`);
  console.log(`${lineMismatches} line(s) whose own money the engine did not reproduce`);
  if (lineMismatches && !shapeOnly) for (const d of mismatched) console.log(`    ${d}`);

  const pct = (v) => (v * 100).toFixed(4).replace(/0+$/, '').replace(/\.$/, '');

  if (shapeOnly) {
    console.log('Cost code\tResult');
    for (const r of codeRows) console.log(`${r.code}\t${r.within ? 'within' : `OUT by ${pct(r.off)}%`}`);
    console.log('\nClass\tResult');
    for (const r of classRows) console.log(`${r.cls}\t${r.within ? 'within' : `OUT by ${pct(r.off)}%`}`);
  } else {
    console.log('Cost code\tName\tPrinted\tRebuilt\tDiff');
    for (const r of codeRows) {
      console.log([r.code, r.name, r.printed.toFixed(2), r.rebuilt.toFixed(2),
        (r.rebuilt - r.printed).toFixed(2), r.within ? 'within' : `OUT ${pct(r.off)}%`].join('\t'));
    }
    console.log('\nClass\tPrinted\tRebuilt\tDiff');
    for (const r of classRows) {
      console.log([r.cls, r.printed.toFixed(2), r.rebuilt.toFixed(2),
        (r.rebuilt - r.printed).toFixed(2), r.within ? 'within' : `OUT ${pct(r.off)}%`].join('\t'));
    }
  }

  const badCodes = codeRows.filter((r) => !r.within).length;
  const badClasses = classRows.filter((r) => !r.within).length;
  console.log(`\n${codeRows.length - badCodes}/${codeRows.length} cost codes and ` +
    `${classRows.length - badClasses}/${classRows.length} classes within ${pct(TOLERANCE)}%`);
}

console.log(`\n${allWithin ? 'PASS' : 'FAIL'} — class subtotals rebuilt from the item lines`);
process.exit(allWithin ? 0 : 1);

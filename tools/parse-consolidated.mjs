// ── Reading a consolidated report ──────────────────────────────────────────
// Turns the text of an Edge consolidated report into item lines this program
// can price, so the money model can be checked against a real job rather than
// against itself.
//
// The report gives every line five things, which is exactly the chain the
// engine models:
//
//   Description | Quantity EU | Ord Qty Ord Un | Unit Price Prc Un | Net Cost
//
// Reads from fixtures/ and writes to fixtures/. Nothing it touches is public;
// this file carries no figures of its own.
//
//   node tools/parse-consolidated.mjs <text file> <output json>

import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';

const [, , input, output] = process.argv;
if (!input || !output) {
  console.error('usage: node tools/parse-consolidated.mjs <text file> <output json>');
  process.exit(1);
}

const text = await readFile(resolve(input), 'utf8');
const lines = text.split('\n');

const num = (s) => Number(String(s).replace(/[$,]/g, ''));

// A cost-code heading: "07-100-100-Roofing Material".
const CODE = /^\s*(\d{2}-\d{3}-\d{3})-(.+?)\s*$/;
// A full item line. The description may be cut short and continue below it.
//
// A unit is not always a tidy uppercase token. Real catalogues sell things by
// the "Carton" and by the "15 GAL" drum, so a unit may be mixed case and may
// carry a size in front of it with a space in between.
const UNIT = '((?:\\d+\\s+)?[A-Za-z0-9/]+)';
const ITEM = new RegExp(
  '^\\s*(.*?)\\s{2,}' +
  '([\\d,]+\\.\\d+)\\s+([A-Z0-9]+)\\s+' +          // quantity + estimating unit
  '([\\d,]+\\.\\d+)\\s+' + UNIT + '\\s+' +          // order quantity + order unit
  '\\$?\\s*([\\d,]+\\.?\\d*)\\s+' + UNIT + '\\s+' + // unit price + price unit
  '\\$\\s*([\\d,]+\\.\\d+)\\s*$',                  // net cost
);
// A cost-code subtotal: the heading repeated with a figure after it.
const SUBTOTAL = /^\s*(\d{2}-\d{3}-\d{3})-.+?\s+\$\s*([\d,]+\.\d+)\s*$/;

const items = [];
const subtotals = {};
const codeNames = {};
let code = null;
let started = false;

for (const raw of lines) {
  const line = raw.replace(/\s+$/, '');
  if (!line.trim()) continue;

  // Skip page furniture — letterheads, print dates, column headings.
  if (/Report Name:|Print Date:|Bid:|Bid Number:|Scenario:|Business Center|P: |Description\s+Quantity EU/.test(line)) {
    if (/Description\s+Quantity EU/.test(line)) started = true;
    continue;
  }
  if (!started) continue;
  if (/^\s*\d+ of \d+\s*$/.test(line)) continue;

  const sub = line.match(SUBTOTAL);
  if (sub) {
    subtotals[sub[1]] = (subtotals[sub[1]] ?? 0) + num(sub[2]);
    continue;
  }

  const heading = line.match(CODE);
  if (heading && !/\$/.test(line)) {
    code = heading[1];
    codeNames[code] = heading[2];
    continue;
  }

  const item = line.match(ITEM);
  if (item) {
    const [, description, qty, unit, ordQty, ordUnit, price, priceUnit, net] = item;
    items.push({
      costCode: code,
      description: description.trim(),
      quantity: num(qty),
      unit,
      orderQuantity: num(ordQty),
      orderUnit: ordUnit,
      unitPrice: num(price),
      priceUnit,
      netCost: num(net),
    });
    continue;
  }

  // A description that wrapped: no figures on it, and an item just above.
  if (items.length && !/[\d]/.test(line) && line.trim().length < 60) {
    const last = items[items.length - 1];
    last.description = `${last.description} ${line.trim()}`.replace(/\s+/g, ' ');
  }
}

await mkdir(dirname(resolve(output)), { recursive: true });
await writeFile(resolve(output), `${JSON.stringify({ codeNames, subtotals, items }, null, 2)}\n`);

// Only counts and code names — never a figure — go to the terminal.
console.log(`${items.length} item line(s) under ${Object.keys(codeNames).length} cost code(s)`);
console.log(`${Object.keys(subtotals).length} cost-code subtotal(s) read`);
const missing = items.filter((i) => !i.costCode).length;
if (missing) console.log(`${missing} line(s) landed outside any cost code — check the parse`);

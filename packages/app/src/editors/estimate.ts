// ── The Estimate Sheet ─────────────────────────────────────────────────────
// At Gate 1 this is the sheet: a line per item, the formula printed on the line
// and editable there, quantities, prices, totals. At Gate 2 the recap underneath
// it becomes real money.
//
// What is here is the other half of the working loop: numbers that come off the
// same document the Plan editor is changing. Tear this editor into its own
// window, drag it to the other monitor, and it keeps following.

import { at, subscribe, type Doc } from '../doc.js';

export function mountEstimate(host: HTMLElement): void {
  host.replaceChildren();

  const title = document.createElement('h2');
  title.textContent = 'Estimate Sheet';
  const note = document.createElement('p');
  note.className = 'editor-note';
  note.textContent =
    'The priced lines arrive at Gate 1 and the money at Gate 2. What this shows now is '
    + 'that it is reading the same job the Plan editor is writing — change a property '
    + 'there and these move, including from another window.';
  host.append(title, note);

  const table = document.createElement('table');
  const body = document.createElement('tbody');
  const head = document.createElement('thead');
  head.append(row(['Condition', 'Measure', 'Quantity', 'Unit'], 'th'));
  table.append(head, body);
  host.append(table);

  subscribe((doc: Doc) => {
    const condition = at('/conditions.json/0', doc) as
      | { name?: string; properties?: { height?: number; sides?: number } }
      | undefined;

    body.replaceChildren();
    if (!condition) {
      body.append(row(['no job open', '', '', ''], 'td'));
      return;
    }

    const name = condition.name ?? 'condition';
    const height = condition.properties?.height;
    const sides = condition.properties?.sides;

    // Gate 0 has no trace, so there is no measured run to work from. One
    // linear foot stands in, which keeps the arithmetic visible and honest
    // about being a placeholder rather than dressing it up as a takeoff.
    const run = 1;
    const wall = height === undefined ? undefined : run * height;
    const corners = sides;

    body.append(row([name, 'Run traced', fixed(run), 'LF'], 'td'));
    body.append(row([name, 'Wall flashing', fixed(wall), 'SF'], 'td'));
    body.append(row([name, 'Corners', fixed(corners), 'EA'], 'td'));

    const total = row(['', 'Total quantity', fixed(sum(run, wall, corners)), ''], 'td');
    total.className = 'total';
    body.append(total);
  });
}

const sum = (...xs: (number | undefined)[]) =>
  xs.every((x) => x !== undefined) ? xs.reduce((a, b) => a! + b!, 0) : undefined;

const fixed = (v: number | undefined) =>
  v === undefined ? '—' : v.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });

function row(cells: string[], kind: 'th' | 'td'): HTMLTableRowElement {
  const tr = document.createElement('tr');
  for (const [i, text] of cells.entries()) {
    const cell = document.createElement(kind);
    cell.textContent = text;
    if (kind === 'td' && i === 2) cell.className = 'num';
    tr.append(cell);
  }
  return tr;
}

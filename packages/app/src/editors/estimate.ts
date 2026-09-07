// ── The Estimate Sheet ─────────────────────────────────────────────────────
// At Gate 1 this is the sheet: a line per item, the formula printed on the line
// and editable there, quantities, prices, totals. At Gate 2 the recap underneath
// it becomes real money.
//
// What is here is the other half of the working loop: numbers that come off the
// same document the Plan editor is changing. Tear this editor into its own
// window, drag it to the other monitor, and it keeps following.

import { measure } from '@roofnerd/engine';
import { at, subscribe, type Doc } from '../doc.js';

type Condition = {
  id: string; name: string; kind: 'area' | 'line' | 'count';
  traces: { id: string; pageId: string; points: { x: number; y: number }[] }[];
  properties: Record<string, number>;
};
type Page = { id: string; feetPerUnit?: number };

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
    const conditions = (at('/conditions', doc) as Condition[]) ?? [];
    const pages = (at('/pages', doc) as Page[]) ?? [];
    const calibrations: Record<string, { feetPerUnit: number } | undefined> = {};
    for (const p of pages) if (p.feetPerUnit) calibrations[p.id] = { feetPerUnit: p.feetPerUnit };

    body.replaceChildren();
    if (!conditions.length) {
      body.append(row(['nothing traced yet', '', '', ''], 'td'));
      return;
    }

    for (const c of conditions) {
      const m = measure(c.kind, c.traces ?? [], c.properties ?? {}, calibrations);
      // Every condition carries all three at once. A blank is a page nobody has
      // scaled, not a zero — an unscaled sheet has no answer and says so.
      for (const [what, value, unit] of [
        ['Area', m.SF, 'SF'],
        ['Run', m.LF, 'LF'],
        ['Count', m.EA, 'EA'],
      ] as const) {
        if (value === null || (value === 0 && what !== 'Count')) continue;
        body.append(row([c.name, what, fixed(value), unit], 'td'));
      }
    }
  });
}

const fixed = (v: number | null | undefined) =>
  v == null ? 'pending' : v.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });

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

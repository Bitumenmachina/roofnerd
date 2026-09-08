// ── Reports ────────────────────────────────────────────────────────────────
// One estimate, filtered for whoever is receiving it.
//
// Pick who it is for on the left; the sheet on the right is the same estimate
// with their columns and nothing else. Nothing here changes a number — a lens
// leaves things out, and that is the whole of what it does.
//
// A concealed cost is never written into the page. Not greyed, not blurred, not
// present: a value that is in the document and merely hidden is one "view
// source" away from the supply house knowing the margin.

import {
  LENSES, conceals, lensById, priceJob, totalSquaresOf, recap as recapOf,
  type Lens,
} from '@roofnerd/engine';
import { at, doc, subscribe, type Doc } from '../doc.js';

let current = LENSES[0]!.id;
let stopSubscribing: (() => void) | null = null;

export function mountReports(host: HTMLElement): void {
  stopSubscribing?.();
  host.replaceChildren();
  host.classList.add('reports-editor');

  const picker = document.createElement('nav');
  picker.className = 'lens-picker';

  const sheet = document.createElement('div');
  sheet.className = 'lens-sheet';

  host.append(picker, sheet);

  const draw = () => {
    drawPicker(picker, () => draw());
    drawLens(sheet, doc());
  };
  stopSubscribing = subscribe(() => draw());
}

function drawPicker(host: HTMLElement, redraw: () => void): void {
  host.replaceChildren();
  const h = document.createElement('h3');
  h.textContent = 'Who is it for?';
  host.append(h);

  for (const lens of LENSES) {
    const b = document.createElement('button');
    b.type = 'button';
    b.className = lens.id === current ? 'lens on' : 'lens';
    const name = document.createElement('strong');
    name.textContent = lens.name;
    const who = document.createElement('span');
    who.textContent = lens.audience;
    b.append(name, who);
    b.addEventListener('click', () => { current = lens.id; redraw(); });
    host.append(b);
  }
}

type Row = Record<string, string>;

function drawLens(host: HTMLElement, d: Doc): void {
  host.replaceChildren();
  const lens = lensById(current);
  if (!lens) return;

  const head = document.createElement('header');
  const title = document.createElement('h2');
  title.textContent = lens.name;
  const who = document.createElement('p');
  who.className = 'lens-audience';
  who.textContent = `For ${lens.audience}.`;
  head.append(title, who);
  if (lens.note) {
    const note = document.createElement('p');
    note.className = 'lens-note';
    note.textContent = lens.note;
    head.append(note);
  }

  const download = document.createElement('button');
  download.type = 'button';
  download.className = 'lens-export';
  download.textContent = 'Copy as CSV';
  head.append(download);
  host.append(head);

  const rows = rowsFor(lens, d);

  const table = document.createElement('table');
  const thead = document.createElement('thead');
  const hr = document.createElement('tr');
  for (const c of lens.columns) {
    const th = document.createElement('th');
    th.textContent = c.heading;
    if (c.numeric) th.className = 'num';
    hr.append(th);
  }
  thead.append(hr);
  const tbody = document.createElement('tbody');
  for (const row of rows) {
    const tr = document.createElement('tr');
    for (const c of lens.columns) {
      const td = document.createElement('td');
      td.textContent = row[c.key] ?? '';
      if (c.numeric) td.className = 'num';
      tr.append(td);
    }
    tbody.append(tr);
  }
  table.append(thead, tbody);
  host.append(table);

  download.addEventListener('click', () => {
    const csv = [
      lens.columns.map((c) => c.heading).join(','),
      ...rows.map((r) => lens.columns.map((c) => quote(r[c.key] ?? '')).join(',')),
    ].join('\n');
    void navigator.clipboard?.writeText(csv);
    download.textContent = 'Copied';
    setTimeout(() => { download.textContent = 'Copy as CSV'; }, 1500);
  });
}

const quote = (v: string) => (/[",\n]/.test(v) ? `"${v.replace(/"/g, '""')}"` : v);
const num = (v: number | null, places = 2) =>
  (v === null ? '' : v.toLocaleString('en-US', { minimumFractionDigits: places, maximumFractionDigits: places }));
const money = (v: number | null) => (v === null ? '' : `$${num(v)}`);

/**
 * The rows a lens shows.
 *
 * Concealed values are never put in the row at all — the column does not exist
 * and neither does the string. That is what "removed rather than hidden" has to
 * mean for it to be worth anything.
 */
function rowsFor(lens: Lens, d: Doc): Row[] {
  const job = at('/job', d) as
    { scenarios?: { id: string; name: string }[]; activeScenarioId?: string } | undefined;
  const scenario = job?.scenarios?.find((s) => s.id === job.activeScenarioId) ?? job?.scenarios?.[0];
  if (!scenario) return [];

  const document_ = { ...(d as object) } as Parameters<typeof priceJob>[0];

  if (lens.id === 'recap') {
    const r = recapOf(document_, scenario as Parameters<typeof recapOf>[1]);
    return r.classes.map((c) => ({
      class: c.name,
      cost: money(c.total),
      perSquare: c.perSquare === null ? '' : `${money(c.perSquare)}/SQ`,
    }));
  }

  const lines = priceJob(document_, scenario as Parameters<typeof priceJob>[1]);
  const showLabor = !conceals(lens, 'labor');
  const showCost = !conceals(lens, 'cost');

  if (lens.id === 'condition-summary') {
    const byCondition = new Map<string, { quantity: number; cost: number; unit: string }>();
    for (const l of lines) {
      const at_ = byCondition.get(l.conditionName) ?? { quantity: 0, cost: 0, unit: l.unit };
      at_.cost += l.extended ?? 0;
      byCondition.set(l.conditionName, at_);
    }
    return [...byCondition].map(([name, v]) => ({
      condition: name,
      quantity: '',
      unit: '',
      ...(showCost ? { cost: money(v.cost) } : {}),
    }));
  }

  return lines
    .filter((l) => showLabor || l.hours === null)
    .map((l) => {
      const row: Row = {
        condition: l.conditionName,
        item: l.item.description,
        costCode: l.item.costCode,
        quantity: num(l.withWaste ?? l.quantity),
        unit: l.unit,
        // With no order step you buy what you measured — that is what the model
        // means by an absent step, and a stocking list that printed a blank
        // there would be sending the supply house a line with no quantity on
        // it. Blank is not a quantity.
        orderQuantity: num(l.orderQuantity ?? l.withWaste, 2),
        orderUnit: l.orderUnitName ?? l.unit,
        sent: '',
        returned: '',
      };
      // Each of these is added only when the lens keeps it. An absent key is an
      // absent cell, and an absent cell is nothing to leak.
      if (showCost) row['cost'] = money(l.extended);
      if (!conceals(lens, 'waste')) row['waste'] = l.item.waste === undefined ? '' : `${l.item.waste}%`;
      if (!conceals(lens, 'derivation')) row['formula'] = l.item.formula;
      if (!conceals(lens, 'notes') && l.item.notes) row['notes'] = l.item.notes;
      return row;
    });
}

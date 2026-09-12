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
  LENSES, conceals, conditionTotal, lensById, priceJob, recap as recapOf,
  type Lens, type LensColumn,
} from '@roofnerd/engine';
import { at, doc, subscribe, writeExport, type Doc } from '../doc.js';
import { sayInStatus } from '../chrome.js';

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

/** How many squares the recap's rates were taken over. Printed beside them. */
let perSquareOver: number | null = null;

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

  // Two ways out of a lens, side by side: onto the clipboard for a message, or
  // into the job's own folder as a file to attach. The text is the same text.
  const actions = document.createElement('div');
  actions.className = 'lens-actions';

  const download = document.createElement('button');
  download.type = 'button';
  download.className = 'lens-export';
  download.textContent = 'Copy as CSV';

  const write = document.createElement('button');
  write.type = 'button';
  // Its own name alongside the shared look, so the check can put its hand on
  // this button rather than on whichever one happens to read 'Export CSV'
  // — and then read the word off it.
  write.className = 'lens-export lens-write';
  write.textContent = 'Export CSV';
  // Nothing open, nothing to write. The shell refuses it as well; this is so
  // the button never looks like it would work.
  write.disabled = !jobIsOpen(d);

  actions.append(download, write);
  head.append(actions);

  // Where the reason goes when nothing could be written: in the window, beside
  // the button that was pressed. Never a browser dialog — this webview does not
  // implement one — and never silence.
  const trouble = document.createElement('p');
  trouble.className = 'lens-trouble';
  trouble.hidden = true;
  head.append(trouble);

  host.append(head);

  perSquareOver = null;
  const rows = rowsFor(lens, d);
  const left = leftOut(lens, d);

  const table = document.createElement('table');
  const thead = document.createElement('thead');
  const hr = document.createElement('tr');
  for (const c of lens.columns) {
    const th = document.createElement('th');
    th.textContent = headingOf(c);
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

  // A total that leaves something out says what it left out. The recap's own
  // arithmetic already knows — it collects every line it could not add — and a
  // sheet that showed the total without it would be the silent zero this
  // program is not allowed to have.
  if (left.length > 0) {
    const note = document.createElement('div');
    note.className = 'lens-pending';
    const h = document.createElement('strong');
    h.textContent = `Not in this total (${left.length}):`;
    const ul = document.createElement('ul');
    for (const p of left) {
      const li = document.createElement('li');
      li.textContent = p;
      ul.append(li);
    }
    note.append(h, ul);
    host.append(note);
  }

  // Built once, from the rows the table above was drawn from. The clipboard and
  // the file cannot disagree because there is nothing for them to disagree about.
  const csv = csvFor(lens, rows, left);

  download.addEventListener('click', () => {
    void navigator.clipboard?.writeText(csv);
    download.textContent = 'Copied';
    setTimeout(() => { download.textContent = 'Copy as CSV'; }, 1500);
  });

  write.addEventListener('click', () => {
    trouble.hidden = true;
    trouble.textContent = '';
    // A text file ends in a newline; a paste does not want one. Same text.
    writeExport(`${lens.id}-${today()}.csv`, `${csv}\n`).then(
      (path) => {
        sayInStatus(host, `Exported to ${path}`);
        write.textContent = 'Exported';
        setTimeout(() => { write.textContent = 'Export CSV'; }, 1500);
      },
      (e: unknown) => {
        const why = typeof e === 'string' ? e : e instanceof Error ? e.message : String(e);
        trouble.textContent = `Nothing was written: ${why}`;
        trouble.hidden = false;
      },
    );
  });
}

/** Is a job open at all? Its name answers it, the way the window chrome asks. */
const jobIsOpen = (d: Doc): boolean => {
  const name = at('/job/name', d);
  return typeof name === 'string' && name.length > 0;
};

/**
 * A column's heading, with the divisor on the rate column.
 *
 * The divisor goes in the heading of the column it made, where it cannot be
 * read apart from the rate — on the screen and in the file both, because the
 * file is the copy that leaves the office.
 */
const headingOf = (c: LensColumn): string =>
  c.key === 'perSquare' && perSquareOver
    ? `${c.heading} · over ${perSquareOver.toFixed(2)} SQ`
    : c.heading;

/**
 * A lens as CSV — one builder, two destinations.
 *
 * The clipboard and the file come through here together. Two builders is how a
 * pasted table and an attached sheet end up a column apart, and nobody finds
 * out until the one that went out was the wrong one.
 *
 * What the total left out travels with the total. A sheet that printed the
 * number and not the footnote under it would be the silent zero at the one
 * place it matters most: after it has left the building. Padded to the table's
 * width so the file is still a rectangle anything can read.
 */
function csvFor(lens: Lens, rows: Row[], left: readonly string[]): string {
  const width = lens.columns.length;
  const line = (cells: readonly string[]) =>
    Array.from({ length: width }, (_, i) => quote(cells[i] ?? '')).join(',');

  const out = [
    line(lens.columns.map(headingOf)),
    ...rows.map((r) => line(lens.columns.map((c) => r[c.key] ?? ''))),
  ];
  if (left.length > 0) {
    out.push(line([]), line([`Not in this total (${left.length})`]));
    for (const p of left) out.push(line([p]));
  }
  return out.join('\n');
}

/**
 * Today, on the calendar of whoever is sitting here.
 *
 * Not the ISO string off a Date: that is UTC, and a sheet exported at eight in
 * the evening on this coast would be named after tomorrow.
 */
function today(): string {
  const now = new Date();
  const two = (n: number) => String(n).padStart(2, '0');
  return `${now.getFullYear()}-${two(now.getMonth() + 1)}-${two(now.getDate())}`;
}

/**
 * What a lens's total could not add.
 *
 * Only the recap has one — it is the only lens that rolls up to a selling
 * price, so it is the only one where a missing line changes a number somebody
 * signs. The other lenses show lines, and a line with no price says so on
 * itself.
 */
function leftOut(lens: Lens, d: Doc): string[] {
  if (lens.id !== 'recap' && lens.id !== 'condition-summary') return [];
  const job = at('/job', d) as
    { scenarios?: { id: string }[]; activeScenarioId?: string } | undefined;
  const scenario = job?.scenarios?.find((s) => s.id === job.activeScenarioId) ?? job?.scenarios?.[0];
  if (!scenario) return [];
  // The Condition Summary adds money up per condition, so it can leave a line
  // out the same way the recap can, and it owes the same footnote.
  if (lens.id === 'condition-summary') {
    return priceJob(
      { ...(d as object) } as Parameters<typeof priceJob>[0],
      scenario as Parameters<typeof priceJob>[1],
    )
      .filter((l) => l.extended === null)
      .map((l) => `${l.conditionName} → ${l.item.description || l.item.id}: ${l.pending ?? 'no money'}`);
  }
  const r = recapOf(
    { ...(d as object) } as Parameters<typeof recapOf>[0],
    scenario as Parameters<typeof recapOf>[1],
  );
  // A rate is a total with a divisor under it, and the divisor leaves things out
  // the same way a total does. An area traced and priced by nothing is correctly
  // out of the per-square — and saying so is what turns $4,274 a square from a
  // number nobody can explain into a number with a reason.
  return [...r.pending, ...r.squaresLeftOut];
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
    // `class`, not `name` — a ClassLine is named by the class it is. Reading a
    // field that is not on the type printed a recap of blank rows with money
    // beside them, and every check passed because none of them read the recap.
    // Found by opening a real job and looking at the sheet.
    perSquareOver = r.totalSquares;
    return r.classes.map((c) => ({
      class: c.class,
      cost: money(c.total),
      perSquare: c.perSquare === null ? '' : `${money(c.perSquare)}/SQ`,
    }));
  }

  const lines = priceJob(document_, scenario as Parameters<typeof priceJob>[1]);
  const showLabor = !conceals(lens, 'labor');
  const showCost = !conceals(lens, 'cost');

  if (lens.id === 'condition-summary') {
    // Keyed by the condition's id, not by the words on it. Two conditions can
    // carry one name — a parapet on the high roof and a parapet on the low one
    // are both "Parapet Wall Flashing" to whoever traced them — and grouping by
    // the label added their money into a single row named after one of them.
    // Nothing on screen said so; the row looked like a condition and was two.
    const names = new Map<string, string>();
    for (const l of lines) if (!names.has(l.conditionId)) names.set(l.conditionId, l.conditionName);
    // `conditionTotal` is the engine's, and it is the same call the panel makes:
    // the cents each line is shown at, added up. A line with no money is not a
    // line worth nothing — it stays out of the sum and comes back under the
    // table as a footnote, the way the recap's do.
    return [...names].map(([id, name]) => {
      const total = conditionTotal(lines, id);
      return {
        condition: name,
        quantity: '',
        unit: '',
        ...(showCost
          ? { cost: total.total === null ? 'nothing priced on it' : money(total.total) }
          : {}),
      };
    });
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

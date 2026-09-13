// ── The lens renderer ──────────────────────────────────────────────────────
// One estimate, filtered for whoever is receiving it.
//
// Who it is for is chosen in the document bar — "Showing …", the handoff's
// "WHO IS LOOKING" menu — and what comes out is drawn here, in the cost panel,
// over the same document. There is no Reports editor any more: a lens is a
// read over the one estimate and it belongs where the money already is, not in
// an editor of its own with a second copy of the picker beside it.
//
// Nothing here changes a number — a lens leaves things out, and that is the
// whole of what it does.
//
// A concealed cost is never written into the page. Not greyed, not blurred, not
// present: a value that is in the document and merely hidden is one "view
// source" away from the supply house knowing the margin.

import {
  conceals, conditionTotal, lensById, priceJob, recap as recapOf,
  type Lens, type LensColumn,
} from '@roofnerd/engine';
import { at, derived, doc, subscribe, writeExport, type Doc } from '../doc.js';
import { sayInStatus } from '../chrome.js';

let stopSubscribing: (() => void) | null = null;

/**
 * Draw one lens into a host and keep it drawn.
 *
 * The lens is named by the caller — the document bar owns the choice — so this
 * has no state of its own beyond the subscription it has to release.
 */
export function mountLens(host: HTMLElement, lensId: string): void {
  stopSubscribing?.();
  host.replaceChildren();
  host.classList.add('reports-editor');

  const sheet = document.createElement('div');
  sheet.className = 'lens-sheet';
  host.append(sheet);

  stopSubscribing = subscribe(() => drawLens(sheet, doc(), lensId));
}

type Row = Record<string, string>;

/**
 * What a row under a class is indented by — the subtotal and each adder.
 *
 * Two spaces of an en quad rather than a stylesheet rule: the same rows go into
 * the CSV, where the indent has to survive as a character or the file reads as
 * a flat list of classes with a tax line among them.
 */
const INSIDE = '  ';

/** How many squares the recap's rates were taken over. Printed beside them. */
let perSquareOver: number | null = null;

/**
 * The figures a recap carries that are not rows in its table.
 *
 * The ladder from job cost to selling price, and then how much roof, how many
 * hours, and hours a square. Held here between building the rows and drawing
 * the sheet, the same way the divisor above is.
 *
 * The ladder is here because the sheet promises it — "the roll-up to selling
 * price, by class" — and printed none of it: an owner could add the six class
 * totals to a job cost and had no way from there to the price in the footer.
 * Overhead, profit and bond were all worked out and all thrown away (C8 §4).
 */
let recapSummary: readonly (readonly [string, string])[] = [];

function drawLens(host: HTMLElement, d: Doc, lensId: string): void {
  host.replaceChildren();
  const lens = lensById(lensId);
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
  recapSummary = [];
  const rows = rowsFor(lens, d);
  const left = leftOut(lens, d);
  const struck = takenOff(lens, d);

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

  // Under the ladder: how much roof, how many hours, and the two divided. The
  // last of those is the figure an estimator checks before any of the money
  // above it — it says whether somebody believed a week the crew can work.
  if (recapSummary.length > 0) {
    const summary = document.createElement('div');
    summary.className = 'lens-summary';
    for (const [name, value] of recapSummary) {
      const row = document.createElement('div');
      row.className = 'summary-row';
      const k = document.createElement('span');
      k.className = 'summary-name';
      k.textContent = name;
      const v = document.createElement('span');
      v.className = 'summary-value num';
      v.textContent = value;
      row.append(k, v);
      summary.append(row);
    }
    host.append(summary);
  }

  // Lines somebody took off the bid, kept where they can be argued about. Not
  // the same list as the one below it: that is what nobody could work out, and
  // this is what somebody decided.
  if (struck.length > 0) {
    const note = document.createElement('div');
    note.className = 'lens-struck';
    const h = document.createElement('strong');
    h.textContent = `On the sheet and out of this total (${struck.length}):`;
    const ul = document.createElement('ul');
    for (const o of struck) {
      const li = document.createElement('li');
      li.textContent = o;
      ul.append(li);
    }
    note.append(h, ul);
    host.append(note);
  }

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
  const csv = csvFor(lens, rows, left, recapSummary, struck);

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
function csvFor(
  lens: Lens, rows: Row[], left: readonly string[],
  summary: readonly (readonly [string, string])[] = [],
  struck: readonly string[] = [],
): string {
  const width = lens.columns.length;
  const line = (cells: readonly string[]) =>
    Array.from({ length: width }, (_, i) => quote(cells[i] ?? '')).join(',');

  const out = [
    line(lens.columns.map(headingOf)),
    ...rows.map((r) => line(lens.columns.map((c) => r[c.key] ?? ''))),
  ];
  // What is on the screen is what is in the file. Hours a square is a figure
  // somebody will check the attached sheet for, and a file that carried the
  // ladder without it would be a different report from the one on screen.
  if (summary.length > 0) {
    out.push(line([]));
    for (const [name, value] of summary) out.push(line([name, value]));
  }
  if (struck.length > 0) {
    out.push(line([]), line([`On the sheet and out of this total (${struck.length})`]));
    for (const o of struck) out.push(line([o]));
  }
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
      derived(),
    )
      .filter((l) => l.extended === null)
      .map((l) => `${l.conditionName} → ${l.item.description || l.item.id}: ${l.pending ?? 'no money'}`);
  }
  const r = recapOf(
    { ...(d as object) } as Parameters<typeof recapOf>[0],
    scenario as Parameters<typeof recapOf>[1],
    derived(),
  );
  // A rate is a total with a divisor under it, and the divisor leaves things out
  // the same way a total does. An area traced and priced by nothing is correctly
  // out of the per-square — and saying so is what turns $4,274 a square from a
  // number nobody can explain into a number with a reason.
  return [...r.pending, ...r.squaresLeftOut];
}

/**
 * The lines this lens is carrying and not adding up, each with its reason.
 *
 * Every lens that totals anything owes this the same way it owes the pending
 * footnote. A sheet that printed the number without the struck lines under it
 * would be the silent zero at the one place it matters most — after the sheet
 * has left the building and somebody is reading it without us in the room.
 */
function takenOff(lens: Lens, d: Doc): string[] {
  if (lens.id !== 'recap' && lens.id !== 'condition-summary') return [];
  const job = at('/job', d) as
    { scenarios?: { id: string }[]; activeScenarioId?: string } | undefined;
  const scenario = job?.scenarios?.find((s) => s.id === job.activeScenarioId) ?? job?.scenarios?.[0];
  if (!scenario) return [];
  return recapOf(
    { ...(d as object) } as Parameters<typeof recapOf>[0],
    scenario as Parameters<typeof recapOf>[1],
    derived(),
  ).off.map((o) => `${o.where} → ${o.what}: ${o.reason}`
    + (o.amount === null ? '' : ` (${money(o.amount)})`));
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
    const r = recapOf(document_, scenario as Parameters<typeof recapOf>[1], derived());
    // `class`, not `name` — a ClassLine is named by the class it is. Reading a
    // field that is not on the type printed a recap of blank rows with money
    // beside them, and every check passed because none of them read the recap.
    // Found by opening a real job and looking at the sheet.
    perSquareOver = r.totalSquares;
    const rate = (v: number | undefined) => (v ? ` · ${num(v, 2).replace(/\.00$/, '')}%` : '');
    const s = scenario as { overhead?: number; profit?: number; bond?: number };
    // Held for the summary block under the table: the ladder this sheet is
    // named after, and then how much roof, how many hours, and the two divided.
    recapSummary = [
      ['Job cost', money(r.jobCost)],
      [`Overhead${rate(s.overhead)}`, money(r.overhead)],
      [`Profit${rate(s.profit)}`, money(r.profit)],
      ['Contract amount', money(r.contractAmount)],
      [`Bond${rate(s.bond)}`, money(r.bond)],
      ['Selling price', money(r.sellingPrice)],
      ['Total SQ', r.totalSquares === null ? '—' : num(r.totalSquares)],
      ['Total hours', num(r.totalHours)],
      ['Hours/SQ', r.hoursPerSquare === null ? '—' : num(r.hoursPerSquare)],
    ];
    // A class, then what is inside it. The class total carries its own adders —
    // tax and escalation on material, burden on labor and on supervision,
    // general liability on a sub — and a sheet that printed only the total put
    // eleven thousand dollars of tax inside a figure called "Material" with
    // nothing on the page saying so. An estimator checking the recap against
    // the sheet found a supervision class half again as big as its one
    // supervision line and could not say why (C8 §4). The engine already builds
    // them named, on the subtotal, and its own words are "never folded in".
    const out: Row[] = [];
    for (const c of r.classes) {
      out.push({
        class: c.class,
        cost: money(c.total),
        perSquare: c.perSquare === null ? '' : `${money(c.perSquare)}/SQ`,
        // Nought hours against Material is a fact and reads as one; a blank would
        // read as a figure nobody worked out.
        hours: num(c.hours),
      });
      if (c.adders.length === 0) continue;
      out.push({ class: `${INSIDE}the lines themselves`, cost: money(c.subtotal), perSquare: '', hours: '' });
      for (const a of c.adders) {
        out.push({
          class: `${INSIDE}${a.name}${rate(a.rate)}`, cost: money(a.amount), perSquare: '', hours: '',
        });
      }
    }
    return out;
  }

  const lines = priceJob(document_, scenario as Parameters<typeof priceJob>[1], derived());
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

// ── The Estimate Sheet ─────────────────────────────────────────────────────
// A line per item, grouped under the condition it comes off.
//
// The formula is printed on the line and edited there. That is not a
// convenience — a library you cannot see into is a library nobody trusts, and
// every estimator has been burned by a number they could not check. If the
// formula is wrong the line says what is wrong and where, and shows no
// quantity rather than a plausible one.
//
// Tear this editor onto the other monitor and trace on the first. Both windows
// are the same job: the total moves while the mouse is still down.

import { measure, priceLine, scopeFor, type Measures } from '@roofnerd/engine';
import { at, doc, set, subscribe, type Doc } from '../doc.js';

type Item = {
  id: string; description: string; costCode: string; unit: string; formula: string;
  unitCost?: number; orderUnit?: { name: string; per: number; waste?: number };
  productionRate?: number; notes?: string;
};
type Condition = {
  id: string; name: string; kind: 'area' | 'line' | 'count';
  traces: { id: string; pageId: string; points: { x: number; y: number }[] }[];
  properties: Record<string, number>; items?: Item[]; from?: string; color?: string;
};
type Page = { id: string; feetPerUnit?: number };

const UNITS = ['SF', 'LF', 'EA', 'SQ'];

export function mountEstimate(host: HTMLElement): void {
  host.replaceChildren();
  host.classList.add('estimate-editor');

  const title = document.createElement('h2');
  title.textContent = 'Estimate Sheet';

  const note = document.createElement('p');
  note.className = 'editor-note';
  note.textContent =
    'Every line shows the formula that produced its quantity, and you can change it here. '
    + 'A line with no price says so instead of counting as nothing.';

  const body = document.createElement('div');
  body.className = 'sheet';

  const foot = document.createElement('div');
  foot.className = 'sheet-total';

  host.append(title, note, body, foot);

  subscribe((d: Doc) => render(body, foot, d));
}

function render(body: HTMLElement, foot: HTMLElement, d: Doc): void {
  const conditions = (at('/conditions', d) as Condition[]) ?? [];
  const pages = (at('/pages', d) as Page[]) ?? [];
  const scenario = activeScenario(d);

  const calibrations: Record<string, { feetPerUnit: number } | undefined> = {};
  for (const p of pages) if (p.feetPerUnit) calibrations[p.id] = { feetPerUnit: p.feetPerUnit };

  const measured = new Map<string, Measures>();
  for (const c of conditions) {
    const source = c.from ? conditions.find((x) => x.id === c.from) ?? c : c;
    measured.set(c.id, measure(source.kind, source.traces ?? [], source.properties ?? {}, calibrations));
  }

  body.replaceChildren();
  if (!conditions.length) {
    const empty = document.createElement('p');
    empty.className = 'muted';
    empty.textContent = 'Nothing traced yet. Trace something on the Plan and it will appear here.';
    body.append(empty);
    foot.textContent = '';
    return;
  }

  let total = 0;
  let anythingPending = false;

  for (const [index, condition] of conditions.entries()) {
    const measures = measured.get(condition.id)!;
    const scope = scopeFor(measures, condition.properties ?? {});

    const group = document.createElement('section');
    group.className = 'sheet-group';

    const header = document.createElement('header');
    const swatch = document.createElement('span');
    swatch.className = 'swatch';
    swatch.style.background = condition.color ?? '#c1440e';
    const name = document.createElement('strong');
    name.textContent = condition.name;
    const summary = document.createElement('span');
    summary.className = 'muted';
    summary.textContent = [
      measures.SF === null ? null : `${fmt(measures.SF)} SF`,
      measures.LF === null ? null : `${fmt(measures.LF)} LF`,
      `${measures.EA} EA`,
      condition.from ? `from ${conditions.find((c) => c.id === condition.from)?.name ?? '—'}` : null,
    ].filter(Boolean).join(' · ');
    header.append(swatch, name, summary);
    group.append(header);

    const table = document.createElement('table');
    // Fixed columns, so the formula keeps its room however long a description is.
    const cols = document.createElement('colgroup');
    for (const name of ['item', 'code', 'formula', 'qty', 'unit', 'waste', 'order', 'cost', 'extended', 'remove']) {
      const col = document.createElement('col');
      col.className = `c-${name}`;
      cols.append(col);
    }
    table.append(cols);
    const head = document.createElement('thead');
    head.append(headRow(['Item', 'Code', 'Formula', 'Qty', 'Unit', 'Waste', 'Order', 'Unit cost', 'Extended', '']));
    const rows = document.createElement('tbody');
    table.append(head, rows);

    const items = condition.items ?? [];
    for (const [itemIndex, item] of items.entries()) {
      const result = priceLine(item as never, scope, scenario?.prices ?? {});
      if (result.extended !== null) total += result.extended;
      if (result.pending) anythingPending = true;
      rows.append(itemRow(index, itemIndex, item, result));
    }

    const add = document.createElement('button');
    add.className = 'add-item';
    add.textContent = '+ item';
    add.addEventListener('click', () => void addItem(index, condition));

    group.append(table, add);
    body.append(group);
  }

  foot.replaceChildren();
  const label = document.createElement('span');
  label.textContent = 'Total';
  const value = document.createElement('strong');
  value.textContent = money(total);
  foot.append(label, value);
  if (anythingPending) {
    const warn = document.createElement('span');
    warn.className = 'pending';
    warn.textContent = 'partial — some lines have no quantity or no price';
    foot.append(warn);
  }
}

function itemRow(conditionIndex: number, itemIndex: number, item: Item, result: ReturnType<typeof priceLine>) {
  const tr = document.createElement('tr');
  const base = `/conditions/${conditionIndex}/items/${itemIndex}`;

  tr.append(cell(textField(item.description, (v) => set(`${base}/description`, v), 'what it is')));
  tr.append(cell(textField(item.costCode, (v) => set(`${base}/costCode`, v), 'code')));

  // The formula, on the line, editable. The point of the whole program.
  const formula = textField(item.formula, (v) => set(`${base}/formula`, v), 'LF * H');
  formula.classList.add('formula');
  const formulaCell = cell(formula);
  if (result.formulaError) {
    formula.classList.add('bad');
    const why = document.createElement('span');
    why.className = 'formula-error';
    why.textContent = result.formulaError;
    formulaCell.append(why);
  }
  tr.append(formulaCell);

  tr.append(numberCell(result.quantity));
  tr.append(cell(selectField(UNITS, item.unit, (v) => set(`${base}/unit`, v))));
  tr.append(cell(numberField(item.orderUnit?.waste, (v) => setOrderUnit(base, item, { waste: v }), '%')));

  const order = document.createElement('span');
  order.textContent = result.orderQuantity === null
    ? '—'
    : `${fmt(result.orderQuantity)} ${result.orderUnitName ?? ''}`.trim();
  tr.append(cell(order, 'num'));

  tr.append(cell(numberField(item.unitCost, (v) => set(`${base}/unitCost`, v), '$')));

  const extended = document.createElement('span');
  extended.textContent = result.extended === null ? (result.pending ?? '—') : money(result.extended);
  if (result.extended === null) extended.className = 'pending';
  tr.append(cell(extended, 'num'));

  const remove = document.createElement('button');
  remove.className = 'remove';
  remove.textContent = '×';
  remove.title = 'take this line off';
  remove.addEventListener('click', () => void removeItem(conditionIndex, itemIndex));
  tr.append(cell(remove));

  return tr;
}

// ── changing the job ───────────────────────────────────────────────────────

async function addItem(conditionIndex: number, condition: Condition): Promise<void> {
  const unit = condition.kind === 'area' ? 'SF' : condition.kind === 'line' ? 'LF' : 'EA';
  const item: Item = {
    id: `item-${Date.now().toString(36)}`,
    description: '',
    costCode: '',
    unit,
    // A new line starts by measuring what was traced. It is the honest default
    // and it shows the estimator what the formula box is for.
    formula: unit,
  };
  const items = [...(condition.items ?? []), item];
  await set(`/conditions/${conditionIndex}/items`, items);
}

async function removeItem(conditionIndex: number, itemIndex: number): Promise<void> {
  const conditions = (at('/conditions', doc()) as Condition[]) ?? [];
  const items = [...(conditions[conditionIndex]?.items ?? [])];
  items.splice(itemIndex, 1);
  await set(`/conditions/${conditionIndex}/items`, items);
}

async function setOrderUnit(base: string, item: Item, patch: { waste?: number | undefined }): Promise<void> {
  const order = { name: item.orderUnit?.name ?? '', per: item.orderUnit?.per ?? 1, ...item.orderUnit, ...patch };
  await set(`${base}/orderUnit`, order);
}

// ── small parts ────────────────────────────────────────────────────────────

function activeScenario(d: Doc) {
  const job = at('/job', d) as { scenarios?: { id: string; prices?: Record<string, number> }[]; activeScenarioId?: string } | undefined;
  return job?.scenarios?.find((s) => s.id === job.activeScenarioId) ?? job?.scenarios?.[0];
}

function cell(child: Node, className?: string): HTMLTableCellElement {
  const td = document.createElement('td');
  if (className) td.className = className;
  td.append(child);
  return td;
}

function numberCell(value: number | null): HTMLTableCellElement {
  const span = document.createElement('span');
  span.textContent = value === null ? 'pending' : fmt(value);
  if (value === null) span.className = 'pending';
  return cell(span, 'num');
}

function headRow(labels: string[]): HTMLTableRowElement {
  const tr = document.createElement('tr');
  for (const label of labels) {
    const th = document.createElement('th');
    th.textContent = label;
    tr.append(th);
  }
  return tr;
}

/**
 * A field that writes to the document as it is typed, and does not fight the
 * cursor when the document comes back around.
 */
function textField(value: string, write: (v: string) => Promise<unknown>, placeholder = ''): HTMLInputElement {
  const input = document.createElement('input');
  input.type = 'text';
  input.value = value ?? '';
  input.placeholder = placeholder;
  input.addEventListener('input', () => void write(input.value));
  return input;
}

function numberField(value: number | undefined, write: (v: number | undefined) => Promise<unknown>, placeholder = ''): HTMLInputElement {
  const input = document.createElement('input');
  input.type = 'number';
  input.step = 'any';
  input.className = 'num';
  input.value = value === undefined ? '' : String(value);
  input.placeholder = placeholder;
  input.addEventListener('input', () => {
    const parsed = input.value === '' ? undefined : Number(input.value);
    void write(parsed !== undefined && Number.isFinite(parsed) ? parsed : undefined);
  });
  return input;
}

function selectField(options: string[], value: string, write: (v: string) => Promise<unknown>): HTMLSelectElement {
  const select = document.createElement('select');
  for (const o of options) {
    const opt = document.createElement('option');
    opt.value = o;
    opt.textContent = o;
    select.append(opt);
  }
  select.value = value;
  select.addEventListener('change', () => void write(select.value));
  return select;
}

const fmt = (v: number) => v.toLocaleString('en-US', { maximumFractionDigits: 2 });
const money = (v: number) => v.toLocaleString('en-US', { style: 'currency', currency: 'USD' });

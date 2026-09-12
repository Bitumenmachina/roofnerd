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

import { CLASS_NAMES, measure, priceLine, recap, scopeFor, type Measures } from '@roofnerd/engine';
import { at, doc, set, subscribe, type Doc } from '../doc.js';
import { icon } from '../icons.js';
import { PENDING_REASON, money, plural, quantity } from '../labels.js';
import { select, selectedConditionId, watchSelection } from '../selection.js';

type UnitStep = { name: string; per?: number; contains?: number; rule: 'ceil' | 'exact' };
type Item = {
  id: string; description: string; costCode: string; unit: string; formula: string;
  waste?: number; order?: UnitStep; price?: UnitStep;
  unitCost?: number; productionRate?: number; crewSize?: number; notes?: string;
};
type Condition = {
  id: string; name: string; kind: 'area' | 'line' | 'count';
  traces: { id: string; pageId: string; points: { x: number; y: number }[] }[];
  properties: Record<string, number>; items?: Item[]; from?: string; color?: string;
};
type Page = { id: string; feetPerUnit?: number };

const UNITS = ['SF', 'LF', 'EA', 'SQ'];

/**
 * The price set this bid is worked out under.
 *
 * Hidden when a job has only one, because a picker with a single choice is
 * furniture. It appears the moment a job has a second, which is when it starts
 * meaning something.
 */
function drawScenarios(host: HTMLElement, d: Doc): void {
  const job = at('/job', d) as
    { scenarios?: { id: string; name: string }[]; activeScenarioId?: string } | undefined;
  const scenarios = job?.scenarios ?? [];
  host.replaceChildren();
  if (scenarios.length < 2) return;

  const label = document.createElement('span');
  label.textContent = 'Scenario';

  const pick = document.createElement('select');
  pick.setAttribute('aria-label', 'Which price set');
  for (const s of scenarios) {
    const o = document.createElement('option');
    o.value = s.id;
    o.textContent = s.name;
    pick.append(o);
  }
  pick.value = job?.activeScenarioId ?? scenarios[0]!.id;
  pick.addEventListener('change', () => void set('/job/activeScenarioId', pick.value));

  host.append(label, pick);
}

export function mountEstimate(host: HTMLElement): void {
  host.replaceChildren();

  // Which price set this bid is being worked out under. A job can hold several
  // — a supply house, a pricing date — and the whole sheet re-prices when it
  // changes, which is the point of having them.
  const scenarioBar = document.createElement('div');
  scenarioBar.className = 'scenario-bar';
  host.append(scenarioBar);

  // The sheet scrolls inside its own area rather than clipping, so a torn-off
  // window narrower than the table still reaches every column.
  const scroller = document.createElement('div');
  scroller.className = 'sheet-scroll';

  const body = document.createElement('div');
  body.className = 'sheet';
  scroller.append(body);

  const foot = document.createElement('div');
  foot.className = 'sheet-footer';

  host.append(scroller, foot);

  subscribe((d: Doc) => {
    drawScenarios(scenarioBar, d);
    render(body, foot, d);
  });
  watchSelection(() => render(body, foot, doc()));
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
    empty.className = 'empty-note';
    empty.textContent = 'Nothing traced yet. Trace something on the Plan and it appears here.';
    body.append(empty);
    foot.replaceChildren();
    return;
  }

  let total_ = 0;
  let anythingPending = false;

  // One table, one header. A header repeated under every condition is three
  // times the furniture and no more information.
  const table = document.createElement('table');
  const cols = document.createElement('colgroup');
  for (const n of ['item', 'code', 'formula', 'qty', 'unit', 'waste', 'order', 'priced', 'cost', 'extended', 'actions']) {
    const col = document.createElement('col');
    col.className = `c-${n}`;
    cols.append(col);
  }
  table.append(cols);
  const head = document.createElement('thead');
  head.append(headRow([
    'Item', 'Code', 'Formula', 'Qty', 'Unit', 'Waste', 'Order', 'Priced', 'Unit cost', 'Extended', '',
  ]));
  const rows = document.createElement('tbody');
  table.append(head, rows);
  body.append(table);

  for (const [index, condition] of conditions.entries()) {
    const measures = measured.get(condition.id)!;
    const scope = scopeFor(measures, condition.properties ?? {});

    // The condition is a tinted row of the same table, so the columns run under
    // it and the group reads as part of the sheet rather than beside it.
    const groupRow = document.createElement('tr');
    groupRow.className = 'group-row';
    if (condition.id === selectedConditionId()) groupRow.classList.add('on');
    const groupCell = document.createElement('td');
    groupCell.colSpan = 10;
    const swatch = document.createElement('span');
    swatch.className = 'swatch';
    swatch.style.background = condition.color ?? '#5f5f5f';
    const name = document.createElement('span');
    name.textContent = condition.name;
    const summary = document.createElement('span');
    summary.className = 'group-measures';
    summary.textContent = [
      condition.kind !== 'area' ? null : (measures.SF === null ? '—' : `${quantity(measures.SF)} SF`),
      condition.kind === 'count' ? null : (measures.LF === null ? '—' : `${quantity(measures.LF)} LF`),
      `${quantity(measures.EA, 0)} EA`,
      condition.from ? `from ${conditions.find((c) => c.id === condition.from)?.name ?? '—'}` : null,
    ].filter(Boolean).join(' · ');
    if (measures.SF === null && measures.LF === null && condition.kind !== 'count') {
      summary.title = PENDING_REASON;
    }
    groupCell.append(swatch, name, summary);
    const groupActions = document.createElement('td');
    groupActions.className = 'actions';
    groupRow.append(groupCell, groupActions);
    groupRow.addEventListener('click', () => select({ kind: 'condition', id: condition.id }));
    rows.append(groupRow);

    for (const [itemIndex, item] of (condition.items ?? []).entries()) {
      const result = priceLine(item as never, scope, scenario?.prices ?? {});
      if (result.extended !== null) total_ += result.extended;
      if (result.pending) anythingPending = true;
      rows.append(itemRow(index, itemIndex, item, result));
    }

    // The button that adds a line sits in the group it adds to.
    const addRow = document.createElement('tr');
    addRow.className = 'add-row';
    const addCell = document.createElement('td');
    addCell.colSpan = 11;
    const add = document.createElement('button');
    add.type = 'button';
    add.className = 'add-item';
    add.append(icon('add', 13));
    const addText = document.createElement('span');
    addText.textContent = 'Add a line';
    add.append(addText);
    add.addEventListener('click', () => void addItem(index, condition));
    addCell.append(add);
    addRow.append(addCell);
    rows.append(addRow);
  }

  // ── the footer: the classes, now that there are classes ─────────────────
  foot.replaceChildren();

  const rolled = recapOf(d);
  if (rolled) {
    const strip = document.createElement('div');
    strip.className = 'class-strip';
    for (const name of CLASS_NAMES) {
      const line = rolled.classes.find((c) => c.class === name);
      if (!line || line.total === 0) continue;
      const cell = document.createElement('div');
      cell.className = 'class-cell';
      const label = document.createElement('span');
      label.className = 'label';
      label.textContent = name;
      const value = document.createElement('span');
      value.className = 'value';
      value.textContent = money(line.total);
      cell.append(label, value);
      strip.append(cell);
    }
    if (strip.children.length) foot.append(strip);

    const selling = document.createElement('div');
    selling.className = 'selling';
    const label = document.createElement('span');
    label.className = 'label';
    label.textContent = 'Selling price';
    const value = document.createElement('span');
    value.className = 'value';
    value.textContent = money(rolled.sellingPrice);
    selling.append(label, value);
    if (rolled.pending.length) {
      const partial = document.createElement('span');
      partial.className = 'partial';
      partial.textContent = `${plural(rolled.pending.length, 'line')} not counted`;
      partial.title = rolled.pending.join('\n');
      selling.append(partial);
    }
    foot.append(selling);
  } else {
    const total = document.createElement('div');
    total.className = 'selling';
    const label = document.createElement('span');
    label.className = 'label';
    label.textContent = 'Total';
    const value = document.createElement('span');
    value.className = 'value';
    value.textContent = money(total_);
    total.append(label, value);
    foot.append(total);
  }
}

/**
 * The recap, when the job has the cost codes to roll one up.
 *
 * Without codes there are no classes, so the sheet shows a plain total rather
 * than pretending to a structure the job has not got yet.
 */
function recapOf(d: Doc) {
  const job = at('/job', d) as { scenarios?: unknown[]; activeScenarioId?: string } | undefined;
  const codes = (at('/costCodes', d) as unknown[]) ?? [];
  if (!job?.scenarios?.length || codes.length === 0) return null;
  try {
    const document_ = {
      job, pages: at('/pages', d) ?? [], conditions: at('/conditions', d) ?? [], costCodes: codes,
    } as never;
    const scenario = activeScenario(d) as never;
    return recap(document_, scenario);
  } catch {
    return null;
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
  } else if (result.unitNote) {
    // The unit does not follow from the formula. Not an error, and it does not
    // change a number — the estimator declared the unit and the unit prices the
    // line. It just stops being silent about the disagreement.
    const note = document.createElement('span');
    note.className = 'formula-note';
    note.textContent = 'check the unit';
    note.title = result.unitNote;
    formulaCell.append(note);
  }
  tr.append(formulaCell);

  tr.append(numberCell(result.quantity));
  tr.append(cell(selectField(UNITS, item.unit, (v) => set(`${base}/unit`, v))));
  // An empty waste field reads "0%", not a bare per-cent sign with nothing
  // in front of it. Zero waste is a real answer; a lone "%" is a shrug.
  tr.append(cell(numberField(item.waste ?? 0, (v) => set(`${base}/waste`, v), '0'), 'num'));

  // What you buy, and what it is priced against — three units, because a real
  // supply house uses three. Membrane is estimated in squares, bought by the
  // roll and quoted by the square foot, and each step is where money hides.
  tr.append(cell(stepCell(result.orderQuantity, result.orderUnitName), 'num'));
  tr.append(cell(stepCell(result.priceQuantity, result.priceUnitName), 'num'));

  tr.append(cell(numberField(item.unitCost, (v) => set(`${base}/unitCost`, v), '$')));

  const extended = document.createElement('span');
  if (result.extended === null) {
    // The reason lives in the tooltip; the cell shows a dash. A red word in a
    // money column reads as an error, and a line with no price yet is not one.
    extended.textContent = '—';
    extended.className = 'none';
    extended.title = result.pending ?? PENDING_REASON;
  } else {
    extended.textContent = money(result.extended);
  }
  tr.append(cell(extended, 'num extended'));

  const remove = document.createElement('button');
  remove.type = 'button';
  remove.className = 'remove';
  remove.title = 'Take this line off';
  remove.setAttribute('aria-label', 'Take this line off');
  remove.append(icon('close', 14));
  remove.addEventListener('click', () => void removeItem(conditionIndex, itemIndex));
  tr.append(cell(remove, 'actions'));

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

/** A quantity in a unit that is not the one it was measured in. */
function stepCell(value: number | null, unit: string | null): HTMLElement {
  const span = document.createElement('span');
  span.textContent = value === null ? '—' : `${quantity(value)} ${unit ?? ''}`.trim();
  if (value === null) span.className = 'none';
  return span;
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
  span.textContent = value === null ? '—' : quantity(value);
  if (value === null) {
    span.className = 'none';
    span.title = PENDING_REASON;
  }
  return cell(span, 'num');
}

function headRow(labels: string[]): HTMLTableRowElement {
  const tr = document.createElement('tr');
  for (const [i, label] of labels.entries()) {
    const th = document.createElement('th');
    th.textContent = label;
    // Numbers right, and the last column is the pinned one the row actions
    // live in so they never scroll out of a narrow window.
    if ([3, 5, 6, 7, 8, 9].includes(i)) th.className = 'num';
    // Extended is pinned with the actions, so its header has to be pinned too
    // or the heading slides off the column it names.
    if (i === 9) th.className = 'num extended';
    if (i === labels.length - 1) th.className = 'actions';
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


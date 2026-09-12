// ── The condition panel ────────────────────────────────────────────────────
// What the drawing cannot tell you. A parapet is a run until somebody says how
// tall it is; a roof is a footprint until somebody says what the slope is.
//
// The live measures come first and come large. They are the reason the panel is
// open — an estimator glances here to see what a formula is about to be handed,
// and a number they have to hunt for is a number they will not check.

import { measure, priceJob, type Condition, type Measures, type Page, type TraceKind } from '@roofnerd/engine';
import { at, set, type Doc } from '../doc.js';
import { icon } from '../icons.js';
import { KIND_LABELS, MEASURE_LABELS, PENDING_REASON, PROPERTY_LABELS, money, plural, quantity } from '../labels.js';

const GROUPS = ['Geometry', 'Metal'] as const;

/**
 * The measures an estimator glances at — but only the ones this kind of thing
 * actually has.
 *
 * A run has no surface at any scale (D3), so printing "— SF" beside one says
 * the number is missing when it is not missing, it does not exist. Worse, the
 * dash carried the tooltip "this sheet has not been scaled yet", which tells
 * the estimator a scale would produce a number. It never would. D42 said each
 * kind shows the measures it actually has; the rail and the sheet did that and
 * this panel did not.
 */
const headlineFor = (kind: TraceKind): readonly { key: keyof Measures; unit: string }[] => {
  if (kind === 'area') {
    return [{ key: 'SF', unit: 'SF' }, { key: 'LF', unit: 'LF' }, { key: 'EA', unit: 'EA' }, { key: 'SQ', unit: 'SQ' }];
  }
  if (kind === 'line') return [{ key: 'LF', unit: 'LF' }, { key: 'EA', unit: 'EA' }];
  return [{ key: 'EA', unit: 'EA' }];
};

export function renderConditionPanel(
  host: HTMLElement,
  conditionId: string | null,
  d: Doc,
  onChanged: () => void,
): void {
  host.replaceChildren();
  const conditions = (at('/conditions', d) as Condition[]) ?? [];
  const index = conditions.findIndex((c) => c.id === conditionId);
  const condition = conditions[index];
  // An empty panel is a panel that looks broken. It stands beside every editor
  // now, so the state it is in most often — a job just opened, nothing picked —
  // is the one state it used to render as a blank column.
  if (!condition) {
    const empty = document.createElement('p');
    empty.className = 'panel-empty';
    empty.textContent = conditionId === null
      ? 'Nothing picked yet. Pick a condition — in the tree, on the drawing, on the sheet or on the roof — and what it measures and what it costs read here.'
      : 'That condition is not in the job any more.';
    host.append(empty);
    return;
  }

  const base = `/conditions/${index}`;
  const properties = condition.properties ?? {};

  // ── name, and what it is ────────────────────────────────────────────────
  const name = document.createElement('input');
  name.type = 'text';
  name.className = 'panel-name';
  name.value = condition.name;
  name.setAttribute('aria-label', 'Condition name');
  name.addEventListener('input', () => void set(`${base}/name`, name.value));
  host.append(name);

  const traces = (condition.traces ?? []).length;
  const kind = document.createElement('p');
  kind.className = 'panel-kind';
  kind.textContent = `${KIND_LABELS[condition.kind]} · ${plural(traces, 'trace')}`;
  host.append(kind);

  // ── what it measures, first and large ───────────────────────────────────
  const measures = measuresFor(condition, conditions, d);
  const grid = document.createElement('div');
  grid.className = 'panel-measures';

  for (const { key, unit } of headlineFor(condition.kind)) {
    const cell = document.createElement('div');

    const label = document.createElement('div');
    label.className = 'measure-label';
    label.textContent = MEASURE_LABELS[key as string] ?? String(key);

    const value = document.createElement('div');
    value.className = 'measure-value';
    const raw = measures[key] as number | null;
    if (raw === null) {
      // A dash, and the reason in the tooltip. Never a coloured word: nothing
      // is broken, the sheet simply has not been scaled.
      value.classList.add('none');
      value.textContent = '—';
      value.title = PENDING_REASON;
    } else {
      value.textContent = quantity(raw, key === 'EA' ? 0 : 2);
      const suffix = document.createElement('span');
      suffix.className = 'unit';
      suffix.textContent = unit;
      value.append(suffix);
    }

    cell.append(label, value);
    grid.append(cell);
  }
  host.append(grid);

  // ── and what it costs ───────────────────────────────────────────────────
  // The same number the sheet shows and the Condition Summary lens groups by,
  // out of the same call: `priceJob`, summed over the lines that came off this
  // condition. Never a second piece of arithmetic — a panel figure that differs
  // from the sheet by a cent is worse than no figure at all, because both look
  // right and one of them is not.
  const cost = costOf(condition, d);
  const row = document.createElement('div');
  row.className = 'panel-money';
  const costLabel = document.createElement('span');
  costLabel.className = 'label';
  costLabel.textContent = 'Cost';
  const costValue = document.createElement('span');
  costValue.className = 'value';
  if (cost.total === null) {
    // Never $0.00. A condition nothing is priced on has no total, and that is a
    // different fact from a total of nothing — one is traced-and-not-yet-priced
    // and the other is free of charge.
    costValue.classList.add('none');
    costValue.textContent = 'nothing priced on it';
  } else {
    costValue.textContent = money(cost.total);
  }
  row.append(costLabel, costValue);
  host.append(row);
  if (cost.unpriced > 0) {
    const short = document.createElement('small');
    short.className = 'panel-money-note';
    short.textContent = cost.total === null
      ? `${plural(cost.unpriced, 'line')} on it with no price yet`
      : `${plural(cost.unpriced, 'line')} not counted`;
    host.append(short);
  }

  // ── where its measures come from ────────────────────────────────────────
  const others = conditions.filter((c) => c.id !== condition.id);
  const from = document.createElement('select');
  from.append(optionEl('', 'Measured from its own traces'));
  for (const other of others) from.append(optionEl(other.id, `From ${other.name}`));
  from.value = condition.from ?? '';
  from.addEventListener('change', () => {
    void set(`${base}/from`, from.value === '' ? undefined : from.value).then(onChanged);
  });
  host.append(field('Measures', from, 'one run, measured once — a coping and its parapet cannot drift apart'));

  // ── the properties, grouped ─────────────────────────────────────────────
  for (const group of GROUPS) {
    const named = Object.entries(PROPERTY_LABELS).filter(([, v]) => v.group === group);
    if (!named.length) continue;

    const heading = document.createElement('p');
    heading.className = 'panel-group';
    heading.textContent = group;
    host.append(heading);

    for (const [key, { label, hint }] of named) {
      const input = numberInput(properties[key], (v) => writeProperty(base, condition, key, v));
      host.append(field(label, input, hint));
    }
  }

  // Anything the estimator named themselves.
  const extra = Object.keys(properties).filter((k) => !(k in PROPERTY_LABELS));
  if (extra.length) {
    const heading = document.createElement('p');
    heading.className = 'panel-group';
    heading.textContent = 'Yours';
    host.append(heading);
    for (const key of extra) {
      const input = numberInput(properties[key], (v) => writeProperty(base, condition, key, v));
      host.append(field(key, input, 'usable in a formula on this condition by this name'));
    }
  }

  const add = document.createElement('button');
  add.type = 'button';
  add.className = 'add-property';
  add.append(icon('add', 14));
  const addText = document.createElement('span');
  addText.textContent = 'Add a property';
  add.append(addText);
  // Naming a property happens in the panel, beside the properties it joins.
  const naming = document.createElement('div');
  naming.className = 'name-property';
  naming.hidden = true;

  const nameField = document.createElement('input');
  nameField.type = 'text';
  nameField.placeholder = 'DECK_GAUGE';
  nameField.setAttribute('aria-label', 'Name the property');

  const nameNote = document.createElement('small');
  nameNote.textContent = 'Formulas on this condition will use that name.';

  const confirmName = document.createElement('button');
  confirmName.type = 'button';
  confirmName.textContent = 'Add it';

  const submitName = () => {
    const clean = nameField.value.trim().toUpperCase().replace(/\s+/g, '_');
    if (!clean || !/^[A-Z_][A-Z0-9_]*$/.test(clean)) {
      nameField.classList.add('bad');
      nameNote.textContent = 'Letters, numbers and underscores, starting with a letter.';
      return;
    }
    naming.hidden = true;
    void writeProperty(base, condition, clean, 0);
  };

  confirmName.addEventListener('click', submitName);
  nameField.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') { e.preventDefault(); submitName(); }
    if (e.key === 'Escape') { e.preventDefault(); naming.hidden = true; }
  });

  naming.append(nameField, confirmName, nameNote);

  add.addEventListener('click', () => {
    naming.hidden = false;
    nameField.value = '';
    nameField.classList.remove('bad');
    nameField.focus();
  });
  host.append(add, naming);
}

/**
 * What the lines on one condition come to, and how many of them have no money.
 *
 * `priceJob` is the whole of the arithmetic; this only picks out the lines whose
 * condition this is. A job with no scenario, or a document too young to price,
 * answers "no total" rather than zero.
 */
function costOf(condition: Condition, d: Doc): {
  total: number | null; priced: number; unpriced: number;
} {
  const job = at('/job', d) as
    { scenarios?: { id: string }[]; activeScenarioId?: string } | undefined;
  const scenario = job?.scenarios?.find((s) => s.id === job.activeScenarioId) ?? job?.scenarios?.[0];
  if (!scenario) return { total: null, priced: 0, unpriced: 0 };
  try {
    // The document as the shell holds it is already this shape — job, pages,
    // conditions, costCodes — which is how Reports hands it to the same call.
    const lines = priceJob(
      { ...(d as object) } as Parameters<typeof priceJob>[0],
      scenario as Parameters<typeof priceJob>[1],
    );
    let total = 0;
    let priced = 0;
    let unpriced = 0;
    for (const line of lines) {
      if (line.conditionId !== condition.id) continue;
      if (line.extended === null) { unpriced += 1; continue; }
      total += line.extended;
      priced += 1;
    }
    return { total: priced > 0 ? total : null, priced, unpriced };
  } catch {
    return { total: null, priced: 0, unpriced: 0 };
  }
}

function measuresFor(condition: Condition, all: Condition[], d: Doc): Measures {
  const pages = (at('/pages', d) as Page[]) ?? [];
  const calibrations: Record<string, { feetPerUnit: number } | undefined> = {};
  for (const p of pages) if (p.feetPerUnit) calibrations[p.id] = { feetPerUnit: p.feetPerUnit };
  const source = condition.from ? all.find((c) => c.id === condition.from) ?? condition : condition;
  return measure(source.kind, source.traces ?? [], source.properties ?? {}, calibrations);
}

async function writeProperty(base: string, condition: Condition, key: string, value: number | undefined) {
  // A mutable copy, which is what the engine's `Properties` is not: it is
  // readonly and its values can be absent, because a property nobody has typed
  // is not a zero. Cleared here by deleting the key, never by writing one.
  const properties: Record<string, number | undefined> = { ...(condition.properties ?? {}) };
  if (value === undefined) delete properties[key];
  else properties[key] = value;
  await set(`${base}/properties`, properties);
}

function field(label: string, control: HTMLElement, hint?: string): HTMLLabelElement {
  const wrap = document.createElement('label');
  const name = document.createElement('span');
  name.textContent = label;
  wrap.append(name, control);
  if (hint) {
    const small = document.createElement('small');
    small.textContent = hint;
    wrap.append(small);
  }
  return wrap;
}

function numberInput(value: number | undefined, write: (v: number | undefined) => Promise<unknown>): HTMLInputElement {
  const input = document.createElement('input');
  input.type = 'number';
  input.step = 'any';
  input.value = value === undefined ? '' : String(value);
  input.addEventListener('input', () => {
    const parsed = input.value === '' ? undefined : Number(input.value);
    void write(parsed !== undefined && Number.isFinite(parsed) ? parsed : undefined);
  });
  return input;
}

function optionEl(value: string, label: string): HTMLOptionElement {
  const o = document.createElement('option');
  o.value = value;
  o.textContent = label;
  return o;
}

// ── The condition panel ────────────────────────────────────────────────────
// What the drawing cannot tell you. A parapet is a run until somebody says how
// tall it is; a roof is a footprint until somebody says what the slope is.
//
// The live measures come first and come large. They are the reason the panel is
// open — an estimator glances here to see what a formula is about to be handed,
// and a number they have to hunt for is a number they will not check.

import { measure, type Measures } from '@roofnerd/engine';
import { at, set, type Doc } from '../doc.js';
import { icon } from '../icons.js';
import { KIND_LABELS, MEASURE_LABELS, PENDING_REASON, PROPERTY_LABELS, plural, quantity } from '../labels.js';

type ConditionShape = {
  id: string; name: string; kind: 'area' | 'line' | 'count';
  traces?: { id: string; pageId: string; points: { x: number; y: number }[] }[];
  properties?: Record<string, number>; from?: string;
};

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
const headlineFor = (kind: 'area' | 'line' | 'count'): readonly { key: keyof Measures; unit: string }[] => {
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
  const conditions = (at('/conditions', d) as ConditionShape[]) ?? [];
  const index = conditions.findIndex((c) => c.id === conditionId);
  const condition = conditions[index];
  if (!condition) return;

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

function measuresFor(condition: ConditionShape, all: ConditionShape[], d: Doc): Measures {
  const pages = (at('/pages', d) as { id: string; feetPerUnit?: number }[]) ?? [];
  const calibrations: Record<string, { feetPerUnit: number } | undefined> = {};
  for (const p of pages) if (p.feetPerUnit) calibrations[p.id] = { feetPerUnit: p.feetPerUnit };
  const source = condition.from ? all.find((c) => c.id === condition.from) ?? condition : condition;
  return measure(source.kind, source.traces ?? [], source.properties ?? {}, calibrations);
}

async function writeProperty(base: string, condition: ConditionShape, key: string, value: number | undefined) {
  const properties = { ...(condition.properties ?? {}) };
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

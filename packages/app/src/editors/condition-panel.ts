// ── The condition panel ────────────────────────────────────────────────────
// What the drawing cannot tell you. A parapet is a line until somebody says how
// tall it is; a roof is a footprint until somebody says what the slope is.
//
// The six named properties are the ones every roofing detail keeps asking for.
// Anything else the estimator needs, they name themselves, and it becomes
// available to every formula on that condition's lines by that name.

import { measure, type Measures } from '@roofnerd/engine';
import { at, set, doc, type Doc } from '../doc.js';

const NAMED: readonly { readonly key: string; readonly label: string; readonly hint: string }[] = [
  { key: 'H', label: 'Height', hint: 'feet — how far the flashing runs up' },
  { key: 'W', label: 'Width', hint: 'feet — coping width, cricket width' },
  { key: 'T', label: 'Thickness', hint: 'inches — insulation, as it is sold' },
  { key: 'PITCH', label: 'Pitch', hint: 'rise per 12 — 5 means 5:12' },
  { key: 'SIDES', label: 'Sides', hint: 'how many' },
  { key: 'STRETCHOUT', label: 'Stretch-out', hint: 'inches — flat metal the profile eats' },
];

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

  const heading = document.createElement('h3');
  heading.textContent = 'Condition';
  host.append(heading);

  // Name.
  host.append(field('Name', textInput(condition.name, (v) => set(`${base}/name`, v))));

  // What it is. Changing this changes what its traces mean, so it is shown.
  const kind = document.createElement('p');
  kind.className = 'muted';
  kind.textContent = `${condition.kind} · ${(condition.traces ?? []).length} trace(s)`;
  host.append(kind);

  // Where its measures come from.
  const others = conditions.filter((c) => c.id !== condition.id);
  const from = document.createElement('select');
  from.append(optionEl('', 'measured from its own traces'));
  for (const other of others) from.append(optionEl(other.id, `from ${other.name}`));
  from.value = condition.from ?? '';
  from.addEventListener('change', () => {
    void set(`${base}/from`, from.value === '' ? undefined : from.value).then(onChanged);
  });
  host.append(field('Measures', from, 'one run, measured once — a coping and its parapet cannot drift apart'));

  // The six named properties.
  for (const { key, label, hint } of NAMED) {
    const input = numberInput(condition.properties?.[key], (v) => writeProperty(base, condition, key, v));
    host.append(field(label, input, hint));
  }

  // Anything the estimator named themselves.
  const extra = Object.keys(condition.properties ?? {}).filter((k) => !NAMED.some((n) => n.key === k));
  for (const key of extra) {
    const input = numberInput(condition.properties?.[key], (v) => writeProperty(base, condition, key, v));
    host.append(field(key, input, 'yours — usable in a formula by this name'));
  }

  const add = document.createElement('button');
  add.textContent = '+ property';
  add.addEventListener('click', () => {
    const name = window.prompt('Name it. Formulas on this condition will use that name.', '');
    const clean = name?.trim().toUpperCase();
    if (!clean || !/^[A-Z_][A-Z0-9_]*$/.test(clean)) return;
    void writeProperty(base, condition, clean, 0);
  });
  host.append(add);

  // What it measures, so the numbers a formula will see are visible here too.
  const measures = measuresFor(condition, conditions, d);
  const shows = document.createElement('table');
  shows.className = 'measures';
  for (const [name, value] of [
    ['SF', measures.SF], ['PLAN_SF', measures.PLAN_SF], ['SQ', measures.SQ],
    ['LF', measures.LF], ['EA', measures.EA],
  ] as const) {
    const tr = document.createElement('tr');
    const th = document.createElement('th');
    th.textContent = name;
    const td = document.createElement('td');
    td.className = 'num';
    td.textContent = value === null ? 'pending' : value.toLocaleString('en-US', { maximumFractionDigits: 2 });
    if (value === null) td.classList.add('pending');
    tr.append(th, td);
    shows.append(tr);
  }
  host.append(shows);
}

type ConditionShape = {
  id: string; name: string; kind: 'area' | 'line' | 'count';
  traces: { id: string; pageId: string; points: { x: number; y: number }[] }[];
  properties: Record<string, number>; from?: string;
};

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

function textInput(value: string, write: (v: string) => Promise<unknown>): HTMLInputElement {
  const input = document.createElement('input');
  input.type = 'text';
  input.value = value ?? '';
  input.addEventListener('input', () => void write(input.value));
  return input;
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

/** Which condition the panel is showing, kept where both editors can read it. */
export const selection = {
  id: null as string | null,
  set(id: string | null) { this.id = id; },
};

export const currentDoc = doc;

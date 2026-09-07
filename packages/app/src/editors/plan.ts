// ── The Plan editor ────────────────────────────────────────────────────────
// The drawing, the tools, and the list of what has been traced off it.
//
// Everything traced goes straight into the one document, so the Estimate Sheet
// — in this window or torn off onto the other monitor — is looking at the same
// numbers before the mouse button comes back up.

import {
  ARCHITECTURAL_SCALES, calibrateFromScale, calibrateFromTwoPoints, distance,
  formatFeetInches, measure, parseFeet, type Point,
} from '@roofnerd/engine';
import { at, addPage, doc, pageBytes, pickFile, set, subscribe, type Doc } from '../doc.js';
import { Surface, svg, pointsAttribute } from '../viewer/surface.js';
import { Tools, type ToolName } from '../viewer/tools.js';
import { loadSheet, type Sheet } from '../viewer/page-source.js';
import { renderConditionPanel } from './condition-panel.js';
import { toolButton, iconButton } from '../chrome.js';
import { hueFor } from '../icons.js';
import { KIND_LABELS, PENDING_REASON, propertyPhrase, quantity } from '../labels.js';
import { select, selectedConditionId, watchSelection } from '../selection.js';

type Trace = { id: string; pageId: string; points: Point[] };
type Condition = {
  id: string; name: string; kind: 'area' | 'line' | 'count';
  traces: Trace[]; properties: Record<string, number>; items: unknown[];
  color?: string; from?: string; hidden?: boolean;
};
type Page = { id: string; name: string; source?: string; pageNumber?: number; feetPerUnit?: number; scaleNote?: string };

let surface: Surface;
let tools: Tools;
let currentPageId: string | null = null;
let loadedSource: string | null = null;

export function mountPlan(host: HTMLElement): void {
  host.replaceChildren();
  host.classList.add('plan-editor');

  const bar = document.createElement('div');
  bar.className = 'toolbar';

  const pageSelect = document.createElement('select');
  pageSelect.setAttribute('aria-label', 'Drawing');
  const addPageButton = toolButton('addDrawing', 'Add a drawing',
    () => addDrawing().catch((e) => report('Adding a drawing', e)));

  const toolButtons = new Map<ToolName, HTMLButtonElement>();
  for (const [tool, label, iconName] of [
    ['select', 'Select', 'select'], ['area', 'Area', 'area'],
    ['line', 'Line', 'line'], ['count', 'Count', 'count'],
  ] as const) {
    toolButtons.set(tool, toolButton(iconName, label, () => chooseTool(tool)));
  }

  // One action, one word. Its label reads Scale or Rescale by state — the sheet
  // either has a scale or it does not, and that is a state, not two commands.
  const scaleButton = toolButton('scale', 'Scale', () => chooseTool('scale'));
  const scaleSelect = document.createElement('select');
  scaleSelect.setAttribute('aria-label', 'Architectural scale');
  scaleSelect.append(option('', 'Or pick a scale…'));
  for (const s of ARCHITECTURAL_SCALES) scaleSelect.append(option(String(s.feetPerInch), s.label));
  scaleSelect.addEventListener('change', () => {
    const feetPerInch = Number(scaleSelect.value);
    if (!feetPerInch || !currentPageId) return;
    const cal = calibrateFromScale(feetPerInch, surface.unitsPerInch);
    void applyScale(cal.feetPerUnit, scaleSelect.selectedOptions[0]?.text ?? 'preset');
    scaleSelect.value = '';
  });

  const spacer = document.createElement('span');
  spacer.className = 'spacer';
  const zoomOut = iconButton('zoomOut', 'Zoom out', () => void surface.setZoom(surface.zoom / 1.25));
  const zoomIn = iconButton('zoomIn', 'Zoom in', () => void surface.setZoom(surface.zoom * 1.25));
  const zoomFit = iconButton('fit', 'Fit the sheet in the window', () => void surface.fit());

  bar.append(pageSelect, addPageButton, divider(), ...toolButtons.values(), divider(),
    scaleButton, scaleSelect, spacer, zoomOut, zoomIn, zoomFit);

  surface = new Surface();

  const hint = document.createElement('p');
  hint.className = 'hint';

  const railHeading = document.createElement('p');
  railHeading.className = 'rail-heading';
  railHeading.textContent = 'Conditions';

  const list = document.createElement('div');
  list.className = 'condition-list';

  const panel = document.createElement('div');
  panel.className = 'condition-panel';

  // One scroll region: the list on top, the selected condition below it. Two
  // scrollers side by side is how the heading ended up clipped under the panel.
  const rail = document.createElement('div');
  rail.className = 'plan-rail';
  rail.append(railHeading, list, panel);

  // A4: the scale badge sits on the drawing, bottom-left, where an estimator
  // looks to check what they are measuring against.
  const scaleBadge = document.createElement('div');
  scaleBadge.className = 'scale-badge';
  surface.root.append(scaleBadge);

  const layout = document.createElement('div');
  layout.className = 'plan-layout';
  layout.append(surface.root, rail);

  host.append(bar, layout, hint);

  tools = new Tools(surface, {
    existingPoints: () => currentPageId ? tracedPoints(conditions(), currentPageId) : [],
    onTrace: (kind, points) => void recordTrace(kind, points),
    onScale: (a, b) => askScale(a, b),
    onHint: (text) => { hint.textContent = text; },
  });

  function chooseTool(tool: ToolName) {
    tools.use(tool);
    for (const [name, b] of toolButtons) b.classList.toggle('on', name === tool);
    scaleButton.classList.toggle('on', tool === 'scale');
  }
  chooseTool('select');

  // Selecting a condition anywhere — here or in the tree — redraws both.
  watchSelection(() => {
    renderConditions(list, doc());
    renderConditionPanel(panel, selectedConditionId(), doc(), () => renderConditions(list, doc()));
    drawTraces(doc());
  });

  pageSelect.addEventListener('change', () => showPage(pageSelect.value).catch((e) => report('opening the drawing', e)));

  subscribe((d: Doc) => {
    const pages = (at('/pages', d) as Page[]) ?? [];
    fillPages(pageSelect, pages);
    if (!currentPageId && pages[0]) showPage(pages[0].id).catch((e) => report('opening the drawing', e));
    else if (currentPageId) refreshSheetIfChanged(pages).catch((e) => report('opening the drawing', e));
    renderConditions(list, d);
    renderConditionPanel(panel, selectedConditionId(), d, () => renderConditions(list, doc()));
    drawTraces(d);
    const page = pages.find((p) => p.id === currentPageId);
    const word = scaleButton.querySelector('span');
    if (word) word.textContent = page?.feetPerUnit ? 'Rescale' : 'Scale';
    scaleBadge.textContent = page?.feetPerUnit
      ? (page.scaleNote ?? 'Scaled')
      : 'Not scaled';
    scaleBadge.classList.toggle('none', !page?.feetPerUnit);
    scaleBadge.hidden = !page;
  });
}

// ── the sheet ──────────────────────────────────────────────────────────────

async function addDrawing(): Promise<void> {
  const picked = await pickFile();
  if (!picked) return;
  const relative = await addPage(picked);
  const name = relative.split('/').pop() ?? relative;
  const pages = (at('/pages') as Page[]) ?? [];
  const page: Page = { id: `page-${Date.now().toString(36)}`, name, source: relative, pageNumber: 1 };
  await set('/pages', [...pages, page]);
  await showPage(page.id);
}

async function showPage(pageId: string): Promise<void> {
  currentPageId = pageId;
  const page = ((at('/pages') as Page[]) ?? []).find((p) => p.id === pageId);
  if (!page?.source) { surface.clear(); loadedSource = null; return; }
  await loadSource(page);
}

/**
 * A drawing that will not open has to say so. A blank sheet with no explanation
 * is the worst thing this editor could do — the estimator would trace nothing
 * and never know the file was the problem.
 */
function report(where: string, e: unknown): void {
  const text = typeof e === 'string' ? e : e instanceof Error ? e.message : String(e);
  const hint = document.querySelector<HTMLElement>('.hint');
  if (hint) hint.textContent = `${where}: ${text}`;
  console.error(`${where}: ${text}`);
}

async function refreshSheetIfChanged(pages: Page[]): Promise<void> {
  const page = pages.find((p) => p.id === currentPageId);
  if (page?.source && page.source !== loadedSource) await loadSource(page);
}

async function loadSource(page: Page): Promise<void> {
  const bytes = await pageBytes(page.source!);
  const sheet: Sheet = await loadSheet(bytes, page.source!, page.pageNumber ?? 1);
  loadedSource = page.source!;
  await surface.show(sheet);
  await surface.fit();
}

// ── scale ──────────────────────────────────────────────────────────────────

/**
 * Ask how long that really is — in the window, not in a browser dialog.
 *
 * The field appears where the instruction already is, under the drawing, with
 * the cursor in it. A dialog would take the estimator off the sheet they are
 * pointing at, and in this webview there is no dialog to take them to anyway.
 */
function askScale(a: Point, b: Point): void {
  const bar = document.querySelector<HTMLElement>('.hint');
  if (!bar) return;

  bar.replaceChildren();
  bar.classList.add('asking');

  const label = document.createElement('label');
  label.className = 'scale-ask';
  const text = document.createElement('span');
  text.textContent = 'How long is that, really?';

  const input = document.createElement('input');
  input.type = 'text';
  input.placeholder = `12'-6"`;
  input.setAttribute('aria-label', 'The real distance between those two points');

  const note = document.createElement('small');
  note.textContent = `12'-6", 6", 4'-6 1/2" and 12.5 all read the same way`;

  const accept = document.createElement('button');
  accept.type = 'button';
  accept.textContent = 'Set the scale';

  const cancel = document.createElement('button');
  cancel.type = 'button';
  cancel.className = 'quiet';
  cancel.textContent = 'Cancel';

  label.append(text, input, accept, cancel, note);
  bar.append(label);
  input.focus();

  const finish = () => {
    bar.classList.remove('asking');
    bar.textContent = 'click a trace to select it';
  };

  const submit = () => {
    const feet = parseFeet(input.value);
    if (feet === null) {
      input.classList.add('bad');
      note.textContent = 'That is not a length. Try 12\'-6", 6", or 12.5.';
      return;
    }
    const cal = calibrateFromTwoPoints(a, b, feet);
    if (!cal) { finish(); return; }
    void applyScale(cal.feetPerUnit, `two points, ${formatFeetInches(feet)}`).then(finish);
  };

  accept.addEventListener('click', submit);
  cancel.addEventListener('click', finish);
  input.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') { e.preventDefault(); submit(); }
    if (e.key === 'Escape') { e.preventDefault(); finish(); }
  });
}

async function applyScale(feetPerUnit: number, note: string): Promise<void> {
  const pages = ((at('/pages') as Page[]) ?? []).map((p) =>
    p.id === currentPageId ? { ...p, feetPerUnit, scaleNote: note } : p);
  await set('/pages', pages);
}

// ── tracing ────────────────────────────────────────────────────────────────

const conditions = (): Condition[] => (at('/conditions') as Condition[]) ?? [];

function tracedPoints(list: Condition[], pageId: string): Point[] {
  const out: Point[] = [];
  for (const c of list) for (const t of c.traces ?? []) if (t.pageId === pageId) out.push(...t.points);
  return out;
}

/**
 * Traces are recorded one at a time, in order.
 *
 * Dropping three counts quickly used to start three conditions: each click read
 * the job before the one before it had finished writing, saw nothing selected
 * that took a count, and made its own. Whoever did it got one marker in each of
 * three conditions and no reason why.
 */
let recording: Promise<void> = Promise.resolve();

function recordTrace(kind: 'area' | 'line' | 'count', points: readonly Point[]): Promise<void> {
  recording = recording.catch(() => undefined).then(() => writeTrace(kind, points));
  return recording;
}

async function writeTrace(kind: 'area' | 'line' | 'count', points: readonly Point[]): Promise<void> {
  if (!currentPageId) return;
  const all = conditions();
  const trace: Trace = { id: `trace-${Date.now().toString(36)}`, pageId: currentPageId, points: [...points] };

  // Into the selected condition if it takes this kind of shape; otherwise a new
  // one, named for what it is, so a trace is never dropped on the floor.
  const selected = all.find((c) => c.id === selectedConditionId() && c.kind === kind);
  if (selected) {
    await set('/conditions', all.map((c) =>
      c.id === selected.id ? { ...c, traces: [...(c.traces ?? []), trace] } : c));
    return;
  }

  const condition: Condition = {
    id: `condition-${Date.now().toString(36)}`,
    name: `${KIND_LABELS[kind]} ${all.filter((c) => c.kind === kind).length + 1}`,
    kind,
    traces: [trace],
    properties: {},
    items: [],
    color: hueFor(all.length),
  };
  await set('/conditions', [...all, condition]);
  select({ kind: 'condition', id: condition.id });
}

function drawTraces(d: Doc): void {
  if (!surface?.hasSheet || !currentPageId) return;
  const g = surface.shapes;
  g.replaceChildren();

  for (const c of ((at('/conditions', d) as Condition[]) ?? [])) {
    if (c.hidden) continue;
    const color = c.color ?? hueFor(0);
    const selected = c.id === selectedConditionId();
    for (const t of c.traces ?? []) {
      if (t.pageId !== currentPageId) continue;
      const common = { stroke: color, class: selected ? 'trace on' : 'trace' };

      if (c.kind === 'count') {
        for (const p of t.points) {
          g.append(svg('circle', { cx: p.x, cy: p.y, r: 4 * surface.pageUnitsPerPixel, fill: color, ...common }));
        }
      } else if (c.kind === 'area') {
        g.append(svg('polygon', { points: pointsAttribute(t.points), fill: color, ...common }));
      } else {
        g.append(svg('polyline', { points: pointsAttribute(t.points), fill: 'none', ...common }));
      }
    }
  }
}

// ── the condition list ─────────────────────────────────────────────────────

function renderConditions(host: HTMLElement, d: Doc): void {
  host.replaceChildren();

  const pages = (at('/pages', d) as Page[]) ?? [];
  const calibrations: Record<string, { feetPerUnit: number } | undefined> = {};
  for (const p of pages) if (p.feetPerUnit) calibrations[p.id] = { feetPerUnit: p.feetPerUnit };

  const list = (at('/conditions', d) as Condition[]) ?? [];
  if (!list.length) {
    const empty = document.createElement('p');
    empty.className = 'muted';
    empty.textContent = 'Nothing traced yet. Add a drawing, set its scale, then pick a tool.';
    host.append(empty);
    return;
  }

  for (const c of list) {
    const m = measure(c.kind, c.traces ?? [], c.properties ?? {}, calibrations);
    const row = document.createElement('button');
    row.type = 'button';
    row.className = c.id === selectedConditionId() ? 'condition on' : 'condition';
    row.addEventListener('click', () => select({ kind: 'condition', id: c.id }));

    const swatch = document.createElement('span');
    swatch.className = 'swatch';
    swatch.style.background = c.color ?? '#c1440e';

    const name = document.createElement('span');
    name.className = 'condition-name';
    name.textContent = c.name;

    const measures = document.createElement('span');
    measures.className = 'condition-measures';
    // A measure with no number yet is a dash with the reason in its tooltip.
    // Never a coloured word — nothing is broken, the sheet is not scaled.
    // An area has a surface; a run does not, and printing "— SF" beside one
    // suggests it might. Each kind shows the measures it actually has.
    const parts: string[] = [];
    if (c.kind === 'area') parts.push(m.SF === null ? '—' : `${quantity(m.SF)} SF`);
    if (c.kind !== 'count') parts.push(m.LF === null ? '—' : `${quantity(m.LF)} LF`);
    parts.push(`${quantity(m.EA, 0)} EA`);
    measures.textContent = parts.join(' · ');
    if (m.LF === null && c.kind !== 'count') measures.title = PENDING_REASON;

    // A4: a condition can be hidden on the sheet without being deleted — a busy
    // roof plan is unreadable with every trace on it at once.
    const visible = c.hidden !== true;
    const eye = document.createElement('span');
    eye.className = 'visibility';
    eye.title = visible ? 'Hide this on the drawing' : 'Show this on the drawing';
    eye.setAttribute('role', 'button');
    eye.textContent = visible ? '●' : '○';
    eye.addEventListener('click', (e) => {
      e.stopPropagation();
      const all = conditions();
      void set('/conditions', all.map((x) => (x.id === c.id ? { ...x, hidden: visible } : x)));
    });
    row.classList.toggle('hidden-condition', !visible);

    row.append(swatch, name, eye, measures);

    // Properties in the words an estimator uses: "4 sides", not "SIDES 4".
    const shown = Object.entries(c.properties ?? {})
      .filter(([, v]) => v !== undefined)
      .map(([k, v]) => propertyPhrase(k, v as number));
    if (shown.length) {
      const props = document.createElement('span');
      props.className = 'condition-props';
      props.textContent = shown.join(' · ');
      row.append(props);
    }

    host.append(row);
  }
}

// ── odds and ends ──────────────────────────────────────────────────────────

const fmt = (v: number) => v.toLocaleString('en-US', { maximumFractionDigits: 2 });

function button(label: string, onClick: () => void): HTMLButtonElement {
  const b = document.createElement('button');
  b.textContent = label;
  b.addEventListener('click', onClick);
  return b;
}

function option(value: string, label: string): HTMLOptionElement {
  const o = document.createElement('option');
  o.value = value;
  o.textContent = label;
  return o;
}

function divider(): HTMLSpanElement {
  const s = document.createElement('span');
  s.className = 'divider';
  return s;
}

function fillPages(select: HTMLSelectElement, pages: Page[]): void {
  const keep = select.value;
  select.replaceChildren();
  if (!pages.length) select.append(option('', 'no drawings yet'));
  for (const p of pages) select.append(option(p.id, p.feetPerUnit ? p.name : `${p.name} (unscaled)`));
  select.value = currentPageId ?? keep;
}

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

type Trace = { id: string; pageId: string; points: Point[] };
type Condition = {
  id: string; name: string; kind: 'area' | 'line' | 'count';
  traces: Trace[]; properties: Record<string, number>; items: unknown[];
  color?: string; from?: string;
};
type Page = { id: string; name: string; source?: string; pageNumber?: number; feetPerUnit?: number; scaleNote?: string };

const COLORS = ['#c1440e', '#1d6a96', '#3f7d20', '#8a3ffc', '#b58900', '#d33682'];

let surface: Surface;
let tools: Tools;
let currentPageId: string | null = null;
let selectedConditionId: string | null = null;
let loadedSource: string | null = null;

export function mountPlan(host: HTMLElement): void {
  host.replaceChildren();
  host.classList.add('plan-editor');

  const bar = document.createElement('div');
  bar.className = 'toolbar';

  const pageSelect = document.createElement('select');
  pageSelect.title = 'which drawing';
  const addPageButton = button('Add a drawing', () => addDrawing().catch((e) => report('adding a drawing', e)));

  const toolButtons = new Map<ToolName, HTMLButtonElement>();
  for (const [tool, label] of [['select', 'Select'], ['area', 'Area'], ['line', 'Line'], ['count', 'Count']] as const) {
    const b = button(label, () => chooseTool(tool));
    toolButtons.set(tool, b);
  }

  const scaleButton = button('Set scale', () => chooseTool('scale'));
  const scaleSelect = document.createElement('select');
  scaleSelect.title = 'or pick an architectural scale';
  scaleSelect.append(option('', 'scale…'));
  for (const s of ARCHITECTURAL_SCALES) scaleSelect.append(option(String(s.feetPerInch), s.label));
  scaleSelect.addEventListener('change', () => {
    const feetPerInch = Number(scaleSelect.value);
    if (!feetPerInch || !currentPageId) return;
    const cal = calibrateFromScale(feetPerInch, surface.unitsPerInch);
    void applyScale(cal.feetPerUnit, scaleSelect.selectedOptions[0]?.text ?? 'preset');
    scaleSelect.value = '';
  });

  const zoomOut = button('−', () => void surface.setZoom(surface.zoom / 1.25));
  const zoomIn = button('+', () => void surface.setZoom(surface.zoom * 1.25));
  const zoomFit = button('Fit', () => void surface.fit());

  bar.append(pageSelect, addPageButton, divider(), ...toolButtons.values(), divider(),
    scaleButton, scaleSelect, divider(), zoomOut, zoomIn, zoomFit);

  surface = new Surface();

  const hint = document.createElement('p');
  hint.className = 'hint';

  const list = document.createElement('div');
  list.className = 'condition-list';

  const layout = document.createElement('div');
  layout.className = 'plan-layout';
  layout.append(surface.root, list);

  host.append(bar, layout, hint);

  tools = new Tools(surface, {
    existingPoints: () => currentPageId ? tracedPoints(conditions(), currentPageId) : [],
    onTrace: (kind, points) => void recordTrace(kind, points),
    onScale: (a, b) => void askScale(a, b),
    onHint: (text) => { hint.textContent = text; },
  });

  function chooseTool(tool: ToolName) {
    tools.use(tool);
    for (const [name, b] of toolButtons) b.classList.toggle('on', name === tool);
    scaleButton.classList.toggle('on', tool === 'scale');
  }
  chooseTool('select');

  pageSelect.addEventListener('change', () => showPage(pageSelect.value).catch((e) => report('opening the drawing', e)));

  subscribe((d: Doc) => {
    const pages = (at('/pages', d) as Page[]) ?? [];
    fillPages(pageSelect, pages);
    if (!currentPageId && pages[0]) showPage(pages[0].id).catch((e) => report('opening the drawing', e));
    else if (currentPageId) refreshSheetIfChanged(pages).catch((e) => report('opening the drawing', e));
    renderConditions(list, d);
    drawTraces(d);
    const page = pages.find((p) => p.id === currentPageId);
    scaleButton.textContent = page?.feetPerUnit ? 'Rescale' : 'Set scale';
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

async function askScale(a: Point, b: Point): Promise<void> {
  const typed = window.prompt(
    `How long is that, really?  (12'-6", 6", 12.5 — all read the same way)`,
    '',
  );
  const feet = parseFeet(typed);
  if (feet === null) return;
  const cal = calibrateFromTwoPoints(a, b, feet);
  if (!cal) return;
  await applyScale(cal.feetPerUnit, `two points, ${formatFeetInches(feet)}`);
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

async function recordTrace(kind: 'area' | 'line' | 'count', points: readonly Point[]): Promise<void> {
  if (!currentPageId) return;
  const all = conditions();
  const trace: Trace = { id: `trace-${Date.now().toString(36)}`, pageId: currentPageId, points: [...points] };

  // Into the selected condition if it takes this kind of shape; otherwise a new
  // one, named for what it is, so a trace is never dropped on the floor.
  const selected = all.find((c) => c.id === selectedConditionId && c.kind === kind);
  if (selected) {
    await set('/conditions', all.map((c) =>
      c.id === selected.id ? { ...c, traces: [...(c.traces ?? []), trace] } : c));
    return;
  }

  const condition: Condition = {
    id: `condition-${Date.now().toString(36)}`,
    name: `${kind[0]!.toUpperCase()}${kind.slice(1)} ${all.filter((c) => c.kind === kind).length + 1}`,
    kind,
    traces: [trace],
    properties: {},
    items: [],
    color: COLORS[all.length % COLORS.length]!,
  };
  selectedConditionId = condition.id;
  await set('/conditions', [...all, condition]);
}

function drawTraces(d: Doc): void {
  if (!surface?.hasSheet || !currentPageId) return;
  const g = surface.shapes;
  g.replaceChildren();

  for (const c of ((at('/conditions', d) as Condition[]) ?? [])) {
    const color = c.color ?? '#c1440e';
    const selected = c.id === selectedConditionId;
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

  const heading = document.createElement('h3');
  heading.textContent = 'Conditions';
  host.append(heading);

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
    row.className = c.id === selectedConditionId ? 'condition on' : 'condition';
    row.addEventListener('click', () => { selectedConditionId = c.id; renderConditions(host, doc()); drawTraces(doc()); });

    const swatch = document.createElement('span');
    swatch.className = 'swatch';
    swatch.style.background = c.color ?? '#c1440e';

    const name = document.createElement('span');
    name.className = 'condition-name';
    name.textContent = c.name;

    const measures = document.createElement('span');
    measures.className = 'condition-measures';
    measures.textContent = [
      m.SF === null ? null : `${fmt(m.SF)} SF`,
      m.LF === null ? null : `${fmt(m.LF)} LF`,
      `${m.EA} EA`,
    ].filter(Boolean).join(' · ');

    row.append(swatch, name, measures);

    const props = document.createElement('span');
    props.className = 'condition-props';
    const shown = Object.entries(c.properties ?? {}).filter(([, v]) => v !== undefined);
    if (shown.length) props.textContent = shown.map(([k, v]) => `${k} ${v}`).join(' · ');
    if (shown.length) row.append(props);

    if (m.SF === null && m.LF === null && c.kind !== 'count') {
      const pending = document.createElement('span');
      pending.className = 'pending';
      pending.textContent = 'sheet not scaled';
      row.append(pending);
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

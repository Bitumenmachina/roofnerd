// ── The Plan view ──────────────────────────────────────────────────────────
// The drawing, and the list of what has been traced off it.
//
// The tools are no longer a row across the top of this editor. They are the
// window's tool rail — one act per button, in the words a roofer uses for it —
// and everything an act needs is in the window's tool settings strip. That is
// the handoff's anatomy, and it is also the arrangement VES has: a rack on the
// left, a spare stage, the depth somewhere that is not on top of the drawing.
//
// Everything traced goes straight into the one document, so the money — in the
// cost panel beside it, or in the panel pulled onto the other monitor — is
// looking at the same numbers before the mouse button comes back up.

import {
  ARCHITECTURAL_SCALES, calibrateFromScale, calibrateFromTwoPoints,
  formatFeetInches, parseFeet,
  type Condition, type ConditionRole, type Page, type Point, type Trace,
} from '@roofnerd/engine';
import { at, addPage, doc, pageBytes, pickFile, set, subscribe, type Doc } from '../doc.js';
import { Surface, svg, pointsAttribute } from '../viewer/surface.js';
import { Tools, type ToolName, type ToolProgress } from '../viewer/tools.js';
import { loadSheet, type Sheet } from '../viewer/page-source.js';
import {
  derivedField, iconButton, stripButton, stripField,
  type RailTool, type SurfaceChrome,
} from '../chrome.js';
import { hueFor, type IconName } from '../icons.js';
import { KIND_LABELS, PENDING_REASON, quantity } from '../labels.js';
import { select, selectedConditionId, watchSelection } from '../selection.js';

// `Condition`, `Page`, `Trace` and `Point` are the engine's. They were copied
// here — three shapes, each a little different from the real one and from the
// other editors' copies — and the copies are how the page bounds ended up
// invisible to everything but this file and how `hidden` ended up in the job
// folder without being in the format's own type. One definition, in the package
// whose job is the shape of a job.

/**
 * One act in the rail: what a roofer calls it, which tool does it, and what
 * kind of condition it makes.
 *
 * The verbs are the handoff's. Only the acts this program can actually carry
 * out are here: "Set high and low", "Box out a curb", "Measure" and "Section"
 * are named in the handoff and are not built, so they are not in the rail. An
 * estimator is never offered a button that does nothing.
 *
 * The act is also what names the condition it makes. A run traced with "Run a
 * parapet" is a parapet and a run traced with "Cricket" is a cricket, and the
 * roles — which counts are drains, which run is a ridge — are set here rather
 * than being read off the name later. The Model cannot solve a heightfield
 * without knowing which counts are drains, and until now nothing in the window
 * could say so: a role could only arrive from a fixture.
 */
interface Act {
  readonly id: string;
  readonly label: string;
  readonly icon: IconName;
  readonly tool: ToolName;
  readonly role?: ConditionRole;
  /** What a new condition off this act is called. */
  readonly names?: string;
}

const ACTS: readonly Act[] = [
  { id: 'select', label: 'Select', icon: 'select', tool: 'select' },
  { id: 'area', label: 'Trace an area', icon: 'area', tool: 'area', names: 'Area' },
  { id: 'parapet', label: 'Run a parapet', icon: 'parapet', tool: 'line', names: 'Parapet' },
  { id: 'drain', label: 'Drop a drain', icon: 'drain', tool: 'count', role: 'drain', names: 'Drain' },
  { id: 'cricket', label: 'Cricket', icon: 'cricket', tool: 'line', role: 'ridge', names: 'Cricket' },
  { id: 'scale', label: 'Scale', icon: 'scale', tool: 'scale' },
];

let surface: Surface;
let tools: Tools;
let currentPageId: string | null = null;
let loadedSource: string | null = null;
let act: Act = ACTS[0]!;
let progress: ToolProgress = { tool: 'select', points: 0, pageArea: null, pageLength: null };
let chrome: SurfaceChrome | null = null;
let stopSubscribing: (() => void) | null = null;
let stopWatching: (() => void) | null = null;

/** The Plan's acts, for the window's rail. */
export const planTools = (pick: (id: string) => void): readonly RailTool[] =>
  ACTS.map((a) => ({ id: a.id, label: a.label, icon: a.icon, onPick: () => pick(a.id) }));

export function mountPlan(host: HTMLElement, surroundings: SurfaceChrome): void {
  unmountPlan();
  host.replaceChildren();
  host.classList.add('plan-editor');
  chrome = surroundings;

  surface = new Surface();

  const hint = document.createElement('p');
  hint.className = 'hint';

  // A4: the scale badge sits on the drawing, bottom-left, where an estimator
  // looks to check what they are measuring against.
  const scaleBadge = document.createElement('div');
  scaleBadge.className = 'scale-badge';
  surface.root.append(scaleBadge);

  // The surface is the drawing (D148, Frame 1). The list of what has been
  // traced folded into the cost panel's Condition tab: it was 320px of the
  // surface drawn twice, and the view switch — an overlay at the drawing's
  // upper right — was landing on top of its first row. The keys that reached
  // it stay: 1 to 9 pick, and the pill says what the next corner goes into.
  host.append(surface.root, hint);

  tools = new Tools(surface, {
    existingPoints: () => currentPageId ? tracedPoints(conditions(), currentPageId) : [],
    onTrace: (kind, points) => void recordTrace(kind, points),
    onScale: (a, b) => askScale(a, b),
    onHint: (text) => { hint.textContent = text; },
    onProgress: (state) => {
      progress = state;
      drawStrip();
      chrome?.setMeasuringInto(state.points > 0 ? measuringInto(state.tool) : null);
    },
    onCursor: (page) => readOut(page),
    // Select picks what is under the pointer. It had nothing to do before,
    // because the list beside the drawing was the only way to pick a condition
    // — and the hint has been saying "click a trace to select it" the whole
    // time. With the list folded into the panel (D148) an act in the rail that
    // did nothing would be exactly the thing the rail is not allowed to have.
    onSelectAt: (event) => pickAt(event),
  });

  // The pointer off the sheet is a real answer: nothing is known here.
  surface.root.addEventListener('pointerleave', () => readOut(null));
  readOut(null);

  surroundings.setTools(planTools((id) => chooseAct(id)));

  // Getting around the sheet. It was a group pinned to the right end of the
  // Plan's own toolbar row, painting over whatever the row scrolled under it
  // (§1.5); it is now an overlay at the top right of the drawing, where the
  // handoff puts view navigation, and an overlay cannot cover a control because
  // there is no row for it to be in.
  surroundings.setViewTools([
    iconButton('zoomOut', 'Zoom out', () => void surface.setZoom(surface.zoom / 1.25)),
    iconButton('zoomIn', 'Zoom in', () => void surface.setZoom(surface.zoom * 1.25)),
    iconButton('fit', 'Fit the sheet in the window', () => void surface.fit()),
  ]);

  chooseAct('select');

  // Selecting a condition anywhere — here, on the sheet, in the Model, or in
  // another window — redraws the list and the drawing.
  stopWatching = watchSelection(() => {
    drawTraces(doc());
    if (progress.points > 0) chrome?.setMeasuringInto(measuringInto(progress.tool));
  });

  stopSubscribing = subscribe((d: Doc) => {
    const pages = (at('/pages', d) as Page[]) ?? [];
    if (!currentPageId && pages[0]) showPage(pages[0].id).catch((e) => report('opening the drawing', e));
    else if (currentPageId) refreshSheetIfChanged(pages).catch((e) => report('opening the drawing', e));
    drawTraces(d);
    const page = pages.find((p) => p.id === currentPageId);
    scaleBadge.textContent = page?.feetPerUnit
      ? (page.scaleNote ?? 'Scaled')
      : 'Not scaled';
    scaleBadge.classList.toggle('none', !page?.feetPerUnit);
    scaleBadge.hidden = !page;
    drawStrip();
  });
}

/**
 * Put the Plan down.
 *
 * The surface switches views in place, so this view is built and taken down
 * repeatedly in one page. Everything it holds outside its own DOM has to come
 * back: two subscriptions, one selection watcher and the tools' keyboard. A
 * subscriber left behind draws into a host that is no longer in the document
 * every time the job changes, and two sets of tools both answer Enter.
 */
export function unmountPlan(): void {
  stopSubscribing?.();
  stopWatching?.();
  stopSubscribing = null;
  stopWatching = null;
  tools?.destroy();
  chrome?.setMeasuringInto(null);
  chrome = null;
  currentPageId = null;
  loadedSource = null;
  progress = { tool: 'select', points: 0, pageArea: null, pageLength: null };
}

/**
 * What the roof is doing under the pointer, on the Plan.
 *
 * Today that is where the cursor is on the sheet, in feet, which the page's own
 * scale answers. The other four — slope, how thick the insulation is there,
 * which way the water goes and how far it has to go to reach a drain — are the
 * Model's to compute from the tapered solve, and the Plan will be able to ask
 * for them when the renderers land. Until then they are em dashes, which is a
 * true thing to say: nothing here knows.
 */
function readOut(page: Point | null): void {
  const sheet = currentPage();
  const where = page === null ? null
    : !sheet?.feetPerUnit ? null
      : `${formatFeetInches(page.x * sheet.feetPerUnit)}, ${formatFeetInches(page.y * sheet.feetPerUnit)}`;
  chrome?.setReadout([
    { key: 'Cursor', value: where },
    { key: 'Slope', value: null },
    { key: 'Insulation here', value: null },
    { key: 'Flow', value: null, water: true },
    { key: '', value: null },
  ]);
}

// ── the act, and what it needs ─────────────────────────────────────────────

function chooseAct(id: string): void {
  act = ACTS.find((a) => a.id === id) ?? ACTS[0]!;
  tools.use(act.tool);
  chrome?.setActiveTool(act.id);
  drawStrip();
}

/** The page the drawing is on, out of the document. */
const currentPage = (d: Doc = doc()): Page | undefined =>
  ((at('/pages', d) as Page[]) ?? []).find((p) => p.id === currentPageId);

/**
 * What the active act needs, and nothing else.
 *
 * Every Plan act measures off a sheet, so every one of them needs the drawing,
 * what it is scaled at and the units the answers come back in — that is not
 * furniture, it is the thing that decides whether a number exists at all. On
 * top of that each act carries its own: the keyed entry an estimator types
 * instead of clicking, the button that closes the shape, and how much has been
 * measured so far.
 */
function drawStrip(): void {
  if (!chrome) return;
  const d = doc();
  const page = currentPage(d);
  const pages = (at('/pages', d) as Page[]) ?? [];

  const fields: Node[] = [];

  // The drawing, and how to add another one. It was a select and a button in a
  // toolbar over the drawing; it is the same select and the same button, in the
  // region the handoff puts them in.
  // Named as TEXT, and then picked from. A <select> carries its chosen option
  // inside itself and `innerText` does not read it — so a strip that offered
  // only the picker named no drawing at all to anything reading the window,
  // which is how section 7's §1.2 went red. With one drawing there is nothing
  // to pick between and the picker is furniture; it appears with the second.
  const named_ = document.createElement('span');
  named_.className = 'strip-field';
  const drawingLabel = document.createElement('span');
  drawingLabel.textContent = 'Drawing';
  const drawingName = document.createElement('span');
  drawingName.className = 'v';
  drawingName.textContent = page ? (page.feetPerUnit ? page.name : `${page.name} (unscaled)`)
    : 'no drawing yet';
  named_.append(drawingLabel, drawingName);

  if (pages.length > 1) {
    const drawing = document.createElement('select');
    drawing.setAttribute('aria-label', 'Drawing');
    for (const p of pages) drawing.append(option(p.id, p.feetPerUnit ? p.name : `${p.name} (unscaled)`));
    drawing.value = currentPageId ?? '';
    drawing.addEventListener('change', () => {
      showPage(drawing.value).catch((e) => report('opening the drawing', e));
    });
    named_.append(drawing);
  }
  fields.push(named_);
  fields.push(stripButton('Add a drawing', () => {
    addDrawing().catch((e) => report('Adding a drawing', e));
  }, true));

  // What the sheet is scaled at, and the one action that sets it. Its word
  // reads Scale or Rescale by state — the sheet either has a scale or it does
  // not, and that is a state, not two commands (D129).
  if (act.id === 'scale') {
    const picker = document.createElement('select');
    picker.setAttribute('aria-label', 'Architectural scale');
    picker.append(option('', 'Pick a scale'));
    for (const s of ARCHITECTURAL_SCALES) picker.append(option(String(s.feetPerInch), s.label));
    picker.addEventListener('change', () => {
      const feetPerInch = Number(picker.value);
      if (!feetPerInch || !currentPageId) return;
      const cal = calibrateFromScale(feetPerInch, surface.unitsPerInch);
      void applyScale(cal.feetPerUnit, picker.selectedOptions[0]?.text ?? 'preset');
      picker.value = '';
    });
    fields.push(labelled('Or', picker));
  }
  fields.push(stripField('Scale', page?.feetPerUnit ? (page.scaleNote ?? 'set') : 'not scaled',
    undefined, true));
  fields.push(stripField('Units', 'SF · LF · EA · SQ'));

  // Snapping, said as what it does. The handoff's strip reads "Snap to 6″",
  // which claims a snap measured in roof inches; this one pulls a corner onto a
  // corner already on the sheet within ten pixels of the screen, at whatever
  // zoom is up. Saying 6″ would be a number the program does not honour.
  if (act.tool === 'area' || act.tool === 'line') {
    fields.push(stripField('Snap', 'to corners already traced'));

    // The keyed entry, from VES muscle memory. A box, or a length carrying the
    // run on in the direction it is already going.
    const keyed = document.createElement('input');
    keyed.type = 'text';
    keyed.className = 'keyed';
    keyed.placeholder = '40x30 · 25';
    keyed.setAttribute('aria-label', 'Type a box or a length');
    keyed.title = 'Type 40x30 for a box off the last corner, or 25 to carry the run on';
    const trouble = document.createElement('span');
    trouble.className = 'why';
    keyed.addEventListener('keydown', (e) => {
      if (e.key !== 'Enter') return;
      e.preventDefault();
      const said = applyKeyed(keyed.value);
      keyed.classList.toggle('bad', said !== null);
      trouble.textContent = said ?? '';
      if (said === null) keyed.value = '';
    });
    fields.push(labelled('Keyed', keyed), trouble);

    fields.push(stripButton(act.tool === 'area' ? 'Close the area' : 'End the run',
      () => tools.close()));
  }

  // How much has been measured so far, in the units the sheet can answer in.
  if (act.tool === 'area') fields.push(soFar('Area so far', progress.pageArea, 2, page, 'SF'));
  if (act.tool === 'line') fields.push(soFar('Run so far', progress.pageLength, 1, page, 'LF'));

  chrome.strip.show(act.id === 'scale' && page?.feetPerUnit ? 'Rescale' : act.label, fields);
}

/**
 * A measure taken off the shape in progress, in feet.
 *
 * Page units times the page's own scale — the same conversion the engine makes
 * for a finished trace, so the strip cannot disagree with the list beside it.
 * An unscaled sheet answers with a dash and the reason, never a number.
 */
function soFar(label: string, pageValue: number | null, power: number, page: Page | undefined, unit: string): HTMLElement {
  if (pageValue === null) return stripField(label, '—', undefined, true);
  if (!page?.feetPerUnit) {
    const field = stripField(label, '—', undefined, true);
    field.title = PENDING_REASON;
    return field;
  }
  const feet = pageValue * page.feetPerUnit ** power;
  return derivedField(label, `${quantity(feet)} ${unit}`,
    'Worked out from the corners on the sheet and the scale the sheet is set at.');
}

/**
 * A keyed entry, applied. Answers null when it worked and the reason when it
 * did not — a field that silently does nothing is the defect this program was
 * started to escape.
 */
function applyKeyed(text: string): string | null {
  const page = currentPage();
  if (!page?.feetPerUnit) return 'this sheet has no scale yet';
  const raw = text.trim();
  if (!raw) return 'type 40x30 for a box, or 25 to carry the run on';

  const box = /^([^x×]+)[x×](.+)$/i.exec(raw);
  if (box) {
    const w = parseFeet(box[1]!.trim());
    const h = parseFeet(box[2]!.trim());
    if (w === null || h === null) return 'that is not two lengths';
    if (act.tool !== 'area') return 'a box is traced with Trace an area';
    if (progress.points === 0) return 'click the corner it starts from first';
    return tools.box(w / page.feetPerUnit, h / page.feetPerUnit)
      ? null : 'that box could not be laid down';
  }

  const length = parseFeet(raw);
  if (length === null) return 'that is not a length';
  if (progress.points < 2) return 'click two corners first, so it has a direction';
  return tools.extend(length / page.feetPerUnit)
    ? null : 'that length could not be carried on';
}

/** What a trace about to be recorded will go into. The pill says this out loud. */
function measuringInto(tool: ToolName): string {
  const kind = tool === 'area' ? 'area' : tool === 'count' ? 'count' : 'line';
  const selected = conditions().find((c) => c.id === selectedConditionId() && c.kind === kind);
  if (selected) return selected.name;
  return `a new ${act.names ?? KIND_LABELS[kind]}`;
}

/**
 * Pick the traced thing under the pointer.
 *
 * Read off what was actually hit rather than worked out from coordinates: every
 * shape the Plan draws carries the condition it belongs to, so the hit test is
 * the browser's own and it agrees with the picture by construction. A click on
 * bare paper picks nothing, which is a real answer — it is how an estimator
 * puts a selection down.
 */
function pickAt(event: PointerEvent): void {
  const target = event.target as Element | null;
  const shape = target?.closest?.('[data-condition]') as HTMLElement | null;
  const id = shape?.dataset['condition'];
  if (!id) return;
  select({ kind: 'condition', id });
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
 * is the worst thing this view could do — the estimator would trace nothing
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

  // Record how big the paper is, once, the first time it is opened. Nothing
  // downstream could tell the drawing from the desk around it without this —
  // a corner clicked past the edge measured the same as one on the roof.
  if (page.width !== sheet.width || page.height !== sheet.height) {
    const pages = ((at('/pages') as Page[]) ?? []).map((p) =>
      (p.id === page.id ? { ...p, width: sheet.width, height: sheet.height } : p));
    await set('/pages', pages);
  }
}

// ── scale ──────────────────────────────────────────────────────────────────

/**
 * Ask how long that really is — in the window, not in a browser dialog.
 *
 * The field appears in the tool settings strip, which is where everything the
 * Scale act needs now lives. A dialog would take the estimator off the sheet
 * they are pointing at, and in this webview there is no dialog to take them to
 * anyway: `prompt` returns nothing and the caller gives up quietly.
 */
function askScale(a: Point, b: Point): void {
  if (!chrome) return;

  const wrap = document.createElement('span');
  wrap.className = 'scale-ask scale-field';

  const text = document.createElement('span');
  text.textContent = 'How long is that, really?';

  const input = document.createElement('input');
  input.type = 'text';
  input.placeholder = `12'-6"`;
  input.setAttribute('aria-label', 'The real distance between those two points');

  const note = document.createElement('small');
  note.textContent = `12'-6", 6", 4'-6 1/2" and 12.5 all read the same way`;

  const accept = stripButton('Set the scale', () => submit());
  const cancel = stripButton('Cancel', () => finish(), true);

  wrap.append(text, input, accept, cancel, note);
  chrome.strip.show('Scale', [wrap]);
  input.focus();

  const finish = () => {
    drawStrip();
    const hint = document.querySelector<HTMLElement>('.hint');
    if (hint) hint.textContent = 'click a trace to select it';
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
  // one, named for the act that made it, so a trace is never dropped on the
  // floor. The act is what carries the role: a count dropped with "Drop a
  // drain" IS a drain, and the Model reads that to find the roof's low points.
  const selected = all.find((c) => c.id === selectedConditionId() && c.kind === kind);
  if (selected) {
    await set('/conditions', all.map((c) =>
      c.id === selected.id ? { ...c, traces: [...(c.traces ?? []), trace] } : c));
    return;
  }

  const word = act.names ?? KIND_LABELS[kind]!;
  const condition: Condition = {
    id: `condition-${Date.now().toString(36)}`,
    name: `${word} ${all.filter((c) => c.name.startsWith(word)).length + 1}`,
    kind,
    traces: [trace],
    properties: {},
    items: [],
    color: hueFor(all.length),
    ...(act.role ? { role: act.role } : {}),
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

      // Every shape says which condition it belongs to, so a click on it can be
      // answered by what was hit rather than by arithmetic over the points.
      const whose = { 'data-condition': c.id };

      if (c.kind === 'count') {
        for (const p of t.points) {
          g.append(svg('circle', { cx: p.x, cy: p.y, r: 4 * surface.pageUnitsPerPixel, fill: color, ...common, ...whose }));
        }
      } else if (c.kind === 'area') {
        g.append(svg('polygon', { points: pointsAttribute(t.points), fill: color, ...common, ...whose }));
      } else {
        g.append(svg('polyline', { points: pointsAttribute(t.points), fill: 'none', ...common, ...whose }));
      }
    }
  }
}

// ── odds and ends ──────────────────────────────────────────────────────────

function option(value: string, label: string): HTMLOptionElement {
  const o = document.createElement('option');
  o.value = value;
  o.textContent = label;
  return o;
}

/** A control in the strip with the word that says what it is. */
function labelled(label: string, control: HTMLElement): HTMLElement {
  const wrap = document.createElement('span');
  wrap.className = 'strip-field';
  const k = document.createElement('span');
  k.textContent = label;
  wrap.append(k, control);
  return wrap;
}

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
import { NOTHING_TO_DRAW, type Drawings } from '@roofnerd/engine';
import {
  at, addPage, doc, pageBytes, pickFile, set, solveDrawings, solveId, subscribe, type Doc,
} from '../doc.js';
import { Surface, svg, pointsAttribute } from '../viewer/surface.js';
import { renderDrawing } from '../viewer/drawing.js';
import { Tools, type ToolName, type ToolProgress } from '../viewer/tools.js';
import { loadSheet, type Sheet } from '../viewer/page-source.js';
import {
  derivedField, iconButton, stripButton, stripField,
  type RailTool, type SurfaceChrome,
} from '../chrome.js';
import { hueFor, type IconName } from '../icons.js';
import { KIND_LABELS, PENDING_REASON, quantity } from '../labels.js';
import { select, selectedConditionId, watchSelection } from '../selection.js';
import {
  clearVerbPicks, isVerb, useVerb, verbMarks, verbNow, verbPick, verbStrip, watchVerb,
} from '../verbs.js';

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
 * The verbs are the handoff's, and all eight of them are here now: trace an
 * area, set high and low, drop a drain, run a parapet, box out a curb, add a
 * cricket, measure, cut a section. The four that were missing are the four that
 * work ON the roof rather than tracing something new onto the sheet, and they
 * live in `verbs.ts` because the Model offers the same four and an act that
 * meant one thing here and another there would be two acts wearing one word.
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
  { id: 'high-low', label: 'Set high and low', icon: 'highLow', tool: 'pick' },
  { id: 'drain', label: 'Drop a drain', icon: 'drain', tool: 'count', role: 'drain', names: 'Drain' },
  { id: 'parapet', label: 'Run a parapet', icon: 'parapet', tool: 'line', names: 'Parapet' },
  { id: 'curb', label: 'Box out a curb', icon: 'curb', tool: 'pick' },
  { id: 'cricket', label: 'Add a cricket', icon: 'cricket', tool: 'line', role: 'ridge', names: 'Cricket' },
  { id: 'measure', label: 'Measure', icon: 'measure', tool: 'pick' },
  { id: 'section', label: 'Cut a section', icon: 'section', tool: 'pick' },
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
let stopWatchingVerb: (() => void) | null = null;

/**
 * Which solved drawing is laid over the sheet, if any.
 *
 * The handoff puts the tapered layout and the fastening plan on the Plan's own
 * view switch, which is right: they are drawings OF the sheet's own roof, in the
 * sheet's own place, and looking at one means looking at the drawing it was
 * taken off. Neither is on until it is asked for — the Plan is for tracing, and
 * seven hundred lettered boards over a drawing somebody is tracing is in the way.
 */
type Overlay = 'layout' | 'fastening' | null;
let overlay: Overlay = null;
let drawings: Drawings = NOTHING_TO_DRAW;
let drawnSolveId = -1;
let pulling = 0;
let refreshOverlayChips: () => void = () => undefined;
let paintOverlay: () => void = () => undefined;

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

  // What the overlay on the sheet is, and what it is drawn at — or why there is
  // nothing on it. It sits on the drawing beside the scale, because that is
  // where somebody reading a drawing looks for what it is of.
  const overlayNote = document.createElement('div');
  overlayNote.className = 'drawing-note';
  overlayNote.hidden = true;
  surface.root.append(overlayNote);

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
    // A place on the sheet is a place on the ROOF once the sheet's own scale is
    // applied. An unscaled sheet has no places on it, which is why the act says
    // so rather than picking somewhere.
    onPick: (page) => {
      const sheet = currentPage();
      if (!sheet?.feetPerUnit) { hint.textContent = PENDING_REASON; return; }
      verbPick({ x: page.x * sheet.feetPerUnit, y: page.y * sheet.feetPerUnit });
    },
    onPickCleared: () => clearVerbPicks(),
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
  // The two solved drawings, on the view switch beside getting around the sheet.
  // They are the same kind of thing as a zoom: a way of looking at what is
  // already there, which changes nothing about the job.
  const overlayChips: HTMLButtonElement[] = [];
  for (const id of ['layout', 'fastening'] as const) {
    const chip = document.createElement('button');
    chip.type = 'button';
    chip.className = 'view-chip';
    chip.textContent = id;
    chip.addEventListener('click', () => {
      overlay = overlay === id ? null : id;
      if (overlay === null) paintOverlay();
      else void pullDrawings();
      refreshOverlayChips();
    });
    overlayChips.push(chip);
  }
  refreshOverlayChips = () => {
    overlayChips[0]!.classList.toggle('on', overlay === 'layout');
    overlayChips[1]!.classList.toggle('on', overlay === 'fastening');
  };
  refreshOverlayChips();
  const chipGroup = document.createElement('div');
  chipGroup.className = 'chip-group';
  chipGroup.append(...overlayChips);

  surroundings.setViewTools([
    chipGroup,
    iconButton('zoomOut', 'Zoom out', () => void surface.setZoom(surface.zoom / 1.25)),
    iconButton('zoomIn', 'Zoom in', () => void surface.setZoom(surface.zoom * 1.25)),
    iconButton('fit', 'Fit the sheet in the window', () => void surface.fit()),
  ]);

  /**
   * Lay the solved drawing over the sheet, in the sheet's own units.
   *
   * A drawing is in FEET on the roof and the sheet is in page units, and the one
   * turns into the other by the scale the estimator set — the same scale, the
   * same direction, so the board grid lands exactly on the outline it was cut
   * from. A sheet nobody has scaled cannot carry an overlay at all, and says so
   * rather than drawing one somewhere.
   */
  paintOverlay = () => {
    const page = currentPage();
    const drawing = overlay === 'layout' ? drawings.layout
      : overlay === 'fastening' ? drawings.fastening : null;
    if (!drawing || !page?.feetPerUnit) {
      surface.drawing.replaceChildren();
      overlayNote.hidden = overlay === null;
      overlayNote.textContent = overlay === null ? ''
        : 'This sheet has no scale yet, so there is nothing to lay over it.';
      return;
    }
    const perFoot = 1 / page.feetPerUnit;
    const drawn = renderDrawing(surface.drawing, drawing, {
      perFootX: perFoot,
      perFootY: perFoot,
      // A word on a plan is drawn at a size on the ROOF, not on the screen, so
      // it stays inside the board it names at every zoom. Two feet of letter on
      // a four foot board is what a tapered layout prints.
      wordSize: perFoot * 2.2,
      letterSize: perFoot * 2,
    });
    overlayNote.hidden = false;
    overlayNote.textContent = drawn.nothingYet ?? drawn.note;
    if (drawn.nothingYet) surface.drawing.replaceChildren();
  };

  // The act the window was already in, if it is in one. A view switch does not
  // end an act: the two picks of "Set high and low" stay on the roof until the
  // next act, and the roof is the same roof in plan.
  chooseAct(verbNow() ?? 'select');

  // What the act has left on the sheet, and the strip that goes with it. Both
  // are redrawn together: a mark on the roof and a reading in the strip are one
  // answer said twice, and a window where they disagree is worse than one that
  // says neither.
  stopWatchingVerb = watchVerb(() => { drawMarks(); drawStrip(); });

  // Selecting a condition anywhere — here, on the sheet, in the Model, or in
  // another window — redraws the list and the drawing.
  stopWatching = watchSelection(() => {
    drawTraces(doc());
    drawMarks();
    if (progress.points > 0) chrome?.setMeasuringInto(measuringInto(progress.tool));
  });

  const pullDrawings = async () => {
    const want = solveId();
    if (want === drawnSolveId) { paintOverlay(); return; }
    const token = pulling + 1;
    pulling = token;
    try {
      const got = await solveDrawings();
      if (token !== pulling) return;
      drawings = got;
      drawnSolveId = got.solveId;
    } catch {
      // A roof that did not solve has no drawing to lay over the sheet. The
      // Model says why; the Plan is still a drawing somebody can trace on.
      if (token !== pulling) return;
      drawings = NOTHING_TO_DRAW;
      drawnSolveId = want;
    }
    paintOverlay();
  };

  stopSubscribing = subscribe((d: Doc) => {
    const pages = (at('/pages', d) as Page[]) ?? [];
    if (overlay !== null) void pullDrawings();
    if (!currentPageId && pages[0]) showPage(pages[0].id).catch((e) => report('opening the drawing', e));
    else if (currentPageId) refreshSheetIfChanged(pages).catch((e) => report('opening the drawing', e));
    drawTraces(d);
    drawMarks();
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
  stopWatchingVerb?.();
  stopSubscribing = null;
  stopWatching = null;
  stopWatchingVerb = null;
  tools?.destroy();
  chrome?.setMeasuringInto(null);
  chrome = null;
  currentPageId = null;
  loadedSource = null;
  progress = { tool: 'select', points: 0, pageArea: null, pageLength: null };
  overlay = null;
  drawings = NOTHING_TO_DRAW;
  drawnSolveId = -1;
  paintOverlay = () => undefined;
  refreshOverlayChips = () => undefined;
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
  // Four of the ten acts work on the roof rather than on the sheet; picking one
  // of the other six puts whichever was up down, which is what takes its marks
  // off the roof.
  useVerb(isVerb(act.id) ? act.id : null);
  drawMarks();
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
  // An act that works on the roof says what it needs itself, in the same words
  // on both views. The sheet's own settings — which drawing, what it is scaled
  // at — are not what that act is doing and would be furniture under it.
  const verb = verbStrip();
  if (verb) { chrome.strip.show(verb.name, verb.fields); return; }
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

/**
 * What the act in the rail has left on the sheet.
 *
 * The marks arrive in FEET, because that is what the core answers in and what
 * the Model draws them in; the sheet is in page units, so they come back through
 * the same scale every trace on it went out through. Nothing is worked out here
 * — where the picks are, where the ruler runs, where the cut goes and where the
 * cricket behind a curb stands are all the act's, and the act got them from the
 * solve.
 */
function drawMarks(): void {
  if (!surface) return;
  const g = surface.marks;
  g.replaceChildren();
  const page = currentPage();
  if (!page?.feetPerUnit) return;
  const unit = 1 / page.feetPerUnit;
  const px = surface.pageUnitsPerPixel;

  for (const mark of verbMarks()) {
    const at = { x: mark.at.x * unit, y: mark.at.y * unit };
    const to = mark.to ? { x: mark.to.x * unit, y: mark.to.y * unit } : null;

    if (mark.ring) {
      g.append(svg('polygon', {
        points: pointsAttribute(mark.ring.map((p) => ({ x: p.x * unit, y: p.y * unit }))),
        class: 'verb-mark verb-box', 'data-verb-mark': mark.kind,
      }));
    }
    if (to) {
      g.append(svg('line', {
        x1: at.x, y1: at.y, x2: to.x, y2: to.y,
        class: 'verb-mark verb-line', 'data-verb-mark': mark.kind,
      }));
    }
    if (mark.kind === 'high' || mark.kind === 'low' || mark.kind === 'corner') {
      g.append(svg('circle', {
        cx: at.x, cy: at.y, r: 5 * px,
        class: 'verb-mark verb-pick', 'data-verb-mark': mark.kind,
      }));
    }
    if (mark.text) {
      const where = to ? { x: (at.x + to.x) / 2, y: (at.y + to.y) / 2 } : at;
      const word = svg('text', {
        x: where.x, y: where.y - 8 * px,
        class: 'verb-mark verb-word', 'data-verb-mark': mark.kind,
        'font-size': 12 * px,
      });
      word.textContent = mark.text;
      g.append(word);
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

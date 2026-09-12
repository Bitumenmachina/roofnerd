// ── Booting a window ───────────────────────────────────────────────────────
// Every window loads this same page and reads off its own label what it is:
// the one the job was opened in, or the cost panel pulled onto a second
// monitor. There is no main window that owns anything — both read the same
// document out of the shell.
//
// The window is the handoff's anatomy: document bar, tool settings strip, the
// body (tool rail · surface · cost panel), status strip. The surface holds the
// views — Plan and Model — and switches between them IN PLACE. It used to
// reload the page with a different editor named in the URL, which threw away
// the selection, the scroll and everything the page had drawn.

import { getCurrentWindow } from '@tauri-apps/api/window';
import { emit, listen } from '@tauri-apps/api/event';
import { LENSES, measure, type Condition, type Page } from '@roofnerd/engine';
import { at, connect, demoFolder, folder, open, pickFolder, save, subscribe, tearOff, type Doc } from './doc.js';
import {
  documentBar, menu, startScreen, statusStrip, toolRail, toolStrip,
  STATUS_SAID, type RailTool, type Reading, type SurfaceChrome,
} from './chrome.js';
import { applyTheme, followTheme } from './theme.js';
import { HELP } from './help.js';
import { followSelection, select, selectedConditionId, watchSelection } from './selection.js';
import { costPanel } from './cost-panel.js';
import { mountPlan, unmountPlan } from './editors/plan.js';
import { mountModel, unmountModel } from './editors/model.js';
import { quantity } from './labels.js';

/**
 * The two views of the one geometry that exist today.
 *
 * Section arrives with the renderers; it is not in here, because a view switch
 * offering a view nobody built is a button that does nothing.
 */
interface View {
  readonly title: string;
  readonly mount: (host: HTMLElement, chrome: SurfaceChrome) => void;
  readonly unmount: () => void;
  readonly help: string;
}

const VIEWS: Record<string, View> = {
  plan: { title: 'Plan', mount: mountPlan, unmount: unmountPlan, help: HELP['Plan']! },
  model: { title: 'Model', mount: mountModel, unmount: unmountModel, help: HELP['Model']! },
};

/** The order they are offered in. Section joins them with the renderers. */
const VIEW_ORDER = ['plan', 'model'] as const;

const params = new URLSearchParams(window.location.search);

/** Which window is this? Its own label answers it. */
function labelOfWindow(): string {
  try {
    return getCurrentWindow().label;
  } catch {
    // The front-end harness runs this page in a plain browser with the shell
    // stubbed, so there is no window to ask and the URL is all there is.
    return params.get('editor') ?? 'main';
  }
}

const role = labelOfWindow();

// The theme goes up before anything is drawn, so no window flashes light on
// its way to dark.
applyTheme();

const docBarHost = document.querySelector<HTMLElement>('#doc-bar')!;
const stripHost = document.querySelector<HTMLElement>('#tool-strip')!;
const bodyRow = document.querySelector<HTMLElement>('#body-row')!;
const railHost = document.querySelector<HTMLElement>('#tool-rail')!;
const surfaceRegion = document.querySelector<HTMLElement>('#surface-region')!;
const host = document.querySelector<HTMLElement>('#editor')!;
const panelHost = document.querySelector<HTMLElement>('#cost-panel')!;
const statusHost = document.querySelector<HTMLElement>('#status-strip')!;

// ── the surface ────────────────────────────────────────────────────────────
// The view mounts into this, and it keeps the id the rest of the program — and
// every check — uses to mean "where the editor is".
const surface = document.createElement('div');
surface.className = 'editor-host';
surface.id = 'editor';
host.replaceWith(surface);

// Getting around the view, upper right where the handoff's pattern lineage puts
// view navigation. The view switch itself joins this column when it arrives.
const viewOverlay = document.createElement('div');
viewOverlay.className = 'view-switch';

// ── the view switch ───────────────────────────────────────────────────────
// Plan and Model are two views of one geometry, and switching between them is
// a swap of what is mounted in the surface — not a navigation. It used to be
// `window.location.assign` with a different editor in the URL: a reload, which
// threw away the selection, the panel, the scroll and every hook in the page,
// and took two and a half seconds doing it. The document is in the shell and
// both views read it, so there was never anything a reload was for.
const viewButtons = document.createElement('div');
viewButtons.className = 'views';
const viewControls = new Map<string, HTMLButtonElement>();
for (const id of VIEW_ORDER) {
  const button = document.createElement('button');
  button.type = 'button';
  button.textContent = VIEWS[id]!.title;
  button.addEventListener('click', () => showView(id));
  viewButtons.append(button);
  viewControls.set(id, button);
}

const viewCaption = document.createElement('span');
viewCaption.className = 'view-caption';
viewCaption.textContent = 'one geometry · two views';

const viewNav = document.createElement('div');
viewNav.className = 'view-nav';

// The way home for a panel that is on the other monitor. Dashed and in the one
// loud colour, because it is a state the estimator put the window into and the
// window has to say so rather than looking like it lost something.
const redock = document.createElement('button');
redock.type = 'button';
redock.className = 'redock';
redock.textContent = 'bring the cost panel back';
redock.hidden = true;
redock.addEventListener('click', () => { void sendHome(); });

viewOverlay.append(viewButtons, viewCaption, viewNav, redock);
surfaceRegion.append(viewOverlay);
viewOverlay.hidden = true;

// The "Measuring into" pill, on the drawing while a shape is open. It is an
// overlay on the surface rather than a line under it, because the question it
// answers — is this going into the right condition? — is asked while looking at
// the drawing.
const pill = document.createElement('div');
pill.className = 'measuring-pill';
pill.hidden = true;
const pillLabel = document.createElement('span');
pillLabel.className = 'label';
pillLabel.textContent = 'Measuring into';
const pillInto = document.createElement('span');
pillInto.className = 'into';
pill.append(pillLabel, pillInto);
surfaceRegion.append(pill);

// ── the tool rail ──────────────────────────────────────────────────────────
const rail = toolRail();
railHost.replaceWith(rail.root);

// ── the status strip ───────────────────────────────────────────────────────
const status = statusStrip();
statusHost.replaceWith(status.root);
status.setReadout(EMPTY_READOUT());

function EMPTY_READOUT() {
  // Em dashes, not blanks. "Nothing is known here" is an answer, and the
  // handoff asks for it in as many words on the first-open frame.
  return [
    { key: 'Cursor', value: null },
    { key: 'Slope', value: null },
    { key: 'Insulation here', value: null },
    { key: 'Flow', value: null, water: true },
    { key: '', value: null },
  ];
}

// An editor that has something to report says it where it stands and it rises
// to here. This window owns its status strip; the editor in it does not.
document.addEventListener(STATUS_SAID, (e) => status.say((e as CustomEvent<string>).detail));

// ── the tool settings strip ────────────────────────────────────────────────
const strip = toolStrip();
stripHost.replaceWith(strip.root);

/**
 * What a view in the surface is handed: the regions around it.
 *
 * The window owns the rail, the strip, the pill and the readout; the view
 * decides what goes in them. That is the division the handoff draws — regions
 * belong to the window, contents belong to whatever is being looked at.
 */
const surfaceChrome: SurfaceChrome = {
  setTools: (tools: readonly RailTool[]) => rail.setTools(tools),
  setActiveTool: (id) => rail.active(id),
  setViewTools: (controls) => viewNav.replaceChildren(...controls),
  strip,
  setMeasuringInto: (into) => {
    pillInto.textContent = into ?? '';
    pill.hidden = into === null;
  },
  setReadout: (readings: readonly Reading[]) => status.setReadout(readings),
};

// ── the document bar ───────────────────────────────────────────────────────
const fileMenu = menu('File', [
  { label: 'Open a job…', onSelect: () => void openJob() },
  { label: 'Save', onSelect: () => void saveJob() },
]);

/**
 * Who the estimate is being shown to.
 *
 * The first row is the estimate itself — nobody's filter — and the rest are the
 * lenses the program already has. Picking one shows it in the cost panel: a
 * lens is a read over the same document, so it belongs where the money is and
 * not in an editor of its own.
 */
const LENS_CHOICES = [
  { id: null, name: 'Full estimate', audience: 'me and my project manager' },
  ...LENSES.map((l) => ({ id: l.id, name: l.name, audience: l.audience })),
];

let lens: string | null = null;

const docBar = documentBar({
  menus: [fileMenu],
  lenses: LENS_CHOICES,
  onLens: (id) => { lens = id; panel.setLens(id); },
  help: { title: 'Plan', body: HELP['Plan']! },
});
docBarHost.replaceWith(docBar.root);

// ── the cost panel ─────────────────────────────────────────────────────────
// Geometry on one side, money on the other, both visible at once. The panel is
// a region of the window; what is in it is its own business.
const panel = costPanel({
  detached: role === 'cost',
  onDetach: () => { void tearOff('cost').catch((e) => status.say(message(e))); },
  onDock: () => { void sendHome(); },
  onLensCleared: () => docBar.setLens(null),
  onShowingChanged: () => setHelp(),
});
panelHost.replaceWith(panel.root);

// ── the window that is only the panel ──────────────────────────────────────
// Detached, this page is the cost panel and its own title bar and nothing
// else: no rail, no surface, no document bar. It reads the same document out
// of the shell and follows the same selection, which is what makes it a second
// window over one job rather than a copy of it.
if (role === 'cost') {
  document.body.classList.add('cost-window');
  document.body.replaceChildren(panel.root);
}

let currentDoc: Doc = {};
let hasJob = false;
let view: keyof typeof VIEWS = 'plan';
let mounted: string | null = null;

// ── the panel, docked or on the other monitor ──────────────────────────────
// The detached panel is a second window over the same document, so "where is
// it" is a fact both windows have to agree on. They agree by saying it out
// loud on the same bus the document and the selection already use: the cost
// window says it has the panel when it boots, and either window can say the
// panel is going home — the main window shows it again and the cost window
// closes itself.
const PANEL_AWAY = 'panel:detached';
const PANEL_HOME = 'panel:docked';

let away = false;

function panelIsAway(yes: boolean): void {
  if (role === 'cost') return;
  away = yes;
  panel.root.hidden = yes;
  bodyRow.classList.toggle('no-panel', yes);
  redock.hidden = !yes;
  sayWhereThePanelIs();
}

async function sendHome(): Promise<void> {
  await emit(PANEL_HOME, {}).catch(() => undefined);
  if (role === 'cost') {
    try {
      await getCurrentWindow().close();
    } catch {
      // A window that cannot close itself still handed the panel back; the
      // estimator is left with two, which is untidy and not wrong.
    }
  }
}

void listen(PANEL_AWAY, () => panelIsAway(true)).catch(() => undefined);
void listen(PANEL_HOME, () => {
  if (role === 'cost') return;
  panelIsAway(false);
}).catch(() => undefined);

if (role === 'cost') {
  // This window IS the panel. It says so, so the window that lost it can give
  // the width to the drawing.
  void emit(PANEL_AWAY, {}).catch(() => undefined);
}

/** What the status strip says at its right end about where the money is. */
function sayWhereThePanelIs(): void {
  status.setAside(away ? 'the cost panel is in its own window' : '');
}

// ── the one roof-area figure ───────────────────────────────────────────────

/**
 * The sum of what the area conditions measure, and what it could not count.
 *
 * Everything on screen reconciles to this number, so it is the areas' own
 * square feet added up — not a second piece of arithmetic. An area on a sheet
 * nobody has scaled has no square feet yet; it is named in the basis and said
 * beside the figure, never quietly left out.
 */
function roofArea(d: Doc): Parameters<typeof docBar.setRoofArea>[0] {
  const pages = (at('/pages', d) as Page[]) ?? [];
  const calibrations: Record<string, { feetPerUnit: number } | undefined> = {};
  for (const p of pages) if (p.feetPerUnit) calibrations[p.id] = { feetPerUnit: p.feetPerUnit };

  const conditions = ((at('/conditions', d) as Condition[]) ?? []).filter((c) => c.kind === 'area');
  const counted: string[] = [];
  const pending: string[] = [];
  let total = 0;
  for (const c of conditions) {
    const source = c.from ? conditions.find((x) => x.id === c.from) ?? c : c;
    const m = measure(source.kind, source.traces ?? [], source.properties ?? {}, calibrations);
    if (m.SF === null) { pending.push(c.name); continue; }
    total += m.SF;
    counted.push(`${c.name} ${quantity(m.SF)} SF`);
  }

  const basis = [
    counted.length ? `Counted: ${counted.join(' · ')}` : 'Nothing with an area is traced yet',
    pending.length ? `Not counted, no scale yet: ${pending.join(' · ')}` : '',
  ].filter(Boolean).join('. ');

  return {
    value: counted.length ? quantity(total) : null,
    unit: 'SF',
    basis,
    leftOut: pending.length
      ? `· ${pending.length === 1 ? 'one area' : `${pending.length} areas`} not counted yet`
      : null,
  };
}

// ── the document ───────────────────────────────────────────────────────────

subscribe((doc: Doc) => {
  currentDoc = doc;
  const name = at('/job/name', doc);
  const named = typeof name === 'string' && name ? name : null;
  docBar.setJob(named);
  docBar.setRoofArea(roofArea(doc));

  const nowHasJob = named !== null;
  if (role === 'cost') { hasJob = nowHasJob; return; }
  if (nowHasJob !== hasJob) {
    hasJob = nowHasJob;
    rail.quiet(!hasJob);
    if (hasJob) {
      viewOverlay.hidden = false;
      showView(view);
    } else {
      viewOverlay.hidden = true;
      showView(null);
      void showStart();
    }
  }
});

/**
 * Show a view in the surface, in place.
 *
 * The selection, the document, the panel and the theme are all outside this
 * function on purpose: they belong to the window and the shell, so a view
 * change cannot touch them. What changes is what is mounted in the surface and
 * what the rail and the strip are holding.
 */
function showView(next: keyof typeof VIEWS | null): void {
  if (mounted !== null && mounted !== next) {
    VIEWS[mounted]!.unmount();
    mounted = null;
  }
  if (next === null) {
    surface.replaceChildren();
    rail.setTools([]);
    return;
  }
  view = next;
  setHelp();
  for (const [id, button] of viewControls) button.classList.toggle('on', id === next);
  if (mounted === next) return;
  mounted = next;
  // A view leaves its own class behind otherwise, and the next one lands on an
  // element still wearing the last one's layout.
  surface.className = 'editor-host';
  setHelp();
  VIEWS[next]!.mount(surface, surfaceChrome);
}

/**
 * What the one "?" opens.
 *
 * One affordance in the document bar, and it answers for what the window is
 * actually showing: the view in the surface, and what the cost panel has up
 * beside it. Two help buttons would be two places to look for one answer.
 */
function setHelp(): void {
  const showing = panel.showing();
  const viewHelp = VIEWS[view]?.help ?? '';
  docBar.setHelp(`${VIEWS[view]?.title ?? 'roofnerd'} · ${showing}`,
    `${viewHelp}\n\n${HELP[showing] ?? ''}`);
}

async function showStart() {
  const demo = await demoFolder().catch(() => null);
  surface.replaceChildren(startScreen({
    onOpen: () => void openJob(),
    recent: [],
    onOpenRecent: (path) => void openJob(path),
    demo: demo ? { path: demo } : null,
  }));
  strip.idle();
}

void start();

async function start() {
  try {
    await connect();
    // And follow what the other windows select and which room they are in, the
    // same way this window follows what they change in the document.
    await followSelection();
    await followTheme();
    await showPath();
    if (!hasJob) await showStart();
  } catch (e) {
    status.say(`Could not reach the document: ${message(e)}`);
  }
}

async function openJob(path?: string) {
  const target = path ?? await pickFolder(await demoFolder());
  if (!target) return;
  try {
    await open(target);
    await showPath();
    select({ kind: 'job' });
    status.say('Opened');
  } catch (e) {
    status.say(message(e));
  }
}

async function saveJob() {
  try {
    await save();
    status.say('Saved');
  } catch (e) {
    status.say(message(e));
  }
}

async function showPath() {
  status.setPath((await folder()) ?? '');
}

const message = (e: unknown) => (typeof e === 'string' ? e : e instanceof Error ? e.message : String(e));

// ── the keyboard, as an estimator already knows it ─────────────────────────
// VES's grammar, because it is the one in the hands: Enter finishes the shape,
// Escape backs out of whatever is open, Backspace takes back the last corner,
// and 1 to 9 pick the nth condition. The first three belong to the shape and
// live with the tools; the numbers and the last rung of Escape are the
// window's, so they are here.
//
// Nothing fires while a field has the caret. An estimator typing "40x30" into
// the keyed entry is not asking for condition 4.
window.addEventListener('keydown', (e) => {
  if (e.altKey || e.ctrlKey || e.metaKey) return;
  const on = document.activeElement;
  if (on instanceof HTMLInputElement || on instanceof HTMLSelectElement
    || on instanceof HTMLTextAreaElement || (on instanceof HTMLElement && on.isContentEditable)) {
    return;
  }

  if (/^[1-9]$/.test(e.key)) {
    const conditions = (at('/conditions', currentDoc) as Condition[]) ?? [];
    const wanted = conditions[Number(e.key) - 1];
    if (!wanted) return;
    e.preventDefault();
    select({ kind: 'condition', id: wanted.id });
    return;
  }

  if (e.key === 'Escape') {
    // The tools take it first when a shape is open — that is what "back out"
    // means while tracing. Otherwise it backs out of the panel's state, one
    // rung at a time, and the last rung is the lens.
    if (backOut()) e.preventDefault();
  }
});

/**
 * One rung out. Answers whether there was anywhere to go.
 *
 * Escape with nothing open used to do nothing at all, which taught the
 * estimator that the key is dead. It is not dead: it puts back the full
 * estimate, which is the state everything else is a departure from.
 */
function backOut(): boolean {
  return panel.backOut();
}

void role;
void bodyRow;

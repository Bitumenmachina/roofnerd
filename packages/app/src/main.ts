// ── Booting a window ───────────────────────────────────────────────────────
// Every window loads this same page and reads off its own URL which editor it
// is. There is no main window that owns anything: a torn-off Estimate Sheet is
// the same code in a different frame, reading the same document.

import { getCurrentWindow } from '@tauri-apps/api/window';
import { at, connect, demoFolder, folder, open, pickFolder, save, subscribe, tearOff, type Doc } from './doc.js';
import { area, menu, startScreen, statusBar, STATUS_SAID } from './chrome.js';
import { HELP } from './help.js';
import { openByDefault, renderTree } from './tree.js';
import { followSelection, select, selectedConditionId, watchSelection } from './selection.js';
import { renderConditionPanel } from './editors/condition-panel.js';
import { mountPlan } from './editors/plan.js';
import { mountEstimate } from './editors/estimate.js';
import { mountModel } from './editors/model.js';
import { mountLibrary } from './editors/library.js';
import { mountReports } from './editors/reports.js';

const EDITORS: Record<string, { title: string; mount: (host: HTMLElement) => void; help: string }> = {
  plan: { title: 'Plan', mount: mountPlan, help: HELP['Plan']! },
  estimate: { title: 'Estimate Sheet', mount: mountEstimate, help: HELP['Estimate Sheet']! },
  model: { title: 'Model', mount: mountModel, help: HELP['Model']! },
  library: { title: 'Library', mount: mountLibrary, help: HELP['Library']! },
  reports: { title: 'Reports', mount: mountReports, help: HELP['Reports']! },
};

const params = new URLSearchParams(window.location.search);
const which = params.get('editor') ?? 'plan';
const editor = EDITORS[which] ?? EDITORS['plan']!;

/**
 * Is this a window that was torn off, or the one the job was opened in?
 *
 * The window's own label answers it: the shell names the first window `main`
 * and names a torn-off one after the editor in it. This used to be read off the
 * URL as "the editor is not the Plan", which is a different question and gave
 * the wrong answer the moment an area's editor picker was used — switching the
 * main window to any other editor made it believe it had been torn off, and it
 * dropped the tree and the menu bar on the floor. Found by section 6's check.
 */
const detached = isDetached();

function isDetached(): boolean {
  try {
    return getCurrentWindow().label !== 'main';
  } catch {
    // The front-end harness runs this page in a plain browser with the shell
    // stubbed, so there is no window to ask. There, one page is one window and
    // the URL is the only thing that distinguishes them.
    return which !== 'plan';
  }
}

const frame = document.querySelector<HTMLElement>('#frame')!;
const sidebar = document.querySelector<HTMLElement>('#sidebar')!;
const host = document.querySelector<HTMLElement>('#editor')!;
const menuBar = document.querySelector<HTMLElement>('#menu-bar')!;
const statusHost = document.querySelector<HTMLElement>('#status')!;

if (detached) {
  frame.classList.add('detached');
  // A torn-off window is the editor and the status bar. A second menu bar in it
  // would be a second copy of the chrome for a window that owns no job.
  menuBar.remove();
}

const status = statusBar();
statusHost.replaceWith(status.root);

// An editor that has something to report says it where it stands and it rises
// to here. This window owns its status bar; the editor in it does not, and a
// torn-off editor is in a different window with a different one.
document.addEventListener(STATUS_SAID, (e) => status.say((e as CustomEvent<string>).detail));

// ── the area, with its own tear-off ────────────────────────────────────────
const { root: areaRoot, body: areaBody, panel: panelHost } = area({
  title: editor.title,
  editor: which,
  help: editor.help,
  editors: Object.entries(EDITORS).map(([id, e]) => ({ id, title: e.title })),
  onSwitch: (id) => {
    // An area can show any editor. Switching is a reload of this same window
    // with a different editor named in its URL, which keeps one code path for
    // "which editor am I" instead of two.
    const url = new URL(window.location.href);
    url.searchParams.set('editor', id);
    window.location.assign(url.toString());
  },
  onTearOff: (name) => {
    // Tearing off the editor you are looking at gives you a second view of it;
    // from the Plan that is the sheet, which is the working pair.
    const target = name === 'plan' ? 'estimate' : name;
    tearOff(target).catch((e) => status.say(message(e)));
  },
});
// The editor mounts into the area's body, and that body keeps the id the rest
// of the program — and every probe — uses to mean "where the editor is".
areaBody.id = 'editor';
host.replaceWith(areaRoot);

// ── the menu bar ───────────────────────────────────────────────────────────
const jobLabel = document.createElement('span');
jobLabel.className = 'menu-job';

menuBar.append(menu('File', [
  { label: 'Open a job…', onSelect: () => void openJob() },
  { label: 'Save', onSelect: () => void saveJob() },
]), jobLabel);

// ── the tree ───────────────────────────────────────────────────────────────
let hasJob = false;

if (!detached) {
  sidebar.addEventListener('tree:changed', () => redrawTree());
  watchSelection(() => redrawTree());
}

function redrawTree() {
  if (detached) return;
  renderTree(sidebar, currentDoc);
}

let currentDoc: Doc = {};

// ── the Properties panel, beside whichever editor this window shows ────────
// The window owns it, not the editor, for the same reason the window owns the
// status bar: it is the same panel in every editor (A3), and a torn-off editor
// is a window with its own. It watches the one selection and the one document,
// which is all it ever needed from the Plan.
//
// It sits below `currentDoc` deliberately: `watchSelection` calls its watcher
// once, immediately, and a watcher reading a `let` declared further down the
// file reads it before it exists.
const drawPanel = () => {
  if (panelHost.hidden) return;
  renderConditionPanel(panelHost, selectedConditionId(), currentDoc, () => drawPanel());
};
watchSelection(() => drawPanel());

subscribe((doc: Doc) => {
  currentDoc = doc;
  drawPanel();
  const name = at('/job/name', doc);
  const named = typeof name === 'string' && name ? name : null;
  jobLabel.textContent = named ?? '';

  const job = at('/job', doc) as { scenarios?: { id: string; name: string }[]; activeScenarioId?: string } | undefined;
  const scenario = job?.scenarios?.find((s) => s.id === job.activeScenarioId) ?? job?.scenarios?.[0];
  const pages = (at('/pages', doc) as { name: string; feetPerUnit?: number; scaleNote?: string }[]) ?? [];
  const scaled = pages.find((p) => p.feetPerUnit);
  status.setFacts({
    ...(named ? { job: named } : {}),
    ...(scenario ? { scenario: scenario.name } : {}),
    ...(pages.length ? { scale: scaled?.scaleNote ?? 'not scaled' } : {}),
    ...(named ? { units: 'SF · LF · EA · SQ' } : {}),
  });

  const nowHasJob = named !== null;
  if (nowHasJob !== hasJob) {
    hasJob = nowHasJob;
    if (hasJob) {
      openByDefault(doc);
      // The panel belongs to an open job. On the start screen there is no
      // editor and nothing to be selected, so it is not there either — and the
      // area gives the whole window to the two buttons that matter.
      panelHost.hidden = false;
      editor.mount(areaBody);
      drawPanel();
    } else {
      panelHost.hidden = true;
      void showStart();
    }
  }
  redrawTree();
});

async function showStart() {
  const demo = await demoFolder().catch(() => null);
  areaBody.replaceChildren(startScreen({
    onOpen: () => void openJob(),
    recent: [],
    onOpenRecent: (path) => void openJob(path),
    demo: demo ? { path: demo } : null,
  }));
}

void start();

async function start() {
  try {
    await connect();
    // And follow what the other windows select, the same way this window
    // follows what they change in the document.
    await followSelection();
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
    status.say('');
    status.setFacts({ saved: 'Saved' });
  } catch (e) {
    status.say(message(e));
  }
}

async function showPath() {
  status.setPath((await folder()) ?? '');
}

const message = (e: unknown) => (typeof e === 'string' ? e : e instanceof Error ? e.message : String(e));

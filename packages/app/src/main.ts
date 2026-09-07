// ── Booting a window ───────────────────────────────────────────────────────
// Every window loads this same page and reads off its own URL which editor it
// is. There is no main window that owns anything: a torn-off Estimate Sheet is
// the same code in a different frame, reading the same document.

import { at, connect, demoFolder, folder, open, save, subscribe, tearOff, type Doc } from './doc.js';
import { area, menu, startScreen, statusBar } from './chrome.js';
import { HELP } from './help.js';
import { openByDefault, renderTree } from './tree.js';
import { select, watchSelection } from './selection.js';
import { mountPlan } from './editors/plan.js';
import { mountEstimate } from './editors/estimate.js';

const EDITORS: Record<string, { title: string; mount: (host: HTMLElement) => void; help: string }> = {
  plan: { title: 'Plan', mount: mountPlan, help: HELP['Plan']! },
  estimate: { title: 'Estimate Sheet', mount: mountEstimate, help: HELP['Estimate Sheet']! },
};

const params = new URLSearchParams(window.location.search);
const which = params.get('editor') ?? 'plan';
const detached = which !== 'plan';
const editor = EDITORS[which] ?? EDITORS['plan']!;

const frame = document.querySelector<HTMLElement>('#frame')!;
const sidebar = document.querySelector<HTMLElement>('#sidebar')!;
const host = document.querySelector<HTMLElement>('#editor')!;
const menuBar = document.querySelector<HTMLElement>('#menu-bar')!;
const statusHost = document.querySelector<HTMLElement>('#status')!;

if (detached) frame.classList.add('detached');

const status = statusBar();
statusHost.replaceWith(status.root);

// ── the area, with its own tear-off ────────────────────────────────────────
const { root: areaRoot, body: areaBody } = area({
  title: editor.title,
  editor: which,
  help: editor.help,
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

subscribe((doc: Doc) => {
  currentDoc = doc;
  const name = at('/job/name', doc);
  const named = typeof name === 'string' && name ? name : null;
  jobLabel.textContent = named ?? '';

  const nowHasJob = named !== null;
  if (nowHasJob !== hasJob) {
    hasJob = nowHasJob;
    if (hasJob) {
      openByDefault(doc);
      editor.mount(areaBody);
    } else {
      showStart();
    }
  }
  redrawTree();
});

function showStart() {
  areaBody.replaceChildren(startScreen({
    onOpen: () => void openJob(),
    recent: [],
    onOpenRecent: (path) => void openJob(path),
  }));
}

void start();

async function start() {
  try {
    await connect();
    await showPath();
    if (!hasJob) showStart();
  } catch (e) {
    status.say(`Could not reach the document: ${message(e)}`);
  }
}

async function openJob(path?: string) {
  const target = path ?? window.prompt('Job folder', (await demoFolder()) ?? '')?.trim();
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

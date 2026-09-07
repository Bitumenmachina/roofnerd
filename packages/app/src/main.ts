// ── Booting a window ───────────────────────────────────────────────────────
// Every window loads this same page. Which editor it becomes it reads off its
// own URL. There is no "main window" that owns anything and no second-class
// pop-out: a torn-off Estimate Sheet is the same code in a different frame,
// reading the same document.

import { connect, demoFolder, folder, open, save, subscribe, tearOff, at, type Doc } from './doc.js';
import { mountPlan } from './editors/plan.js';
import { mountEstimate } from './editors/estimate.js';

const EDITORS: Record<string, (host: HTMLElement) => void> = {
  plan: mountPlan,
  estimate: mountEstimate,
};

const params = new URLSearchParams(window.location.search);
const which = params.get('editor') ?? 'plan';
const detached = which !== 'plan';

const host = document.querySelector<HTMLElement>('#editor');
const status = document.querySelector<HTMLElement>('#status');
if (!host) throw new Error('the page is missing its editor area');

if (detached) document.body.classList.add('detached');

const mount = EDITORS[which] ?? EDITORS['plan']!;
mount(host);

// The tree and the buttons only exist in the window that has them; a torn-off
// editor hides the panel rather than shipping a second set of controls.
if (!detached) {
  wireShell();
}

void start();

async function start() {
  try {
    await connect();
    await showFolder();
  } catch (e) {
    say(`could not reach the document: ${message(e)}`);
  }
}

function wireShell() {
  const jobName = document.querySelector<HTMLElement>('#job-name');
  const treeBody = document.querySelector<HTMLElement>('#tree-body');

  subscribe((doc: Doc) => {
    const name = at('/job.json/name', doc);
    if (jobName) jobName.textContent = typeof name === 'string' ? name : 'no job open';
    if (treeBody) renderTree(treeBody, doc);
  });

  on('#open-demo', async () => {
    // Gate 0 opens the demo job by a path typed into the program, not by a file
    // dialog. The dialog is a Gate 1 concern; what is being proved here is that
    // one folder becomes one document that every window follows.
    const guess = await promptFolder();
    if (!guess) return;
    try {
      await open(guess);
      await showFolder();
      say('opened');
    } catch (e) {
      say(message(e));
    }
  });

  on('#save', async () => {
    try {
      say(`saved to ${await save()}`);
    } catch (e) {
      say(message(e));
    }
  });

  on('#tear-off', async () => {
    try {
      await tearOff('estimate');
    } catch (e) {
      say(message(e));
    }
  });
}

function renderTree(hostEl: HTMLElement, doc: Doc) {
  hostEl.replaceChildren();
  const conditions = at('/conditions.json', doc);
  if (!Array.isArray(conditions) || conditions.length === 0) return;

  const list = document.createElement('ul');
  list.className = 'tree';
  const pages = document.createElement('li');
  pages.textContent = 'Pages';
  const inner = document.createElement('ul');
  for (const c of conditions as { name?: string }[]) {
    const item = document.createElement('li');
    item.textContent = c.name ?? 'condition';
    inner.append(item);
  }
  pages.append(inner);
  list.append(pages);
  hostEl.append(list);
}

async function showFolder() {
  const where = document.querySelector<HTMLElement>('#job-folder');
  if (where) where.textContent = (await folder()) ?? '';
}

/**
 * Gate 0 has no file dialog — that is Gate 1. A one-line prompt keeps the folder
 * path visible instead of hard-coding one and calling it "open". It is prefilled
 * with the demo job when the shell can find one, because a relative path would
 * resolve against wherever the program happens to be running from, and that is
 * not the repository root.
 */
async function promptFolder(): Promise<string | null> {
  const answer = window.prompt('Job folder', (await demoFolder()) ?? '');
  return answer && answer.trim() ? answer.trim() : null;
}

function on(selector: string, handler: () => void | Promise<void>) {
  document.querySelector(selector)?.addEventListener('click', () => void handler());
}

function say(text: string) {
  if (status) status.textContent = text;
}

const message = (e: unknown) => (typeof e === 'string' ? e : e instanceof Error ? e.message : String(e));

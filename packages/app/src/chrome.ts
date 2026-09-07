// ── The window ─────────────────────────────────────────────────────────────
// Every window has the same anatomy, whether it is the one the program opened
// with or an editor torn off onto another monitor:
//
//   menu bar        File, and nothing that is not a file action
//   tree            the shape of the job (hidden in a torn-off window)
//   area            one editor, with its own header and its own tear-off
//   status bar      where the job is, and what just happened
//
// A torn-off editor is not a lesser window. It is the same anatomy with the
// tree folded away, which is why the button that tears an editor off lives on
// the editor rather than in the sidebar.

import { icon, type IconName } from './icons.js';
import { openHelp } from './help.js';

export interface AreaOptions {
  readonly title: string;
  /** Which editor this is, for the tear-off. Absent means it cannot be torn off. */
  readonly editor?: string;
  readonly help?: string;
  readonly onTearOff?: (editor: string) => void;
}

/**
 * An area: a titled region with one editor in it.
 *
 * The header carries the editor's name, a help affordance, and — on every area,
 * not on one of them — the button that pulls it into its own window.
 */
export function area(options: AreaOptions): { root: HTMLElement; body: HTMLElement } {
  const root = document.createElement('section');
  root.className = 'area';

  const header = document.createElement('header');
  header.className = 'area-header';

  const title = document.createElement('h2');
  title.textContent = options.title;
  header.append(title);

  const actions = document.createElement('div');
  actions.className = 'area-actions';

  if (options.help) {
    actions.append(iconButton('help', 'What this is for', () => openHelp(options.title, options.help!)));
  }
  if (options.editor && options.onTearOff) {
    actions.append(iconButton('tearOff', 'Open in its own window', () => options.onTearOff!(options.editor!)));
  }
  header.append(actions);

  const body = document.createElement('div');
  body.className = 'area-body';

  root.append(header, body);
  return { root, body };
}

export function iconButton(name: IconName, label: string, onClick: () => void): HTMLButtonElement {
  const button = document.createElement('button');
  button.type = 'button';
  button.className = 'icon-button';
  button.title = label;
  button.setAttribute('aria-label', label);
  button.append(icon(name));
  button.addEventListener('click', onClick);
  return button;
}

/** A toolbar button: an icon and its word, because a pictogram alone teaches nobody. */
export function toolButton(name: IconName, label: string, onClick: () => void): HTMLButtonElement {
  const button = document.createElement('button');
  button.type = 'button';
  button.className = 'tool';
  button.append(icon(name));
  const text = document.createElement('span');
  text.textContent = label;
  button.append(text);
  button.addEventListener('click', onClick);
  return button;
}

export interface MenuItem {
  readonly label: string;
  readonly onSelect: () => void;
  readonly disabled?: boolean;
}

/** The File menu. Opening, saving and recent jobs — nothing else lives here. */
export function menu(label: string, items: readonly MenuItem[]): HTMLElement {
  const wrap = document.createElement('div');
  wrap.className = 'menu';

  const button = document.createElement('button');
  button.type = 'button';
  button.className = 'menu-button';
  button.textContent = label;

  const list = document.createElement('div');
  list.className = 'menu-list';
  list.hidden = true;

  for (const item of items) {
    const entry = document.createElement('button');
    entry.type = 'button';
    entry.className = 'menu-item';
    entry.textContent = item.label;
    entry.disabled = item.disabled ?? false;
    entry.addEventListener('click', () => { list.hidden = true; item.onSelect(); });
    list.append(entry);
  }

  button.addEventListener('click', (e) => {
    e.stopPropagation();
    list.hidden = !list.hidden;
  });
  document.addEventListener('click', () => { list.hidden = true; });

  wrap.append(button, list);
  return wrap;
}

/** The status bar: where the job is, and what just happened. */
export function statusBar(): { root: HTMLElement; setPath(p: string): void; say(text: string): void } {
  const root = document.createElement('footer');
  root.className = 'status-bar';

  const message = document.createElement('span');
  message.className = 'status-message';

  const path = document.createElement('span');
  path.className = 'status-path';

  root.append(message, path);
  return {
    root,
    setPath: (p) => { path.textContent = p; path.title = p; },
    say: (text) => { message.textContent = text; },
  };
}

/**
 * The start screen — what a window shows before a job is open.
 *
 * Not an empty grid with a paragraph explaining that it is empty. Two things to
 * do, and the recent jobs, which is what somebody opening the program actually
 * wants.
 */
export function startScreen(options: {
  onOpen: () => void;
  recent: readonly { path: string; name: string }[];
  onOpenRecent: (path: string) => void;
}): HTMLElement {
  const root = document.createElement('div');
  root.className = 'start';

  const title = document.createElement('h1');
  title.textContent = 'roofnerd';

  const line = document.createElement('p');
  line.className = 'start-line';
  line.textContent = 'Trace the roof on one screen, watch the money on the other.';

  const open = document.createElement('button');
  open.type = 'button';
  open.className = 'start-open';
  open.append(icon('job', 18));
  const openText = document.createElement('span');
  openText.textContent = 'Open a job';
  open.append(openText);
  open.addEventListener('click', options.onOpen);

  root.append(title, line, open);

  if (options.recent.length) {
    const heading = document.createElement('h3');
    heading.textContent = 'Recent';
    const list = document.createElement('ul');
    list.className = 'start-recent';
    for (const job of options.recent) {
      const li = document.createElement('li');
      const button = document.createElement('button');
      button.type = 'button';
      button.textContent = job.name;
      button.title = job.path;
      button.addEventListener('click', () => options.onOpenRecent(job.path));
      li.append(button);
      list.append(li);
    }
    root.append(heading, list);
  }

  return root;
}

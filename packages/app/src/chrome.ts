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
  /** Every editor this area could show, for the picker at its top-left. */
  readonly editors?: readonly { readonly id: string; readonly title: string }[];
  readonly onSwitch?: (editor: string) => void;
}

/**
 * An area: a titled region with one editor in it, and the Properties panel
 * beside it.
 *
 * The header carries the editor's name, a help affordance, and — on every area,
 * not on one of them — the button that pulls it into its own window.
 *
 * The panel is built HERE, once, rather than by each editor, because A3 asks for
 * the same panel in every editor and the Plan was the only one that had it: an
 * estimator on the sheet, in the Library or in the Model could see what was
 * selected and not what it was. It rides with the area (D113), so a torn-off
 * editor gets its own — the same way a torn-off window gets its own status bar.
 */
export function area(options: AreaOptions): { root: HTMLElement; body: HTMLElement; panel: HTMLElement } {
  const root = document.createElement('section');
  root.className = 'area';

  const header = document.createElement('header');
  header.className = 'area-header';

  // An area is a frame that can show any editor, so which one it shows is a
  // choice made on the area — not a fact about the window it happens to be in.
  if (options.editors && options.editors.length > 1 && options.onSwitch) {
    const picker = document.createElement('select');
    picker.className = 'editor-picker';
    picker.setAttribute('aria-label', 'Which editor');
    for (const e of options.editors) {
      const option = document.createElement('option');
      option.value = e.id;
      option.textContent = e.title;
      picker.append(option);
    }
    picker.value = options.editor ?? options.editors[0]!.id;
    picker.addEventListener('change', () => options.onSwitch!(picker.value));
    header.append(picker);
  } else {
    const title = document.createElement('h2');
    title.textContent = options.title;
    header.append(title);
  }

  const actions = document.createElement('div');
  actions.className = 'area-actions';

  if (options.help) {
    actions.append(iconButton('help', 'What this is for', () => openHelp(options.title, options.help!)));
  }
  if (options.editor && options.onTearOff) {
    actions.append(iconButton('tearOff', 'Open in its own window', () => options.onTearOff!(options.editor!)));
  }
  header.append(actions);

  // The body is two columns: the editor, and the panel. The editor keeps the
  // flex column it has always mounted into — it is now a child of the body
  // rather than the body itself, so nothing inside an editor changes.
  const body = document.createElement('div');
  body.className = 'area-body';

  const host = document.createElement('div');
  host.className = 'editor-host';

  const panel = document.createElement('aside');
  panel.className = 'condition-panel';
  panel.setAttribute('aria-label', 'Properties');
  // A window opens on the start screen, where there is no job to have a
  // condition in. The window shows it when a job opens.
  panel.hidden = true;

  body.append(host, panel);
  root.append(header, body);
  return { root, body: host, panel };
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

/**
 * What an editor dispatches when it has something for the status bar.
 *
 * An editor does not hold the status bar — the window does, and a torn-off
 * editor has its own. So the editor says it where it stands and lets the
 * message rise to whichever window it is in, which is the same way the tree
 * tells the window it changed (`tree:changed`).
 */
export const STATUS_SAID = 'status:said';

/** Say something in the status bar of whatever window this element is in. */
export function sayInStatus(from: HTMLElement, text: string): void {
  from.dispatchEvent(new CustomEvent<string>(STATUS_SAID, { bubbles: true, detail: text }));
}

export interface StatusFacts {
  readonly job?: string;
  readonly scenario?: string;
  readonly scale?: string;
  readonly units?: string;
  readonly saved?: string;
}

/**
 * The status bar: the facts about the job that are true whatever editor is
 * open — its name, which price set is active, what the sheet is scaled at,
 * what units are in use, and whether the work is saved. Nothing else.
 */
export function statusBar(): {
  root: HTMLElement;
  setPath(p: string): void;
  setFacts(f: StatusFacts): void;
  say(text: string): void;
} {
  const root = document.createElement('footer');
  root.className = 'status-bar';

  const facts = document.createElement('span');
  facts.className = 'status-facts';

  const message = document.createElement('span');
  message.className = 'status-message';

  const path = document.createElement('span');
  path.className = 'status-path';
  // Two parts, because only one of them can be given up. The last folder of a
  // job's path is the job; everything in front of it is the machine it happens
  // to be on. So the tail holds its ground and the head is what shortens.
  const pathHead = document.createElement('span');
  pathHead.className = 'path-head';
  const pathTail = document.createElement('span');
  pathTail.className = 'path-tail';
  path.append(pathHead, pathTail);

  const held: StatusFacts & Record<string, string | undefined> = {};

  root.append(facts, message, path);

  return {
    root,
    setPath: (p) => {
      const cut = p.lastIndexOf('/');
      pathHead.textContent = cut > 0 ? p.slice(0, cut) : '';
      pathTail.textContent = cut > 0 ? p.slice(cut) : p;
      path.title = p;
    },
    // The whole of it on hover, the way the path beside it already does: a
    // message that names a file is longer than the bar and the estimator still
    // has to be able to read where the file went.
    say: (text) => { message.textContent = text; message.title = text; },
    setFacts: (f) => {
      // Merged, not replaced: saying "Saved" must not blank the job's name.
      Object.assign(held, f);
      const shown = held;
      facts.replaceChildren();
      const pairs: [string, string][] = [];
      if (shown.job) pairs.push(['Job', shown.job]);
      if (shown.scenario) pairs.push(['Scenario', shown.scenario]);
      if (shown.scale) pairs.push(['Scale', shown.scale]);
      if (shown.units) pairs.push(['Units', shown.units]);
      if (shown.saved) pairs.push(['', shown.saved]);
      for (const [i, [key, value]] of pairs.entries()) {
        if (i) {
          const sep = document.createElement('span');
          sep.className = 'status-sep';
          sep.textContent = '│';
          facts.append(sep);
        }
        const field = document.createElement('span');
        field.className = 'status-field';
        if (key) {
          const k = document.createElement('span');
          k.className = 'k';
          k.textContent = key;
          field.append(k);
        }
        const v = document.createElement('span');
        v.textContent = value;
        field.append(v);
        facts.append(field);
      }
    },
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
  /** The demo job, when this build can still find one. */
  demo?: { path: string } | null;
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

  // The demo job, when there is one. It is the product's first screen: somebody
  // opening this for the first time should see a roof, not a file browser. It
  // is also the one way into a job that does not go through an operating-system
  // dialog, which makes it the path a check can actually walk.
  if (options.demo) {
    const demo = document.createElement('button');
    demo.type = 'button';
    demo.className = 'start-demo';
    demo.append(icon('page', 16));
    const demoText = document.createElement('span');
    demoText.textContent = 'Open the demo job';
    demo.append(demoText);
    demo.title = options.demo.path;
    demo.addEventListener('click', () => options.onOpenRecent(options.demo!.path));
    root.append(demo);
  }

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

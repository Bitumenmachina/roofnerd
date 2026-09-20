// ── The window ─────────────────────────────────────────────────────────────
// Every window has the same anatomy, whether it is the one the program opened
// with or the cost panel pulled onto a second monitor:
//
//   document bar        44px  what file is open, the one roof-area figure
//                             everything reconciles to, who it is being shown
//                             to, help, and which theme the room is in.
//                             Never holds a tool.
//   tool settings strip 42px  everything the active tool needs and nothing
//                             else. It swaps wholesale with the tool.
//   body                      tool rail 52 · surface · cost panel 364
//   status strip        30px  what the roof does under the cursor, where the
//                             job is, and what just happened.
//
// The regions are fixed and only the surface flexes. That is the README's
// "Responsive behavior" and it is in the grid rather than in a comment: a
// track written as `30px` cannot be talked into 44 by its contents, which is
// how one long export message once made the status bar two lines high.
//
// What is NOT here any more: the tree, the editor picker and the area header.
// A bid does not need a second navigator when the conditions are in the cost
// panel's Condition tab and in the Plan's own list, and an area that can show
// any editor was the A3 anatomy — the handoff has one surface with two views
// switched in place and a panel beside it (D132, Addendum 7).

import { icon, type IconName } from './icons.js';
import { openHelp } from './help.js';
import { flipTheme, theme, watchTheme } from './theme.js';

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
    // The list is fixed-positioned so the bar's own overflow clip cannot hide
    // it (R1); it stands just under the button, wherever the button is.
    const box = button.getBoundingClientRect();
    list.style.top = `${Math.round(box.bottom + 2)}px`;
    list.style.left = `${Math.round(box.left)}px`;
    list.hidden = !list.hidden;
  });
  document.addEventListener('click', () => { list.hidden = true; });

  wrap.append(button, list);
  return wrap;
}

// ── the document bar ───────────────────────────────────────────────────────

/** A quantity and where it came from: derived off the model, or typed. */
export interface Figure {
  readonly value: string | null;
  readonly unit: string;
  /** What went into it. On the title, so it can be checked rather than trusted. */
  readonly basis: string;
  /**
   * What the figure could not count, in words, beside it.
   *
   * A total that leaves something out says what it left out — and says it on
   * screen, not only on a tooltip. An unscaled sheet has area on it that has
   * no square feet yet, and a roof-area figure that quietly skipped it would
   * be the silent zero this program is not allowed to have.
   */
  readonly leftOut?: string | null;
}

export interface LensChoice {
  readonly id: string | null;
  readonly name: string;
  readonly audience: string;
}

export function documentBar(options: {
  readonly menus: readonly HTMLElement[];
  readonly lenses: readonly LensChoice[];
  readonly onLens: (id: string | null) => void;
  readonly help: { title: string; body: string };
}): {
  root: HTMLElement;
  setJob(name: string | null): void;
  setRoofArea(figure: Figure): void;
  setLens(id: string | null): void;
  /** What the "?" opens. It follows whatever the surface is showing. */
  setHelp(title: string, body: string): void;
} {
  const root = document.createElement('header');
  root.className = 'doc-bar';

  const job = document.createElement('span');
  job.className = 'doc-job none';
  job.textContent = 'no file open';

  // The one roof-area figure. Everything on screen reconciles to it, so its
  // basis travels with it on the title rather than living in a document
  // somewhere: an estimator who cannot see what went into a number cannot use
  // it to check anything.
  const area = document.createElement('span');
  area.className = 'roof-area none';
  const areaLabel = document.createElement('span');
  areaLabel.className = 'label';
  areaLabel.textContent = 'Roof area';
  const areaValue = document.createElement('span');
  areaValue.className = 'value';
  areaValue.textContent = '—';
  const areaUnit = document.createElement('span');
  areaUnit.className = 'unit';
  const areaMark = document.createElement('span');
  areaMark.className = 'derived';
  areaMark.textContent = '◈';
  areaMark.hidden = true;
  const areaLeftOut = document.createElement('span');
  areaLeftOut.className = 'left-out';
  areaLeftOut.hidden = true;
  area.append(areaLabel, areaValue, areaUnit, areaMark, areaLeftOut);

  const spacer = document.createElement('span');
  spacer.className = 'doc-spacer';

  // ── who is looking ──────────────────────────────────────────────────────
  const selector = document.createElement('div');
  selector.className = 'lens-selector';

  const button = document.createElement('button');
  button.type = 'button';
  button.className = 'lens-button';
  const showing = document.createElement('span');
  showing.className = 'label';
  showing.textContent = 'Showing';
  const shown = document.createElement('span');
  shown.className = 'value';
  shown.textContent = options.lenses[0]?.name ?? 'Full estimate';
  button.append(showing, shown);

  const list = document.createElement('div');
  list.className = 'lens-menu';
  list.hidden = true;
  const head = document.createElement('p');
  head.className = 'lens-menu-head';
  head.textContent = 'WHO IS LOOKING';
  list.append(head);

  let chosen: string | null = null;
  const rows: { choice: LensChoice; row: HTMLButtonElement }[] = [];
  for (const choice of options.lenses) {
    const row = document.createElement('button');
    row.type = 'button';
    row.className = 'lens';
    const name = document.createElement('strong');
    name.textContent = choice.name;
    const who = document.createElement('span');
    who.textContent = choice.audience;
    row.append(name, who);
    row.addEventListener('click', () => {
      list.hidden = true;
      setLens(choice.id);
      options.onLens(choice.id);
    });
    list.append(row);
    rows.push({ choice, row });
  }
  const foot = document.createElement('p');
  foot.className = 'lens-menu-foot';
  foot.textContent = 'A lens hides lines and columns. It never edits the estimate.';
  list.append(foot);

  button.addEventListener('click', (e) => {
    e.stopPropagation();
    // The list is fixed-positioned so the bar's own overflow clip cannot hide
    // it (R1); it stands just under the button, wherever the button is.
    const box = button.getBoundingClientRect();
    list.style.top = `${Math.round(box.bottom + 2)}px`;
    list.style.left = `${Math.round(box.left)}px`;
    list.hidden = !list.hidden;
  });
  document.addEventListener('click', () => { list.hidden = true; });
  selector.append(button, list);

  function setLens(id: string | null): void {
    chosen = id;
    for (const { choice, row } of rows) row.classList.toggle('on', choice.id === chosen);
    const match = rows.find((r) => r.choice.id === chosen);
    shown.textContent = match?.choice.name ?? 'Full estimate';
    button.classList.toggle('on', chosen !== null);
  }

  let helping = options.help;
  const help = iconButton('help', 'What this is for',
    () => openHelp(helping.title, helping.body));

  // Light is the default and stays it. The toggle says which room you are
  // going to, not which one you are in — a button labelled with the state it
  // is already in is a button nobody can read.
  const themeToggle = document.createElement('button');
  themeToggle.type = 'button';
  themeToggle.className = 'theme-toggle';
  themeToggle.addEventListener('click', () => { flipTheme(); });
  watchTheme((now) => {
    const going = now === 'dark' ? 'a light room' : 'a dark room';
    themeToggle.replaceChildren(icon(now === 'dark' ? 'sun' : 'moon'));
    themeToggle.title = `Work in ${going}`;
    themeToggle.setAttribute('aria-label', `Work in ${going}`);
  });

  const divider = document.createElement('span');
  divider.className = 'doc-divider';

  root.append(...options.menus, job, divider, area, spacer, selector, help, themeToggle);
  setLens(null);

  return {
    root,
    setJob: (name) => {
      job.textContent = name ?? 'no file open';
      job.classList.toggle('none', name === null);
      job.title = name ?? '';
    },
    setRoofArea: (figure) => {
      area.classList.toggle('none', figure.value === null);
      areaValue.textContent = figure.value ?? '—';
      areaUnit.textContent = figure.value === null ? '' : figure.unit;
      areaMark.hidden = figure.value === null;
      areaLeftOut.textContent = figure.leftOut ?? '';
      areaLeftOut.hidden = !figure.leftOut;
      area.title = figure.basis;
    },
    setLens,
    setHelp: (title, body) => { helping = { title, body }; },
  };
}

// ── the tool settings strip ────────────────────────────────────────────────

/** What the strip says when no tool has been picked: nothing, in words. */
const STRIP_IDLE = 'Tool settings appear here once a tool is active';

export function toolStrip(): {
  root: HTMLElement;
  show(name: string, fields: readonly Node[]): void;
  idle(): void;
} {
  const root = document.createElement('div');
  root.className = 'tool-strip';

  const name = document.createElement('span');
  name.className = 'tool-strip-name';
  const fields = document.createElement('div');
  fields.className = 'tool-strip-fields';

  const idle = document.createElement('span');
  idle.className = 'tool-strip-idle';
  idle.textContent = STRIP_IDLE;

  root.append(idle);

  return {
    root,
    // Wholesale: the strip is emptied and refilled, never patched. Half of one
    // tool's settings beside half of another's is worse than none.
    show: (label, contents) => {
      name.textContent = label;
      fields.replaceChildren(...contents);
      root.replaceChildren(name, fields);
    },
    idle: () => { root.replaceChildren(idle); },
  };
}

/** One reading in the strip: what it is, what it says, and why. */
export function stripField(label: string, value: string, why?: string, mono = false): HTMLElement {
  const wrap = document.createElement('span');
  wrap.className = 'strip-field';
  if (label) {
    const k = document.createElement('span');
    k.textContent = label;
    wrap.append(k);
  }
  const v = document.createElement('span');
  v.className = mono ? 'v mono' : 'v';
  v.textContent = value;
  wrap.append(v);
  if (why) {
    const w = document.createElement('span');
    w.className = 'why';
    w.textContent = why;
    wrap.append(w);
  }
  return wrap;
}

/** A reading the model worked out, with the cyan mark and its basis. */
export function derivedField(label: string, value: string, basis: string): HTMLElement {
  const wrap = stripField(label, value, undefined, true);
  const mark = document.createElement('span');
  mark.className = 'derived';
  mark.textContent = '◈';
  wrap.append(mark);
  wrap.title = basis;
  return wrap;
}

export function stripButton(label: string, onClick: () => void, quiet = false): HTMLButtonElement {
  const b = document.createElement('button');
  b.type = 'button';
  if (quiet) b.className = 'quiet';
  b.textContent = label;
  b.addEventListener('click', onClick);
  return b;
}

// ── the tool rail ──────────────────────────────────────────────────────────

/**
 * What a view in the surface is handed so it can fill the chrome around it.
 *
 * A view does not own the rail or the strip — the window does, because they are
 * regions of the window and the handoff says so. But the acts in the rail and
 * the settings in the strip belong to whatever is in the surface, so the view
 * is given the handles and puts its own tools in them. Switching views swaps
 * the contents of both, wholesale.
 */
export interface SurfaceChrome {
  /** Put this view's acts in the rail. */
  setTools(tools: readonly RailTool[]): void;
  /** Which one is up. Null for none. */
  setActiveTool(id: string | null): void;
  /** Everything the active tool needs, and nothing else. */
  strip: { show(name: string, fields: readonly Node[]): void; idle(): void };
  /**
   * Getting around the view: zoom, fit, and later the Model's preset corners.
   *
   * Upper right, under the view switch, which is where the pattern lineage the
   * handoff names puts view navigation. It used to be a pinned group at the end
   * of the Plan's own toolbar row, which is what §1.5 was about: a sticky item
   * reserves no room, so it painted over whatever the row scrolled under it.
   * An overlay on the drawing cannot cover a control, because there is no row
   * for it to be in.
   */
  setViewTools(controls: readonly Node[]): void;
  /** "Measuring into: <condition>", on the drawing, or nothing. */
  setMeasuringInto(into: string | null): void;
  /** What the roof is doing under the cursor, for the status strip. */
  setReadout(readings: readonly Reading[]): void;
}

export interface RailTool {
  readonly id: string;
  /** The act, in a roofer's words. It is the hover label and the whole point. */
  readonly label: string;
  readonly icon: IconName;
  readonly onPick: () => void;
}

export function toolRail(): {
  root: HTMLElement;
  setTools(tools: readonly RailTool[]): void;
  active(id: string | null): void;
  quiet(on: boolean): void;
} {
  const root = document.createElement('nav');
  root.className = 'tool-rail';
  root.setAttribute('aria-label', 'Tools');

  let buttons = new Map<string, HTMLButtonElement>();

  return {
    root,
    // Wholesale, when the surface changes what it is showing: the acts belong
    // to the view, and half of one view's acts beside half of another's would
    // be a rail offering something that cannot be done.
    setTools: (tools) => {
      buttons = new Map();
      root.replaceChildren();
      for (const tool of tools) {
        const button = document.createElement('button');
        button.type = 'button';
        button.className = 'rail-tool';
        button.setAttribute('aria-label', tool.label);
        button.title = tool.label;
        button.append(icon(tool.icon, 20));
        const chip = document.createElement('span');
        chip.className = 'rail-label';
        chip.textContent = tool.label;
        button.append(chip);
        button.addEventListener('click', tool.onPick);
        root.append(button);
        buttons.set(tool.id, button);
      }
    },
    active: (id) => { for (const [name, b] of buttons) b.classList.toggle('on', name === id); },
    quiet: (on) => { root.classList.toggle('quiet', on); },
  };
}

// ── the status strip ───────────────────────────────────────────────────────

/**
 * What an editor dispatches when it has something for the status strip.
 *
 * An editor does not hold the strip — the window does, and the detached cost
 * panel has its own. So the editor says it where it stands and lets the
 * message rise to whichever window it is in.
 */
export const STATUS_SAID = 'status:said';

/** Say something in the status strip of whatever window this element is in. */
export function sayInStatus(from: HTMLElement, text: string): void {
  from.dispatchEvent(new CustomEvent<string>(STATUS_SAID, { bubbles: true, detail: text }));
}

/** One field of the cursor readout. An em dash is a real answer: it says nothing is known. */
export interface Reading {
  readonly key: string;
  readonly value: string | null;
  /** Water — where it goes and how far. The one loud colour, spent on drainage. */
  readonly water?: boolean;
  /** A flat area with no fall. The only other colour there is, and only here. */
  readonly warn?: boolean;
}

export function statusStrip(): {
  root: HTMLElement;
  setReadout(readings: readonly Reading[]): void;
  setPath(p: string): void;
  say(text: string): void;
  /** A standing fact about the window, beside the message rather than instead of it. */
  setAside(text: string): void;
} {
  const root = document.createElement('footer');
  root.className = 'status-strip';

  const readout = document.createElement('div');
  readout.className = 'readout';

  const spacer = document.createElement('span');
  spacer.className = 'status-spacer';

  const message = document.createElement('span');
  message.className = 'status-message';

  // Standing facts about the window itself — where the cost panel is, today.
  // Separate from the message so that saying where a file went does not wipe
  // out the reason half the window is missing, and the other way round.
  const aside = document.createElement('span');
  aside.className = 'status-aside';
  aside.hidden = true;

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

  root.append(readout, spacer, message, aside, path);

  return {
    root,
    setReadout: (readings) => {
      readout.replaceChildren();
      for (const reading of readings) {
        const field = document.createElement('span');
        field.className = 'readout-field';
        if (reading.water) field.classList.add('water');
        if (reading.warn) field.classList.add('warn');
        if (reading.value === null) field.classList.add('none');
        if (reading.key) {
          const k = document.createElement('span');
          k.className = 'k';
          k.textContent = reading.key;
          field.append(k);
        }
        const v = document.createElement('span');
        v.className = 'v';
        v.textContent = reading.value ?? '—';
        field.append(v);
        readout.append(field);
      }
    },
    setPath: (p) => {
      const cut = p.lastIndexOf('/');
      pathHead.textContent = cut > 0 ? p.slice(0, cut) : '';
      pathTail.textContent = cut > 0 ? p.slice(cut) : p;
      path.title = p;
    },
    // The whole of it on hover, the way the path beside it already does: a
    // message that names a file is longer than the strip and the estimator
    // still has to be able to read where the file went.
    say: (text) => { message.textContent = text; message.title = text; },
    setAside: (text) => { aside.textContent = text; aside.hidden = text === ''; },
  };
}

/**
 * The start screen — what a window shows before a job is open.
 *
 * Not an empty grid with a paragraph explaining that it is empty. Two things to
 * do, and the recent jobs, which is what somebody opening the program actually
 * wants. It sits in the surface, which is where the handoff's first-open frame
 * puts it, with the chrome present and quiet around it.
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

/** The theme this window is in, for anything that has to paint outside CSS. */
export { theme };

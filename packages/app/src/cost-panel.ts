// ── The cost panel ─────────────────────────────────────────────────────────
// Geometry on one side, money on the other, both visible at once. That is the
// first rule the handoff gives and it is the whole point of the program: the
// estimator clicks a condition and watches the bid move.
//
//   Estimate   the lines, compact, one opening in place to its whole chain
//   Condition  what is traced, and everything the drawing cannot say about the
//              one that is picked — plus the book, when an assembly is wanted
//   Recap      where the money actually sits, by class, with what it left out
//
// The running totals are pinned under all three and never scroll away. Beside
// the tabs is the button that pulls the whole panel into a window of its own,
// 600 by 860, over the same document — a second window, not a copy.
//
// The panel sets its own width: 364 docked, 420 with a line open or the book
// out, and what a lens needs when a lens is showing. It is a width and never a
// flex — only the surface flexes (README, "Responsive behavior").

import {
  CLASS_NAMES, conceals, lensById, recap as recapOf,
  type Lens,
} from '@roofnerd/engine';
import { at, doc, subscribe, type Doc } from './doc.js';
import { iconButton } from './chrome.js';
import { renderConditionList } from './condition-list.js';
import { renderConditionPanel } from './editors/condition-panel.js';
import { closeOpenLine, mountEstimate, unmountEstimate } from './editors/estimate.js';
import { mountLibrary } from './editors/library.js';
import { mountLens } from './editors/reports.js';
import { money, plural } from './labels.js';
import { selectedConditionId, watchSelection } from './selection.js';

export type Tab = 'estimate' | 'condition' | 'recap';

const TABS: readonly { id: Tab; title: string }[] = [
  { id: 'estimate', title: 'Estimate' },
  { id: 'condition', title: 'Condition' },
  { id: 'recap', title: 'Recap' },
];

/** Docked. The handoff's number, and it holds the compact line. */
const DOCKED = 364;
/** With a line open to its chain, or the book out. The handoff's number. */
const WIDE = 420;
/**
 * What a lens needs, from the columns it actually has.
 *
 * Measured rather than guessed: each column is given the width its heading and
 * its figures need at this stylesheet's sizes, and the panel takes the sum. The
 * internal lens has nine columns and comes to more than the window can spare
 * beside a drawing, so it is capped and its table scrolls inside the panel —
 * which is the honest answer, and it is said here rather than found later.
 */
const LENS_COLUMN: Readonly<Record<string, number>> = {
  class: 120, condition: 130, item: 150, costCode: 96, formula: 160,
  quantity: 84, unit: 48, waste: 62, orderQuantity: 84, orderUnit: 80,
  cost: 92, perSquare: 96, sent: 60, returned: 74, notes: 140,
};
const LENS_CAP = 620;

const lensWidth = (lens: Lens): number => Math.min(LENS_CAP, Math.max(DOCKED,
  24 + lens.columns.reduce((sum, c) => sum + (LENS_COLUMN[c.key] ?? 90), 0)));

export interface CostPanel {
  readonly root: HTMLElement;
  /** Which tab is up. */
  setTab(tab: Tab): void;
  /** Which lens the document bar has chosen, or none. */
  setLens(id: string | null): void;
  /** One rung out: the book, then an open line, then the lens. */
  backOut(): boolean;
  /** What the panel is showing, for the window's one help affordance. */
  showing(): string;
  /** Put it down. */
  unmount(): void;
}

export function costPanel(options: {
  /** True in the window that is only the panel. */
  readonly detached: boolean;
  /** Docked: pull it into its own window. Detached: bring it home. */
  readonly onDetach?: () => void;
  readonly onDock?: () => void;
  readonly onLensCleared?: () => void;
  /** Told when the panel changes what it is showing, so the "?" follows it. */
  readonly onShowingChanged?: () => void;
}): CostPanel {
  const root = document.createElement('aside');
  root.className = 'cost-panel';
  root.setAttribute('aria-label', 'Cost');

  let tab: Tab = 'estimate';
  let lens: string | null = null;
  let conditionHosts: { list: HTMLElement; properties: HTMLElement; load: HTMLButtonElement } | null = null;
  let titleName: HTMLElement | null = null;
  let book = false;
  let lineOpen = false;
  let mountedTab: string | null = null;

  // ── the tabs, and the button that detaches the whole panel ──────────────
  const tabs = document.createElement('div');
  tabs.className = 'cost-tabs';
  const tabButtons = new Map<Tab, HTMLButtonElement>();
  for (const t of TABS) {
    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'cost-tab';
    button.textContent = t.title;
    button.addEventListener('click', () => setTab(t.id));
    tabs.append(button);
    tabButtons.set(t.id, button);
  }
  if (!options.detached) {
    tabs.append(iconButton('tearOff', 'Open the cost panel in its own window',
      () => options.onDetach?.()));
  }

  const body = document.createElement('div');
  body.className = 'cost-body';

  // ── the running totals, pinned ──────────────────────────────────────────
  const totals = document.createElement('div');
  totals.className = 'cost-totals';

  root.append(tabs, body, totals);

  // The detached window's own title bar — the handoff draws one, and a window
  // that is only a panel has to say which job's money it is showing and how to
  // send it home.
  if (options.detached) {
    const bar = document.createElement('div');
    bar.className = 'cost-title';
    const name = document.createElement('span');
    name.className = 'cost-title-name';
    const home = document.createElement('button');
    home.type = 'button';
    home.className = 'cost-dock';
    home.textContent = 'dock it back';
    home.addEventListener('click', () => options.onDock?.());
    bar.append(name, home);
    root.prepend(bar);
    root.classList.add('detached');
    titleName = name;
  }

  function width(): number {
    if (options.detached) return 0;
    if (lens) {
      const chosen = lensById(lens);
      if (chosen) return lensWidth(chosen);
    }
    return lineOpen || book ? WIDE : DOCKED;
  }

  function applyWidth(): void {
    if (options.detached) return;
    root.style.setProperty('--panel-w', `${width()}px`);
  }

  function setTab(next: Tab): void {
    tab = next;
    book = false;
    options.onShowingChanged?.();
    draw();
  }

  function setLens(id: string | null): void {
    lens = id;
    mountedTab = null;
    if (id) tab = 'estimate';
    draw();
  }

  /**
   * One rung out.
   *
   * The book, then a line open to its chain, then the lens. Escape has meant
   * "back out" in VES for years and an estimator reaches for it; each rung is
   * a state the estimator put the panel into, in the order they did it.
   */
  function backOut(): boolean {
    if (book) { book = false; draw(); return true; }
    if (closeOpenLine()) { lineOpen = false; applyWidth(); draw(); return true; }
    if (lens !== null) { lens = null; options.onLensCleared?.(); setLens(null); return true; }
    return false;
  }

  function draw(): void {
    const d = doc();
    for (const [id, button] of tabButtons) button.classList.toggle('on', id === tab);
    const chosen = lens ? lensById(lens) : undefined;
    tabButtons.get('estimate')!.textContent = chosen ? chosen.name : 'Estimate';
    tabs.classList.toggle('lens-on', lens !== null);

    const key = `${tab}:${lens ?? ''}:${book ? 'book' : ''}`;
    if (mountedTab !== key) {
      mountedTab = key;
      unmountEstimate();
      body.replaceChildren();
      const host = document.createElement('div');
      body.append(host);
      if (tab === 'estimate' && lens) {
        mountLens(host, lens);
      } else if (tab === 'estimate') {
        mountEstimate(host, {
          onOpenLine: (open) => { lineOpen = open; applyWidth(); },
        });
      } else if (tab === 'recap') {
        mountLens(host, 'recap');
      } else if (book) {
        options.onShowingChanged?.();
        host.className = 'cost-book';
        const back = document.createElement('button');
        back.type = 'button';
        back.className = 'cost-back';
        back.textContent = 'Back to the condition';
        back.addEventListener('click', () => { book = false; draw(); });
        const shelf = document.createElement('div');
        host.append(back, shelf);
        mountLibrary(shelf);
      } else {
        options.onShowingChanged?.();
        host.className = 'cost-condition';
        const list = document.createElement('div');
        list.className = 'condition-list';
        const properties = document.createElement('aside');
        properties.className = 'condition-panel';
        properties.setAttribute('aria-label', 'Properties');
        const load = document.createElement('button');
        load.type = 'button';
        load.className = 'cost-load';
        load.textContent = 'Load an assembly';
        load.addEventListener('click', () => { book = true; draw(); });
        host.append(list, properties, load);
        conditionHosts = { list, properties, load };
      }
    }
    if (tab === 'condition' && !book && conditionHosts) {
      renderConditionList(conditionHosts.list, d);
      renderConditionPanel(conditionHosts.properties, selectedConditionId(), d, () => draw());
      conditionHosts.load.disabled = selectedConditionId() === null;
      conditionHosts.load.title = selectedConditionId() === null
        ? 'Pick a condition first — an assembly is loaded onto one'
        : 'Load a saved set of items onto the condition that is picked';
    }
    // Chrome present and quiet before a job is open — the handoff's first-open
    // frame. Not hidden: a panel that vanishes makes the window look like it
    // lost something, and a panel standing there blank makes it look broken.
    const name = at('/job/name', d);
    root.classList.toggle('quiet', !(typeof name === 'string' && name));
    drawTotals(totals, d, lens);
    if (titleName) {
      const name = at('/job/name', d);
      titleName.textContent = typeof name === 'string' && name ? `Cost — ${name}` : 'Cost';
    }
    applyWidth();
  }

  const stopSubscribing = subscribe(() => draw());
  const stopWatching = watchSelection(() => {
    // A condition picked anywhere opens it on the Condition tab, which is the
    // handoff's rule about selection: clicking a condition in any view opens it
    // where its properties are. It does not steal the tab from a lens.
    if (tab === 'condition' || lens) { draw(); return; }
    draw();
  });

  applyWidth();

  return {
    root,
    setTab,
    setLens,
    backOut,
    showing: () => {
      if (book) return 'Library';
      if (lens) return 'Lens';
      return tab === 'estimate' ? 'Estimate' : tab === 'recap' ? 'Recap' : 'Condition';
    },
    unmount: () => {
      stopSubscribing();
      stopWatching();
      unmountEstimate();
    },
  };
}

// ── the pinned totals ──────────────────────────────────────────────────────

/**
 * Material, Labor and the selling price, pinned under whatever tab is up.
 *
 * Straight off `recap()` — the same arithmetic the Recap tab renders and the
 * sheet's footer used to. Nothing here adds anything up on its own: a panel
 * figure that differs from the recap by a cent is worse than no figure at all,
 * because both look right and one of them is not.
 *
 * A lens that conceals cost gets no money here either. The estimate is not
 * hidden, it is absent: the numbers are never written into the page, which is
 * what "removed rather than hidden" has to mean to be worth anything.
 */
function drawTotals(host: HTMLElement, d: Doc, lensId: string | null): void {
  host.replaceChildren();
  const lens = lensId ? lensById(lensId) : undefined;
  const rolled = recapOf_(d);

  if (lens && conceals(lens, 'cost')) {
    const row = document.createElement('div');
    row.className = 'selling';
    const label = document.createElement('span');
    label.className = 'label';
    label.textContent = 'Selling price';
    const value = document.createElement('span');
    value.className = 'value none';
    value.textContent = 'hidden by this lens';
    row.append(label, value);
    host.append(row);
    return;
  }

  if (!rolled) {
    const row = document.createElement('div');
    row.className = 'selling';
    const label = document.createElement('span');
    label.className = 'label';
    label.textContent = 'Selling price';
    const value = document.createElement('span');
    value.className = 'value none';
    value.textContent = 'nothing priced yet';
    row.append(label, value);
    host.append(row);
    return;
  }

  // Material and Labor, and then the selling price. Two headline classes,
  // because the panel is 364px wide and six of them wrap to three rows in it —
  // a pinned total that is a third of the panel high is not pinned, it is a
  // wall. The rest are on the Recap tab, whole, and this says so rather than
  // leaving the estimator to wonder what the other four came to.
  const HEADLINE: readonly string[] = ['Material', 'Labor'];
  const strip = document.createElement('div');
  strip.className = 'class-strip';
  for (const name of CLASS_NAMES) {
    if (!HEADLINE.includes(name)) continue;
    const line = rolled.classes.find((c) => c.class === name);
    if (!line || line.total === 0) continue;
    const cell = document.createElement('div');
    cell.className = 'class-cell';
    const label = document.createElement('span');
    label.className = 'label';
    label.textContent = name;
    const value = document.createElement('span');
    value.className = 'value';
    value.textContent = money(line.total);
    cell.append(label, value);
    strip.append(cell);
  }
  const others = rolled.classes.filter((c) => !HEADLINE.includes(c.class) && c.total !== 0);
  if (others.length) {
    const rest = document.createElement('div');
    rest.className = 'class-cell rest';
    const label = document.createElement('span');
    label.className = 'label';
    label.textContent = others.map((c) => c.class).join(' · ');
    const value = document.createElement('span');
    value.className = 'value';
    value.textContent = money(others.reduce((sum, c) => sum + c.total, 0));
    value.title = 'By class, whole, on the Recap tab.';
    rest.append(label, value);
    strip.append(rest);
  }
  if (strip.children.length) host.append(strip);

  const selling = document.createElement('div');
  selling.className = 'selling';
  const label = document.createElement('span');
  label.className = 'label';
  label.textContent = 'Selling price';
  const value = document.createElement('span');
  value.className = 'value';
  value.textContent = money(rolled.sellingPrice);
  selling.append(label, value);
  if (rolled.pending.length) {
    const partial = document.createElement('span');
    partial.className = 'partial';
    partial.textContent = `${plural(rolled.pending.length, 'line')} not counted`;
    partial.title = rolled.pending.join('\n');
    selling.append(partial);
  }
  host.append(selling);
}

/**
 * The recap, when the job has the cost codes to roll one up.
 *
 * Without codes there are no classes, so the panel says so rather than
 * pretending to a structure the job has not got yet.
 */
function recapOf_(d: Doc) {
  const job = at('/job', d) as { scenarios?: unknown[]; activeScenarioId?: string } | undefined;
  const codes = (at('/costCodes', d) as unknown[]) ?? [];
  if (!job?.scenarios?.length || codes.length === 0) return null;
  try {
    const document_ = {
      job, pages: at('/pages', d) ?? [], conditions: at('/conditions', d) ?? [], costCodes: codes,
    } as never;
    const scenario = (job.scenarios as { id: string }[])
      .find((s) => s.id === job.activeScenarioId) ?? job.scenarios[0];
    return recapOf(document_, scenario as never);
  } catch {
    return null;
  }
}

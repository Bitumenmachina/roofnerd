// ── The Estimate tab ───────────────────────────────────────────────────────
// The lines, in the compact form the handoff draws: what it is, how much of
// it, in what unit, and what it costs. Four columns, one line per row, grouped
// under the condition they come off — because the panel is 364px wide and an
// estimator glancing at it while clicking geometry is reading the money move,
// not auditing the chain.
//
// The chain is still there. A line opens IN PLACE to the whole of it — the
// formula editable on the line, every step from the traced measure to the
// money, and the arithmetic written out — and the panel widens to 420 while it
// is open. That is the handoff's Frame 5 and it is the point of the program: a
// library you cannot see into is a library nobody trusts, and every estimator
// has been burned by a number they could not check.
//
// Nothing about the arithmetic changed. `priceLine` prices a line, `priceJob`
// and `recap` roll it up, and every field writes to the one document by the
// same pointer it always did.

import {
  BASIS_WORDS, classOf, linesFrom, measure, priceLine, PRODUCTION_PER, PRODUCTION_PER_WORDS,
  scopeFor,
  type Condition, type CostCode, type Item, type Measures, type Page, type ProductionPer,
} from '@roofnerd/engine';
import { at, derivedFor, doc, set, subscribe, type Doc } from '../doc.js';
import { cutOnScreen, watchCut } from '../cut.js';
import { heldCaret, named, restoreCaret } from '../caret.js';
import { icon } from '../icons.js';
import { PENDING_REASON, money, plural, quantity } from '../labels.js';
import { select, selectedConditionId, selectedItemId, watchSelection } from '../selection.js';

const UNITS = ['SF', 'LF', 'EA', 'SQ'];

/** Which line is open to its whole chain. One at a time: the panel is 420 wide. */
let openLineId: string | null = null;

let stopSubscribing: (() => void) | null = null;
let stopWatching: (() => void) | null = null;
let stopWatchingCut: (() => void) | null = null;
let widthChanged: (open: boolean) => void = () => undefined;
/**
 * Draw the lines again.
 *
 * Opening a line also selects it, so the selection watcher redrew and the chain
 * appeared; CLOSING one changes no selection and changed no document, so
 * Escape set the state and nothing put it on screen. The state and the drawing
 * are the same act and they go together here.
 */
let redraw: () => void = () => undefined;

export interface EstimateOptions {
  /** Told when a line opens or closes, because the panel's width follows it. */
  readonly onOpenLine: (open: boolean) => void;
}

export function mountEstimate(host: HTMLElement, options: EstimateOptions): void {
  unmountEstimate();
  host.replaceChildren();
  host.className = 'cost-estimate';
  widthChanged = options.onOpenLine;

  const head = document.createElement('div');
  head.className = 'cost-head';

  const columns = document.createElement('div');
  columns.className = 'cost-columns';
  for (const [label, cls] of [['ITEM', 'c-what'], ['QTY', 'c-qty'], ['', 'c-unit'], ['COST', 'c-money']] as const) {
    const cell = document.createElement('span');
    cell.className = cls;
    cell.textContent = label;
    columns.append(cell);
  }

  const lines = document.createElement('div');
  lines.className = 'cost-lines';

  // What is below the fold, counted rather than claimed. The handoff's frame
  // reads "14 more lines below"; a figure like that has to be the lines that
  // are actually not on screen, or it is a number that lies in the one place
  // an estimator would trust it.
  const more = document.createElement('p');
  more.className = 'cost-more';
  more.hidden = true;

  const countBelow = () => {
    const rows = Array.from(lines.querySelectorAll<HTMLElement>('.cost-line'));
    const bottom = lines.getBoundingClientRect().bottom;
    const hidden = rows.filter((r) => r.getBoundingClientRect().top >= bottom - 1).length;
    more.textContent = hidden ? `${plural(hidden, 'more line')} below` : '';
    more.hidden = hidden === 0;
  };
  lines.addEventListener('scroll', countBelow);

  host.append(head, columns, lines, more);

  const draw = (d: Doc) => {
    render(head, lines, d);
    // After layout, because it is a question about where the rows actually are.
    requestAnimationFrame(countBelow);
  };

  redraw = () => draw(doc());
  stopSubscribing = subscribe((d: Doc) => draw(d));
  stopWatching = watchSelection(() => draw(doc()));
  // And the cut on the surface beside it, when there is one. A section is a
  // reading of the same job the lines are, and the handoff's Frame 3 says so at
  // the top of this panel: these are the lines the plane goes through.
  stopWatchingCut = watchCut(() => draw(doc()));
}

export function unmountEstimate(): void {
  stopSubscribing?.();
  stopWatching?.();
  stopWatchingCut?.();
  stopSubscribing = null;
  stopWatching = null;
  stopWatchingCut = null;
  redraw = () => undefined;
}

/** Close whatever line is open. Answers whether there was one — Escape asks. */
export function closeOpenLine(): boolean {
  if (openLineId === null) return false;
  openLine(null);
  return true;
}

function openLine(id: string | null): void {
  openLineId = id;
  widthChanged(id !== null);
  redraw();
  if (id === null) return;
  // The chain is taller than the panel on a line with a rate on it, so the two
  // buttons at the end of it can fall below the fold — and in the torn-off
  // window the one that takes a line off was landing under the footnote that
  // counts the lines below, where nothing can click it. A button that cannot be
  // reached is a button that does not exist.
  //
  // The smallest scroll that brings them in, after the panel has been rebuilt
  // and re-measured. Nothing moves at all when the whole chain already fits, and
  // this happens on OPENING only — a redraw while somebody is typing a rate into
  // the chain must not pull the view out from under them.
  requestAnimationFrame(() => {
    document.querySelector('.line-chain .chain-actions')
      ?.scrollIntoView({ block: 'nearest' });
  });
}

// ── the lines ──────────────────────────────────────────────────────────────

function render(head: HTMLElement, host: HTMLElement, d: Doc): void {
  // Which field is being typed into, before it is destroyed — see caret.ts.
  // The panel is rebuilt whole on every change to the job, which is how a line
  // typed here moves the total in the other window.
  const caret = heldCaret(host);

  const conditions = (at('/conditions', d) as Condition[]) ?? [];
  const pages = (at('/pages', d) as Page[]) ?? [];
  const costCodes = (at('/costCodes', d) as CostCode[]) ?? [];
  const scenario = activeScenario(d);

  const calibrations: Record<string, { feetPerUnit: number } | undefined> = {};
  for (const p of pages) if (p.feetPerUnit) calibrations[p.id] = { feetPerUnit: p.feetPerUnit };

  const measured = new Map<string, Measures>();
  for (const c of conditions) {
    const source = c.from ? conditions.find((x) => x.id === c.from) ?? c : c;
    measured.set(c.id, measure(source.kind, source.traces ?? [], source.properties ?? {}, calibrations));
  }

  // ── the header: what this estimate is of, and which price set it is under ──
  // Not the file's name — the document bar carries that, and printing it again
  // three inches below is furniture. What belongs here is what the estimate is
  // OF: how much roof, which is the figure everything reconciles to.
  head.replaceChildren();
  const name = at('/job/name', d);
  const what = document.createElement('span');
  what.className = 'cost-head-what';
  let roof = 0;
  let roofKnown = false;
  for (const c of conditions) {
    if (c.kind !== 'area') continue;
    const m = measured.get(c.id);
    if (!m || m.SF === null) continue;
    roof += m.SF;
    roofKnown = true;
  }
  what.textContent = roofKnown ? `${quantity(roof)} SF of roof` : 'nothing traced yet';

  // The price set this bid is worked out under. It is a price set, so it lives
  // with the prices — the status bar used to carry it as a fact about the job,
  // which it is not: it is a choice about this estimate.
  const scenarios = (at('/job/scenarios', d) as { id: string; name: string }[]) ?? [];
  const bar = document.createElement('span');
  bar.className = 'scenario-bar';
  if (scenarios.length > 1) {
    const label = document.createElement('span');
    label.textContent = 'Scenario';
    const pick = document.createElement('select');
    pick.setAttribute('aria-label', 'Which price set');
    for (const s of scenarios) {
      const o = document.createElement('option');
      o.value = s.id;
      o.textContent = s.name;
      pick.append(o);
    }
    pick.value = scenario?.id ?? scenarios[0]!.id;
    pick.addEventListener('change', () => void set('/job/activeScenarioId', pick.value));
    bar.append(label, named(pick, '/job/activeScenarioId'));
  } else if (scenarios.length === 1) {
    // A picker with one choice is furniture; the name of the set is not.
    bar.append(document.createTextNode(`Scenario ${scenarios[0]!.name}`));
  }

  const count = document.createElement('span');
  count.className = 'cost-head-count';
  // Filled in once the rows are drawn (R8): an item that stands for one line
  // per letter is several rows, and a count of items read "42 lines" over 45.
  count.textContent = '';

  head.append(what, bar, count);

  // ── what the cut on the surface goes through ─────────────────────────────
  //
  // Frame 3 puts it at the top of this panel and it is the right place: a
  // section is a reading of the same job these lines are, and the question an
  // estimator asks looking at one is "what am I buying here". So the conditions
  // the plane passes through are named, their lines counted, and their groups
  // marked — no filter, because hiding the rest would be a lens, and a lens is a
  // thing somebody chooses.
  const cut = cutOnScreen();
  const through = cut === null
    ? []
    : conditions.filter((c) => cut.through.includes(c.id));

  // ── the lines ────────────────────────────────────────────────────────────
  host.replaceChildren();
  if (cut !== null) {
    const banner = document.createElement('p');
    banner.className = 'cut-lines';
    const lines = through.reduce((n, c) => n + (c.items ?? []).length, 0);
    banner.textContent = `LINES CUT BY SECTION ${cut.named.replace(/^Section /, '')} · ${lines}`;
    banner.title = through.length === 0
      ? 'The plane passes through nothing that carries a line.'
      : `The plane passes through ${through.map((c) => c.name).join(', ')}.`;
    host.append(banner);
  }
  if (!conditions.length) {
    const empty = document.createElement('p');
    empty.className = 'empty-note';
    // Two different states, said differently. No file open is the first thing
    // anybody sees and the handoff writes the sentence for it; a job with
    // nothing traced yet is a different fact and points at what to do.
    empty.textContent = typeof name === 'string' && name
      ? 'Nothing traced yet. Trace something on the Plan and it appears here.'
      : 'Cost shows up here as soon as there is something traced. Nothing to price yet.';
    host.append(empty);
    restoreCaret(host, caret);
    return;
  }

  for (const [index, condition] of conditions.entries()) {
    const measures = measured.get(condition.id)!;
    // What the trace measures, what the estimator typed, and what the model
    // solved for this condition — one scope, in that order, so a solved number
    // can be read by name on a line. Empty until C2 puts the board bill in it.
    const scope = scopeFor(measures, condition.properties ?? {}, derivedFor(condition.id));

    const group = document.createElement('button');
    group.type = 'button';
    group.className = condition.id === selectedConditionId() ? 'cost-group on' : 'cost-group';
    if (through.some((c) => c.id === condition.id)) group.classList.add('cut');
    const swatch = document.createElement('span');
    swatch.className = 'swatch';
    swatch.style.background = condition.color ?? '#5f5f5f';
    const groupName = document.createElement('span');
    groupName.className = 'group-name';
    groupName.textContent = condition.name;
    const summary = document.createElement('span');
    summary.className = 'group-measures';
    summary.textContent = [
      condition.kind !== 'area' ? null : (measures.SF === null ? '—' : `${quantity(measures.SF)} SF`),
      condition.kind === 'count' ? null : (measures.LF === null ? '—' : `${quantity(measures.LF)} LF`),
      `${quantity(measures.EA, 0)} EA`,
      condition.from ? `from ${conditions.find((c) => c.id === condition.from)?.name ?? '—'}` : null,
    ].filter(Boolean).join(' · ');
    if (measures.SF === null && measures.LF === null && condition.kind !== 'count') {
      summary.title = PENDING_REASON;
    }
    group.append(swatch, groupName, summary);
    group.addEventListener('click', () => select({ kind: 'condition', id: condition.id }));
    host.append(group);

    for (const [itemIndex, stored] of (condition.items ?? []).entries()) {
      // One stored line can be a family — one per lettered panel the layout
      // reached. They all open to the SAME chain, because they are one line in
      // the job file and editing one of them is editing the family.
      for (const item of linesFrom(stored, scope)) {
        const result = priceLine(
          item, scope, scenario?.prices ?? {}, condition.waste ?? 0,
          classOf(costCodes, item.costCode) ?? null,
        );
        host.append(compactLine(condition, index, itemIndex, item, result));
        if (item.id === openLineId) {
          host.append(chain(condition, conditions, index, itemIndex, stored, result));
        }
      }
    }

    // The button that adds a line sits in the group it adds to.
    const add = document.createElement('button');
    add.type = 'button';
    add.className = 'cost-add';
    add.append(icon('add', 13));
    const addText = document.createElement('span');
    addText.textContent = 'Add a line';
    add.append(addText);
    add.addEventListener('click', () => void addItem(index, condition));
    host.append(add);
  }

  // The header counts the rows a person can scroll to, not the items in the file.
  const drawn = head.querySelector<HTMLElement>('.cost-head-count');
  if (drawn) drawn.textContent = plural(host.querySelectorAll('.cost-line').length, 'line');

  restoreCaret(host, caret);
}

/**
 * Is this quantity the model's or the estimator's?
 *
 * The handoff's contract: derived carries the cyan mark and is not editable,
 * typed is dotted-underlined and is the estimator's own, and there is no third
 * state. A formula that names a measure or a property is derived — it is worked
 * out from what was traced. A formula that is nothing but a number is a
 * quantity somebody typed, and it reads as one.
 */
const isTyped = (formula: string): boolean => /^\s*-?[\d.,]+\s*$/.test(formula ?? '');

function compactLine(
  condition: Condition, index: number, itemIndex: number, item: Item,
  result: ReturnType<typeof priceLine>,
): HTMLElement {
  const row = document.createElement('div');
  row.className = 'cost-line';
  if (item.id === openLineId) row.classList.add('open');
  if (item.id === selectedItemId()) row.classList.add('on');
  // Struck, and still here. A line taken off the total does not leave the
  // sheet — the argument about whether it belongs is the reason it exists.
  if (result.off !== null) row.classList.add('off');

  const what = document.createElement('button');
  what.type = 'button';
  what.className = 'line-what';
  // The name, in two pieces, and the SECOND one is the piece that survives.
  //
  // "Base layer fasteners, field", "…, perimeter" and "…, corner" are three
  // different quantities at three different prices, and the column is about
  // 130px wide, so all three read "Base layer fasteners, …" and an estimator
  // has three identical rows with different numbers on them. The word that
  // tells them apart is the last one, which is exactly the word an ellipsis at
  // the end eats (C8 §3c). So the tail after the last comma is held back from
  // shrinking and the head gives way instead: "Base layer faste…, corner".
  // One span, ellipsized at its end, the whole name in its title. The two-span
  // form kept the tail and lost the middle, and drew a gap and an orphaned comma
  // ("Continuous cleat     , 20 ga") where a name should be; every line stays
  // one height, which is the rule the sheet keeps (R10).
  const title = document.createElement('span');
  title.className = 'line-item';
  const whole = item.description || 'Untitled line';
  title.textContent = whole;
  title.title = whole;
  what.append(title);
  // The second row, and EVERY line has one, which is what makes them all the
  // same height. It was inside the item's own cell, sharing about 126 px with
  // the description's column — and once a labor line put its crew and its rate
  // there too, the condition's name was ellipsised down to "Main Roof F…". It
  // spans the whole line now, where there is room for the run it came off, the
  // formula, the crew, the name on a quote and why the line is struck, with the
  // hours at the right end under the money they produced.
  const basis = document.createElement('span');
  basis.className = 'line-basis';
  const from = document.createElement('span');
  from.textContent = condition.name;
  // `formula` is on it as well as `line-formula`: a formula IS code, typed on
  // purpose by the person reading it, and the vocabulary check exempts the
  // formula wherever the program renders it — it used to be only ever in an
  // input, and now it is a chip on the line and a term in the working too.
  const formula = document.createElement('span');
  formula.className = 'formula line-formula';
  formula.textContent = item.formula;
  basis.append(from, formula);
  // What you BUY, where that is not what the line is measured in.
  //
  // A line shows its estimating quantity and is charged on its order quantity,
  // and where an order step is on the item those are different numbers: the
  // dumpster line read "6.86 EA" against $5,813.50, which is seven dumpsters'
  // money, and the coping read "650.00 SF" against a price per pound. An
  // estimator multiplies the two figures in front of him, gets a third, and
  // stops trusting the column (C8 §3b). Frame 3 of the handoff prints this line
  // as 826.5 LB for the same reason. The chain is on the priced line already;
  // this is it reaching the sheet, beside the measure it came off rather than
  // instead of it.
  if (result.orderQuantity !== null && result.orderUnitName !== null
    && (result.orderUnitName !== result.unit
      || Math.abs(result.orderQuantity - (result.quantity ?? result.orderQuantity)) > 0.005)) {
    const bought = document.createElement('span');
    bought.className = 'line-ordered';
    bought.textContent = `buy ${quantity(result.orderQuantity)} ${result.orderUnitName}`;
    bought.title = 'What this line orders, once waste and the supplier\'s package are on it.'
      + ' The money is on this figure.';
    basis.append(bought);
  }

  // The crew and the rate, on the line. A labor line whose rate is only visible
  // three clicks away is a labor line nobody checks, and "how many squares a
  // day did you give it" is the first question asked of one.
  if (result.rateSaid !== null) {
    const crew = document.createElement('span');
    crew.className = 'line-crew';
    crew.textContent = result.rateSaid;
    crew.title = `${BASIS_WORDS[result.basis]} — what this rate assumes.`;
    basis.append(crew);
  }
  // And the name on a quote. A subcontract number nobody can chase is a number
  // that gets carried into a buyout meeting and cannot be defended there.
  if (result.basis === 'sub') {
    const quoted = document.createElement('span');
    quoted.className = 'line-quoted';
    quoted.textContent = result.quotedBy ?? 'quoted by nobody named';
    if (result.quotedBy === null) quoted.classList.add('none');
    basis.append(quoted);
  }
  // Why it is off, in the words you would say across the table. On this row and
  // not on one of its own: a struck line that stood a third of a line taller
  // than everything around it read as a different kind of thing, and it is not
  // — it is a line on this estimate that somebody decided about.
  if (result.off !== null) {
    const why = document.createElement('span');
    why.className = 'line-off';
    why.textContent = result.off;
    why.title = `Struck: ${result.off}. It is on the sheet and out of every total.`;
    basis.append(why);
  }
  // Opening a line selects it too, so the geometry it came off lights up in
  // whatever view the surface is showing.
  what.addEventListener('click', () => {
    openLine(item.id === openLineId ? null : item.id);
    select({ kind: 'item', conditionId: condition.id, id: item.id });
  });

  const qty = document.createElement('span');
  qty.className = 'line-qty num';
  if (result.quantity === null) {
    qty.textContent = '—';
    qty.classList.add('none');
    qty.title = result.formulaError ?? result.pending ?? PENDING_REASON;
  } else {
    qty.textContent = quantity(result.quantity);
  }

  const unit = document.createElement('span');
  unit.className = 'line-unit';
  const unitWord = document.createElement('span');
  unitWord.textContent = item.unit;
  unit.append(unitWord);
  if (isTyped(item.formula)) {
    unitWord.classList.add('typed');
    unitWord.title = 'Typed by the estimator.';
  } else {
    const mark = document.createElement('span');
    mark.className = 'derived';
    mark.textContent = '◈';
    // The formula is beside this mark, in the mono face, exempt because it is
    // code the estimator typed. A tooltip is not exempt and never can be — it
    // is read as a sentence — so it says what the mark MEANS rather than
    // printing `VERTICES` at somebody who never asked to see one.
    mark.title = 'Worked out from what was traced, by the formula on this line.';
    unit.append(mark);
  }

  // Three figures on a labor line, the way a report prints them: how much of
  // it, how long it takes, what it costs. The panel is 364 wide and there is no
  // room for a fifth column, so the hours sit at the right end of the row below,
  // under the money they produced.
  const cost = document.createElement('span');
  cost.className = 'line-money num';
  if (result.extended === null) {
    // The reason lives in the tooltip; the cell shows a dash. A red word in a
    // money column reads as an error, and a line with no price yet is not one.
    cost.textContent = '—';
    cost.classList.add('none');
    cost.title = result.pending ?? PENDING_REASON;
  } else {
    cost.textContent = money(result.extended);
  }

  if (result.hours !== null) {
    const hours = document.createElement('span');
    hours.className = 'line-hours num';
    hours.textContent = `${quantity(result.hours)} hrs`;
    const mark = document.createElement('span');
    mark.className = 'derived';
    mark.textContent = '◈';
    mark.title = 'Worked out from the rate and the crew on this line.';
    hours.append(mark);
    basis.append(hours);
  }

  row.append(what, qty, unit, cost, basis);
  // A note about the line goes under it, spanning the row: the formula would
  // not run, or the unit does not follow from it.
  if (result.formulaError) {
    const why = document.createElement('span');
    why.className = 'formula-error';
    why.textContent = result.formulaError;
    row.append(why);
  } else if (result.unitNote) {
    // The sentence itself, on the line. The engine writes it to be read — "this
    // works out to an area, and the line is priced as LF" — and it was being
    // hidden behind "check the unit" with the reason in a tooltip. That is the
    // defect §1.9 already names in the other direction: what the program
    // noticed goes on screen, not only where a hand has to hover to find it.
    // It advises and never overrules: the declared unit is untouched.
    const note = document.createElement('span');
    note.className = 'formula-note';
    note.textContent = result.unitNote;
    row.append(note);
  }
  void index;
  void itemIndex;
  return row;
}

/**
 * One line, open to the whole of it.
 *
 * The formula is edited here, on the line, which is the rule this program was
 * started to keep. Under it is every step from the traced measure to the money
 * — each one a field that writes to the same pointer it always did — and then
 * the arithmetic written out, so the number can be checked without arithmetic
 * of the reader's own.
 */
function chain(
  condition: Condition, conditions: readonly Condition[],
  index: number, itemIndex: number, item: Item,
  result: ReturnType<typeof priceLine>,
): HTMLElement {
  const base = `/conditions/${index}/items/${itemIndex}`;
  const wrap = document.createElement('div');
  wrap.className = 'line-chain';

  // ── the formula, editable, where it belongs ──────────────────────────────
  const formulaRow = document.createElement('label');
  formulaRow.className = 'chain-formula';
  const formulaLabel = document.createElement('span');
  formulaLabel.textContent = 'Formula';
  const formula = named(textField(item.formula, (v) => set(`${base}/formula`, v), 'LF * H'),
    `${base}/formula`);
  formula.classList.add('formula');
  if (result.formulaError) formula.classList.add('bad');
  formulaRow.append(formulaLabel, formula);
  wrap.append(formulaRow);
  if (result.formulaError) {
    const why = document.createElement('span');
    why.className = 'formula-error';
    why.textContent = result.formulaError;
    wrap.append(why);
  }

  // ── where it comes from ─────────────────────────────────────────────────
  const source = document.createElement('p');
  source.className = 'chain-source';
  source.textContent = `Comes from ${condition.name}`
    + (condition.from ? ` → ${conditions.find((c) => c.id === condition.from)?.name ?? '—'}` : '');
  wrap.append(source);

  // ── the chain, step by step ─────────────────────────────────────────────
  const table = document.createElement('div');
  table.className = 'chain-rows';

  table.append(chainRow('Code',
    named(textField(item.costCode, (v) => set(`${base}/costCode`, v), 'code'), `${base}/costCode`)));
  // What kind of cost this is. A WORD and not a picker: it is read off the line
  // — a rate at the brake is fabrication, anything else with a rate is labor,
  // and with no rate the cost code above already answered it. Asking the
  // estimator to classify what they just wrote is a question the sheet can
  // answer for itself.
  table.append(chainRow('Kind', word(BASIS_WORDS[result.basis])));
  table.append(chainRow('Quantity', figure(result.quantity, item.unit, !isTyped(item.formula),
    result.formulaError ?? result.pending)));
  table.append(chainRow('Unit',
    named(selectField(UNITS, item.unit, (v) => set(`${base}/unit`, v)), `${base}/unit`)));
  table.append(chainRow('Waste',
    named(numberField(item.waste ?? 0, (v) => set(`${base}/waste`, v), '0'), `${base}/waste`)));
  table.append(chainRow('With waste', figure(result.withWaste, item.unit, true, result.pending)));
  // What you buy, and what it is priced against — three units, because a real
  // supply house uses three. Membrane is estimated in squares, bought by the
  // roll and quoted by the square foot, and each step is where money hides.
  // The crew and the rate, where they are typed. A production rate is the
  // order-step conversion on a labor line, so it sits in the chain exactly
  // where a roll conversion sits on a material one.
  if (item.production !== undefined) {
    table.append(chainRow('Crew', named(numberField(
      item.production.crew,
      (v) => writeProduction(base, item, { crew: v }),
      'how many',
    ), `${base}/production/crew`)));
    const rate = document.createElement('span');
    rate.className = 'chain-rate';
    rate.append(named(numberField(
      item.production.rate,
      (v) => writeProduction(base, item, { rate: v ?? 0 }),
      item.unit,
    ), `${base}/production/rate`));
    rate.append(selectField(
      [...PRODUCTION_PER],
      item.production.per,
      (v) => writeProduction(base, item, { per: v as ProductionPer }),
      (v) => `${item.unit} ${PRODUCTION_PER_WORDS[v as ProductionPer]}`,
    ));
    table.append(chainRow('Rate', rate));
    table.append(chainRow('Hours', figure(result.hours, 'hrs', true, result.pending)));
    if (result.crewDays !== null) {
      table.append(chainRow('Crew days', figure(result.crewDays, 'days', true, result.pending)));
    }
  }
  if (result.basis === 'sub') {
    table.append(chainRow('Quoted by', named(textField(
      item.priceSource?.from ?? '',
      (v) => writeQuote(base, item, v),
      'whose number this is',
    ), `${base}/priceSource/from`)));
  }
  table.append(chainRow('Order', figure(result.orderQuantity, result.orderUnitName, true, result.pending)));
  table.append(chainRow('Priced', figure(result.priceQuantity, result.priceUnitName, true, result.pending)));
  table.append(chainRow('Unit cost',
    named(numberField(item.unitCost, (v) => set(`${base}/unitCost`, v), '$'), `${base}/unitCost`)));
  table.append(chainRow('Extended', moneyCell(result.extended, result.pending), 'chain-total'));
  // Off the sheet, and why. Left empty the line is on the bid; filled in, it is
  // struck, out of every total, and the reason is the words beside it.
  table.append(chainRow('Off the sheet', named(textField(
    item.off?.reason ?? '',
    (v) => writeOff(base, item, v),
    'on the sheet',
  ), `${base}/off/reason`)));
  wrap.append(table);

  // ── the arithmetic, written out ─────────────────────────────────────────
  const trace = document.createElement('div');
  trace.className = 'chain-trace';
  for (const line of workings(item, result)) {
    const p = document.createElement('p');
    for (const part of line) {
      const span = document.createElement('span');
      if (part.formula) span.className = 'formula';
      span.textContent = part.text;
      p.append(span);
    }
    trace.append(p);
  }
  if (trace.children.length) wrap.append(trace);

  // ── where else this formula runs ────────────────────────────────────────
  const elsewhere: string[] = [];
  for (const c of conditions) {
    for (const other of c.items ?? []) {
      if (other.id !== item.id && other.formula === item.formula) {
        elsewhere.push(`${c.name} → ${other.description || 'untitled line'}`);
      }
    }
  }
  if (elsewhere.length) {
    const also = document.createElement('p');
    also.className = 'chain-also';
    also.textContent = `The same formula runs on ${plural(elsewhere.length, 'other line')}: ${elsewhere.join(' · ')}`;
    wrap.append(also);
  }

  // ── the two things there are to do ──────────────────────────────────────
  // There is no "Keep it" and no "Cancel". Every field here writes to the
  // document as it is typed — that is how the money moves in the other window
  // while the mouse is still down — so a button that claimed to keep the change
  // would be claiming something that already happened, and one that claimed to
  // cancel it would be claiming an Undo this program does not have yet (D136).
  const actions = document.createElement('div');
  actions.className = 'chain-actions';
  const close = document.createElement('button');
  close.type = 'button';
  close.className = 'chain-close';
  close.textContent = 'Close the line';
  close.addEventListener('click', () => { openLine(null); });
  const off = document.createElement('button');
  off.type = 'button';
  off.className = 'remove';
  off.textContent = 'Take this line off';
  off.addEventListener('click', () => {
    openLine(null);
    void removeItem(index, itemIndex);
  });
  actions.append(close, off);
  wrap.append(actions);

  return wrap;
}

/**
 * The arithmetic, in the order it happened.
 *
 * Every figure here is off `priceLine`'s own answer — nothing is recomputed, so
 * these lines cannot disagree with the number above them. The one that matters
 * is the sheet-metal case: linear feet become pounds through a stretch-out that
 * belongs to the detail and cannot be a constant on the material.
 */
interface Part { readonly text: string; readonly formula?: boolean }

function workings(item: Item, result: ReturnType<typeof priceLine>): Part[][] {
  const out: Part[][] = [];
  if (result.quantity === null) return out;
  const waste = item.waste ?? 0;
  const first: Part[] = [
    { text: item.formula, formula: true },
    { text: ` = ${quantity(result.quantity)} ${result.unit}` },
  ];
  if (result.withWaste !== null && waste) {
    first.push({ text: ` → with ${waste}% waste ${quantity(result.withWaste)} ${result.unit}` });
  }
  out.push(first);
  if (result.hours !== null && result.rateSaid !== null) {
    // The rate as it was stated, not as it was divided. An estimator checking
    // this line wants to see the crew and the day they gave it.
    const days = result.crewDays === null ? '' : `, ${quantity(result.crewDays)} crew-days`;
    out.push([{
      text: `at ${result.rateSaid} that is ${quantity(result.hours)} hours${days}`,
    }]);
  } else if (result.orderQuantity !== null && result.orderUnitName) {
    out.push([{ text: `bought as ${quantity(result.orderQuantity)} ${result.orderUnitName}` }]);
  }
  if (result.priceQuantity !== null && result.priceUnitName && result.unitCost !== null) {
    out.push([{
      text: `${quantity(result.priceQuantity)} ${result.priceUnitName}`
        + ` at ${money(result.unitCost)} = ${money(result.extended)}`,
    }]);
  }
  // What the line has to say for itself — which row of a table it was priced
  // from, what it stands in for. The job file carried these words all along and
  // no screen printed them (R6).
  if (item.notes) out.push([{ text: item.notes }]);
  return out;
}

/**
 * Write a change to the rate, the crew or what the rate is per.
 *
 * The WHOLE item goes back, not a pointer into it. The shell's `doc_set` walks
 * to a place that already exists and refuses one that does not, so writing
 * `…/production/rate` onto a line that has never had a rate would fail with
 * "nothing at … in this job" — and the field would look like it worked while
 * nothing happened. An item is small; the whole of it goes.
 */
const writeProduction = (
  base: string, item: Item, patch: Partial<{ rate: number; crew: number | undefined; per: ProductionPer }>,
): Promise<unknown> => {
  const now = item.production ?? { rate: 0, per: 'crew-day' as ProductionPer };
  const production = { ...now, ...patch };
  // A crew cleared out is a crew nobody stated, which is not the same as nought
  // of them — and the line says so rather than dividing by it.
  if (production.crew === undefined || !(production.crew > 0)) delete (production as { crew?: number }).crew;
  return set(base, { ...item, production });
};

/** The name on a quote. Same reason the whole item goes back. */
const writeQuote = (base: string, item: Item, from: string): Promise<unknown> => {
  const source = { ...(item.priceSource ?? {}), from };
  if (from.trim() === '') delete (source as { from?: string }).from;
  const next: Item = Object.keys(source).length === 0
    ? stripped(item, 'priceSource')
    : { ...item, priceSource: source };
  return set(base, next);
};

/** Struck, or back on the sheet. An empty reason is a line nobody took off. */
const writeOff = (base: string, item: Item, reason: string): Promise<unknown> =>
  set(base, reason.trim() === '' ? stripped(item, 'off') : { ...item, off: { reason } });

/** The same item without one field. A key left at undefined is written as null. */
function stripped(item: Item, key: keyof Item): Item {
  const copy = { ...item } as Record<string, unknown>;
  delete copy[key];
  return copy as unknown as Item;
}

/** A word the program worked out. Read, never typed — so it is not a field. */
function word(text: string): HTMLElement {
  const span = document.createElement('span');
  span.className = 'chain-word';
  span.textContent = text;
  return span;
}

/** A step in the chain: what it is, and what it reads. */
function chainRow(label: string, value: Node, className = ''): HTMLElement {
  const row = document.createElement('div');
  row.className = className ? `chain-row ${className}` : 'chain-row';
  const k = document.createElement('span');
  k.className = 'chain-name';
  k.textContent = label;
  const v = document.createElement('span');
  v.className = 'chain-value';
  v.append(value);
  row.append(k, v);
  return row;
}

/** A quantity the program worked out: the number, its unit, and the cyan mark. */
function figure(value: number | null, unit: string | null, derived: boolean, why?: string | null): HTMLElement {
  const span = document.createElement('span');
  span.className = 'num';
  if (value === null) {
    span.textContent = '—';
    span.classList.add('none');
    span.title = why ?? PENDING_REASON;
    return span;
  }
  span.textContent = `${quantity(value)} ${unit ?? ''}`.trim();
  if (derived) {
    const mark = document.createElement('span');
    mark.className = 'derived';
    mark.textContent = '◈';
    span.append(mark);
  } else {
    span.classList.add('typed');
  }
  return span;
}

function moneyCell(value: number | null, why: string | null): HTMLElement {
  const span = document.createElement('span');
  span.className = 'num';
  if (value === null) {
    span.textContent = '—';
    span.classList.add('none');
    span.title = why ?? PENDING_REASON;
    return span;
  }
  span.textContent = money(value);
  return span;
}

// ── changing the job ───────────────────────────────────────────────────────

async function addItem(conditionIndex: number, condition: Condition): Promise<void> {
  const unit = condition.kind === 'area' ? 'SF' : condition.kind === 'line' ? 'LF' : 'EA';
  const item: Item = {
    id: `item-${Date.now().toString(36)}`,
    description: '',
    costCode: '',
    unit,
    // A new line starts by measuring what was traced. It is the honest default
    // and it shows the estimator what the formula is for.
    formula: unit,
  };
  const items = [...(condition.items ?? []), item];
  await set(`/conditions/${conditionIndex}/items`, items);
  openLine(item.id);
}

async function removeItem(conditionIndex: number, itemIndex: number): Promise<void> {
  const conditions = (at('/conditions', doc()) as Condition[]) ?? [];
  const items = [...(conditions[conditionIndex]?.items ?? [])];
  items.splice(itemIndex, 1);
  await set(`/conditions/${conditionIndex}/items`, items);
}

// ── small parts ────────────────────────────────────────────────────────────

function activeScenario(d: Doc) {
  const job = at('/job', d) as { scenarios?: { id: string; name: string; prices?: Record<string, number> }[]; activeScenarioId?: string } | undefined;
  return job?.scenarios?.find((s) => s.id === job.activeScenarioId) ?? job?.scenarios?.[0];
}

/**
 * A field that writes to the document as it is typed.
 *
 * The rebuild that follows destroys it; `caret.ts` puts the caret back, keyed
 * on the pointer this field writes to. Without that a formula typed "LF * H"
 * got as far as "L".
 */
function textField(value: string, write: (v: string) => Promise<unknown>, placeholder = ''): HTMLInputElement {
  const input = document.createElement('input');
  input.type = 'text';
  input.value = value ?? '';
  input.placeholder = placeholder;
  input.addEventListener('input', () => void write(input.value));
  return input;
}

function numberField(value: number | undefined, write: (v: number | undefined) => Promise<unknown>, placeholder = ''): HTMLInputElement {
  const input = document.createElement('input');
  input.type = 'number';
  input.step = 'any';
  input.className = 'num';
  input.value = value === undefined ? '' : String(value);
  input.placeholder = placeholder;
  input.addEventListener('input', () => {
    const parsed = input.value === '' ? undefined : Number(input.value);
    void write(parsed !== undefined && Number.isFinite(parsed) ? parsed : undefined);
  });
  return input;
}

function selectField(
  options: string[], value: string, write: (v: string) => Promise<unknown>,
  say: (v: string) => string = (v) => v,
): HTMLSelectElement {
  const select_ = document.createElement('select');
  for (const o of options) {
    const opt = document.createElement('option');
    opt.value = o;
    opt.textContent = say(o);
    select_.append(opt);
  }
  select_.value = value;
  select_.addEventListener('change', () => void write(select_.value));
  return select_;
}

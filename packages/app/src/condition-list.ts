// ── The list of what has been traced ───────────────────────────────────────
// Swatch, name, the number that picks it, what it measures right now, and
// whether it is drawn on the sheet. There is no tree any more (D132): this list
// is where conditions live, and it stands in two places — the Plan's own rail,
// beside the drawing, and the cost panel's Condition tab, above the properties.
//
// One definition, because two would drift. The Plan had this list private to
// itself and the Condition tab needs the same rows; a second copy is how the
// two ended up disagreeing about a measure in every product that has tried it.

import { measure, strayCorners, type Condition, type Page } from '@roofnerd/engine';
import { at, set, type Doc } from './doc.js';
import { PENDING_REASON, plural, propertyPhrase, quantity } from './labels.js';
import { hueFor } from './icons.js';
import { select, selectedConditionId } from './selection.js';

/**
 * How many of a condition's corners were clicked past the edge of the paper.
 *
 * The check itself is `strayCorners` in the engine, because the page bounds it
 * needs used to live on one editor's private copy of `Page` — so the Model, the
 * panel and every probe were shut out of a question they all want to ask. A
 * page only counts once it knows its own size: a sheet still loading has no
 * bounds to be outside of.
 */
const offSheet = (c: Condition, pages: Page[]): number => strayCorners(c.traces, pages);

export function renderConditionList(host: HTMLElement, d: Doc): void {
  host.replaceChildren();

  const pages = (at('/pages', d) as Page[]) ?? [];
  const calibrations: Record<string, { feetPerUnit: number } | undefined> = {};
  for (const p of pages) if (p.feetPerUnit) calibrations[p.id] = { feetPerUnit: p.feetPerUnit };

  const list = (at('/conditions', d) as Condition[]) ?? [];
  if (!list.length) {
    const empty = document.createElement('p');
    empty.className = 'muted';
    empty.textContent = 'Nothing traced yet. Add a drawing, set its scale, then pick an act.';
    host.append(empty);
    return;
  }

  for (const [index, c] of list.entries()) {
    const m = measure(c.kind, c.traces ?? [], c.properties ?? {}, calibrations);
    const row = document.createElement('button');
    row.type = 'button';
    row.className = c.id === selectedConditionId() ? 'condition on' : 'condition';
    row.addEventListener('click', () => select({ kind: 'condition', id: c.id }));

    const swatch = document.createElement('span');
    swatch.className = 'swatch';
    swatch.style.background = c.color ?? hueFor(index);

    const name = document.createElement('span');
    name.className = 'condition-name';
    name.textContent = c.name;

    // The number that picks it. VES has 1 to 9 and an estimator who has used it
    // reaches for the key rather than the mouse, so the key is printed on the
    // row it belongs to — a hotkey nobody can see is a hotkey nobody uses.
    if (index < 9) {
      const key = document.createElement('span');
      key.className = 'condition-key';
      key.textContent = String(index + 1);
      key.title = `Press ${index + 1} to pick this`;
      row.append(key);
    }

    const measures = document.createElement('span');
    measures.className = 'condition-measures';
    // A measure with no number yet is a dash with the reason in its tooltip.
    // Never a coloured word — nothing is broken, the sheet is not scaled.
    // An area has a surface; a run does not, and printing "— SF" beside one
    // suggests it might. Each kind shows the measures it actually has.
    const parts: string[] = [];
    if (c.kind === 'area') parts.push(m.SF === null ? '—' : `${quantity(m.SF)} SF`);
    if (c.kind !== 'count') parts.push(m.LF === null ? '—' : `${quantity(m.LF)} LF`);
    parts.push(`${quantity(m.EA, 0)} EA`);
    measures.textContent = parts.join(' · ');
    if (m.LF === null && c.kind !== 'count') measures.title = PENDING_REASON;

    // Corners clicked past the edge of the paper.
    //
    // The desk around a sheet is not the drawing, and area measured out there
    // is measured off nothing — but the program used to take it, price it and
    // save it without a word. On a full-size sheet that is easy to do and
    // impossible to notice, because the whole drawing is a thumbnail until you
    // zoom. It stays a measurement; it just stops being a silent one.
    const stray = offSheet(c, pages);
    if (stray > 0) {
      const off = document.createElement('span');
      off.className = 'condition-off-sheet';
      off.textContent = `${plural(stray, 'corner')} off the sheet`;
      off.title = 'Traced past the edge of the drawing. What is out there was measured off the desk, not off the roof.';
      measures.append(document.createTextNode(' · '), off);
    }

    // A4: a condition can be hidden on the sheet without being deleted — a busy
    // roof plan is unreadable with every trace on it at once.
    const visible = c.hidden !== true;
    const eye = document.createElement('span');
    eye.className = 'visibility';
    eye.title = visible ? 'Hide this on the drawing' : 'Show this on the drawing';
    eye.setAttribute('role', 'button');
    eye.textContent = visible ? '●' : '○';
    eye.addEventListener('click', (e) => {
      e.stopPropagation();
      const all = (at('/conditions') as Condition[]) ?? [];
      void set('/conditions', all.map((x) => (x.id === c.id ? { ...x, hidden: visible } : x)));
    });
    row.classList.toggle('hidden-condition', !visible);

    row.append(swatch, name, eye, measures);

    // Properties in the words an estimator uses: "4 sides", not "SIDES 4".
    const shown = Object.entries(c.properties ?? {})
      .filter(([, v]) => v !== undefined)
      .map(([k, v]) => propertyPhrase(k, v as number));
    if (shown.length) {
      const props = document.createElement('span');
      props.className = 'condition-props';
      props.textContent = shown.join(' · ');
      row.append(props);
    }

    host.append(row);
  }
}

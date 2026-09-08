// ── The Library ────────────────────────────────────────────────────────────
// The book: saved sets of items, and the profiles the sheet-metal ones are
// formed from.
//
// It is optional depth. A job traces, measures, prices and recaps with no
// library open at all — this is where an estimator stops retyping the same
// build-up on every bid, not something the program needs to work.
//
// Loading an assembly COPIES its items onto the condition. It does not link
// them. An estimator who changes a price on a bid is changing that bid, and a
// library that reached back into finished jobs would be a library nobody dared
// edit.

import { girthOf, type Profile } from '@roofnerd/engine';
import { at, doc, set, subscribe, type Doc } from '../doc.js';
import { selectedConditionId, watchSelection } from '../selection.js';

type Item = {
  id: string; description: string; costCode: string; unit: string; formula: string;
  layer?: string;
  priceSource?: { from?: string; firmness?: string; on?: string; doubt?: string };
};
type Assembly = {
  id: string; name: string; generic?: boolean; manufacturer?: string;
  items: Item[]; notes?: string;
};
type Condition = { id: string; name: string; kind: string; items?: unknown[] };

let stopWatching: (() => void) | null = null;
let stopSubscribing: (() => void) | null = null;

export function mountLibrary(host: HTMLElement): void {
  stopWatching?.();
  stopSubscribing?.();
  host.replaceChildren();
  host.classList.add('library-editor');

  const body = document.createElement('div');
  body.className = 'library-body';
  host.append(body);

  const draw = () => render(body, doc());
  stopSubscribing = subscribe(() => draw());
  stopWatching = watchSelection(() => draw());
}

function render(host: HTMLElement, d: Doc): void {
  host.replaceChildren();

  const assemblies = (at('/library/assemblies', d) as Assembly[]) ?? [];
  const profiles = (at('/library/profiles', d) as Profile[]) ?? [];

  if (assemblies.length === 0 && profiles.length === 0) {
    const empty = document.createElement('p');
    empty.className = 'muted';
    empty.textContent = 'No library on this job. A job prices perfectly well without one — '
      + 'the library is where you stop retyping the same build-up on every bid.';
    host.append(empty);
    return;
  }

  const conditions = (at('/conditions', d) as Condition[]) ?? [];
  const selected = conditions.find((c) => c.id === selectedConditionId()) ?? null;

  host.append(section('Assemblies', assemblies.map((a) => assemblyCard(a, selected))));
  if (profiles.length) {
    host.append(section('Profiles', profiles.map((p) => profileCard(p))));
  }
}

function section(title: string, cards: HTMLElement[]): HTMLElement {
  const wrap = document.createElement('section');
  wrap.className = 'library-section';
  const h = document.createElement('h3');
  h.textContent = title;
  wrap.append(h, ...cards);
  return wrap;
}

function assemblyCard(a: Assembly, selected: Condition | null): HTMLElement {
  const card = document.createElement('article');
  card.className = 'library-card';

  const head = document.createElement('header');
  const name = document.createElement('strong');
  name.textContent = a.name;
  const kind = document.createElement('span');
  kind.className = 'library-kind';
  // "Generic" and a manufacturer's system are different things to an estimator:
  // one is what you bid when nothing is specified, the other is what you bid
  // when something is.
  kind.textContent = a.manufacturer ?? (a.generic ? 'Generic' : '');
  head.append(name, kind);

  const load = document.createElement('button');
  load.type = 'button';
  load.className = 'library-load';
  if (selected) {
    load.textContent = `Load onto ${selected.name}`;
    load.addEventListener('click', () => void loadOnto(a, selected));
  } else {
    load.textContent = 'Pick a condition to load it onto';
    load.disabled = true;
  }
  head.append(load);
  card.append(head);

  const list = document.createElement('ul');
  list.className = 'library-items';
  for (const item of a.items) {
    const li = document.createElement('li');
    const what = document.createElement('span');
    what.textContent = item.description;
    if (item.layer) {
      const layer = document.createElement('em');
      layer.className = 'library-layer';
      layer.textContent = item.layer;
      what.append(' ', layer);
    }
    li.append(what, firmnessChip(item), Object.assign(document.createElement('code'), {
      textContent: `${item.formula} → ${item.unit}`,
    }));
    list.append(li);
  }
  card.append(list);

  if (a.notes) {
    const note = document.createElement('p');
    note.className = 'library-note';
    note.textContent = a.notes;
    card.append(note);
  }
  return card;
}

/**
 * How good the price on this line is, and why it is doubtful when it is.
 *
 * Its own axis. A price can be firm on a quantity that is still pending, and a
 * placeholder is not the same thing as no price at all — one is a number
 * standing in for a number, the other is nothing. Silence means nobody has said.
 */
function firmnessChip(item: Item): HTMLElement {
  const chip = document.createElement('span');
  chip.className = 'library-firmness';
  const source = item.priceSource;
  if (!source?.firmness) return chip;
  chip.classList.add(`is-${source.firmness}`);
  chip.textContent = source.firmness;
  const why = [source.from, source.doubt].filter(Boolean).join(' — ');
  if (why) chip.title = why;
  return chip;
}

function profileCard(p: Profile): HTMLElement {
  const card = document.createElement('article');
  card.className = 'library-card';

  const head = document.createElement('header');
  const name = document.createElement('strong');
  name.textContent = p.name;
  const girth = document.createElement('span');
  girth.className = 'library-girth';
  // The number the formula language exists for. Shown as the sum it is, so an
  // estimator can see it is the legs and the hems rather than a magic figure.
  girth.textContent = `${girthOf(p)} in girth`;
  head.append(name, girth);
  card.append(head);

  const made = document.createElement('p');
  made.className = 'library-note';
  const legs = p.legs.join(' + ');
  const hems = (p.hems ?? []).length ? ` + ${(p.hems ?? []).join(' + ')} for the hems` : '';
  made.textContent = `${legs}${hems}`;
  card.append(made);

  if (p.weightPerSF) {
    const weight = document.createElement('p');
    weight.className = 'library-note';
    weight.textContent = `${p.weightPerSF} lb per square foot of coil`;
    card.append(weight);
  }
  return card;
}

/**
 * Copy an assembly's items onto a condition.
 *
 * Ids are made fresh so loading the same assembly onto two conditions does not
 * give them items that share an id — which would make a price typed on one
 * appear on the other, silently.
 */
async function loadOnto(a: Assembly, condition: Condition): Promise<void> {
  const conditions = ((at('/conditions') as Condition[]) ?? []).map((c) => {
    if (c.id !== condition.id) return c;
    const existing = (c.items ?? []) as Item[];
    const added = a.items.map((item, n) => ({
      ...item,
      id: `${condition.id}-${a.id}-${n}`,
      // Keep the book's id so the price book can still price this line.
      libraryId: item.id,
    }));
    return { ...c, items: [...existing, ...added], assemblyId: a.id };
  });
  await set('/conditions', conditions);
}

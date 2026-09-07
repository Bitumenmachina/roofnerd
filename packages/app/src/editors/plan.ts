// ── The Plan editor ────────────────────────────────────────────────────────
// At Gate 1 this is the drawing: a PDF or an aerial, a two-point scale, and the
// trace tools. None of that is here yet.
//
// What is here is the half of the working loop that a drawing would sit in — a
// condition, and a number on it you can change. Change it and watch the
// Estimate Sheet move, in this window or in a torn-off one. That is the thing
// Gate 0 exists to prove, and it is proved with one field rather than with a
// takeoff surface nobody asked for yet.

import { at, set, subscribe, type Doc } from '../doc.js';

const HEIGHT = '/conditions.json/0/properties/height';
const SIDES = '/conditions.json/0/properties/sides';

export function mountPlan(host: HTMLElement): void {
  host.replaceChildren();

  const title = document.createElement('h2');
  title.textContent = 'Plan';
  const note = document.createElement('p');
  note.className = 'editor-note';
  note.textContent =
    'The drawing goes here at Gate 1 — PDF or aerial, a two-point scale, and the trace '
    + 'tools. For now this is a condition off the demo job with two of its properties. '
    + 'Change one and watch the Estimate Sheet.';
  host.append(title, note);

  const name = document.createElement('h3');
  host.append(name);

  const height = field('Wall flashing height, feet', HEIGHT);
  const sides = field('Number of sides', SIDES);
  host.append(height.label, sides.label);

  subscribe((doc: Doc) => {
    const condition = at('/conditions.json/0', doc) as { name?: string } | undefined;
    name.textContent = condition?.name ?? '— no condition —';
    height.show(at(HEIGHT, doc));
    sides.show(at(SIDES, doc));
    const open = condition !== undefined;
    height.input.disabled = !open;
    sides.input.disabled = !open;
  });
}

/** A number on the document. Typing in it asks the shell to change the job. */
function field(caption: string, pointer: string) {
  const label = document.createElement('label');
  const span = document.createElement('span');
  span.textContent = caption;
  const input = document.createElement('input');
  input.type = 'number';
  input.step = 'any';
  label.append(span, input);

  let editing = false;
  input.addEventListener('focus', () => { editing = true; });
  input.addEventListener('blur', () => { editing = false; });
  input.addEventListener('input', () => {
    const value = Number(input.value);
    if (input.value !== '' && Number.isFinite(value)) void set(pointer, value);
  });

  return {
    label,
    input,
    // Do not overwrite what someone is typing into. The document still decides
    // what the value is; this only decides when to redraw from it.
    show(value: unknown) {
      if (editing) return;
      input.value = typeof value === 'number' ? String(value) : '';
    },
  };
}

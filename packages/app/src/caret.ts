// ── The field being typed into ─────────────────────────────────────────────
// Every editor here rebuilds itself when the document changes, and that is the
// right design: one document, one direction, no editor holding its own copy of
// a number. The cost is that the element being typed into is destroyed between
// one keystroke and the next — `replaceChildren` removes it, removing it blurs
// it, and the next character is delivered to the body of the document and
// discarded. A height typed "12" is a height of 1.
//
// So the rebuild stands and the caret is put back: which field had it, and where
// in the text it was. Two functions, either side of the rebuild.
//
// The alternative — leaving the field being edited alone and rebuilding the rest
// — is a diffing render, and it would take from the panel the one thing the
// panel is for: the measures and the money under them move as the height is
// typed. This is the smaller change, and it keeps the direction of the data.
//
// A field says which one it is with `data-field`, and the name is the pointer it
// writes to (`/conditions/2/items/0/formula`) or the property it holds
// (`properties/H`). Two fields with one name would be the defect; a pointer
// cannot be ambiguous.

export interface HeldCaret {
  readonly key: string;
  /** Null on the controls that cannot answer — a number input is one. */
  readonly start: number | null;
  readonly end: number | null;
}

/** What is being typed into inside `host`, if anything is. */
export function heldCaret(host: HTMLElement): HeldCaret | null {
  const active = document.activeElement;
  if (!(active instanceof HTMLElement) || !host.contains(active)) return null;
  const key = active.dataset['field'];
  if (!key) return null;

  let start: number | null = null;
  let end: number | null = null;
  if (active instanceof HTMLInputElement || active instanceof HTMLTextAreaElement) {
    // `selectionStart` is null on an input of type number and THROWS on some
    // others, which is why this is both guarded and allowed to answer null.
    try {
      start = active.selectionStart;
      end = active.selectionEnd;
    } catch {
      start = null;
      end = null;
    }
  }
  return { key, start, end };
}

/**
 * Put it back, after the rebuild.
 *
 * Only ever restores a focus that was just taken: `heldCaret` answers nothing
 * unless the focus was already on a named field inside this host, so this can
 * never steal the caret from somewhere else in the window.
 */
export function restoreCaret(host: HTMLElement, held: HeldCaret | null): void {
  if (!held) return;
  // Scanned rather than selected: a property name comes out of the job file, and
  // a quote in one would turn `[data-field="…"]` into a syntax error that broke
  // the whole render.
  let to: HTMLElement | null = null;
  for (const el of Array.from(host.querySelectorAll<HTMLElement>('[data-field]'))) {
    if (el.dataset['field'] === held.key) { to = el; break; }
  }
  if (!to) return;

  to.focus();
  if (held.start === null) return;
  if (to instanceof HTMLInputElement || to instanceof HTMLTextAreaElement) {
    try {
      to.setSelectionRange(held.start, held.end);
    } catch {
      // A number input cannot be told where its caret is. Focus is the half
      // that matters there: typing appends, which is what the hand expected.
    }
  }
}

/** Name a control so the caret can find it again. */
export function named<T extends HTMLElement>(control: T, key: string): T {
  control.dataset['field'] = key;
  return control;
}

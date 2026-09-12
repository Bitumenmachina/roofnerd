// ── What is selected ───────────────────────────────────────────────────────
// Selecting a condition anywhere selects it everywhere. That only works if
// there is one selection that everybody watches — the same reason there is one
// document.
//
// It lives in the window rather than in the job: which condition you are
// looking at is not part of the bid, and saving it into the file would put it
// in a git diff every time anybody clicked anything.
//
// "Everywhere" used to stop at the window frame. The selection was a module
// variable plus `sessionStorage`, and sessionStorage is per-window on purpose,
// so Addendum 4 §4 — "selecting a condition on the sheet highlights it in plan
// and in 3D; selecting in 3D highlights it on the sheet" — could not be true of
// a torn-off sheet, which is the arrangement the program is built around. Now a
// selection is also said out loud on the event bus the document already uses,
// and every other window hears it.

import { emit, listen } from '@tauri-apps/api/event';

export type Selected =
  | { readonly kind: 'job' }
  | { readonly kind: 'page'; readonly id: string }
  | { readonly kind: 'condition'; readonly id: string }
  | { readonly kind: 'item'; readonly conditionId: string; readonly id: string };

type Watcher = (selected: Selected) => void;

/**
 * Selection survives an editor switch, because an editor switch is a reload.
 *
 * Changing which editor an area shows re-navigates the window with a different
 * editor in the URL — one code path for "which editor am I" instead of two,
 * which is the right trade. But the selection lived in a module variable, so
 * picking a condition on the Plan and then switching that area to the Library
 * arrived with nothing selected and a button saying "pick a condition". Found
 * by driving a real job end to end; no section's own check caught it, because
 * each of them selects after it has finished switching.
 *
 * `sessionStorage` is the right shelf for it: it survives the reload, it is
 * per-window so a torn-off sheet keeps its own, and it never touches the job —
 * which condition you are looking at is still not part of the bid.
 */
const REMEMBERED = 'roofnerd:selected';

function remembered(): Selected {
  try {
    const raw = sessionStorage.getItem(REMEMBERED);
    if (raw) return JSON.parse(raw) as Selected;
  } catch {
    // A window with no storage still works; it just starts at the job.
  }
  return { kind: 'job' };
}

let current: Selected = remembered();
const watchers = new Set<Watcher>();

export const selected = (): Selected => current;

/** What a window says when it selects something, and who said it. */
const CHANGED = 'selection:changed';
interface Said {
  readonly from: string;
  readonly selected: Selected;
}

/**
 * This window, for as long as this page lives.
 *
 * `emit` reaches every window, the sender included — so without a name on the
 * message a window would hear its own selection come back, apply it, and fire
 * every watcher a second time. Worse, a payload that had travelled through JSON
 * is a different object each time round, so nothing downstream could tell the
 * echo from a real change. The token is the whole of the guard: a window ignores
 * what it said itself.
 *
 * An editor switch reloads the window and mints a new one, which is correct —
 * it is a new page, and the only thing the token has to be is unique to a page.
 */
const ME = `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 10)}`;

function apply(next: Selected): void {
  current = next;
  try {
    sessionStorage.setItem(REMEMBERED, JSON.stringify(next));
  } catch {
    // Not being able to remember it is not a reason to refuse the selection.
  }
  for (const w of watchers) w(current);
}

export function select(next: Selected): void {
  apply(next);
  // Said out loud, after it is true here. A window that waited for the round
  // trip before showing its own selection would feel slower than the mouse.
  //
  // Nothing is saved by this: the message carries the selection and the
  // document is not touched. And nothing depends on it — a window with no shell
  // to talk to (the front-end harness) or a window whose capabilities do not
  // reach the event bus still selects, locally, exactly as it did before.
  void emit(CHANGED, { from: ME, selected: next } satisfies Said).catch(() => undefined);
}

/**
 * Follow what the other windows select. Called once, when a window boots.
 *
 * It never emits in return — an arriving selection is applied and that is the
 * end of it, which is the second half of not having a loop.
 */
export async function followSelection(): Promise<void> {
  try {
    await listen<Said>(CHANGED, (event) => {
      const said = event.payload;
      if (!said || said.from === ME || !said.selected) return;
      apply(said.selected);
    });
  } catch {
    // A window that cannot hear the others still works on its own.
  }
}

export function watchSelection(w: Watcher): () => void {
  watchers.add(w);
  w(current);
  return () => watchers.delete(w);
}

/** The condition in view, whichever way it came to be selected. */
export function selectedConditionId(): string | null {
  if (current.kind === 'condition') return current.id;
  if (current.kind === 'item') return current.conditionId;
  return null;
}

export const selectedPageId = (): string | null => (current.kind === 'page' ? current.id : null);

// ── What is selected ───────────────────────────────────────────────────────
// Selecting a node in the tree selects it in every editor. That only works if
// there is one selection, in one place, that everybody watches — the same
// reason there is one document.
//
// It lives in the window rather than in the job: which condition you are
// looking at is not part of the bid, and saving it into the file would put it
// in a git diff every time anybody clicked anything.

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

export function select(next: Selected): void {
  current = next;
  try {
    sessionStorage.setItem(REMEMBERED, JSON.stringify(next));
  } catch {
    // Not being able to remember it is not a reason to refuse the selection.
  }
  for (const w of watchers) w(current);
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

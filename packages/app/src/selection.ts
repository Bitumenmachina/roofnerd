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

let current: Selected = { kind: 'job' };
const watchers = new Set<Watcher>();

export const selected = (): Selected => current;

export function select(next: Selected): void {
  current = next;
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

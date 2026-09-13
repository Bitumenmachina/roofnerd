// ── The cut that is on screen ──────────────────────────────────────────────
// Which section the estimator is looking at, and what it went through.
//
// The Section view draws it and the Estimate names the lines it passes through —
// two regions of the one window, so they need somewhere to agree. That is this
// file, and it is the smallest thing that does the job: a value and a set of
// watchers, the same shape `selection.ts` has.
//
// It is NOT said out loud on the event bus the way a selection is, and that is
// deliberate. A selection is a fact about the job that every window shares; a
// cut is a fact about one window's surface, and a cost panel torn off onto the
// other monitor has no Section view in it to be showing. Telling it about a cut
// it cannot see would put a banner over money that had nothing to do with
// anything on that screen.

/** The cut on screen, and the conditions its plane passes through. */
export interface Cut {
  /** What it is called on the drawing: "Cut A–A". */
  readonly named: string;
  /** Which conditions the plane goes through, by id. */
  readonly through: readonly string[];
}

type Watcher = (cut: Cut | null) => void;

let current: Cut | null = null;
const watchers = new Set<Watcher>();

export const cutOnScreen = (): Cut | null => current;

/** The Section view says what it is showing. Null when it is put down. */
export function showingCut(next: Cut | null): void {
  current = next;
  for (const w of watchers) w(current);
}

export function watchCut(w: Watcher): () => void {
  watchers.add(w);
  w(current);
  return () => watchers.delete(w);
}

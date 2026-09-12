// ── Which theme the room is in ─────────────────────────────────────────────
// Light is the default and stays the default (Q5). Dark is the design
// handoff's palette, and both are the same set of token names in `app.css` —
// so the toggle sets one attribute on the root element and nothing else in the
// program knows or cares which one is up.
//
// It is remembered in `localStorage` rather than in the job: which theme an
// estimator works in is a fact about the estimator, not about the bid, and
// writing it into the file would put it in a diff every time anybody flipped
// it. `localStorage` rather than `sessionStorage` because it is shared by every
// window of this program, so a window opened later comes up in the same room.

import { emit, listen } from '@tauri-apps/api/event';

export type Theme = 'light' | 'dark';

const REMEMBERED = 'roofnerd:theme';
const CHANGED = 'theme:changed';

/** This window, so it can ignore its own message coming back round. */
const ME = `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 10)}`;

const watchers = new Set<(t: Theme) => void>();

function remembered(): Theme {
  try {
    return localStorage.getItem(REMEMBERED) === 'dark' ? 'dark' : 'light';
  } catch {
    // A window that cannot remember still works; it comes up light.
    return 'light';
  }
}

let current: Theme = remembered();

export const theme = (): Theme => current;

function paint(next: Theme): void {
  current = next;
  // Light is the default, so it is the absence of the attribute rather than a
  // second value to keep in step with the stylesheet.
  if (next === 'dark') document.documentElement.setAttribute('data-theme', 'dark');
  else document.documentElement.removeAttribute('data-theme');
  for (const w of watchers) w(next);
}

/** Put the remembered theme up. Called once, before anything is drawn. */
export function applyTheme(): void {
  paint(current);
}

export function setTheme(next: Theme): void {
  try {
    localStorage.setItem(REMEMBERED, next);
  } catch {
    // Not being able to remember it is not a reason to refuse it.
  }
  paint(next);
  void emit(CHANGED, { from: ME, theme: next }).catch(() => undefined);
}

export const flipTheme = (): Theme => {
  const next: Theme = current === 'dark' ? 'light' : 'dark';
  setTheme(next);
  return next;
};

/** Follow the other windows, the same way selection does. Never emits back. */
export async function followTheme(): Promise<void> {
  try {
    await listen<{ from: string; theme: Theme }>(CHANGED, (event) => {
      const said = event.payload;
      if (!said || said.from === ME || !said.theme) return;
      paint(said.theme);
    });
  } catch {
    // A window that cannot hear the others keeps its own.
  }
}

export function watchTheme(w: (t: Theme) => void): () => void {
  watchers.add(w);
  w(current);
  return () => watchers.delete(w);
}

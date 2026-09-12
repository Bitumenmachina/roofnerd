// ── A trace, in feet ───────────────────────────────────────────────────────
// A trace is stored in page units, never in feet: zoom in and the numbers do not
// move, which is the difference between a takeoff you can come back to and one
// that was only ever right at one zoom (`viewer/surface.ts`). Feet come from the
// page the trace was drawn on, and a page that nobody has scaled has none.
//
// This lived inside the Model editor, where the IFC writer could not reach it
// without dragging three.js and the window in behind it. It reads the job and
// nothing else — no shell, no document plumbing, no drawing — so both can use
// the one copy. Five copies of the job's shape were deleted from this package
// once already; this is the same lesson, one file earlier.

import type { Page, Point, Trace } from '@roofnerd/engine';

/** Feet to the page unit, per page. A page with no scale is simply absent. */
export function feetPerUnit(pages: readonly Page[]): Record<string, number> {
  const out: Record<string, number> = {};
  for (const p of pages) {
    if (p.feetPerUnit) out[p.id] = p.feetPerUnit;
  }
  return out;
}

/**
 * A trace's points in feet.
 *
 * Empty when its sheet has no scale — never zero. An unscaled trace has no
 * length, and a zero would quietly become a zero-foot wall.
 */
export function feet(trace: Trace, scales: Record<string, number>): Point[] {
  const s = scales[trace.pageId];
  if (!s) return [];
  return trace.points.map((p) => ({ x: p.x * s, y: p.y * s }));
}

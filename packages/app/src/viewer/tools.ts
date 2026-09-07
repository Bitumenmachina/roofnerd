// ── The tools ──────────────────────────────────────────────────────────────
// Four things an estimator does on a sheet: outline an area, run a line, drop a
// count, and tell the program how big the sheet is.
//
// All three trace tools work the same way, because that is what a hand expects:
// click to put a point down, keep clicking, and finish with a double-click or
// Enter. Escape throws the whole shape away; Backspace takes back the last
// point. Holding Shift keeps the segment square to the sheet.

import type { Point } from '@roofnerd/engine';
import { distance, polygonArea, polylineLength, polygonPerimeter } from '@roofnerd/engine';
import { Surface, svg, pointsAttribute } from './surface.js';

export type ToolName = 'select' | 'area' | 'line' | 'count' | 'scale';

/** Snap to a nearby point already on the sheet, within this many screen pixels. */
const SNAP_PIXELS = 10;

export interface ToolHost {
  /** Points already on the sheet, for snapping to. */
  existingPoints(): readonly Point[];
  /** A finished trace. */
  onTrace(kind: 'area' | 'line' | 'count', points: readonly Point[]): void;
  /** Two points picked for a scale, and how far apart they are on the sheet. */
  onScale(a: Point, b: Point): void;
  /** Something for the status line — the running length, or what to do next. */
  onHint(text: string): void;
}

export class Tools {
  private tool: ToolName = 'select';
  private points: Point[] = [];
  private cursor: Point | null = null;
  private orthogonal = false;

  constructor(private readonly surface: Surface, private readonly host: ToolHost) {
    surface.on({
      onDown: (page, event) => this.down(page, event),
      onMove: (page, event) => this.move(page, event),
    });
    window.addEventListener('keydown', (e) => this.key(e));
    window.addEventListener('keyup', (e) => {
      if (e.key === 'Shift') { this.orthogonal = false; this.draw(); }
    });
  }

  get current(): ToolName {
    return this.tool;
  }

  use(tool: ToolName): void {
    this.tool = tool;
    this.abandon();
    this.host.onHint(HINTS[tool]);
  }

  /** Throw away the shape in progress without recording it. */
  abandon(): void {
    this.points = [];
    this.surface.scratch.replaceChildren();
  }

  private down(page: Point, event: PointerEvent): void {
    if (this.tool === 'select') return;
    const point = this.snap(this.constrain(page));

    // A double-click finishes rather than adding a point on top of the last one.
    if (event.detail > 1 && this.tool !== 'count') {
      this.finish();
      return;
    }

    if (this.tool === 'count') {
      this.host.onTrace('count', [point]);
      return;
    }

    this.points.push(point);

    if (this.tool === 'scale' && this.points.length === 2) {
      const [a, b] = this.points as [Point, Point];
      this.points = [];
      this.surface.scratch.replaceChildren();
      this.host.onScale(a, b);
      return;
    }

    this.draw();
  }

  private move(page: Point, event: PointerEvent): void {
    this.orthogonal = event.shiftKey;
    this.cursor = this.snap(this.constrain(page));
    if (this.points.length) this.draw();
  }

  private key(e: KeyboardEvent): void {
    if (e.key === 'Shift') { this.orthogonal = true; this.draw(); return; }
    if (!this.points.length) return;

    if (e.key === 'Escape') { this.abandon(); this.host.onHint(HINTS[this.tool]); e.preventDefault(); }
    else if (e.key === 'Enter') { this.finish(); e.preventDefault(); }
    else if (e.key === 'Backspace') { this.points.pop(); this.draw(); e.preventDefault(); }
  }

  private finish(): void {
    const kind = this.tool;
    if (kind !== 'area' && kind !== 'line') { this.abandon(); return; }

    // An area needs three corners; a run needs two ends. Below that there is
    // nothing to measure, so nothing is recorded.
    const enough = kind === 'area' ? 3 : 2;
    if (this.points.length >= enough) this.host.onTrace(kind, this.points);
    this.abandon();
    this.host.onHint(HINTS[kind]);
  }

  /** Hold Shift and the next segment squares up to the sheet. */
  private constrain(page: Point): Point {
    const last = this.points[this.points.length - 1];
    if (!this.orthogonal || !last) return page;
    return Math.abs(page.x - last.x) > Math.abs(page.y - last.y)
      ? { x: page.x, y: last.y }
      : { x: last.x, y: page.y };
  }

  /**
   * Pull the cursor onto a point already on the sheet when it is close enough.
   * Corners that ought to be the same corner become the same corner, which is
   * what stops a roof plan from leaking area at every junction.
   */
  private snap(page: Point): Point {
    const within = SNAP_PIXELS * this.surface.pageUnitsPerPixel;
    let best: Point | null = null;
    let bestDistance = within;

    const candidates = this.tool === 'scale' ? [] : this.host.existingPoints();
    for (const candidate of [...candidates, ...this.points]) {
      const d = distance(page, candidate);
      if (d < bestDistance) { bestDistance = d; best = candidate; }
    }
    return best ?? page;
  }

  /** Redraw the shape in progress. */
  private draw(): void {
    const scratch = this.surface.scratch;
    scratch.replaceChildren();
    if (!this.points.length) return;

    const preview = this.cursor ? [...this.points, this.cursor] : this.points;

    if (this.tool === 'area' && preview.length > 2) {
      scratch.append(svg('polygon', { points: pointsAttribute(preview), class: 'draft-area' }));
    } else {
      scratch.append(svg('polyline', { points: pointsAttribute(preview), class: 'draft-line' }));
    }

    for (const p of this.points) {
      scratch.append(svg('circle', { cx: p.x, cy: p.y, r: 3 * this.surface.pageUnitsPerPixel, class: 'draft-vertex' }));
    }

    this.host.onHint(this.progress(preview));
  }

  /** What the shape measures so far, in page units — the caller turns it into feet. */
  private progress(preview: readonly Point[]): string {
    if (this.tool === 'scale') return 'click the other end of a known dimension';
    const raw = this.tool === 'area'
      ? `area ${polygonArea(preview).toFixed(0)} · perimeter ${polygonPerimeter(preview).toFixed(0)} page units`
      : `${polylineLength(preview).toFixed(0)} page units`;
    return `${this.points.length} point(s) · ${raw} · Enter or double-click to finish, Escape to drop it`;
  }
}

const HINTS: Record<ToolName, string> = {
  select: 'click a trace to select it',
  area: 'click the corners of an area — Enter or double-click to close it',
  line: 'click along a run — Enter or double-click to end it',
  count: 'click each one',
  scale: 'click one end of a dimension you know',
};

// ── The tools ──────────────────────────────────────────────────────────────
// Four things an estimator does on a sheet: outline an area, run a line, drop a
// count, and tell the program how big the sheet is.
//
// All three trace tools work the same way, because that is what a hand expects:
// click to put a point down, keep clicking, and finish with a double-click or
// Enter. Escape throws the whole shape away; Backspace takes back the last
// point. Holding Shift keeps the segment square to the sheet.

import { plural } from '../labels.js';
import type { Point } from '@roofnerd/engine';
import { distance, polygonArea, polylineLength, polygonPerimeter } from '@roofnerd/engine';
import { Surface, svg, pointsAttribute } from './surface.js';

export type ToolName = 'select' | 'area' | 'line' | 'count' | 'scale' | 'pick';

/** Snap to a nearby point already on the sheet, within this many screen pixels. */
const SNAP_PIXELS = 10;

export interface ToolHost {
  /** Points already on the sheet, for snapping to. */
  existingPoints(): readonly Point[];
  /** A finished trace. */
  onTrace(kind: 'area' | 'line' | 'count', points: readonly Point[]): void;
  /** Two points picked for a scale, and how far apart they are on the sheet. */
  onScale(a: Point, b: Point): void;
  /**
   * A place picked on the sheet, for an act that is not tracing anything.
   *
   * "Set high and low", "Measure", "Box out a curb" and "Cut a section" put a
   * finger on the roof and nothing goes into the document at the click. So they
   * are not a trace tool with the recording taken out — they get their own,
   * which snaps and squares up like every other pick and records nothing.
   */
  onPick(page: Point): void;
  /** Escape, with an act up and places picked: drop them. */
  onPickCleared(): void;
  /** Something for the status line — the running length, or what to do next. */
  onHint(text: string): void;
  /**
   * How the shape in progress stands, every time it changes.
   *
   * In page units, because that is what this class knows: the caller has the
   * page's scale and turns it into square feet and linear feet. It is what the
   * tool settings strip reads ("Area so far 9,600 SF") and what puts the
   * "Measuring into" pill on the drawing and takes it away again.
   */
  onProgress(state: ToolProgress): void;
  /**
   * Where the pointer is on the sheet, in page units, on every move.
   *
   * The status strip reads it. It comes through here rather than off a second
   * listener because `Surface.on` holds one set of handlers, and two callers
   * both registering would mean whichever went second silently took the
   * pointer away from the first.
   */
  onCursor(page: Point | null): void;
  /**
   * A click with the Select act up.
   *
   * The event travels rather than the point: the caller reads what was HIT —
   * every shape on the drawing says which condition it belongs to — so the
   * answer agrees with the picture by construction instead of by arithmetic
   * over the same points a second time.
   */
  onSelectAt(event: PointerEvent): void;
}

export interface ToolProgress {
  readonly tool: ToolName;
  /** Corners put down so far. Zero means nothing is open. */
  readonly points: number;
  /** The area the shape encloses, in page units, when it encloses one. */
  readonly pageArea: number | null;
  /** How far along it runs, in page units. */
  readonly pageLength: number | null;
}

export class Tools {
  private tool: ToolName = 'select';
  private points: Point[] = [];
  private cursor: Point | null = null;
  private orthogonal = false;

  /** The two window listeners, held so they can be taken off again. */
  private readonly onKeyDown = (e: KeyboardEvent) => this.key(e);
  private readonly onKeyUp = (e: KeyboardEvent) => {
    if (e.key === 'Shift') { this.orthogonal = false; this.draw(); }
  };

  constructor(private readonly surface: Surface, private readonly host: ToolHost) {
    surface.on({
      onDown: (page, event) => this.down(page, event),
      onMove: (page, event) => this.move(page, event),
    });
    window.addEventListener('keydown', this.onKeyDown);
    window.addEventListener('keyup', this.onKeyUp);
  }

  /**
   * Give the keyboard back.
   *
   * The surface switches between Plan and Model in place now, so this class is
   * built and thrown away repeatedly in one page. Its listeners are on `window`
   * — they have to be, a keystroke is not aimed at the drawing — and a set left
   * behind would go on drawing into a scratch layer that is no longer in the
   * document. Two Tools both answering Enter is the shape of that defect.
   */
  destroy(): void {
    window.removeEventListener('keydown', this.onKeyDown);
    window.removeEventListener('keyup', this.onKeyUp);
    this.points = [];
  }

  get current(): ToolName {
    return this.tool;
  }

  /** Is a shape open? Escape belongs to the shape while one is. */
  get tracing(): boolean {
    return this.points.length > 0;
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
    this.report();
  }

  /** Close the shape and record it, the way Enter does. */
  close(): void {
    this.finish();
  }

  /**
   * A box, typed rather than traced: "40x30" off the corner already put down.
   *
   * VES has had this for years and an estimator who has used it does not want
   * a dialog. It goes right and down from the last corner on the sheet, which
   * is the direction a drawing reads in, and it is recorded as an ordinary
   * traced area — there is nothing special about it once it exists.
   */
  box(width: number, height: number): boolean {
    if (this.tool !== 'area') return false;
    const from = this.points[this.points.length - 1];
    if (!from || !(width > 0) || !(height > 0)) return false;
    this.host.onTrace('area', [
      from,
      { x: from.x + width, y: from.y },
      { x: from.x + width, y: from.y + height },
      { x: from.x, y: from.y + height },
    ]);
    this.abandon();
    this.host.onHint(HINTS[this.tool]);
    return true;
  }

  /**
   * A length, typed: carry the run on in the direction it is already going.
   *
   * It needs a direction, so it needs two corners already down. With one it
   * refuses and the caller says why — a keyed length that guessed a bearing
   * would put a parapet somewhere nobody asked for.
   */
  extend(length: number): boolean {
    if (this.tool !== 'line' && this.tool !== 'area') return false;
    const n = this.points.length;
    const a = this.points[n - 2];
    const b = this.points[n - 1];
    if (!a || !b || !(length > 0)) return false;
    const run = Math.hypot(b.x - a.x, b.y - a.y);
    if (run === 0) return false;
    this.points.push({
      x: b.x + ((b.x - a.x) / run) * length,
      y: b.y + ((b.y - a.y) / run) * length,
    });
    this.draw();
    return true;
  }

  private down(page: Point, event: PointerEvent): void {
    if (this.tool === 'select') { this.host.onSelectAt(event); return; }
    const point = this.snap(this.constrain(page));

    // An act that only picks places records nothing and draws nothing of its
    // own: what the picks MEAN is the act's, and what they look like on the
    // sheet is the view's.
    if (this.tool === 'pick') { this.host.onPick(point); return; }

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
    this.report();

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
    this.host.onCursor(this.cursor);
    if (this.points.length) this.draw();
  }

  private key(e: KeyboardEvent): void {
    if (e.key === 'Shift') { this.orthogonal = true; this.draw(); return; }
    // An act that picks places has no shape open, so the rung below never
    // reaches it. Escape still means back out of what has been picked.
    if (this.tool === 'pick' && e.key === 'Escape') {
      this.host.onPickCleared();
      e.preventDefault();
      return;
    }
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
    this.report();
  }

  /** What the shape measures right now, in page units, for the chrome. */
  private report(): void {
    const preview = this.cursor ? [...this.points, this.cursor] : this.points;
    const enough = preview.length > 2;
    this.host.onProgress({
      tool: this.tool,
      points: this.points.length,
      pageArea: this.tool === 'area' && enough ? polygonArea(preview) : null,
      pageLength: this.points.length > 1
        ? (this.tool === 'area' ? polygonPerimeter(preview) : polylineLength(preview))
        : null,
    });
  }

  /** What the shape measures so far, in page units — the caller turns it into feet. */
  private progress(preview: readonly Point[]): string {
    if (this.tool === 'scale') return 'click the other end of a known dimension';
    const raw = this.tool === 'area'
      ? `area ${polygonArea(preview).toFixed(0)} · perimeter ${polygonPerimeter(preview).toFixed(0)} page units`
      : `${polylineLength(preview).toFixed(0)} page units`;
    return `${plural(this.points.length, 'point')} · ${raw} · Enter or double-click to finish, Escape to drop it`;
  }
}

const HINTS: Record<ToolName, string> = {
  select: 'click a trace to select it — scroll to move, Ctrl+scroll to zoom',
  area: 'click the corners of an area — Enter or double-click to close it',
  line: 'click along a run — Enter or double-click to end it',
  count: 'click each one',
  scale: 'click one end of a dimension you know',
  pick: 'click a place on the roof — Escape clears what you picked',
};

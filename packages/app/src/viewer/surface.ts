// ── The trace surface ──────────────────────────────────────────────────────
// The drawing, and the shapes drawn on top of it.
//
// Two layers, deliberately: the sheet is painted into a canvas, and the traces
// live in an SVG over it. That split is why a trace stays crisp at any zoom and
// why picking one is a matter of asking the browser what is under the cursor
// rather than hit-testing pixels.
//
// Coordinates: every point that gets stored is in PAGE units, never screen
// pixels. Zoom in and the numbers do not move. That is the difference between a
// takeoff you can come back to and one that was only ever right at one zoom.

import type { Point } from '@roofnerd/engine';
import type { Sheet } from './page-source.js';

const SVG_NS = 'http://www.w3.org/2000/svg';

export interface SurfaceEvents {
  /** The cursor moved, in page units. */
  onMove?(page: Point, event: PointerEvent): void;
  onDown?(page: Point, event: PointerEvent): void;
  /** A trace was clicked. */
  onPick?(traceId: string, conditionId: string): void;
  onZoom?(zoom: number): void;
}

export class Surface {
  readonly root: HTMLDivElement;
  private readonly scroller: HTMLDivElement;
  private readonly stack: HTMLDivElement;
  private readonly canvas: HTMLCanvasElement;
  readonly overlay: SVGSVGElement;
  /** Where the tool in use draws its work in progress. Cleared between strokes. */
  readonly scratch: SVGGElement;
  /** Where finished traces are drawn. */
  readonly shapes: SVGGElement;

  private sheet: Sheet | null = null;
  private zoomLevel = 1;
  /** Paints run one at a time and in order, however fast the zoom is worked. */
  private painting: Promise<void> = Promise.resolve();
  private events: SurfaceEvents = {};
  private panning: { x: number; y: number; left: number; top: number } | null = null;

  constructor() {
    this.root = document.createElement('div');
    this.root.className = 'surface';

    this.scroller = document.createElement('div');
    this.scroller.className = 'surface-scroll';

    this.stack = document.createElement('div');
    this.stack.className = 'surface-stack';

    this.canvas = document.createElement('canvas');
    this.canvas.className = 'surface-sheet';

    this.overlay = document.createElementNS(SVG_NS, 'svg');
    this.overlay.classList.add('surface-overlay');
    this.shapes = document.createElementNS(SVG_NS, 'g');
    this.scratch = document.createElementNS(SVG_NS, 'g');
    this.overlay.append(this.shapes, this.scratch);

    this.stack.append(this.canvas, this.overlay);
    this.scroller.append(this.stack);
    this.root.append(this.scroller);

    this.wire();
  }

  on(events: SurfaceEvents): void {
    this.events = events;
  }

  get zoom(): number {
    return this.zoomLevel;
  }

  /** Page units in one paper inch — what an architectural scale preset needs. */
  get unitsPerInch(): number {
    return this.sheet?.unitsPerInch ?? 72;
  }

  get hasSheet(): boolean {
    return this.sheet !== null;
  }

  async show(sheet: Sheet, zoom = this.zoomLevel): Promise<void> {
    this.sheet = sheet;
    await this.setZoom(zoom);
  }

  clear(): void {
    this.sheet = null;
    const context = this.canvas.getContext('2d');
    context?.clearRect(0, 0, this.canvas.width, this.canvas.height);
    this.canvas.width = 0;
    this.canvas.height = 0;
    this.shapes.replaceChildren();
    this.scratch.replaceChildren();
  }

  async setZoom(zoom: number): Promise<void> {
    this.zoomLevel = Math.min(8, Math.max(0.1, zoom));
    const sheet = this.sheet;
    if (!sheet) return;

    // The overlay is sized in PAGE units and stretched over the painted sheet,
    // so everything drawn into it is written in page coordinates directly. It
    // is set BEFORE the paint: a trace must be pickable and correctly placed
    // even while a big sheet is still rendering underneath it.
    const zoomAtCall = this.zoomLevel;
    this.overlay.setAttribute('viewBox', `0 0 ${sheet.width} ${sheet.height}`);
    const width = Math.floor(sheet.width * zoomAtCall);
    const height = Math.floor(sheet.height * zoomAtCall);
    this.overlay.setAttribute('width', String(width));
    this.overlay.setAttribute('height', String(height));
    this.stack.style.width = `${width}px`;
    this.stack.style.height = `${height}px`;

    // Keep strokes a constant width on screen however far in the estimator is.
    this.overlay.style.setProperty('--hair', String(1 / zoomAtCall));
    this.events.onZoom?.(zoomAtCall);

    this.painting = this.painting
      .catch(() => undefined)
      .then(() => sheet.render(this.canvas, zoomAtCall));
    await this.painting;
  }

  /** Fit the whole sheet in the window — where an estimator starts. */
  async fit(): Promise<void> {
    if (!this.sheet) return;
    const box = this.scroller.getBoundingClientRect();
    const margin = 24;
    const zoom = Math.min(
      (box.width - margin) / this.sheet.width,
      (box.height - margin) / this.sheet.height,
    );
    await this.setZoom(zoom);
  }

  /**
   * Screen point → page units. Everything stored goes through here.
   *
   * Measured off the overlay, not the canvas: the overlay is sized the moment
   * the zoom changes, while the canvas only catches up when the paint finishes.
   * Taking it off the canvas would put a click in the wrong place on a big sheet.
   */
  toPage(event: { clientX: number; clientY: number }): Point {
    const box = this.overlay.getBoundingClientRect();
    return {
      x: (event.clientX - box.left) / this.zoomLevel,
      y: (event.clientY - box.top) / this.zoomLevel,
    };
  }

  /** How many page units make up one screen pixel — for snap distances. */
  get pageUnitsPerPixel(): number {
    return 1 / this.zoomLevel;
  }

  private wire(): void {
    this.overlay.addEventListener('pointermove', (e) => {
      if (this.panning) {
        this.scroller.scrollLeft = this.panning.left - (e.clientX - this.panning.x);
        this.scroller.scrollTop = this.panning.top - (e.clientY - this.panning.y);
        return;
      }
      this.events.onMove?.(this.toPage(e), e);
    });

    this.overlay.addEventListener('pointerdown', (e) => {
      // Middle button, or space held, pans. The left button belongs to the tool.
      if (e.button === 1 || e.shiftKey && e.button === 0 && !this.events.onDown) {
        this.panning = {
          x: e.clientX, y: e.clientY,
          left: this.scroller.scrollLeft, top: this.scroller.scrollTop,
        };
        this.overlay.setPointerCapture(e.pointerId);
        e.preventDefault();
        return;
      }
      if (e.button !== 0) return;
      this.events.onDown?.(this.toPage(e), e);
    });

    const stopPan = (e: PointerEvent) => {
      if (!this.panning) return;
      this.panning = null;
      this.overlay.releasePointerCapture(e.pointerId);
    };
    this.overlay.addEventListener('pointerup', stopPan);
    this.overlay.addEventListener('pointercancel', stopPan);

    // Ctrl and the wheel zooms, which is what every drawing program does.
    this.scroller.addEventListener('wheel', (e) => {
      if (!e.ctrlKey) return;
      e.preventDefault();
      void this.setZoom(this.zoomLevel * (e.deltaY < 0 ? 1.15 : 1 / 1.15));
    }, { passive: false });
  }
}

/** Make an SVG element with attributes, since doing it by hand is four lines every time. */
export function svg<K extends keyof SVGElementTagNameMap>(
  tag: K,
  attributes: Record<string, string | number>,
): SVGElementTagNameMap[K] {
  const element = document.createElementNS(SVG_NS, tag);
  for (const [name, value] of Object.entries(attributes)) {
    element.setAttribute(name, String(value));
  }
  return element;
}

export const pointsAttribute = (points: readonly Point[]): string =>
  points.map((p) => `${p.x},${p.y}`).join(' ');

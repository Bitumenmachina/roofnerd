// ── The Section view ───────────────────────────────────────────────────────
// The roof, cut open at a drain.
//
// The handoff's Frame 3, and the point of it is that a tapered package is bought
// as boards and nobody can see a board in a plan or in a shaded surface. A
// section shows the stack: the deck it lands on, the flat board by the course,
// the lettered panel with its own letter on it, the sump cut down to the bowl,
// and the wall it stops against with the flashing, the coping, the cleat and the
// bar. Every one of those is a line on the estimate, so the drawing and the money
// are two readings of one thing.
//
// TWO PANES OF ONE ROOF. On the left the solved roof with a clipping plane
// through the drain and the face it exposes drawn in — the SAME three.js world
// the Model view stands up, mounted here with a cut (`mountModel`'s options), so
// there is one place that knows how to draw a facet. On the right the section
// itself, drawn through the one drawing renderer the Plan's overlays use (D155).
//
// NOTHING HERE COMPUTES. The cut, where it runs, what it passes through, the
// layers, the letters and every dimension on it are solved in
// `src-tauri/core/src/drawing.rs` and arrive through `solve_get("drawing")`. This
// file chooses which cut is showing and paints. The vertical is stretched four
// times and the drawing says so on itself: a board is half an inch on a run of
// twenty-two feet, and at true scale the whole build-up is a line.

import { NOTHING_TO_DRAW, type Drawing, type Drawings } from '@roofnerd/engine';
import { solveDrawings, solveId, subscribe } from '../doc.js';
import { cutWanted, showingCut, watchWantedCut } from '../cut.js';
import { stripField, type RailTool, type SurfaceChrome } from '../chrome.js';
import { extentOf, fitDrawing, renderDrawing, spreadWords } from '../viewer/drawing.js';
import { mountModel, recutModel, unmountModel } from './model.js';

const SVG_NS = 'http://www.w3.org/2000/svg';

/**
 * A CSS pixel is a ninety-sixth of an inch, so 1" = 4'-0" is twenty-four pixels
 * to the foot. The drawing states the scale; this is what the state means on a
 * screen.
 */
const PIXELS_PER_INCH = 96;

/** How much white there is round the drawing, in pixels. */
const MARGIN = 22;

/**
 * The scales a section may be drawn at, in feet to the inch, biggest drawing
 * first.
 *
 * An architect's rule and nothing else: 3" = 1'-0" down to 1/16" = 1'-0". The
 * pane picks the first of them the drawing fits in, so a cut through a short
 * run fills its sheet at 1/2" = 1'-0" and a cut across the building comes down
 * the ladder until it fits — and whichever it lands on is a scale somebody can
 * put a rule on, which is the whole reason a drawing states one.
 *
 * Before this the drawing was always laid out at the core's 1" = 4'-0": a short
 * cut sat in the middle of a pane a third empty, and a long one was too big and
 * got squashed to fit, under a note that admitted it — "brought down to fit the
 * window" — which is a drawing stating a scale it is not drawn at (C8 §8).
 */
const SCALES_FEET_PER_INCH = [1 / 3, 2 / 3, 1, 4 / 3, 2, 8 / 3, 4, 16 / 3, 8, 32 / 3, 16];

let chrome: SurfaceChrome | null = null;
let stopSubscribing: (() => void) | null = null;
let drawings: Drawings = NOTHING_TO_DRAW;
/** Which cut is showing, by its place in the list. */
let showing = 0;
let drawnSolveId = -1;
let pulling = 0;
/** Why the drawings did not arrive, when they did not. */
let trouble: string | null = null;
let paint: () => void = () => undefined;
let refreshChips: () => void = () => undefined;

/**
 * The Section's own acts. One, and it is the same one the Model has: pointing at
 * the roof picks what you are looking at.
 *
 * "Cut a section" is not offered HERE. It is a pick on the roof, so it belongs
 * where the roof is — the Plan and the Model both carry it — and this view is
 * where its answer lands.
 */
export const sectionTools = (pick: (id: string) => void): readonly RailTool[] =>
  [{ id: 'select', label: 'Select', icon: 'select', onPick: () => pick('select') }];

export function mountSection(host: HTMLElement, surroundings: SurfaceChrome): void {
  unmountSection();
  chrome = surroundings;
  host.replaceChildren();
  host.className = 'editor-host section-editor';
  // An act asked for a cut before this view was mounted: honour it on the way
  // up rather than opening on the first one and jumping a moment later.
  showing = cutWanted() ?? 0;
  drawnSolveId = -1;
  drawings = NOTHING_TO_DRAW;
  trouble = null;

  // The roof, cut. It is the Model's own world with a plane through it, so the
  // facets, the boards, the sumps and the crickets are drawn by the one file
  // that knows how — and the face the plane exposes is the section drawing
  // itself, stood back up where it was cut from.
  const cutPane = document.createElement('div');
  cutPane.className = 'section-cut';

  // And the section beside it, at the scale it states.
  const sheet = document.createElement('div');
  sheet.className = 'section-drawing';
  const surface = document.createElementNS(SVG_NS, 'svg');
  surface.classList.add('drawing');
  const marks = document.createElementNS(SVG_NS, 'g');
  surface.append(marks);
  const note = document.createElement('p');
  note.className = 'drawing-note';
  const nothing = document.createElement('p');
  nothing.className = 'drawing-nothing';
  nothing.hidden = true;
  sheet.append(surface, note, nothing);

  host.append(cutPane, sheet);

  surroundings.setTools(sectionTools(() => undefined));
  surroundings.setActiveTool('select');

  paint = () => {
    const cut = drawings.sections[showing];
    if (!cut) {
      marks.replaceChildren();
      note.textContent = '';
      nothing.hidden = false;
      nothing.textContent = trouble
        ? `This roof did not solve, so there is nothing to cut. ${trouble}`
        : 'There is no drain on a tapered area yet, so there is nothing to cut through.'
          + ' Drop a drain on the Plan and the section appears here.';
      showingCut(null);
      unmountModel();
      cutPane.replaceChildren();
      return;
    }
    nothing.hidden = true;

    // Down the rule until it fits, and then it is drawn at that and says so.
    //
    // The room is the pane less the footer note, which sits over the bottom of
    // it: the drain's own label was landing on that sentence because nothing
    // reserved the band it occupies.
    const room = sheet.getBoundingClientRect();
    const footer = note.getBoundingClientRect().height || 0;
    const wide = room.width - MARGIN;
    const tall = room.height - footer - MARGIN;
    let drawn = null as ReturnType<typeof renderDrawing> | null;
    let feetPerInch = SCALES_FEET_PER_INCH[SCALES_FEET_PER_INCH.length - 1]!;
    for (const scale of SCALES_FEET_PER_INCH) {
      const perFoot = PIXELS_PER_INCH / scale;
      drawn = renderDrawing(marks, cut, {
        perFootX: perFoot,
        perFootY: perFoot * cut.verticalExaggeration,
        wordSize: 11,
        letterSize: 13,
      });
      spreadWords(marks);
      feetPerInch = scale;
      const box = extentOf(marks);
      if (box && box.width + MARGIN <= wide && box.height + MARGIN <= tall) break;
    }
    const { drawnToFit } = fitDrawing(surface, marks, MARGIN / 2, footer);
    // The horizontal is the PANE's to state, because the pane is what knows how
    // much room there is; everything else on the line is the drawing's own.
    const said = `${cut.title} · horizontal 1" = ${feetInches(feetPerInch)} · ${drawn!.note}`;
    note.textContent = drawnToFit ? `${said} · brought down to fit the window` : said;

    // The same cut, in three dimensions, beside it.
    if (!cutPane.firstElementChild) {
      mountModel(cutPane, surroundings, { cut, ownsChrome: false });
    } else {
      recutModel(cut);
    }

    // And the money is told what the plane went through, so the Estimate can
    // name the lines this cut is of.
    showingCut({ named: cut.title, through: cut.through });
    drawStrip(cut);
    viewChips();
  };

  const refresh = async () => {
    const want = solveId();
    if (want !== drawnSolveId) {
      const token = pulling + 1;
      pulling = token;
      try {
        const got = await solveDrawings();
        if (token !== pulling) return;
        drawings = got;
        drawnSolveId = got.solveId;
        trouble = null;
      } catch (e) {
        if (token !== pulling) return;
        drawings = NOTHING_TO_DRAW;
        drawnSolveId = want;
        trouble = e instanceof Error ? e.message : String(e);
      }
      if (showing >= drawings.sections.length) showing = 0;
    }
    paint();
  };

  stopSubscribing = subscribe(() => void refresh());
  stopWatchingWanted = watchWantedCut((index) => {
    if (index === showing) return;
    showing = index;
    if (drawings.sections[showing]) paint();
  });
  // A window that opened AFTER the job did has heard no change, so it has no
  // solve id to chase. Ask once on the way up.
  void refresh();

  // The scale it can be drawn at depends on how much room there is, so a
  // resized window is a repaint. Guarded on the box actually changing: the fit
  // sets the drawing's own width and height, and an observer that fired on that
  // would paint forever.
  let wasWide = 0;
  let wasTall = 0;
  const watchBox = new ResizeObserver(() => {
    const box = sheet.getBoundingClientRect();
    if (Math.abs(box.width - wasWide) < 2 && Math.abs(box.height - wasTall) < 2) return;
    wasWide = box.width;
    wasTall = box.height;
    if (drawings.sections[showing]) paint();
  });
  watchBox.observe(sheet);
  boxWatch = () => watchBox.disconnect();

}

let boxWatch: (() => void) | null = null;
let stopWatchingWanted: (() => void) | null = null;

export function unmountSection(): void {
  unmountModel();
  stopSubscribing?.();
  stopWatchingWanted?.();
  stopSubscribing = null;
  stopWatchingWanted = null;
  boxWatch?.();
  boxWatch = null;
  paint = () => undefined;
  refreshChips = () => undefined;
  showingCut(null);
  chrome = null;
}

/**
 * One chip per cut, in the words on the drawing.
 *
 * A cut per drain, because a drain is where a section is taken: every one of
 * them has its own run, its own stack and its own end condition, and the one
 * through Drain 1 says nothing about the one that stops at a ridge. The handoff
 * draws this as "cut A–A of 3"; it is a chip apiece here, so the estimator can
 * see how many there are and go straight to the one they want.
 */
function viewChips(): void {
  if (!chrome) return;
  const chips: HTMLElement[] = [];
  const buttons: HTMLButtonElement[] = [];
  drawings.sections.forEach((cut, i) => {
    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'view-chip';
    button.textContent = cut.title.replace(/^Section /, 'cut ');
    button.addEventListener('click', () => {
      showing = i;
      paint();
    });
    buttons.push(button);
    chips.push(button);
  });
  refreshChips = () => {
    buttons.forEach((b, i) => b.classList.toggle('on', i === showing));
  };
  refreshChips();
  const group = document.createElement('div');
  group.className = 'chip-group';
  group.append(...chips);
  chrome.setViewTools([group]);
}

/** What the strip says about the cut that is up. */
function drawStrip(cut: Drawing): void {
  if (!chrome) return;
  chrome.strip.show('Cut a section', [
    stripField('Showing', cut.subtitle ?? cut.title),
    stripField('Vertical scale', `×${cut.verticalExaggeration}`),
    stripField('Horizontal', `1" = ${feetInches(cut.feetPerInch)}`),
    stripField('Run length', `${feetInches(cut.cutRunFeet)}`, undefined, true),
  ]);
}

/** Feet the way a roofer writes them. The core writes every other length. */
function feetInches(v: number): string {
  const whole = Math.floor(v);
  const inches = Math.round((v - whole) * 12);
  return inches === 12 ? `${whole + 1}'-0"` : `${whole}'-${inches}"`;
}

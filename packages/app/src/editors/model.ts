// ── The Model editor ───────────────────────────────────────────────────────
// The roof, stood up.
//
// Nothing here is modelled. Every surface in this view is derived from the same
// traces and properties that drive the estimate, which is the whole point: a 3D
// view built by hand is a second set of numbers, and a second set of numbers
// disagrees with the first one eventually. This one cannot, because it holds
// none of its own.
//
// What it draws, and what each thing is made of:
//
//   the datum   the traced polygon at its elevation, which is the top of
//               structural deck — the surface the whole build-up is
//               dimensioned from
//   the build-up what the layers on that condition say they are thick, stood
//               up from the datum. Absent when no layer says, because a roof
//               whose build-up nobody has stated is not a roof nine inches thick
//   a parapet   a line condition with a height, stood up off its base
//   a curb      the same, and said to be a curb, because a parapet is the edge
//               of the roof and a curb is a thing standing on it
//   the coping  formed metal over a wall or a curb, as wide as the detail says
//               and not as wide as the wall. A surface, not a plate: 24 ga is
//               twenty-eight thousandths of an inch
//   the taper   the field laid out from its drains: one drainage region per
//               drain, four planes falling from each, drawn as the facets those
//               planes govern
//   a sump      the square the field is cut down in at each drain, at the side
//               the condition states
//   a valley    where two planes of a region tie, at 45 degrees to the boards,
//               drawn on the surface it lies in rather than through it
//   a ridge     the boundary between two regions: equidistant between their
//               drains and square to the line joining them
//   a cricket   two planes falling from a ridge the drains put there, not a
//               ridge somebody drew
//   no fall     where the build-up has hit its ceiling and the roof goes flat,
//               so the water stops moving
//
// It is seen through a fixed isometric from one of four corners, with a fit and
// a vertical exaggeration and an orbit that is off until it is asked for. All
// four of those move the camera and nothing else.
//
// The conventions above are not this file's opinion. They live in the engine at
// `roof.ts`, each with the manual or the tool it came from, and they are tested
// there against the source rather than against this drawing. This file draws
// what they say. That order matters: it was the other way round once, and the
// check passed fourteen out of fourteen against geometry nobody had checked.
//
// WHERE THE GEOMETRY COMES FROM (C0, 2026-09-12; C1, D149). This file used to work
// the roof out. It does not any more: the regions, the planes, the facets, the
// valleys, the ridges, the no-fall ground and the crickets are solved in
// `roofnerd-core` and arrive through `solve_get("taper")` when the solve moves.
// C1 replaced the cone with the layout NRCA lays out, and this file changed by
// drawing the facets it is handed instead of a lattice of samples — which is the
// point of having put the solve behind a seam. Patrick's ruling is that the model
// is the function rather than a view — the same solved roof has to feed the
// Section, the tapered layout drawing and the estimate, and a second arithmetic
// in here is a second answer. So the drawing draws. The one number it still
// works out is where a vertex goes on the screen.
//
// The seam was proven by a change nobody can see: the picture at C0 is the
// picture before it, pixel for pixel, and `tools/probe/section-core.mjs` asks
// the window where its roof came from to tell the difference.
//
// There is no editing in this window. Clicking selects, and selecting is not
// editing — it is the same one selection every other editor watches.

import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import {
  CRICKET_SLOPE_MULTIPLE, MIN_FLASHING_INCHES, NOTHING_SOLVED, NOTHING_SOLVED_WALLS,
  SINGLE_LAYER_MAX_INCHES,
  buildUpInches, cricketOf, fieldAt, fieldOf, formatFeetInches, pointInPolygon, readFacetAt,
  readTaperAt, selfIntersects, sumpOf, wallOf,
  type CapReason, type Condition, type Drawing, type Page, type Point, type Properties,
  type SolvedCrease, type SolvedCricket, type SolvedRefusal, type SolvedRidge, type SolvedSump,
  type SolvedTaper, type SolvedTaperField, type SolvedTaperRegion, type SolvedWallRun,
  type SolvedWalls,
} from '@roofnerd/engine';
import { at, doc, solveId, solveTaper, solveWalls, subscribe, type Doc } from '../doc.js';
import { inches as inchLabel } from '../labels.js';
import { feet, feetPerUnit } from '../scale.js';
import { select, selectedConditionId, watchSelection } from '../selection.js';
import {
  derivedField, stripField,
  type RailTool, type Reading as StripReading, type SurfaceChrome,
} from '../chrome.js';
import { watchTheme } from '../theme.js';
import { hueFor } from '../icons.js';
import {
  VERB_LABELS, clearVerbPicks, isVerb, useVerb, verbMarks, verbNow, verbPick, verbStrip,
  watchVerb, type VerbName,
} from '../verbs.js';

// The four shapes this file draws from are the engine's, not its own. The copies
// that were here said `role?: 'drain' | 'ridge'` where the engine names the
// roles, and `items?: { thickness?: number }[]` — a line reduced to the one
// field this file reads, which is how a file ends up believing that is all a
// line has.

/** What the status strip says when nothing is under the pointer. */
const EMPTY_READING: readonly StripReading[] = [
  { key: 'Cursor', value: null },
  { key: 'Slope', value: null },
  { key: 'Insulation here', value: null },
  { key: 'Flow', value: null, water: true },
  { key: '', value: null },
];

/** Inches to feet, because a property is in inches and the world is in feet. */
const inches = (n: number) => n / 12;

// Where a tapered field stops rising, and why, is `capInches` in the engine.
// It used to be four inches a board here, which is not a number from anywhere:
// Carlisle runs a single layer to 4.5 and GAF to 4.6, and past that you buy
// another layer rather than the roof going flat. The ceiling an estimator
// actually hits is flashing height against a wall the building already has.

// Nothing here is sampled. The surface arrives as facets, each one exactly planar,
// so a corner lifted to its own depth is the roof and not an approximation of it.

/**
 * The thickness ramp, from the handoff's own token table: charcoal where the
 * roof is thin and grey where it is thick.
 *
 * It used to be the CONDITION's colour lightened toward white, which made the
 * roof read as a blue shape rather than as a roof and put the field's identity
 * colour — a thing for telling one traced condition from another — on a scale
 * that means a depth. Frame 2 shades by thickness in four greys and nothing
 * else, and it is right: the one loud colour on this drawing is the water.
 */
const RAMP = ['#2b3136', '#4a5359', '#7c878d', '#a9b4b9'] as const;

/** Where a fraction of the way up the ramp lands, as a colour. */
function rampAt(t: number): THREE.Color {
  const held = Math.min(1, Math.max(0, Number.isFinite(t) ? t : 0));
  const along = held * (RAMP.length - 1);
  const first = Math.min(RAMP.length - 2, Math.floor(along));
  return new THREE.Color(RAMP[first]!).lerp(new THREE.Color(RAMP[first + 1]!), along - first);
}

/**
 * A wall is masonry and reads as masonry.
 *
 * The handoff draws the parapet in three greys by face and its edges in a fourth
 * — stone, not the condition's identity colour, which belongs in the swatch
 * beside its name in the list. The light already separates the faces, so one
 * stone and a sun is the whole of it.
 */
const MASONRY = '#3b4247';

/** The dashes a valley and a ridge are drawn in: pale, and not the water's. */
const CREASE = '#cfd8dd';

/**
 * The four corners the roof is offered from.
 *
 * North is up the sheet, which is how a roof plan is read, so north is negative
 * z in the world and the compass below reads the way it does. Each one is a
 * direction to stand in, not a position: the distance comes off the roof.
 */
const CORNERS = {
  NE: { x: 1, z: -1 },
  NW: { x: -1, z: -1 },
  SE: { x: 1, z: 1 },
  SW: { x: -1, z: 1 },
} as const;

type Corner = keyof typeof CORNERS;

/** What the chips offer, in the words on them. */
const CORNER_LABEL: Record<Corner, string> = {
  NE: 'NE corner', NW: 'NW corner', SE: 'SE corner', SW: 'SW corner',
};

/** The two vertical exaggerations. A quarter in twelve is invisible at true scale. */
const EXAGGERATIONS = [1, 3] as const;

let scene: THREE.Scene;
let camera: THREE.OrthographicCamera;
/** Which corner the fixed view is taken from. */
let corner: Corner = 'NE';
/** How far the vertical is stretched. One is true scale. */
let exaggeration = 1;
/** Has the estimator asked to look around? Until then the camera is fixed. */
let looking = false;
/** Redraw the chips when one of the three above moves. */
let refreshChips: () => void = () => undefined;
let stopWatchingTheme: (() => void) | null = null;
let chrome: SurfaceChrome | null = null;
let renderer: THREE.WebGLRenderer | null = null;
let controls: OrbitControls | null = null;
let world: THREE.Group;
let frameHandle = 0;
let stopWatching: (() => void) | null = null;
/** Has the estimator taken the camera? Until then the view follows the roof. */
let framed = false;
/** Whether there is a roof to frame at all. A box change on nothing is nothing. */
let lastSpan = 0;
let stopObserving: (() => void) | null = null;
let stopSubscribing: (() => void) | null = null;
/** The section this drawing is cut along, or none — the Model view has none. */
let cutting: Drawing | null = null;
/** Does this mounting own the rail and the strip, or is it a pane in something? */
let ownsChrome = true;
/** The canvas this view is drawn on, for turning world points into screen ones. */
let surface: HTMLCanvasElement | null = null;

/**
 * The projection: a fixed isometric, from one of four corners.
 *
 * The design work asked for this and the reason is the roof itself. A quarter
 * inch in twelve is a fortieth of a degree; at true scale in perspective it is
 * not a slope, it is a flat grey field, and an estimator fighting an orbit
 * camera to find an angle where it reads is an estimator who stops opening this
 * view. So the camera does not move unless it is asked to: four corners, a fit,
 * and a vertical stretch that makes the fall visible without moving a vertex.
 *
 * Orthographic, and that is not a style. Two lengths the same on the roof have
 * to be the same on the screen or the picture cannot be measured off, which is
 * what perspective gives up. A direction with the same magnitude on all three
 * axes is what makes the two plan axes foreshorten equally — the isometric the
 * handoff draws, arrived at rather than approximated.
 *
 * The orbit is still here and still off. "Look around" turns it on for anybody
 * who wants to get behind a curb, and any of the presets puts it back.
 */
function projection(canvas: HTMLCanvasElement) {
  camera = new THREE.OrthographicCamera(-1, 1, 1, -1, -20_000, 20_000);
  camera.up.set(0, 1, 0);

  const orbit = new OrbitControls(camera, canvas);
  orbit.enableDamping = true;
  orbit.dampingFactor = 0.08;
  // Under the roof there is nothing to see and it is easy to get lost there.
  orbit.maxPolarAngle = Math.PI * 0.49;
  orbit.enabled = false;
  orbit.addEventListener('start', () => { framed = true; });
  return orbit;
}

/**
 * Stand the camera at the chosen corner and put the whole roof in the window.
 *
 * Called on open, on every document change until somebody takes the camera, on
 * every change to the box the canvas is in, and on each of the chips. The last
 * of those is why this is one function: a preset that framed differently from a
 * fit would be two cameras, and the two would disagree the first time a roof
 * was traced off centre.
 */
function frame() {
  if (!camera || !controls || !world) return;
  const box = new THREE.Box3().setFromObject(world);
  if (box.isEmpty()) return;
  const centre = box.getCenter(new THREE.Vector3());
  const span = Math.max(box.getSize(new THREE.Vector3()).length(), 20);

  const dir = CORNERS[corner];
  const from = new THREE.Vector3(dir.x, 1, dir.z).normalize().multiplyScalar(Math.max(span * 2, 80));
  controls.target.copy(centre);
  camera.position.copy(centre).add(from);
  camera.up.set(0, 1, 0);
  camera.lookAt(centre);
  camera.zoom = 1;
  controls.update();
  fitFrustum(box);
}

/**
 * Size the window on the world so the whole roof is in it, with a margin.
 *
 * The box's own corners, put through the camera rather than guessed at from the
 * span: a diagonal view of a long roof needs a wider window than a square one,
 * and a span-times-a-constant rule is how the roof ends up a postage stamp in a
 * wide window and cropped in a tall one. Both were on screen this week.
 */
function fitFrustum(known?: THREE.Box3) {
  if (!camera || !surface || !world) return;
  const box = known ?? new THREE.Box3().setFromObject(world);
  if (box.isEmpty()) return;
  const w = surface.clientWidth;
  const h = surface.clientHeight;
  if (w < 2 || h < 2) return;

  camera.updateMatrixWorld();
  // Computed here rather than read: `matrixWorldInverse` is filled in by the
  // renderer on its way to drawing a frame, so on the frame this is called from
  // it is whatever the last frame left. The roof fitted to the previous camera
  // is the small-roof defect wearing a different hat.
  const toCamera = camera.matrixWorld.clone().invert();
  let minX = Infinity; let maxX = -Infinity; let minY = Infinity; let maxY = -Infinity;
  const p = new THREE.Vector3();
  for (const x of [box.min.x, box.max.x]) {
    for (const y of [box.min.y, box.max.y]) {
      for (const z of [box.min.z, box.max.z]) {
        p.set(x, y, z).applyMatrix4(toCamera);
        minX = Math.min(minX, p.x); maxX = Math.max(maxX, p.x);
        minY = Math.min(minY, p.y); maxY = Math.max(maxY, p.y);
      }
    }
  }
  const MARGIN = 1.1;
  const halfW = Math.max(((maxX - minX) / 2) * MARGIN, 1);
  const halfH = Math.max(((maxY - minY) / 2) * MARGIN, 1);
  // Whichever of the two needs more room decides, so nothing is ever cut off.
  const scale = Math.max(halfW / (w / 2), halfH / (h / 2));
  const cx = (minX + maxX) / 2;
  const cy = (minY + maxY) / 2;
  camera.left = cx - (w / 2) * scale;
  camera.right = cx + (w / 2) * scale;
  camera.top = cy + (h / 2) * scale;
  camera.bottom = cy - (h / 2) * scale;
  camera.updateProjectionMatrix();
}

/** The ground the drawing lies on, in whichever room the estimator is in. */
function paintBackground() {
  if (!scene) return;
  const token = getComputedStyle(document.documentElement).getPropertyValue('--desk').trim();
  scene.background = new THREE.Color(token || '#e6e6ea');
}

/**
 * What the roof is doing under the pointer.
 *
 * Thickness, the fall, which drain the water reaches and how far it has to go.
 * All of it comes off the same arithmetic the surface is drawn from, so it can
 * never say something the picture does not.
 */
function readAt(canvas: HTMLCanvasElement, event: PointerEvent): readonly StripReading[] {
  const nothing = (why: string | null): readonly StripReading[] => [
    { key: 'Cursor', value: why },
    { key: 'Slope', value: null },
    { key: 'Insulation here', value: null },
    { key: 'Flow', value: null, water: true },
    { key: '', value: null },
  ];
  if (!camera || fieldsNow.length === 0) return nothing(null);
  const rect = canvas.getBoundingClientRect();
  raycaster.setFromCamera(new THREE.Vector2(
    ((event.clientX - rect.left) / rect.width) * 2 - 1,
    -((event.clientY - rect.top) / rect.height) * 2 + 1,
  ), camera);
  const hits = raycaster.intersectObjects(world.children, false);
  if (hits.length === 0) return nothing(null);

  // The roof under the pointer, not the first thing standing on it.
  //
  // A curb, the cricket behind it, a parapet and the coping on top of the
  // parapet all STAND ON the roof. An estimator pointing at one of them is
  // still pointing at a place on the roof, and the readout that answers there
  // with three em dashes reads as the program not knowing its own roof — which
  // is what it did: the pointer sat at the middle of the field and the strip
  // said "off the tapered field", because the ray struck the thing in front and
  // the answer was taken off that.
  //
  // The tapered surface is asked first on purpose. A point on it is inside the
  // field by construction, where a point on the deck can land on the polygon's
  // own edge, which is neither in nor out — one traced corner was enough to put
  // the whole reading out.
  const nearestOf = (kind: string) =>
    hits.find((h) => String(h.object.userData['kind'] ?? '') === kind);
  const hit = nearestOf('taper') ?? nearestOf('build-up') ?? nearestOf('datum') ?? hits[0]!;

  const p = { x: hit.point.x, y: hit.point.z };
  const where = `${formatFeetInches(p.x)}, ${formatFeetInches(p.y)}`;
  const field = fieldsNow.find((f) => pointInPolygon(p, f.ring));
  if (!field) {
    return [
      { key: 'Cursor', value: where },
      { key: 'Slope', value: null },
      { key: 'Insulation here', value: null },
      { key: 'Flow', value: 'off the tapered field', water: true },
      { key: '', value: null },
    ];
  }

  // The solved field, read where the pointer is: the greatest of the four planes
  // of whichever drainage region holds this place. Nothing is sampled and nothing
  // is recomputed — the planes were solved in Rust and this multiplies a run by a
  // slope, which is the one expression that exists in both languages.
  const here = readTaperAt(field, p);
  if (!here) {
    // Inside the trace and on no region, which the layout should not allow. Said
    // out loud rather than answered with a depth nobody solved.
    return [
      { key: 'Cursor', value: where },
      { key: 'Slope', value: `${inchLabel(field.slopeInchesPerFoot)} in 12` },
      { key: 'Insulation here', value: null },
      { key: 'Flow', value: 'no drainage region covers this point', warn: true },
      { key: '', value: null },
    ];
  }
  const thickness = here.thicknessInches;
  if (here.flat) {
    // No fall is the one thing on a low-slope roof that decides whether the
    // water leaves, so it says which ceiling was hit rather than only that one
    // was. Amber, and only here.
    const why = field.capReason === 'flashing'
      ? 'no fall — as high as the wall it flashes against allows'
      : 'no fall — the boards have run out here';
    return [
      { key: 'Cursor', value: where },
      { key: 'Slope', value: `${inchLabel(field.slopeInchesPerFoot)} in 12` },
      { key: 'Insulation here', value: inchLabel(thickness) },
      { key: 'Flow', value: why, warn: true },
      { key: '', value: null },
    ];
  }
  return [
    { key: 'Cursor', value: where },
    { key: 'Slope', value: `${inchLabel(field.slopeInchesPerFoot)} in 12` },
    { key: 'Insulation here', value: inchLabel(thickness) },
    { key: 'Flow', value: `runs ${compass(p, here.toward)}`, water: true },
    { key: '', value: `${here.runFeet.toFixed(1)} ft of run to the drain it falls to` },
  ];
}

/**
 * The Model's own acts, for the window's rail.
 *
 * Pointing at the roof to pick what you are looking at, and the four acts that
 * work ON the roof. Frame 2 is headed "Set high and low" and this is the view it
 * is headed over: the high side and the drain are places, and a place is easier
 * to put a finger on standing up than in plan. The other three are here for the
 * same reason — a ruler, a curb and a cut are all two picks on the roof.
 *
 * Not here: tracing an area, running a parapet, dropping a drain, scaling the
 * sheet. Those put something new on a DRAWING, and this view has no drawing
 * under it.
 */
export const modelTools = (pick: (id: string) => void): readonly RailTool[] => [
  { id: 'select', label: 'Select', icon: 'select', onPick: () => pick('select') },
  { id: 'high-low', label: VERB_LABELS['high-low'], icon: 'highLow', onPick: () => pick('high-low') },
  { id: 'curb', label: VERB_LABELS.curb, icon: 'curb', onPick: () => pick('curb') },
  { id: 'measure', label: VERB_LABELS.measure, icon: 'measure', onPick: () => pick('measure') },
  { id: 'section', label: VERB_LABELS.section, icon: 'section', onPick: () => pick('section') },
];

/** One chip, in the words Frame 2 writes on it. */
function chip(label: string, onPick: () => void): HTMLButtonElement {
  const button = document.createElement('button');
  button.type = 'button';
  button.className = 'view-chip';
  button.textContent = label;
  button.addEventListener('click', onPick);
  return button;
}

const chipGroup = (...chips: HTMLElement[]) => {
  const group = document.createElement('div');
  group.className = 'chip-group';
  group.append(...chips);
  return group;
};

/**
 * Getting around the roof: four corners, a fit, the vertical, and the orbit.
 *
 * Every one of these moves the camera and nothing else. That is the whole rule
 * of this view — a preset that changed a thickness or a slope would be a second
 * roof, and two roofs disagree. The vertical stretch is the one that looks like
 * an exception and is not: it stretches the picture, the vertices stay where the
 * heightfield put them, and the readout under the pointer goes on answering in
 * the inches the roof actually is.
 */
function viewChips(surroundings: SurfaceChrome): void {
  const corners: HTMLButtonElement[] = [];
  const verts: HTMLButtonElement[] = [];

  for (const id of Object.keys(CORNERS) as Corner[]) {
    corners.push(chip(CORNER_LABEL[id], () => {
      corner = id;
      looking = false;
      if (controls) controls.enabled = false;
      framed = false;
      frame();
      refreshChips();
    }));
  }

  const fit = chip('fit roof', () => { framed = false; frame(); refreshChips(); });

  for (const times of EXAGGERATIONS) {
    verts.push(chip(`vert \u00d7${times}`, () => {
      exaggeration = times;
      if (world) world.scale.set(1, times, 1);
      framed = false;
      frame();
      refreshChips();
    }));
  }

  // The board grid, beside the vertical, because it is the same kind of thing:
  // a way of looking at the one roof that changes nothing about it.
  const boards = chip('boards', () => {
    showBoards = !showBoards;
    redrawWorld();
    refreshChips();
  });

  const look = chip('look around', () => {
    looking = !looking;
    if (controls) controls.enabled = looking;
    refreshChips();
  });

  refreshChips = () => {
    boards.classList.toggle('on', showBoards);
    (Object.keys(CORNERS) as Corner[]).forEach((id, i) => {
      corners[i]!.classList.toggle('on', id === corner && !looking);
    });
    EXAGGERATIONS.forEach((times, i) => {
      verts[i]!.classList.toggle('on', times === exaggeration);
    });
    look.classList.toggle('on', looking);
  };
  refreshChips();

  surroundings.setViewTools([
    chipGroup(...corners), chipGroup(fit), chipGroup(...verts),
    chipGroup(boards), chipGroup(look),
  ]);
}

/** Draw the roof again with nothing re-pulled. A chip that changes what is on
 *  the roof and not what the roof IS needs no solve. */
let redrawWorld: () => void = () => undefined;

/**
 * What a view that is not the Model view wants of it.
 *
 * The Section stands the same roof up behind a clipping plane, and it is the
 * SAME roof: one solve, one set of meshes, one place that knows how to draw a
 * facet. Two copies of this file would be two roofs, and two roofs disagree.
 */
export interface ModelOptions {
  /**
   * A section drawing to cut the roof along. The plane is where the drawing says
   * the cut is, the face it exposes is the drawing's own polygons stood up, and
   * the vertical is stretched by the exaggeration the drawing states.
   */
  readonly cut?: Drawing | null;
  /** False where the view around this one owns the rail and the strip. */
  readonly ownsChrome?: boolean;
}

export function mountModel(
  host: HTMLElement, surroundings: SurfaceChrome, options: ModelOptions = {},
): void {
  unmount();
  chrome = surroundings;
  cutting = options.cut ?? null;
  ownsChrome = options.ownsChrome !== false;
  if (ownsChrome) {
    surroundings.setTools(modelTools((id) => chooseAct(id)));
    surroundings.setActiveTool(verbNow() ?? 'select');
    surroundings.setViewTools([]);
    surroundings.setReadout(EMPTY_READING);
    if (verbNow()) showVerbStrip();
    else {
      surroundings.strip.show('Select', [
        stripField('On the roof', 'click to pick what you are looking at'),
        stripField('Units', 'SF · LF · EA · SQ'),
      ]);
    }
  }
  framed = false;
  corner = 'NE';
  exaggeration = cutting ? cutting.verticalExaggeration : 1;
  looking = false;
  showBoards = false;
  labelsNow = [];
  host.replaceChildren();
  host.classList.add('model-editor');

  // One canvas, filling the surface. It carried a second one beside it for a
  // while — the same roof written out as IFC and read back through a viewer
  // toolkit, so the two could be held against each other by eye. What that
  // actually put on screen was a white slab next to the roof, taking half the
  // room the roof had. The IFC file is still written and still checked; it is
  // not a second picture in the window an estimator works in.
  const canvas = document.createElement('canvas');
  canvas.className = 'model-canvas';
  canvas.setAttribute('aria-label', 'The roof in three dimensions');
  surface = canvas;

  // Upper left, over the drawing, which is where Frame 2 puts them: what the
  // shading means, and what the marks on the roof are. An overlay rather than a
  // row underneath, because a row underneath is room taken off the roof.
  const overlay = document.createElement('div');
  overlay.className = 'model-overlay';

  const legend = document.createElement('div');
  legend.className = 'model-legend';

  const note = document.createElement('p');
  note.className = 'model-note';

  overlay.append(legend, note);

  // The words that stand on the roof. Their own layer over the canvas, because
  // they have to be readable at any camera and a mesh cannot be: a word lying on
  // a tapered plane in a fixed isometric is a word at an angle. Frame 2 draws
  // them upright, and so does every roof plan ever printed.
  const words = document.createElement('div');
  words.className = 'model-labels';
  labelHost = words;

  // What the roof does where the pointer is reads in the window's status strip
  // now, which is where the handoff puts it: a continuous readout under the
  // cursor, in the same region whatever the surface is showing. It used to be a
  // line of its own under this canvas, which meant the Plan had nowhere to put
  // the same four facts when it learns to compute them.
  host.append(canvas, words, overlay);

  viewChips(surroundings);

  renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  // A section is a plane through the roof with everything in front of it taken
  // away. three.js will take it away and will not cap what it cut, so the face
  // is drawn: the section drawing's own polygons, stood up at the plane.
  renderer.localClippingEnabled = cutting !== null;

  scene = new THREE.Scene();
  // The ground the drawing lies on is the room the estimator is in — the same
  // token the window itself paints its viewport with, so a dark room does not
  // leave one pale rectangle in it.
  stopWatchingTheme = watchTheme(() => paintBackground());

  // Flat light plus one sun. A tapered field reads by its shading, so the light
  // has to come from somewhere rather than being uniform.
  scene.add(new THREE.AmbientLight(0xffffff, 1.5));
  const sun = new THREE.DirectionalLight(0xffffff, 1.6);
  sun.position.set(-0.4, 1, 0.6);
  scene.add(sun);

  world = new THREE.Group();
  world.scale.set(1, exaggeration, 1);
  scene.add(world);

  /**
   * A rebuild that throws must never leave the last good roof on screen.
   *
   * This is the one failure mode in a drawing that yields wrong geometry rather
   * than a refusal. `build` clears the world and repopulates it; if it throws
   * half way, the render loop keeps painting whatever survived — a roof that is
   * partly the old job and partly the new one, with a legend beside it that
   * looks right. Nothing downstream can see that. No check, no probe and no
   * export would catch it, because every surface agrees perfectly on the wrong
   * roof.
   *
   * The prior lineage on this machine found exactly this with a PDF pane that
   * kept its last raster under a correctly-drawn overlay, and named it the only
   * failure in that build that produced wrong geometry instead of an error. Its
   * answer is the right one: empty the view and say so, loudly, in the window.
   */
  const draw = () => {
    let built: Built;
    try {
      built = build(doc(), solvedNow, solvedWallsNow);
    } catch (e) {
      world.clear();
      legend.replaceChildren();
      roofLabels = [];
      labelsNow = [];
      marks = null;
      words.replaceChildren();
      note.textContent = `This roof did not draw, so nothing is shown — what you would have seen would not have been this job. ${e instanceof Error ? e.message : String(e)}`;
      note.classList.add('model-broke');
      return;
    }
    note.classList.remove('model-broke');
    legend.replaceChildren(...built.legend);
    note.textContent = built.note;
    roofLabels = cutting ? [] : built.labels;
    labelsNow = roofLabels;
    // While an act is up the strip is the act's: it is what the estimator is
    // doing, and the readout of the roof as it stands is what they are doing it
    // to. Frame 2's six readings are in the act's own strip, in the same order.
    if (ownsChrome && !verbNow()) chrome?.strip.show('Select', built.strip);
    if (ownsChrome && verbNow()) showVerbStrip();
    drawMarks();
    if (solveTrouble) {
      // The roof did not solve. Nothing tapered is drawn, and the window says
      // why rather than showing a deck with no fall on it as though that were
      // the answer.
      note.textContent = `This roof did not solve, so the taper is not shown. ${solveTrouble}`;
      note.classList.add('model-broke');
    }
    if (!controls) controls = projection(canvas);
    // The plane has to be back on every material every time the world is rebuilt
    // — the meshes are new objects and the old materials went with the old ones.
    if (cutting) clipTo(cutting);
    lastSpan = built.span;
    resize(canvas);
    // Once they have looked around it is their camera. Until then the view
    // follows the roof, so tracing something new does not leave it off screen —
    // and so the roof is in the window the first time it is opened, in every
    // window it is opened in. A section looks at its own cut instead.
    if (!framed && built.span > 0) {
      if (cutting) aimAtCut(cutting);
      else frame();
    }
    controls.update();
  };

  /**
   * What is actually in the scene, read off the meshes.
   *
   * Not the bookkeeping `build` kept while it drew — the vertices themselves.
   * That distinction is the whole reason this exists: the old check for this
   * editor asked the legend whether there was a cricket, and the legend said yes
   * because the code had counted one. A check that reads its producer's own
   * tally measures nothing, and it passed fourteen out of fourteen doing it.
   *
   * So the check recomputes the trade's rules from the drawn geometry and the
   * document, and this hands it the geometry. Read-only, and nothing in the
   * program reads it — the job is still opened through the menu, traced through
   * the sheet, and read through the window.
   */
  (window as unknown as Record<string, unknown>)['__roofnerdModel'] = () => {
    const meshes = world.children.map((object) => {
      const mesh = object as THREE.Mesh;
      const box = new THREE.Box3().setFromObject(mesh);
      const kind = mesh.userData['kind'] ?? null;
      const position = mesh.geometry?.getAttribute('position');
      // The shapes whose construction is a trade convention travel whole, so the
      // check can work the convention out from the vertices instead of asking
      // the legend. A heightfield is thousands of points and is not one of them.
      const wholeShape = kind === 'cricket' || kind === 'parapet' || kind === 'parapet-ribbon';
      const material = mesh.material as THREE.MeshLambertMaterial | undefined;
      return {
        conditionId: mesh.userData['conditionId'] ?? null,
        kind,
        min: [box.min.x, box.min.y, box.min.z],
        max: [box.max.x, box.max.y, box.max.z],
        points: wholeShape && position ? Array.from(position.array as Float32Array) : null,
        // What `highlight()` actually did to this surface, read off the
        // material rather than off the selection it was given. A check that
        // reads back the state it just set measures nothing; this is the
        // picture's own answer to "is it lit".
        emissive: material?.emissive ? `#${material.emissive.getHexString()}` : null,
        emissiveIntensity: material?.emissiveIntensity ?? null,
      };
    });
    // Two answers hung on the same array, so the checks that already read it go
    // on reading it: `JSON.stringify` of an array carries the elements and
    // nothing else, and these are asked for by name.
    return Object.assign(meshes, {
      // Where this roof came from. Before C0 there was nothing to ask: the
      // drawing worked the roof out itself, so a check could not tell a view
      // that draws a solved model from one that solves its own.
      source: 'solve_get',
      solveId: drawnSolveId,
      selectedConditionId: selectedConditionId(),
      screenPointOf: (id: string) => screenPointOf(id),
      screenPointAt: (x: number, y: number) => screenPointAt(x, y),
      // Where the camera is standing and how the vertical is stretched, so a
      // check can hold a preset against what it claims to have changed. The
      // meshes above are the other half of that: a preset moves this and
      // nothing in the list.
      view: camera ? {
        corner,
        vert: exaggeration,
        looking,
        position: [camera.position.x, camera.position.y, camera.position.z],
        target: controls ? [controls.target.x, controls.target.y, controls.target.z] : null,
        frustum: [camera.left, camera.right, camera.top, camera.bottom],
        onScreen: roofOnScreen(),
      } : null,
    });
  };

  /**
   * Pull the solved roof when the solve moves, then draw.
   *
   * The document event carries the solve's id and the named numbers — hundreds
   * of bytes, which can ride a keystroke — and the geometry is pulled, because on
   * a real roof it is hundreds of kilobytes and must not. A pull that has been
   * overtaken by a later one is dropped rather than drawn: two changes in flight
   * would otherwise land in whichever order the shell answered in.
   */
  const refresh = async () => {
    const want = solveId();
    if (want !== drawnSolveId) {
      const token = pulling + 1;
      pulling = token;
      try {
        const [got, gotWalls] = await Promise.all([solveTaper(), solveWalls()]);
        if (token !== pulling) return;
        solvedNow = got;
        solvedWallsNow = gotWalls;
        drawnSolveId = got.solveId;
        solveTrouble = null;
      } catch (e) {
        if (token !== pulling) return;
        solvedNow = NOTHING_SOLVED;
        solvedWallsNow = NOTHING_SOLVED_WALLS;
        drawnSolveId = want;
        solveTrouble = e instanceof Error ? e.message : String(e);
      }
    }
    draw();
  };

  redrawWorld = () => draw();
  stopWatchingVerb = watchVerb(() => {
    drawMarks();
    if (ownsChrome) showVerbStrip();
  });
  stopSubscribing = subscribe(() => void refresh());
  stopWatching = watchSelection(() => highlight());

  canvas.addEventListener('click', (e) => {
    // An act that works on the roof takes the click: a pick is a place, not a
    // selection. With no act up, a click on the roof selects what it hit, which
    // is what this view has always done.
    // Only the view that owns the rail takes a pick: the Section stands this
    // same roof up behind a clipping plane, and a click in THAT pane is not an
    // act being carried out on the roof.
    if (ownsChrome && verbNow()) { pickPlace(canvas, e); return; }
    pick(canvas, e);
  });
  canvas.addEventListener('pointermove', (e) => chrome?.setReadout(readAt(canvas, e)));
  canvas.addEventListener('pointerleave', () => chrome?.setReadout(EMPTY_READING));
  window.addEventListener('resize', () => resize(canvas));

  // Watch the box, not the window.
  //
  // The Plan learned this first and the Model never got it: a `window` resize
  // listener misses every way the area changes size without the window doing —
  // the legend wrapping to a second row, a sheet torn off, an area re-laid. The
  // camera framing here is computed from the roof's span in feet rather than
  // from the canvas, so a stale box does not put the roof at the wrong scale;
  // what it does put wrong is the aspect and the render buffer, and until the
  // next document change nothing corrects it. Same fix as
  // `packages/app/src/viewer/surface.ts`: while nobody has taken the camera,
  // re-fit whenever the box changes, including the first time it gets a real one.
  const watchBox = new ResizeObserver(() => {
    resize(canvas);
    if (!framed && lastSpan > 0) frame();
  });
  watchBox.observe(canvas);
  stopObserving = () => watchBox.disconnect();

  const tick = () => {
    frameHandle = requestAnimationFrame(tick);
    controls?.update();
    if (renderer && camera) renderer.render(scene, camera);
    placeWords();
  };
  tick();
}

/**
 * Cut the roof along the plane the section states, and draw the face it exposes.
 *
 * Three things, and none of them is geometry this file worked out. The PLANE is
 * where the drawing says the cut is — a vertical plane through the drain, square
 * to the way the cut runs — and everything on the near side of it is taken away,
 * which is what a section IS. The FACE is the section drawing's own polygons,
 * stood up at that plane: the boards by the course, the lettered panel, the sump,
 * the wall. three.js caps nothing it clips, so without this the roof would be
 * hollow where it was cut, which is the one thing a section must not be. And the
 * CAMERA looks square at the plane from the side that was taken away, tipped a
 * little so the roof running away behind the cut reads as roof.
 *
 * The vertical stretch is the world's own — the same one the Model's chips set —
 * so the face and the roof behind it are stretched together and the picture has
 * one scale in it.
 */
function clipTo(drawing: Drawing) {
  if (!camera || !controls || !renderer || !world) return;
  const at = drawing.cutAt;
  const along = drawing.cutAlong;
  if (!at || !along) return;

  // Square to the way the cut runs, pointing at the half that is kept.
  const away = new THREE.Vector3(-along.y, 0, along.x).normalize();
  const through = new THREE.Vector3(at.x, drawing.cutElevation, at.y);
  const plane = new THREE.Plane().setFromNormalAndCoplanarPoint(away, through);

  world.traverse((object) => {
    const mesh = object as THREE.Mesh;
    const material = mesh.material as THREE.Material | THREE.Material[] | undefined;
    if (!material) return;
    // The face is ON the plane; clipping it against its own plane is a coin toss
    // decided by the last bit of a float.
    const planes = mesh.userData['kind'] === 'cut-face' ? null : [plane];
    for (const one of Array.isArray(material) ? material : [material]) {
      one.clippingPlanes = planes;
      one.needsUpdate = true;
    }
  });
}

/**
 * Stand where the half that was taken away used to be, and look at the cut.
 *
 * Square on, and a little above: a section looked at dead square is an
 * elevation, and an elevation drawn in three dimensions is a picture that could
 * have been a flat one. The window is sized on the FACE rather than on the whole
 * roof, because most of the roof is behind the plane and fitting to it would put
 * the cut in the corner of the pane as a sliver.
 */
function aimAtCut(drawing: Drawing) {
  if (!camera || !controls || !world) return;
  const at = drawing.cutAt;
  const along = drawing.cutAlong;
  if (!at || !along) return;
  const away = new THREE.Vector3(-along.y, 0, along.x).normalize();

  const face = new THREE.Box3();
  world.traverse((object) => {
    if (object.userData['kind'] === 'cut-face') face.expandByObject(object);
  });
  const middle = face.isEmpty()
    ? new THREE.Vector3(
      at.x - along.x * drawing.cutRunFeet * 0.5,
      drawing.cutElevation,
      at.y - along.y * drawing.cutRunFeet * 0.5,
    )
    : face.getCenter(new THREE.Vector3());

  const back = away.clone().negate().add(new THREE.Vector3(0, 0.35, 0)).normalize();
  const far = Math.max(drawing.cutRunFeet * 4, 120);
  controls.target.copy(middle);
  camera.position.copy(middle).add(back.multiplyScalar(far));
  camera.up.set(0, 1, 0);
  camera.lookAt(middle);
  camera.zoom = 1;
  controls.update();
  // A margin round the face, so the roof beyond the cut is visible as the thing
  // the cut was taken out of.
  if (!face.isEmpty()) {
    const wide = face.getSize(new THREE.Vector3()).length() * 0.35;
    face.expandByScalar(Math.max(wide, 2));
    fitFrustum(face);
  }
}

/**
 * Cut the roof somewhere else, without taking the view down and putting it up.
 *
 * A drain picker is pressed while the roof is on screen; rebuilding the renderer
 * for it would drop a WebGL context on the floor every time, and the roof itself
 * has not changed at all — only the plane through it has.
 */
export function recutModel(drawing: Drawing | null): void {
  cutting = drawing;
  if (renderer) renderer.localClippingEnabled = drawing !== null;
  if (drawing) exaggeration = drawing.verticalExaggeration;
  if (world) world.scale.set(1, exaggeration, 1);
  framed = false;
  redrawWorld();
}

/**
 * The cut face itself: the section drawing stood back up where it was cut from.
 *
 * The drawing is in its own frame — x from the far end of the cut to the drain,
 * y the height off the top of deck with up negative — so putting it back is two
 * lines of arithmetic and no decisions. Every polygon in it was cut in the core
 * off the solved planes and the boards C2 billed, so the face is the bill, seen
 * edge on.
 */
function cutFace(drawing: Drawing): THREE.Object3D[] {
  const at = drawing.cutAt;
  const along = drawing.cutAlong;
  if (!at || !along) return [];
  const away = { x: -along.y, y: along.x };
  const out: THREE.Object3D[] = [];

  // Forward of the plane by a hair, so the face is not clipped by the plane it
  // lies in. A thousandth of a foot is a hundredth of an inch on the roof.
  const forward = 0.02;
  const place = (x: number, y: number) => new THREE.Vector3(
    at.x - along.x * (drawing.cutRunFeet - x) - away.x * forward,
    drawing.cutElevation - y,
    at.y - along.y * (drawing.cutRunFeet - x) - away.y * forward,
  );

  for (const shape of drawing.shapes) {
    const ring = shape.shape.ring;
    if (ring.length < 3) continue;
    const flat = new THREE.Shape(ring.map((p) => new THREE.Vector2(p.x, p.y)));
    for (const hole of shape.shape.holes) {
      if (hole.length < 3) continue;
      flat.holes.push(new THREE.Path(hole.map((p) => new THREE.Vector2(p.x, p.y))));
    }
    const cut = new THREE.ShapeGeometry(flat);
    const xy = cut.getAttribute('position');
    const index = cut.getIndex();
    const order = index ? Array.from(index.array) : [...Array(xy.count).keys()];
    const positions: number[] = [];
    for (const i of order) {
      const p = place(xy.getX(i), xy.getY(i));
      positions.push(p.x, p.y, p.z);
    }
    cut.dispose();
    if (positions.length === 0) continue;

    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
    geometry.computeVertexNormals();
    const mesh = new THREE.Mesh(geometry, new THREE.MeshBasicMaterial({
      color: new THREE.Color(FACE[shape.role] ?? '#6b757c'),
      side: THREE.DoubleSide,
    }));
    mesh.raycast = () => {};
    mesh.userData = { conditionId: null, kind: 'cut-face' };
    out.push(mesh);
  }
  return out;
}

/** What each course of the cut face is drawn in. The stylesheet's greys. */
const FACE: Readonly<Record<string, string>> = {
  fill: '#4a5359',
  panel: '#a9b4b9',
  sump: '#1f7a8c',
  cover: '#7c878d',
  wall: MASONRY,
};

/**
 * Put the words where the roof they name is drawn, this frame.
 *
 * They follow the camera, so this runs with the render rather than with the
 * document: a preset corner, a fit, the vertical stretch and the orbit all move
 * the picture without changing the roof, and a word that stayed put would be
 * pointing at nothing. It is a handful of labels, and each one is a projection
 * and two style properties.
 *
 * A word whose place is off the canvas is hidden rather than clamped to the
 * edge: a label stuck to the frame is a label pointing at the wrong thing.
 */
function placeWords() {
  if (!labelHost || !camera || !surface || !world) return;
  const rect = surface.getBoundingClientRect();
  if (rect.width < 2 || rect.height < 2) return;

  // One element per label, kept between frames. Rebuilding them every frame
  // would be a hundred nodes a second for a picture nobody is touching.
  while (labelHost.children.length > labelsNow.length) labelHost.lastElementChild?.remove();
  while (labelHost.children.length < labelsNow.length) {
    const el = document.createElement('span');
    el.className = 'model-label';
    labelHost.append(el);
  }

  labelsNow.forEach((label, i) => {
    const el = labelHost!.children[i] as HTMLElement;
    if (el.textContent !== label.text) el.textContent = label.text;
    if (el.getAttribute('data-label-kind') !== label.kind) {
      el.setAttribute('data-label-kind', label.kind);
    }
    const v = world.localToWorld(label.at.clone()).project(camera);
    const x = (v.x * 0.5 + 0.5) * rect.width;
    const y = (-v.y * 0.5 + 0.5) * rect.height;
    const on = v.z < 1 && x >= 0 && x <= rect.width && y >= 0 && y <= rect.height;
    el.hidden = !on;
    if (!on) return;
    el.style.left = `${Math.round(x)}px`;
    el.style.top = `${Math.round(y)}px`;
  });
}

/**
 * Put the Model down. The surface switches views in place; this is the other
 * half.
 *
 * The act in the rail is NOT put down with it. "Until the next act" is what the
 * handoff says about the marks on the roof, and a view switch is not an act: an
 * estimator who sets the high side standing up and then looks at it in plan
 * should find their two picks still on the roof. What ends an act is choosing
 * another one.
 */
export function unmountModel(): void {
  unmount();
  chrome = null;
}

function unmount() {
  if (frameHandle) cancelAnimationFrame(frameHandle);
  frameHandle = 0;
  stopWatching?.();
  stopSubscribing?.();
  stopWatching = null;
  stopSubscribing = null;
  stopObserving?.();
  stopObserving = null;
  stopWatchingVerb?.();
  stopWatchingVerb = null;
  marks?.clear();
  marks = null;
  markWords = [];
  controls?.dispose();
  controls = null;
  renderer?.dispose();
  renderer = null;
  surface = null;
  stopWatchingTheme?.();
  stopWatchingTheme = null;
  refreshChips = () => undefined;
  redrawWorld = () => undefined;
  labelHost = null;
  labelsNow = [];
  roofLabels = [];
  cutting = null;
  ownsChrome = true;
}

/**
 * Size the canvas to the box it is actually in — or decline.
 *
 * It used to substitute 1 for a zero measurement, which sounds harmless and is
 * not: a canvas measured before layout settles reads 0, gets a 1x1 buffer at
 * aspect 1, and **keeps it**, because nothing measures again until the document
 * next changes. A roof drawn into a one-pixel buffer at the wrong aspect is the
 * same class of defect as the Plan's, which fitted to 355 px of a box that ended
 * up nearly a thousand. Declining leaves the last good size, and the observer
 * below calls back the moment there is a real one.
 */
function resize(canvas: HTMLCanvasElement) {
  if (!renderer || !camera) return;
  const w = canvas.clientWidth;
  const h = canvas.clientHeight;
  if (w < 2 || h < 2) return;
  renderer.setSize(w, h, false);
  if (!framed) {
    fitFrustum();
    return;
  }
  // Their camera. Keep what they are looking at and give the new width to the
  // sides, which is what a window getting wider does to a drawing.
  const halfH = (camera.top - camera.bottom) / 2;
  const cx = (camera.left + camera.right) / 2;
  const halfW = halfH * (w / h);
  camera.left = cx - halfW;
  camera.right = cx + halfW;
  camera.updateProjectionMatrix();
}

// ── reading the job ────────────────────────────────────────────────────────

const conditionsOf = (d: Doc): Condition[] => (at('/conditions', d) as Condition[]) ?? [];

// Which drains a cricket serves, which of them the roof falls to, what the
// lowest thing the membrane flashes against is, and therefore where the field
// stops rising — all of that is `src-tauri/core/src/taper.rs` now. It was here,
// and a copy of it was in the engine's `capInches`, and the two agreed until
// one of them was edited. What is left in this file is drawing.

/**
 * Which way the water leaves this point, as a bearing a person can act on.
 *
 * North is up the sheet, which is the direction a roof plan is read in and the
 * same convention the prior 3D work on this machine settled on. Eight points is
 * as fine as anybody can use standing on a roof: "north-east" sends somebody to
 * the right corner, "N 43\u00b0 E" does not.
 */
function compass(from: Point, to: Point): string {
  // Plan y grows down the sheet, so north is negative y.
  const deg = (Math.atan2(to.x - from.x, from.y - to.y) * 180) / Math.PI;
  const points = ['north', 'north-east', 'east', 'south-east', 'south', 'south-west', 'west', 'north-west'];
  return points[Math.round(((deg + 360) % 360) / 45) % 8]!;
}

// Which drain the water falls to, and how far the fall runs to get there, is the
// solved field read at a point — `readTaperAt` in the engine, which is the one
// expression the core is pinned to. Point-in-polygon is the engine's
// `pointInPolygon`; this file carried its own copy of the same ray cast.

// ── building the world ─────────────────────────────────────────────────────

type Built = {
  centre: THREE.Vector3; span: number; legend: HTMLElement[]; note: string;
  thin: number | null; thick: number | null;
  /** The words standing on the roof, for the overlay to place. */
  labels: RoofLabel[];
  /** What the tool settings strip says about this roof. */
  strip: Node[];
};

/**
 * The tapered fields ON SCREEN, for the readout.
 *
 * The solve's own list is not the same list: a trace that crosses itself is
 * solved and not drawn, and the strip must not answer over a surface nobody can
 * see. These are the ones that got drawn, in the solve's own words.
 */
let fieldsNow: SolvedTaperField[] = [];

/**
 * The words that stand ON the roof, and where each of them stands.
 *
 * Frame 2's whole argument: each drain says its own sump depth, the curb says
 * what stands behind it, and the flat patch says how many square feet of it
 * there are and what stopped the fall. They are HTML over the canvas rather than
 * geometry in it — a label has to stay upright and readable at any camera, which
 * a mesh does not — and they are placed every frame by projecting the point on
 * the roof they belong to.
 *
 * Every number in one of them is the solve's. There is no second arithmetic here
 * and there must never be: a word on a roof is a number an estimator will act on.
 */
type RoofLabel = { kind: string; at: THREE.Vector3; text: string };
let labelsNow: RoofLabel[] = [];
/** The words the ROOF says, apart from the ones an act is saying. */
let roofLabels: RoofLabel[] = [];
/** The words an act is saying, which come and go with the act. */
let markWords: RoofLabel[] = [];
/** Where an act's marks live in the scene. Rebuilt whole, never patched. */
let marks: THREE.Group | null = null;
let stopWatchingVerb: (() => void) | null = null;
/** Where they are drawn. One element per label, moved rather than rebuilt. */
let labelHost: HTMLElement | null = null;
/** Has the estimator asked for the board grid? */
let showBoards = false;

/** The solved roof this drawing was built from. */
let solvedNow: SolvedTaper = NOTHING_SOLVED;
/** The solved walls and curbs this drawing was built from. */
let solvedWallsNow: SolvedWalls = NOTHING_SOLVED_WALLS;
/** Which solve is on screen. Nothing is, until the first one is pulled. */
let drawnSolveId = -1;
/** The pull in flight, so an overtaken answer is dropped rather than drawn. */
let pulling = 0;
/** Why the roof did not solve, when it did not. */
let solveTrouble: string | null = null;

function build(d: Doc, taper: SolvedTaper, walls: SolvedWalls): Built {
  world.clear();

  const list = conditionsOf(d);
  const scales = feetPerUnit((at('/pages', d) as Page[]) ?? []);
  const drains = taper.drains;
  // The tapered field's own colour, which the ramp and the key are drawn in.
  const tapered = list.find(
    (c) => c.kind === 'area' && !c.hidden && c.properties?.['TAPER'] !== undefined,
  );

  const box = new THREE.Box3();
  let anything = false;
  let noFallSquares = 0;
  let taperedFields = 0;
  let regions = 0;
  let valleys = 0;
  let ridges = 0;
  /// The worst a ridge disagrees with its neighbour, in inches. Nought is a roof
  /// whose two patterns meet at one thickness, which is what NRCA asks for.
  let ridgeUneven = 0;
  let crickets = 0;
  let thin = Infinity;
  let thick = -Infinity;
  const fields: SolvedTaperField[] = [];
  const unscaled = list.some((c) => (c.traces ?? []).some((t) => !scales[t.pageId]));
  // Things the drawing has to say out loud rather than paper over.
  let unstatedBuildUp = 0;
  let capReason: CapReason = null;
  let outOfProportion = 0;
  let ridgeOffSquare = 0;
  let cricketSlopeOff = 0;
  let overOneLayer = false;
  let unbuiltCrickets = 0;
  let crossedTraces = 0;
  let unstatedWall = 0;
  let unstatedCap = 0;
  let boards = 0;
  const labels: RoofLabel[] = [];
  let sumps = 0;
  let unsumpedDrains = 0;
  let clippedSumps = 0;
  let copings = 0;
  let curbs = 0;

  list.forEach((c, index) => {
    if (c.hidden) return;
    const colour = c.color ?? hueFor(index);
    const props = c.properties ?? {};
    const elevation = props['ELEV'] ?? 0;

    for (const [traceIndex, trace] of (c.traces ?? []).entries()) {
      const ring = feet(trace, scales);
      if (ring.length === 0) continue;
      // A ring that crosses itself triangulates to overlapping faces and gaps,
      // with nothing raised — earcut says so in its own documentation. Worse,
      // its area cancels to zero. Neither shows up as an error, so the shape is
      // left out and named rather than drawn.
      if (c.kind === 'area' && selfIntersects(ring)) {
        crossedTraces += 1;
        continue;
      }

      if (c.kind === 'area') {
        const solvedField = fieldOf(taper, c.id, traceIndex);
        const tapered = props['TAPER'] !== undefined && drains.length > 0;
        if (tapered) {
          // Solved and undrawable, or not solved yet: no surface, and the key
          // says how many tapered facets there are, so nothing is invented to
          // stand in for one.
          let drawn = 0;
          if (solvedField) {
            solvedField.regions.forEach((region, regionIndex) => {
              const mesh = regionSurface(solvedField, region, c);
              if (!mesh) return;
              world.add(mesh);
              drawn += 1;
              // The layout drawing, in three dimensions: C2's bill on the roof
              // it was billed off. Off until the chip is pressed, because seven
              // hundred marks over the fall is not what Frame 2 is for.
              if (showBoards) {
                for (const board of boardGrid(solvedField, region)) world.add(board);
                boards += region.boards.length;
              }
              for (const valley of region.valleys) {
                world.add(creaseLine(solvedField, valley, 'valley'));
                valleys += 1;
              }
              // A ridge belongs to both the regions it divides, so it is drawn
              // once — by the earlier of the two — rather than twice in the same
              // place, which would double every count in the key.
              for (const ridge of region.ridges) {
                if (ridge.meets < regionIndex) continue;
                world.add(creaseLine(solvedField, ridge, 'ridge'));
                ridges += 1;
                if (ridge.disagreementInches > 1 / 8) {
                  ridgeUneven = Math.max(ridgeUneven, ridge.disagreementInches);
                }
              }
            });
          }
          if (solvedField && drawn > 0) {
            world.add(flowField(solvedField));
            noFallSquares += solvedField.noFallSquareFeet;
            // The flat ground, named where it lies. How much of it there is and
            // what stopped the fall are the two facts an estimator acts on —
            // extend the run, add a drain, or accept that it will hold water —
            // and they belong on the patch rather than in a key three inches
            // away from it.
            if (solvedField.noFallSquareFeet > 0 && solvedField.capInches !== null) {
              const patch = widestFlat(solvedField);
              if (patch) {
                labels.push({
                  kind: 'nofall',
                  at: new THREE.Vector3(
                    patch.x,
                    solvedField.elevation + inches(solvedField.capInches) + 0.4,
                    patch.y,
                  ),
                  text: `no fall · ${Math.round(solvedField.noFallSquareFeet).toLocaleString('en-US')} SF`
                    + ` · ${solvedField.capReason === 'flashing'
                      ? 'the wall will not take another layer'
                      : 'the boards run out'}`,
                });
              }
            }
            taperedFields += 1;
            regions += drawn;
            if (solvedField.thinInches !== null) thin = Math.min(thin, solvedField.thinInches);
            if (solvedField.thickInches !== null) thick = Math.max(thick, solvedField.thickInches);
            if (solvedField.thickInches !== null && solvedField.thickInches > SINGLE_LAYER_MAX_INCHES) {
              overOneLayer = true;
            }
            if (solvedField.capReason !== null) capReason = solvedField.capReason;
            fields.push(solvedField);
          }
        } else {
          // Not tapered, so the build-up is whatever the layers on it say. Null
          // when none of them says, and null draws nothing — the legend carries
          // it instead, because an unstated build-up is a fact about the job.
          const stack = buildUpInches(c.items ?? []);
          if (stack === null) unstatedBuildUp += 1;
          else world.add(buildUp(ring, elevation, stack, colour, c));
        }
        world.add(datumSurface(ring, elevation, colour, c, tapered));
        anything = true;
      } else if (c.kind === 'line') {
        const height = props['H'] ?? 0;
        if (c.role === 'ridge') {
          // Which pair of drains a cricket serves, or which curb it stands
          // behind, and therefore where its ridge lies and how wide it is, is
          // the core's (`taper.rs`). What reaches here is the wedge.
          const solvedCricket = cricketOf(taper, c.id, traceIndex);
          if (solvedCricket) {
            world.add(cricketMesh(solvedCricket, colour, c));
            crickets += 1;
            if (solvedCricket.lengthToWidth > solvedCricket.maxLengthToWidth) outOfProportion += 1;
            if (solvedCricket.offBy !== null && solvedCricket.offBy > 15) ridgeOffSquare += 1;
            if (solvedCricket.slopeDisagrees) cricketSlopeOff += 1;
          } else {
            // Nothing to derive it from. A ridge line on its own is a line, and
            // drawing a cricket off it would be inventing the shape again.
            world.add(runLine(ring, elevation, colour, c));
            unbuiltCrickets += 1;
          }
        } else if (height > 0) {
          // The solid and the cap are both C3's (`walls.rs`, `coping.rs`):
          // `WALL` grows the traced line inboard by a true polyline offset,
          // not a per-segment guess at where the ring's own middle is, and
          // the cap is that same offset again, centred on the wall, at
          // whatever width `W` states or `WALL + 2 x CAP_OVERHANG_IN` derives.
          // This file draws the rings it is handed; it computes no offset.
          const run = wallOf(walls, c.id, traceIndex);
          if (run?.wallInches === null) unstatedWall += 1;
          if (run) {
            world.add(wallSolid(run, c));
            const cap = copingCap(run);
            if (cap) { world.add(cap); copings += 1; }
            // A cap width is always derivable once a wall thickness is —
            // `WALL + 2 x CAP_OVERHANG_IN` sees to that — so the only way a
            // stated or derived width draws no cap is a width with no wall to
            // centre it on: `W` typed, `WALL` not.
            else if (run.coping.capWidthFeet !== null) unstatedCap += 1;
          }
          if (c.role === 'curb') {
            curbs += 1;
            // Its own name, and the cricket behind it where one is traced
            // against it. Which cricket that is, is the job's own answer: a
            // ridge condition names what it serves in `between`, and that is
            // what the core sizes the wedge off.
            const behind = list.find((other) => other.role === 'ridge' && !other.hidden
              && (other.between ?? []).includes(c.id));
            const middle = {
              x: ring.reduce((sum, q) => sum + q.x, 0) / ring.length,
              y: ring.reduce((sum, q) => sum + q.y, 0) / ring.length,
            };
            labels.push({
              kind: 'curb',
              at: new THREE.Vector3(middle.x, elevation + height + 0.4, middle.y),
              text: behind ? `${c.name} · cricket behind` : c.name,
            });
          }
        } else {
          world.add(runLine(ring, elevation, colour, c));
        }
        anything = true;
      } else if (c.kind === 'count') {
        for (const p of ring) {
          const sump = sumpOf(taper, c.id, p);
          if (c.role === 'drain') {
            if (sump === null) unsumpedDrains += 1;
            else {
              sumps += 1;
              if (sump.clipped) clippedSumps += 1;
            }
            // Its own name and the sump it sits in, on the roof beside it. The
            // number is the depth the layout falls from the sump's rim to the
            // drain, read off the solved planes — the same figure the section
            // dimensions and the same one the sump panel is bought against.
            const which = drains.findIndex((d) => d.x === p.x && d.y === p.y) + 1;
            const deckHere = fieldAt(taper, p)?.elevation ?? elevation;
            labels.push({
              kind: 'drain',
              at: new THREE.Vector3(p.x, deckHere + inches(sump?.rimInches ?? 0) + 0.6, p.y),
              text: sump === null
                ? `Drain ${which} · no sump stated`
                : `Drain ${which} · sump ${inchLabel(sump.depthInches)} deep`,
            });
          }
          // The deck the sump is cut into is the field's, read off the solve. A
          // drain on no tapered field has none, and the drain's own datum stands.
          const deck = fieldAt(taper, p)?.elevation ?? elevation;
          world.add(...marker(p, elevation, c, sump, deck));
        }
        anything = true;
      }

      for (const p of ring) box.expandByPoint(new THREE.Vector3(p.x, elevation, p.y));
    }
  });

  // The valleys and the ridges were drawn with the regions they belong to: both
  // are folds in the solved surface rather than lines of their own, which is the
  // whole difference between this and the version that drew a valley down the
  // line joining two drains. There is no such line in a four-way layout — between
  // two drains the ground between the two 45 degree ties is level across, so the
  // water runs to the valleys either side of it and not along the middle.

  // And, where this is a section, the face the cut exposes. It goes in last
  // because it stands in front of everything the plane took away.
  if (cutting) for (const piece of cutFace(cutting)) world.add(piece);

  highlight();

  const centre = anything ? box.getCenter(new THREE.Vector3()) : new THREE.Vector3();
  const span = anything ? Math.max(box.getSize(new THREE.Vector3()).length(), 20) : 0;

  fieldsNow = fields;

  return {
    centre,
    span,
    thin: Number.isFinite(thin) ? thin : null,
    thick: Number.isFinite(thick) ? thick : null,
    legend: legendFor({
      taperedFields, regions, noFallSquareFeet: noFallSquares, drains: drains.length, crickets,
      thin, thick, capReason, unstatedBuildUp, outOfProportion, ridgeOffSquare,
      cricketSlopeOff, overOneLayer, unbuiltCrickets, crossedTraces, unstatedWall,
      unstatedCap, copings, curbs, valleys, ridges, ridgeUneven, boards,
      sumps, unsumpedDrains, clippedSumps,
      refused: taper.refused,
      ramp: RAMP[2],
    }),
    labels,
    strip: stripFor(fields, list),
    note: anything
      ? (unscaled ? 'Some of this roof is on a sheet nobody has scaled, so it is not drawn.' : '')
      : 'Nothing traced yet. Trace a roof on the Plan and it stands up here.',
  };
}

/**
 * The widest piece of ground the cap holds flat, and where the middle of it is.
 *
 * For the word that stands on it. The largest flat facet rather than all of
 * them, because one label on the biggest patch is a drawing and one on every
 * patch is a lawn — and the number on it is the WHOLE field's no-fall area,
 * which is what an estimator is deciding about.
 */
function widestFlat(field: SolvedTaperField): Point | null {
  let best: { size: number; at: Point } | null = null;
  for (const region of field.regions) {
    for (const facet of region.facets) {
      if (!facet.flat || facet.shape.ring.length < 3) continue;
      const ring = facet.shape.ring;
      let twice = 0;
      let cx = 0;
      let cy = 0;
      for (let i = 0; i < ring.length; i += 1) {
        const a = ring[i]!;
        const b = ring[(i + 1) % ring.length]!;
        const cross = a.x * b.y - b.x * a.y;
        twice += cross;
        cx += (a.x + b.x) * cross;
        cy += (a.y + b.y) * cross;
      }
      if (Math.abs(twice) < 1e-9) continue;
      const size = Math.abs(twice) / 2;
      if (best === null || size > best.size) {
        best = { size, at: { x: cx / (3 * twice), y: cy / (3 * twice) } };
      }
    }
  }
  return best?.at ?? null;
}

/**
 * What the tool settings strip says about this roof.
 *
 * Frame 2's strip, in this program's own numbers: the high side and the low
 * point, the slope and the sump the estimator typed, and — the one that is worth
 * the room — how many courses of board the layout NEEDS beside how many it was
 * allowed. On the demo those are four and three, and the difference is a
 * purchase somebody has to make (C2's pushback 8).
 *
 * This is what the strip says with NO act up: the roof as it stands, under
 * Select. The same six readings under the act that changes them are the act's
 * own (`verbs.ts`), where two of them are fields somebody types into.
 */
function stripFor(fields: SolvedTaperField[], list: Condition[]): Node[] {
  const out: Node[] = [
    stripField('On the roof', 'click to pick what you are looking at'),
  ];
  const field = fields[0];
  if (!field) {
    out.push(stripField('Units', 'SF · LF · EA · SQ'));
    return out;
  }

  if (field.thickInches !== null) {
    out.push(derivedField('High', inchLabel(field.thickInches),
      'The thickest the layout gets: the longest run on it, times the slope, held to the ceiling.'));
  }
  out.push(derivedField('Low', `${inchLabel(field.startInches)} at the drains`,
    'What the estimator stated at the drain. Everything on the roof is measured up from it.'));
  out.push(stripField('Slope', `${inchLabel(field.slopeInchesPerFoot)} in 12`));

  const allowed = list.find((c) => c.id === field.conditionId)?.properties?.['BOARDS'];
  const needed = field.bill?.coursesMax;
  if (needed !== undefined) {
    out.push(derivedField('Boards', allowed === undefined
      ? `${needed} courses`
      : `${needed} courses against ${allowed} allowed`,
    'How many layers of board the deepest position takes, the tapered panel included,'
      + ' beside the ceiling the job typed.'));
  }

  const sump = field.sumps[0];
  if (sump) {
    out.push(stripField('Sump width', formatFeetInches(sump.sideFeet)));
    out.push(derivedField('Sump depth', inchLabel(sump.depthInches),
      'The roof at the sump\'s own rim, less the thickness at the drain:'
      + ' the thickness at the drain, the slope, the sump width and the board module.'));
  }
  return out;
}

/**
 * The datum — the traced polygon at its elevation, which is the top of deck.
 *
 * A surface, with no depth, because no depth has been stated. This used to be an
 * extrusion three quarters of a foot deep, with a comment admitting it was "not
 * a real assembly — enough to be a building". The argument for it was that a
 * bare surface floats and reads as a coloured shape in space rather than a roof.
 * That argument is true and it is not a reason: a shape that reads as a building
 * because somebody picked a thickness is a picture of a building, and this view
 * exists so the estimator can read the roof rather than admire it.
 *
 * The convention it now follows is the one both real tools keep. A wall in
 * FreeCAD holds a link to its baseline and rebuilds its solid from it on every
 * recompute; an IFC slab's layer stack is placed against a reference plane. The
 * datum is the low-dimensional thing, and the solid is what falls out of it.
 */
function datumSurface(ring: Point[], elevation: number, colour: string, c: Condition, faded: boolean) {
  const shape = new THREE.Shape(ring.map((p) => new THREE.Vector2(p.x, p.y)));
  const geometry = new THREE.ShapeGeometry(shape);
  // Plan (x, y) becomes world (x, elevation, y): the trace keeps its orientation.
  geometry.rotateX(Math.PI / 2);
  geometry.translate(0, elevation, 0);
  const mesh = new THREE.Mesh(geometry, new THREE.MeshLambertMaterial({
    color: new THREE.Color(colour),
    side: THREE.DoubleSide,
    transparent: faded,
    opacity: faded ? 0.35 : 1,
  }));
  mesh.userData = { conditionId: c.id, kind: 'datum' };
  return mesh;
}

/**
 * The build-up, at the thickness its own layers say it is.
 *
 * Only drawn when something says. `buildUpInches` returns null rather than zero
 * for a stack that carries no thicknesses, and null means nothing is drawn and
 * the legend says the build-up is not stated — which is a true thing about the
 * job, and a prompt to go and put it in.
 *
 * It grows **up** from the datum, and that direction is a decision rather than
 * an accident of which way an extrusion happened to point. IFC keeps the
 * magnitude on the layer as a non-negative length and the direction beside it as
 * `DirectionSense`; FreeCAD clamps a negative `Height` to zero and steers with a
 * separate `Normal`. A roof build-up sits on the deck, so: up.
 */
function buildUp(ring: Point[], elevation: number, thick: number, colour: string, c: Condition) {
  const shape = new THREE.Shape(ring.map((p) => new THREE.Vector2(p.x, p.y)));
  const depth = inches(thick);
  const geometry = new THREE.ExtrudeGeometry(shape, { depth, bevelEnabled: false });
  // The same rotation as the datum, so the plan is not mirrored; then lifted by
  // its own depth, which puts its underside on the deck and its top at the
  // finished surface. That is the stack growing up, written out.
  geometry.rotateX(Math.PI / 2);
  geometry.translate(0, elevation + depth, 0);
  const mesh = new THREE.Mesh(geometry, new THREE.MeshLambertMaterial({
    color: new THREE.Color(colour), side: THREE.DoubleSide,
  }));
  mesh.userData = { conditionId: c.id, kind: 'build-up' };
  return mesh;
}

/**
 * One drainage region, drawn from the facets the core cut it into.
 *
 * A facet is a piece of ground with one plane over it — or with the cap over it,
 * and then the roof is flat there. Either way the surface is exactly planar on it,
 * so triangulating the polygon and lifting each corner to its own depth IS the
 * roof rather than a sampling of it: a few dozen triangles to a region where the
 * cone needed a lattice of two and a half thousand. Lighter where it is thicker,
 * amber where the fall has run out.
 *
 * Every number in here arrived solved — the polygons, which plane governs each of
 * them, where the cap binds. The one thing this works out is the height of a
 * corner, which is `readFacetAt` in the engine: the same expression the core is
 * pinned to, evaluated at a point the core chose.
 */
function regionSurface(
  field: SolvedTaperField, region: SolvedTaperRegion, c: Condition,
) {
  const elevation = field.elevation;
  const positions: number[] = [];
  const colours: number[] = [];
  const noFall = new THREE.Color('#c98a1d');
  // The ramp is read across the whole FIELD, not across this region, so one
  // depth is one grey wherever it stands. Shading each region against its own
  // thin and thick would make two places at the same thickness two colours,
  // which is the one thing a scale must not do.
  const thin = field.thinInches ?? 0;
  const thick = field.thickInches ?? thin + 1;
  const span = thick - thin > 1e-9 ? thick - thin : 1;

  for (const facet of region.facets) {
    const ring = facet.shape.ring;
    if (ring.length < 3) continue;
    const shape = new THREE.Shape(ring.map((p) => new THREE.Vector2(p.x, p.y)));
    for (const hole of facet.shape.holes) {
      if (hole.length < 3) continue;
      shape.holes.push(new THREE.Path(hole.map((p) => new THREE.Vector2(p.x, p.y))));
    }
    // Earcut, through three.js, on a polygon that is planar by construction. The
    // corners it hands back are the corners it was given: it adds no points, so
    // every vertex has a depth the solved plane can be asked for.
    const flat = new THREE.ShapeGeometry(shape);
    const xy = flat.getAttribute('position');
    const index = flat.getIndex();
    const order = index ? Array.from(index.array) : [...Array(xy.count).keys()];
    for (const i of order) {
      const at = { x: xy.getX(i), y: xy.getY(i) };
      const depth = readFacetAt(field, region, facet, at);
      positions.push(at.x, elevation + inches(depth), at.y);
      const shade = facet.flat ? noFall : rampAt((depth - thin) / span);
      colours.push(shade.r, shade.g, shade.b);
    }
    flat.dispose();
  }

  if (positions.length === 0) return null;

  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
  geometry.setAttribute('color', new THREE.Float32BufferAttribute(colours, 3));
  geometry.computeVertexNormals();

  const mesh = new THREE.Mesh(geometry, new THREE.MeshLambertMaterial({
    vertexColors: true, side: THREE.DoubleSide,
  }));
  mesh.userData = { conditionId: c.id, kind: 'taper' };
  return mesh;
}

/**
 * Where the water goes: one long arrow down each piece of ground one plane
 * governs.
 *
 * The core places them, points them and says how long each one has to be — down
 * the gradient of whichever plane governs, which on a four-way layout is square
 * to the boards and not at the drain, because water runs down the fall and then
 * along the valley. Flat ground gets none: there is nothing to point at when the
 * fall has run out.
 *
 * ONE TO A FACET (C5, D156). It was one every eight feet, which on the demo is a
 * hundred and thirty-nine identical darts — a lawn, and the thing Frame 2 most
 * obviously is not. A facet is the ground a single plane governs, so the water
 * does exactly one thing over the whole of it; one mark that reaches across it
 * says so, and a scatter of short ones only implies it.
 *
 * A shaft and a head, because a dart with no shaft reads as a blob — which is
 * what the instanced version looked like, caught by opening the screenshot and
 * not by any check. There are a dozen of them now, so they are ordinary meshes.
 */
function flowField(field: SolvedTaperField) {
  const flow = new THREE.Group();
  const water = new THREE.MeshLambertMaterial({ color: new THREE.Color(WATER) });
  for (const a of field.flow) {
    const long = Math.max(a.lengthFeet, 3);
    const head = Math.min(long * 0.3, 3.5);
    const shaft = Math.max(long - head, 0.5);
    const wide = Math.max(long * 0.022, 0.18);

    const mark = new THREE.Group();
    const stick = new THREE.Mesh(new THREE.BoxGeometry(shaft, wide * 0.7, wide), water);
    stick.position.x = -head / 2;
    const point = new THREE.ConeGeometry(wide * 1.9, head, 7);
    point.rotateZ(-Math.PI / 2);
    const tip = new THREE.Mesh(point, water);
    tip.position.x = shaft / 2;
    mark.add(stick, tip);

    mark.position.set(a.x, field.elevation + inches(a.thicknessInches) + 0.1, a.y);
    mark.rotation.set(0, -a.angle, 0);
    // Not pickable: an arrow is the drawing explaining itself, not a thing on
    // the roof. Clicking one should select the field under it.
    stick.raycast = () => {};
    tip.raycast = () => {};
    flow.add(mark);
  }
  return flow;
}

/**
 * The board grid, on the roof, with the letter of each board on it.
 *
 * This is C2's bill, drawn: one mesh per board POSITION, cut to the same shape
 * the boards were cut to, tinted by the letter printed on it, and a letter
 * standing on every one of them. Nothing here works a board out — the grid, the
 * cut, the letter and the depth all arrive solved, and the same list is what the
 * Estimate is priced off. A board drawn here is a board that got bought.
 *
 * It is off until the estimator asks for it, because it is seven hundred marks
 * on this roof and the thing Frame 2 is for is the fall.
 */
function boardGrid(field: SolvedTaperField, region: SolvedTaperRegion): THREE.Object3D[] {
  const out: THREE.Object3D[] = [];
  const byLetter = new Map<string, { x: number; y: number; z: number }[]>();

  for (const board of region.boards) {
    const positions: number[] = [];
    const top = field.elevation + inches(board.thicknessInches) + 0.06;
    for (const piece of board.shapes) {
      if (piece.ring.length < 3) continue;
      const shape = new THREE.Shape(piece.ring.map((p) => new THREE.Vector2(p.x, p.y)));
      for (const hole of piece.holes) {
        if (hole.length < 3) continue;
        shape.holes.push(new THREE.Path(hole.map((p) => new THREE.Vector2(p.x, p.y))));
      }
      const flat = new THREE.ShapeGeometry(shape);
      const xy = flat.getAttribute('position');
      const index = flat.getIndex();
      const order = index ? Array.from(index.array) : [...Array(xy.count).keys()];
      for (const i of order) positions.push(xy.getX(i), top, xy.getY(i));
      flat.dispose();
    }
    if (positions.length === 0) continue;

    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
    geometry.computeVertexNormals();
    const mesh = new THREE.Mesh(geometry, new THREE.MeshBasicMaterial({
      color: tintOf(board.letter),
      side: THREE.DoubleSide,
      transparent: true,
      opacity: board.cut ? 0.92 : 0.8,
    }));
    // A board is the layout drawn on the roof, not a thing to pick: clicking one
    // should select the field it is laid on, and seven hundred of them in the
    // raycast would make the pointer readout crawl.
    mesh.raycast = () => {};
    mesh.userData = { conditionId: null, kind: 'board' };
    out.push(mesh);

    if (board.letter) {
      const where = byLetter.get(board.letter) ?? [];
      where.push({ x: board.centre.x, y: top + 0.04, z: board.centre.y });
      byLetter.set(board.letter, where);
    }
  }

  // The letters, one instanced quad per letter of the chart. Four textures for
  // seven hundred boards: a canvas each, drawn once and kept, so a re-solve does
  // not leave a texture behind every time anybody types.
  for (const [letter, where] of byLetter) {
    const quad = new THREE.PlaneGeometry(2.4, 2.4);
    quad.rotateX(-Math.PI / 2);
    const faces = new THREE.InstancedMesh(
      quad,
      new THREE.MeshBasicMaterial({
        map: letterFace(letter), transparent: true, depthWrite: false,
      }),
      where.length,
    );
    const placed = new THREE.Object3D();
    where.forEach((p, i) => {
      placed.position.set(p.x, p.y, p.z);
      placed.updateMatrix();
      faces.setMatrixAt(i, placed.matrix);
    });
    faces.instanceMatrix.needsUpdate = true;
    faces.raycast = () => {};
    faces.userData = { conditionId: null, kind: 'board-letter' };
    out.push(faces);
  }

  return out;
}

/** One tint per letter, off the same ramp the field is shaded on. */
function tintOf(letter: string | null): THREE.Color {
  if (letter === null) return new THREE.Color('#8a5a1d');
  let sum = 0;
  for (const ch of letter) sum += ch.charCodeAt(0) - 64;
  return rampAt(((sum % 7) + 1) / 8);
}

/** The letters already drawn, so a re-solve reuses them rather than leaking. */
const letterFaces = new Map<string, THREE.CanvasTexture>();

/**
 * A letter, drawn once into a canvas and used as a texture.
 *
 * There is no font loader here and there is no network, so a glyph in three
 * dimensions is a picture of a glyph. Seven hundred boards share four of them.
 */
function letterFace(letter: string): THREE.CanvasTexture {
  const had = letterFaces.get(letter);
  if (had) return had;
  const canvas = document.createElement('canvas');
  canvas.width = 96;
  canvas.height = 96;
  const ink = canvas.getContext('2d');
  if (ink) {
    ink.clearRect(0, 0, 96, 96);
    ink.fillStyle = '#10161a';
    ink.font = 'bold 54px system-ui, sans-serif';
    ink.textAlign = 'center';
    ink.textBaseline = 'middle';
    ink.fillText(letter, 48, 52);
  }
  const face = new THREE.CanvasTexture(canvas);
  letterFaces.set(letter, face);
  return face;
}

/**
 * A parapet or a curb: the solid the solve already built, stood up from its
 * base to its own height (C3, `walls.rs`).
 *
 * `run.outer` is the traced line and `run.inboard` — where `WALL` is typed —
 * is that same line offset toward the roof by a true polyline offset
 * (`cavalier_contours`, not a per-segment guess at the ring's own middle). A
 * closed ring with both is an annulus: `THREE.Shape` with `run.inboard`'s
 * ring as a hole, extruded from the deck to the wall's own height by
 * `THREE.ExtrudeGeometry`'s own triangulator, which owes the two rings no
 * correspondence in vertex count — the offset can and does add one at a
 * reflex corner the outer trace never had. Without an inboard line the wall
 * stays the ribbon it has always been: one vertical face, no thickness.
 *
 * This function computes no offset. It draws the two rings it is handed.
 */
function wallSolid(run: SolvedWallRun, c: Condition): THREE.Object3D {
  const low = run.elevationFeet;
  const material = new THREE.MeshLambertMaterial({
    color: new THREE.Color(MASONRY), side: THREE.DoubleSide,
  });
  // A curb is a run with a height, the same as a parapet is, and it is drawn
  // by the same solid. What it is called here is what it is: a parapet is the
  // edge of the roof and a curb is a thing standing on it, and a check
  // counting one must not find the other.
  const kind = c.role === 'curb' ? 'curb' : (run.inboard ? 'parapet' : 'parapet-ribbon');

  if (!run.inboard) {
    const mesh = ribbon(run.outer.ring, low, low + run.heightFeet, material);
    mesh.userData = { conditionId: c.id, kind };
    return mesh;
  }

  const shape = new THREE.Shape(run.outer.ring.map((p) => new THREE.Vector2(p.x, p.y)));
  shape.holes.push(new THREE.Path(run.inboard.ring.map((p) => new THREE.Vector2(p.x, p.y))));
  const geometry = new THREE.ExtrudeGeometry(shape, { depth: run.heightFeet, bevelEnabled: false });
  // The same rotation and lift `buildUp` uses: the extrusion runs from the
  // deck up, so its top lands at the wall's own height and not the other way.
  geometry.rotateX(Math.PI / 2);
  geometry.translate(0, low + run.heightFeet, 0);
  const mesh = new THREE.Mesh(geometry, material);
  mesh.userData = { conditionId: c.id, kind };
  return mesh;
}

/** A vertical face along a ring, no thickness — the ribbon a `WALL`-less run has always been. */
function ribbon(ring: readonly Point[], low: number, high: number, material: THREE.Material): THREE.Mesh {
  const positions: number[] = [];
  for (let i = 0; i < ring.length - 1; i += 1) {
    const a = ring[i]!;
    const b = ring[i + 1]!;
    positions.push(a.x, low, a.y, b.x, low, b.y, a.x, high, a.y);
    positions.push(b.x, low, b.y, b.x, high, b.y, a.x, high, a.y);
  }
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
  geometry.computeVertexNormals();
  return new THREE.Mesh(geometry, material);
}

/**
 * The formed metal over a wall or a curb, as wide as the detail says — C3's
 * `coping.rs`, centred on the wall by the same kind of offset the wall's own
 * inboard face is. Drawn as a surface and not a solid on purpose: 24 ga is
 * twenty-eight thousandths of an inch, and a plate thick enough to see would
 * be a thickness nobody stated. `null` where there is no wall thickness to
 * centre a cap on, even if a width is typed or derived.
 */
function copingCap(run: SolvedWallRun): THREE.Mesh | null {
  const { capOuter, capInner } = run.coping;
  if (!capOuter) return null;
  const shape = new THREE.Shape(capOuter.ring.map((p) => new THREE.Vector2(p.x, p.y)));
  if (capInner) shape.holes.push(new THREE.Path(capInner.ring.map((p) => new THREE.Vector2(p.x, p.y))));
  const geometry = new THREE.ShapeGeometry(shape);
  geometry.rotateX(Math.PI / 2);
  geometry.translate(0, run.elevationFeet + run.heightFeet + 0.02, 0);
  const mesh = new THREE.Mesh(geometry, new THREE.MeshLambertMaterial({
    color: new THREE.Color(GALVANIZED), side: THREE.DoubleSide,
  }));
  mesh.userData = { conditionId: run.conditionId, kind: 'coping' };
  return mesh;
}

/** Formed metal, in the colour of the material rather than of the condition. */
const GALVANIZED = '#b9c4c9';

/**
 * A line the boards are cut along, drawn on the surface it lies in.
 *
 * Which lines those are is solved (`taper.rs`). A valley is where two of a
 * region's four planes have risen equally, which is at 45 degrees to the boards
 * exactly — NRCA p.166, and the reason the boards can be cut corner to corner. A
 * ridge is the boundary between two regions: equidistant between their drains and
 * square to the line joining them. Both arrive as two ends with the depth at each,
 * because the surface is planar along them and two numbers say all of it.
 *
 * Dashed, because that is what a low line is on a roof plan and what Frame 2
 * draws. A ridge is drawn the same way in the same colour: it is the same kind of
 * mark, and the key names both.
 */
function creaseLine(field: SolvedTaperField, crease: SolvedCrease | SolvedRidge, kind: string) {
  const geometry = new THREE.BufferGeometry().setFromPoints([
    new THREE.Vector3(crease.a.x, field.elevation + inches(crease.aInches) + 0.12, crease.a.y),
    new THREE.Vector3(crease.b.x, field.elevation + inches(crease.bInches) + 0.12, crease.b.y),
  ]);
  const line = new THREE.Line(geometry, new THREE.LineDashedMaterial({
    color: new THREE.Color(CREASE), dashSize: 2, gapSize: 1.4, transparent: true, opacity: 0.75,
  }));
  line.computeLineDistances();
  // A crease is the drawing explaining the roof, not a thing on it: a click here
  // should pick the field it lies in.
  line.raycast = () => {};
  line.userData = { conditionId: null, kind };
  return line;
}

/** The one loud colour: drainage, and everything the water does. */
const WATER = '#1f7a8c';

/**
 * A cricket: the wedge the core solved, drawn.
 *
 * Everything that makes a cricket what it is happens in `taper.rs` now, and it
 * is worth saying what that is, because it is the part of this drawing that was
 * wrong longest. NRCA is explicit (pp.166–168): the ridge of a saddle between
 * two drains sits equidistant between them and runs perpendicular to the line
 * joining them — it is where two drainage fields meet, not a line somebody drew
 * and hung planes off. Behind a curb it is the curb's own upslope face that
 * sizes it (§8.1 p.252, Figs. 10-13/10-14). Width is derived either way, and
 * width is the load-bearing number: widening a cricket is what steepens its
 * valley, and steepening the surface does nothing for it.
 *
 * What the traced line is still worth is how far the cricket runs, which no
 * amount of drain geometry knows. That division — their line for the extent, the
 * thing served for the shape — is the core's; this places triangles.
 */
function cricketMesh(cricket: SolvedCricket, colour: string, c: Condition) {
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.Float32BufferAttribute([...cricket.positions], 3));
  geometry.computeVertexNormals();
  const mesh = new THREE.Mesh(geometry, new THREE.MeshLambertMaterial({
    color: new THREE.Color(colour), side: THREE.DoubleSide,
  }));
  mesh.userData = { conditionId: c.id, kind: 'cricket' };
  return mesh;
}

/** A run with no height: drawn as a line on the deck, so it is still visible. */
function runLine(ring: Point[], elevation: number, colour: string, c: Condition) {
  const geometry = new THREE.BufferGeometry().setFromPoints(
    ring.map((p) => new THREE.Vector3(p.x, elevation + 0.05, p.y)),
  );
  const line = new THREE.Line(geometry, new THREE.LineBasicMaterial({ color: new THREE.Color(colour) }));
  line.userData = { conditionId: c.id, kind: 'run' };
  return line;
}

/**
 * A counted thing. A drain gets its sump and its bowl; anything else gets a mark.
 *
 * THE SUMP IS SOLVED NOW, NOT DRAWN. It used to be a 4 ft square — four feet
 * because that was the fallback when nothing said otherwise — set at a height this
 * file worked out for itself out of `T`, `SUMP` and `TAPER`. Both are gone: the
 * shape is the cut the core made, clipped to the roof where the roof's own edge
 * takes some of it, and the height is the depth the core read off the solved planes
 * at the sump's rim. A drain with no sump stated gets no sump, and the legend says
 * so rather than the drawing inventing one.
 */
function marker(
  p: Point, elevation: number, c: Condition, sump: SolvedSump | null, deck: number,
): THREE.Object3D[] {
  if (c.role !== 'drain') {
    const mark = new THREE.Mesh(
      new THREE.SphereGeometry(0.75, 12, 8),
      new THREE.MeshLambertMaterial({ color: new THREE.Color(c.color ?? '#555555') }),
    );
    mark.position.set(p.x, elevation + 0.75, p.y);
    mark.userData = { conditionId: c.id, kind: 'count' };
    return [mark];
  }

  // The bowl sits at the roof the drain is in, which is the deck plus whatever the
  // tapered package is there. With no sump solved that is the drain's own datum.
  const at = sump === null ? elevation : deck + inches(sump.rimInches);
  const bowl = new THREE.Mesh(
    new THREE.CylinderGeometry(0.7, 0.55, 0.35, 16),
    new THREE.MeshLambertMaterial({ color: new THREE.Color(WATER) }),
  );
  bowl.position.set(p.x, at + 0.22, p.y);
  bowl.userData = { conditionId: c.id, kind: 'drain' };
  if (sump === null) return [bowl];

  // The cut, as the core cut it. Earcut on a polygon the core handed over; nothing
  // here decides its shape or its height.
  const positions: number[] = [];
  for (const piece of sump.shapes) {
    if (piece.ring.length < 3) continue;
    const shape = new THREE.Shape(piece.ring.map((q) => new THREE.Vector2(q.x, q.y)));
    for (const hole of piece.holes) {
      if (hole.length < 3) continue;
      shape.holes.push(new THREE.Path(hole.map((q) => new THREE.Vector2(q.x, q.y))));
    }
    const flat = new THREE.ShapeGeometry(shape);
    const xy = flat.getAttribute('position');
    const index = flat.getIndex();
    const order = index ? Array.from(index.array) : [...Array(xy.count).keys()];
    for (const i of order) positions.push(xy.getX(i), at + 0.03, xy.getY(i));
    flat.dispose();
  }
  if (positions.length === 0) return [bowl];

  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
  geometry.computeVertexNormals();
  const cut = new THREE.Mesh(geometry, new THREE.MeshLambertMaterial({
    color: new THREE.Color(WATER), side: THREE.DoubleSide, transparent: true, opacity: 0.55,
  }));
  cut.userData = { conditionId: c.id, kind: 'sump' };
  return [cut, bowl];
}

// ── the legend, in trade words ─────────────────────────────────────────────

/**
 * The thickness ramp, with depths written on it.
 *
 * "Thicker where it is lighter" is a sentence, not a scale — nobody can read a
 * depth off it, which is what an estimator actually wants from a shaded field.
 */
function thicknessScale(thin: number, thick: number): HTMLElement {
  const wrap = document.createElement('span');
  wrap.className = 'model-scale';

  const head = document.createElement('b');
  head.className = 'model-head';
  head.textContent = 'Insulation thickness';
  wrap.append(head);

  const ramp = document.createElement('i');
  ramp.className = 'model-ramp';
  // The same four greys the surface is shaded in, in the same order, so the ramp
  // IS the field's colouring rather than a second palette that looks like it.
  ramp.style.background = `linear-gradient(to right, ${RAMP.join(', ')})`;
  wrap.append(ramp);

  const ticks = document.createElement('span');
  ticks.className = 'model-ticks';
  for (const v of [thin, (thin + thick) / 2, thick]) {
    const t = document.createElement('span');
    t.textContent = inchLabel(v);
    ticks.append(t);
  }
  wrap.append(ticks);

  const caption = document.createElement('em');
  caption.className = 'model-caption';
  caption.textContent = 'thin at the drains, thick at the high side';
  wrap.append(caption);
  return wrap;
}

/** Everything the legend has to be able to say. */
type Reading = {
  taperedFields: number; regions: number; noFallSquareFeet: number; drains: number;
  crickets: number;
  thin: number; thick: number;
  capReason: CapReason;
  unstatedBuildUp: number; outOfProportion: number; ridgeOffSquare: number;
  cricketSlopeOff: number; overOneLayer: boolean; unbuiltCrickets: number;
  crossedTraces: number; unstatedWall: number;
  unstatedCap: number; copings: number; curbs: number;
  valleys: number; ridges: number; ridgeUneven: number;
  /** Board positions drawn on the roof, when the grid is up. */
  boards: number;
  sumps: number; unsumpedDrains: number; clippedSumps: number;
  refused: readonly SolvedRefusal[];
  /** The tapered field's own colour, which the ramp is drawn in. */
  ramp: string;
};

/**
 * What the drawing is telling you, in words.
 *
 * Half of this is the legend a picture needs and half is the drawing admitting
 * what it does not know. Both belong here. A view that quietly draws its way
 * around a missing build-up, or calls a cricket well-proportioned because it
 * never checked, is worse than one that says so — that is the whole lesson of
 * the version of this file that passed fourteen checks it had written itself.
 */
function legendFor(r: Reading): HTMLElement[] {
  const out: HTMLElement[] = [];
  const row = (swatch: string, text: string) => {
    const el = document.createElement('span');
    el.className = 'model-key';
    const dot = document.createElement('i');
    dot.style.background = swatch;
    el.append(dot, document.createTextNode(text));
    out.push(el);
  };
  const say = (text: string) => {
    // Not a `model-key`: what the drawing cannot say is not a mark ON it, and a
    // key that grows a row every time the job has a gap in it stops being a key.
    const el = document.createElement('span');
    el.className = 'model-said';
    el.textContent = text;
    out.push(el);
  };

  if (r.taperedFields > 0 && Number.isFinite(r.thin) && Number.isFinite(r.thick)) {
    out.push(thicknessScale(r.thin, r.thick));
  }

  // ── the key: four rows, and the ramp ──────────────────────────────────────
  //
  // It had ten and it read as a table of contents. Frame 2 gives a drawing three
  // marks and a scale, and everything else it has to say it says ON THE ROOF —
  // each drain with its sump depth, the curb with what stands behind it, the
  // flat ground with its square feet. That is the difference between a drawing
  // and a debugger, and it is C5's acceptance (D156).
  //
  // What a key must never do is name a mark that is not on this roof, so every
  // row still comes off what actually got drawn.
  if (r.drains > 0) row(WATER, 'arrows follow the fall down the plane it is on');
  if (r.valleys > 0 || r.ridges > 0) {
    const valleys = r.valleys > 0 ? 'the valleys, at 45° to the boards' : '';
    const ridges = r.ridges > 0
      ? `the ${r.ridges === 1 ? 'ridge' : `${r.ridges} ridges`}, halfway between two drains`
      : '';
    row(CREASE, `dashed: ${[valleys, ridges].filter(Boolean).join(', and ')}`);
  }
  if (r.noFallSquareFeet > 0) {
    const why = r.capReason === 'flashing'
      ? 'the roof has reached 8" under the wall it flashes against'
      : 'the boards run out before the water gets anywhere';
    row('#c98a1d', `${Math.round(r.noFallSquareFeet).toLocaleString('en-US')} SF with no fall — ${why}`);
  }
  if (r.regions > 1) row(r.ramp, `the field is split between its ${r.regions} drains`);
  if (r.boards > 0) {
    row(r.ramp, `${r.boards.toLocaleString('en-US')} boards, each one lettered off the table`);
  }

  // ── and what the drawing cannot say, said out loud ────────────────────────
  //
  // Half of what follows is the drawing admitting what it does not know, and it
  // belongs under the key rather than in it. A view that quietly draws its way
  // around a missing build-up, or calls a cricket well-proportioned because it
  // never looked, is worse than one that says so — that is the whole lesson of
  // the version of this file that passed fourteen checks it had written itself.
  if (r.unsumpedDrains > 0) {
    say(r.unsumpedDrains === 1
      ? 'a drain has no sump: nothing states its bowl or its width'
      : `${r.unsumpedDrains} drains have no sump: nothing states a bowl or a width`);
  }
  if (r.clippedSumps > 0) {
    say(r.clippedSumps === 1
      ? 'a sump runs off the edge of the roof it is cut into'
      : `${r.clippedSumps} sumps run off the edge of the roof they are cut into`);
  }
  if (r.overOneLayer) say(`over ${SINGLE_LAYER_MAX_INCHES}" — this is a second layer of board, not a thicker one`);
  // NRCA p.166: two adjoining patterns should meet at one common thickness, and
  // where they cannot an area divider is what carries the step.
  if (r.ridgeUneven > 0) {
    say(`the two patterns meet ${inchLabel(r.ridgeUneven)} apart at a ridge`
      + ' — NRCA wants one thickness there, or an area divider to carry the step');
  }
  for (const refused of r.refused) {
    say(`a tapered area is not laid out: ${refused.why}`);
  }
  if (r.capReason === 'flashing') {
    say(`capped by flashing height — NRCA wants ${MIN_FLASHING_INCHES}" above the finished roof`);
  }
  if (r.crossedTraces > 0) {
    say(r.crossedTraces === 1
      ? 'a traced area crosses itself, so it is not drawn — its square footage would come out zero'
      : `${r.crossedTraces} traced areas cross themselves and are not drawn`);
  }
  if (r.unstatedWall > 0) {
    say(r.unstatedWall === 1
      ? '1 run has no wall thickness stated, so it is drawn as a face — fill it in and it stands up'
      : `${r.unstatedWall} runs have no wall thickness stated`);
  }
  if (r.unstatedCap > 0) {
    say(r.unstatedCap === 1
      ? '1 wall has a cap width typed but no wall thickness to centre it on, so no coping is drawn'
      : `${r.unstatedCap} walls have a cap width typed but no wall thickness to centre it on`);
  }
  if (r.unstatedBuildUp > 0) {
    say(r.unstatedBuildUp === 1
      ? '1 area has no build-up stated — put thicknesses on its layers and it stands up'
      : `${r.unstatedBuildUp} areas have no build-up stated`);
  }
  if (r.unbuiltCrickets > 0) {
    say(r.unbuiltCrickets === 1
      ? '1 ridge is not between two drains yet, so there is no cricket to draw'
      : `${r.unbuiltCrickets} ridges are not between two drains yet`);
  }
  if (r.outOfProportion > 0) {
    say(r.outOfProportion === 1
      ? 'a cricket is too long for its width — the valley will pond however steep it is cut'
      : `${r.outOfProportion} crickets are too long for their width`);
  }
  if (r.ridgeOffSquare > 0) {
    say('a traced ridge is not square to the drains it serves — one of the two is wrong');
  }
  if (r.cricketSlopeOff > 0) {
    say(`a cricket is not cut at ${CRICKET_SLOPE_MULTIPLE}× the field beside it`);
  }
  return out;
}

// ── selecting, which is not editing ────────────────────────────────────────

const raycaster = new THREE.Raycaster();

function pick(canvas: HTMLCanvasElement, event: MouseEvent) {
  if (!camera) return;
  const rect = canvas.getBoundingClientRect();
  const pointer = new THREE.Vector2(
    ((event.clientX - rect.left) / rect.width) * 2 - 1,
    -((event.clientY - rect.top) / rect.height) * 2 + 1,
  );
  raycaster.setFromCamera(pointer, camera);
  const hit = raycaster.intersectObjects(world.children, false)[0];
  const id = hit?.object.userData['conditionId'];
  if (typeof id === 'string') select({ kind: 'condition', id });
}

// ── the acts, on the roof ──────────────────────────────────────────────────

/**
 * Pick an act off the rail.
 *
 * Select is the absence of an act, not an act of its own: it is what the view
 * does when nobody has asked for anything else. Choosing any other act puts the
 * one before it down, which is what takes its marks off the roof.
 */
function chooseAct(id: string): void {
  const wanted: VerbName | null = isVerb(id) ? id : null;
  useVerb(wanted);
  chrome?.setActiveTool(wanted ?? 'select');
  if (wanted) showVerbStrip();
  else redrawWorld();
  drawMarks();
}

/** The act's own strip, in the window that owns one. */
function showVerbStrip(): void {
  const verb = verbStrip();
  if (verb) chrome?.strip.show(verb.name, verb.fields);
}

/**
 * A place on the roof, picked by hand.
 *
 * The same ray the selection uses, read for WHERE it struck rather than for
 * what: a place is a point on the plan in feet, which is what the core answers
 * in. The tapered surface is asked for first, for the reason the readout asks
 * for it first — a curb, a cricket and a parapet all stand on the roof, and an
 * estimator pointing at one of them is still pointing at a place.
 */
function pickPlace(canvas: HTMLCanvasElement, event: MouseEvent): void {
  if (!camera) return;
  const rect = canvas.getBoundingClientRect();
  raycaster.setFromCamera(new THREE.Vector2(
    ((event.clientX - rect.left) / rect.width) * 2 - 1,
    -((event.clientY - rect.top) / rect.height) * 2 + 1,
  ), camera);
  const hits = raycaster.intersectObjects(world.children, false);
  if (hits.length === 0) return;
  const nearestOf = (kind: string) =>
    hits.find((h) => String(h.object.userData['kind'] ?? '') === kind);
  const hit = nearestOf('taper') ?? nearestOf('build-up') ?? nearestOf('datum') ?? hits[0]!;
  verbPick({ x: hit.point.x, y: hit.point.z });
}

/**
 * What the act has left on the roof, in three dimensions.
 *
 * The marks arrive in feet on the plan and stand at whatever depth the solved
 * package is at that place, so a pick sits ON the roof rather than hovering at
 * the datum under it. Its own group, rebuilt whole and re-added after every
 * rebuild of the world, because `build` empties the world and the marks are not
 * the roof.
 */
function drawMarks(): void {
  markWords = [];
  if (!world || cutting) return;
  if (marks) {
    world.remove(marks);
    marks.clear();
  }
  marks = new THREE.Group();
  world.add(marks);

  const standsOn = (p: { x: number; y: number }) => {
    const field = fieldsNow.find((f) => pointInPolygon(p, f.ring));
    if (!field) return 0;
    const here = readTaperAt(field, p);
    return field.elevation + inches(here?.thicknessInches ?? field.startInches);
  };

  for (const mark of verbMarks()) {
    const lift = 0.2;
    const a = new THREE.Vector3(mark.at.x, standsOn(mark.at) + lift, mark.at.y);
    if (mark.ring) {
      const ring = mark.ring.map((p) => new THREE.Vector3(p.x, standsOn(p) + lift, p.y));
      const shape = new THREE.Line(
        new THREE.BufferGeometry().setFromPoints(ring),
        new THREE.LineBasicMaterial({ color: new THREE.Color(WATER) }),
      );
      shape.raycast = () => {};
      marks.add(shape);
    }
    if (mark.to) {
      const b = new THREE.Vector3(mark.to.x, standsOn(mark.to) + lift, mark.to.y);
      const line = new THREE.Line(
        new THREE.BufferGeometry().setFromPoints([a, b]),
        new THREE.LineBasicMaterial({ color: new THREE.Color(WATER) }),
      );
      line.raycast = () => {};
      marks.add(line);
    }
    if (mark.kind === 'high' || mark.kind === 'low' || mark.kind === 'corner') {
      const dot = new THREE.Mesh(
        new THREE.SphereGeometry(0.8, 10, 8),
        new THREE.MeshBasicMaterial({ color: new THREE.Color(WATER) }),
      );
      dot.position.copy(a);
      dot.raycast = () => {};
      marks.add(dot);
    }
    if (mark.text) {
      const at = mark.to
        ? new THREE.Vector3(
          (mark.at.x + mark.to.x) / 2,
          (standsOn(mark.at) + standsOn(mark.to)) / 2 + lift,
          (mark.at.y + mark.to.y) / 2,
        )
        : a;
      markWords.push({ kind: 'pick', at, text: mark.text });
    }
  }
  labelsNow = [...roofLabels, ...markWords];
}

/**
 * Where on the screen a condition is drawn, in the canvas's own client
 * coordinates — the coordinates a pointer event carries and `pick()` reads.
 *
 * For checking only, like the rest of `__roofnerdModel`. Nothing in the program
 * calls it: a person aims with their hand, and this exists so that a check can
 * aim at the roof field without a coordinate typed into it by somebody who had
 * the camera in front of them.
 *
 * It does not merely project a centre and hope. A heightfield's bounding-box
 * centre hangs in the air above the surface, a parapet's sits inside the wall,
 * and a ring's sits in the hole — so each candidate point is put back through
 * the SAME raycast `pick()` uses, and the first one that comes back as this
 * condition is the one returned. `hits` says what a click there would select,
 * whatever that turns out to be, so a check can never read a point that would
 * have picked something else as proof of a point that would not.
 */
function screenPointOf(conditionId: string): {
  x: number; y: number; inside: boolean; of: string | null; hits: string | null;
} | null {
  if (!camera || !surface || !world) return null;
  const rect = surface.getBoundingClientRect();

  const toScreen = (p: THREE.Vector3) => {
    const v = p.clone().project(camera);
    return {
      x: rect.left + (v.x * 0.5 + 0.5) * rect.width,
      y: rect.top + (-v.y * 0.5 + 0.5) * rect.height,
    };
  };
  const within = (at: { x: number; y: number }) =>
    at.x >= rect.left && at.x <= rect.right && at.y >= rect.top && at.y <= rect.bottom;

  const candidates: { at: { x: number; y: number }; of: string }[] = [];
  for (const object of world.children) {
    if (object.userData['conditionId'] !== conditionId) continue;
    const mesh = object as THREE.Mesh;
    const position = mesh.geometry?.getAttribute('position');
    if (!position) continue;
    const of = String(mesh.userData['kind'] ?? '');

    // The centroid of the vertices first — on a surface it is on the surface,
    // which a box centre is not — then a scattering of the vertices themselves,
    // for the shapes a centroid can miss.
    const centre = new THREE.Vector3();
    const v = new THREE.Vector3();
    for (let i = 0; i < position.count; i += 1) {
      centre.add(v.fromBufferAttribute(position, i));
    }
    centre.multiplyScalar(1 / Math.max(1, position.count));
    candidates.push({ at: toScreen(mesh.localToWorld(centre)), of });

    const step = Math.max(1, Math.floor(position.count / 12));
    for (let i = 0; i < position.count; i += step) {
      const point = new THREE.Vector3().fromBufferAttribute(position, i);
      candidates.push({ at: toScreen(mesh.localToWorld(point)), of });
    }
  }
  if (candidates.length === 0) return null;

  for (const candidate of candidates) {
    // Rounded BEFORE it is tested, because rounded is what it is used at. A
    // point verified at 640.4 and handed back as 640 is a point nothing ever
    // cast a ray through, and at the corner of a roof one pixel is the whole
    // difference between the deck and the wall standing on it: a check aimed at
    // the field, was told the field was there, clicked the number it was given
    // and selected the parapet.
    const at = { x: Math.round(candidate.at.x), y: Math.round(candidate.at.y) };
    if (!within(at)) continue;
    const hits = whatIsAt(rect, at);
    if (hits === conditionId) {
      return { x: at.x, y: at.y, inside: true, of: candidate.of, hits };
    }
  }

  // Nothing on this condition answers a ray from where it is drawn — it is
  // behind something, or off screen. Answer with the best point there is and
  // what is actually there, rather than with a point nothing would pick.
  const first = candidates[0]!;
  const at = { x: Math.round(first.at.x), y: Math.round(first.at.y) };
  return {
    x: at.x, y: at.y,
    inside: within(at), of: first.of,
    hits: within(at) ? whatIsAt(rect, at) : null,
  };
}

/**
 * Where a place ON THE ROOF lands on the screen, in feet on the plan.
 *
 * For checking only, like the rest of `__roofnerdModel`. `screenPointOf` aims at
 * a condition, which is enough to pick one; a check that reads the thickness at
 * three places has to aim at the places, and the only thing that knows where a
 * place on the roof is drawn is the drawing. The height comes off the solved
 * field, so the point is on the surface rather than over it, and the answer says
 * what a ray through it would actually strike.
 */
function screenPointAt(x: number, y: number): {
  x: number; y: number; inside: boolean; of: string | null; hits: string | null;
} | null {
  if (!camera || !surface || !world) return null;
  const at = { x, y };
  const field = fieldsNow.find((f) => pointInPolygon(at, f.ring));
  if (!field) return null;
  const here = readTaperAt(field, at);
  if (!here) return null;
  const height = field.elevation + inches(here.thicknessInches);
  const rect = surface.getBoundingClientRect();
  const v = world.localToWorld(new THREE.Vector3(x, height, y)).project(camera);
  const on = {
    x: Math.round(rect.left + (v.x * 0.5 + 0.5) * rect.width),
    y: Math.round(rect.top + (-v.y * 0.5 + 0.5) * rect.height),
  };
  const within = on.x >= rect.left && on.x <= rect.right && on.y >= rect.top && on.y <= rect.bottom;
  return {
    x: on.x, y: on.y, inside: within, of: 'taper',
    hits: within ? whatIsAt(rect, on) : null,
  };
}

/**
 * Where the whole roof lands on the canvas, in the canvas's own pixels.
 *
 * For checking only, like the rest of `__roofnerdModel`. "The roof fits the
 * window" is the one claim in this view that a count of meshes cannot reach: a
 * roof drawn at a twentieth of the size it should be has every mesh a check
 * could ask for and is still the defect. This answers with the rectangle the
 * roof actually occupies and the rectangle it had to fill, and lets the check do
 * the division.
 */
function roofOnScreen(): {
  x: number; y: number; w: number; h: number; of: { w: number; h: number };
} | null {
  if (!camera || !surface || !world) return null;
  const box = new THREE.Box3().setFromObject(world);
  if (box.isEmpty()) return null;
  const rect = surface.getBoundingClientRect();
  let minX = Infinity; let maxX = -Infinity; let minY = Infinity; let maxY = -Infinity;
  for (const x of [box.min.x, box.max.x]) {
    for (const y of [box.min.y, box.max.y]) {
      for (const z of [box.min.z, box.max.z]) {
        const v = new THREE.Vector3(x, y, z).project(camera);
        const sx = (v.x * 0.5 + 0.5) * rect.width;
        const sy = (-v.y * 0.5 + 0.5) * rect.height;
        minX = Math.min(minX, sx); maxX = Math.max(maxX, sx);
        minY = Math.min(minY, sy); maxY = Math.max(maxY, sy);
      }
    }
  }
  return {
    x: Math.round(minX), y: Math.round(minY),
    w: Math.round(maxX - minX), h: Math.round(maxY - minY),
    of: { w: Math.round(rect.width), h: Math.round(rect.height) },
  };
}

/** What a click at this point would select — the same ray `pick()` casts. */
function whatIsAt(rect: DOMRect, at: { x: number; y: number }): string | null {
  if (!camera || !world) return null;
  raycaster.setFromCamera(new THREE.Vector2(
    ((at.x - rect.left) / rect.width) * 2 - 1,
    -((at.y - rect.top) / rect.height) * 2 + 1,
  ), camera);
  const hit = raycaster.intersectObjects(world.children, false)[0];
  const id = hit?.object.userData['conditionId'];
  return typeof id === 'string' ? id : null;
}

/** Selected reads as the accent everywhere else does; nothing else changes. */
function highlight() {
  if (!world) return;
  const id = selectedConditionId();
  for (const object of world.children) {
    const mesh = object as THREE.Mesh;
    const material = mesh.material as THREE.MeshLambertMaterial | undefined;
    if (!material || !('emissive' in material)) continue;
    const mine = mesh.userData['conditionId'] === id;
    material.emissive = new THREE.Color(mine ? '#2f6fd0' : '#000000');
    material.emissiveIntensity = mine ? 0.55 : 0;
  }
}

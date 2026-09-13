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
  CRICKET_SLOPE_MULTIPLE, MIN_FLASHING_INCHES, NOTHING_SOLVED, SINGLE_LAYER_MAX_INCHES,
  buildUpInches, cricketOf, fieldAt, fieldOf, formatFeetInches, pointInPolygon, readFacetAt,
  readTaperAt, selfIntersects, sumpOf,
  type CapReason, type Condition, type Page, type Point, type Properties,
  type SolvedCrease, type SolvedCricket, type SolvedRefusal, type SolvedRidge, type SolvedSump,
  type SolvedTaper, type SolvedTaperField, type SolvedTaperRegion,
} from '@roofnerd/engine';
import { at, doc, solveId, solveTaper, subscribe, type Doc } from '../doc.js';
import { feet, feetPerUnit } from '../scale.js';
import { select, selectedConditionId, watchSelection } from '../selection.js';
import { stripField, type RailTool, type Reading as StripReading, type SurfaceChrome } from '../chrome.js';
import { watchTheme } from '../theme.js';
import { hueFor } from '../icons.js';

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
 * How far apart the flow arrows are placed, in feet.
 *
 * The core places them — this is the same spacing, and it is here only to size
 * the dart against it. A dart is a mark on a drawing and how big it is drawn is
 * the drawing's business; where it goes and which way it points are the roof's.
 */
const ARROW_FEET = 8;

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
 * One, today: pointing at the roof to pick what you are looking at. The
 * handoff's Model frame carries "Set high and low", and that arrives with the
 * renderers — it is not in the rail until it does something.
 */
export const modelTools = (pick: (id: string) => void): readonly RailTool[] =>
  [{ id: 'select', label: 'Select', icon: 'select', onPick: () => pick('select') }];

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

  const look = chip('look around', () => {
    looking = !looking;
    if (controls) controls.enabled = looking;
    refreshChips();
  });

  refreshChips = () => {
    (Object.keys(CORNERS) as Corner[]).forEach((id, i) => {
      corners[i]!.classList.toggle('on', id === corner && !looking);
    });
    EXAGGERATIONS.forEach((times, i) => {
      verts[i]!.classList.toggle('on', times === exaggeration);
    });
    look.classList.toggle('on', looking);
  };
  refreshChips();

  surroundings.setViewTools([chipGroup(...corners), chipGroup(fit), chipGroup(...verts), chipGroup(look)]);
}

export function mountModel(host: HTMLElement, surroundings: SurfaceChrome): void {
  unmount();
  chrome = surroundings;
  surroundings.setTools(modelTools(() => undefined));
  surroundings.setActiveTool('select');
  surroundings.setViewTools([]);
  surroundings.setReadout(EMPTY_READING);
  surroundings.strip.show('Select', [
    stripField('On the roof', 'click to pick what you are looking at'),
    stripField('Units', 'SF · LF · EA · SQ'),
  ]);
  framed = false;
  corner = 'NE';
  exaggeration = 1;
  looking = false;
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

  // What the roof does where the pointer is reads in the window's status strip
  // now, which is where the handoff puts it: a continuous readout under the
  // cursor, in the same region whatever the surface is showing. It used to be a
  // line of its own under this canvas, which meant the Plan had nowhere to put
  // the same four facts when it learns to compute them.
  host.append(canvas, overlay);

  viewChips(surroundings);

  renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));

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
      built = build(doc(), solvedNow);
    } catch (e) {
      world.clear();
      legend.replaceChildren();
      note.textContent = `This roof did not draw, so nothing is shown — what you would have seen would not have been this job. ${e instanceof Error ? e.message : String(e)}`;
      note.classList.add('model-broke');
      return;
    }
    note.classList.remove('model-broke');
    legend.replaceChildren(...built.legend);
    note.textContent = built.note;
    if (solveTrouble) {
      // The roof did not solve. Nothing tapered is drawn, and the window says
      // why rather than showing a deck with no fall on it as though that were
      // the answer.
      note.textContent = `This roof did not solve, so the taper is not shown. ${solveTrouble}`;
      note.classList.add('model-broke');
    }
    if (!controls) controls = projection(canvas);
    lastSpan = built.span;
    resize(canvas);
    // Once they have looked around it is their camera. Until then the view
    // follows the roof, so tracing something new does not leave it off screen —
    // and so the roof is in the window the first time it is opened, in every
    // window it is opened in.
    if (!framed && built.span > 0) frame();
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
        const got = await solveTaper();
        if (token !== pulling) return;
        solvedNow = got;
        drawnSolveId = got.solveId;
        solveTrouble = null;
      } catch (e) {
        if (token !== pulling) return;
        solvedNow = NOTHING_SOLVED;
        drawnSolveId = want;
        solveTrouble = e instanceof Error ? e.message : String(e);
      }
    }
    draw();
  };

  stopSubscribing = subscribe(() => void refresh());
  stopWatching = watchSelection(() => highlight());

  canvas.addEventListener('click', (e) => pick(canvas, e));
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
  };
  tick();
}

/** Put the Model down. The surface switches views in place; this is the other half. */
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
  controls?.dispose();
  controls = null;
  renderer?.dispose();
  renderer = null;
  surface = null;
  stopWatchingTheme?.();
  stopWatchingTheme = null;
  refreshChips = () => undefined;
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
};

/**
 * The tapered fields ON SCREEN, for the readout.
 *
 * The solve's own list is not the same list: a trace that crosses itself is
 * solved and not drawn, and the strip must not answer over a surface nobody can
 * see. These are the ones that got drawn, in the solve's own words.
 */
let fieldsNow: SolvedTaperField[] = [];

/** The solved roof this drawing was built from. */
let solvedNow: SolvedTaper = NOTHING_SOLVED;
/** Which solve is on screen. Nothing is, until the first one is pulled. */
let drawnSolveId = -1;
/** The pull in flight, so an overtaken answer is dropped rather than drawn. */
let pulling = 0;
/** Why the roof did not solve, when it did not. */
let solveTrouble: string | null = null;

function build(d: Doc, taper: SolvedTaper): Built {
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
              const mesh = regionSurface(solvedField, region, colour, c);
              if (!mesh) return;
              world.add(mesh);
              drawn += 1;
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
          const wall = props['WALL'];
          if (wall === undefined) unstatedWall += 1;
          const stood = standUp(ring, elevation, height, wall, colour, c);
          world.add(stood.mesh);
          if (c.role === 'curb') curbs += 1;
          // The cap on top of it. A coping is formed metal over a wall and a
          // curb cap is formed metal over a curb; both are as wide as the
          // detail says and neither is as wide as the wall. With no width
          // stated nothing is drawn and the legend says so, which is the same
          // rule the wall thickness already follows.
          if (stood.cap) { world.add(stood.cap); copings += 1; }
          else if (wall !== undefined && props['W'] === undefined) unstatedCap += 1;
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
      unstatedCap, copings, curbs, valleys, ridges, ridgeUneven,
      sumps, unsumpedDrains, clippedSumps,
      refused: taper.refused,
      ramp: (tapered?.color ?? '#4a6b8a'),
    }),
    note: anything
      ? (unscaled ? 'Some of this roof is on a sheet nobody has scaled, so it is not drawn.' : '')
      : 'Nothing traced yet. Trace a roof on the Plan and it stands up here.',
  };
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
  field: SolvedTaperField, region: SolvedTaperRegion, colour: string, c: Condition,
) {
  const elevation = field.elevation;
  const positions: number[] = [];
  const colours: number[] = [];
  const base = new THREE.Color(colour);
  const noFall = new THREE.Color('#c98a1d');

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
      const shade = facet.flat
        ? noFall
        : base.clone().lerp(new THREE.Color('#ffffff'), Math.min(0.65, inches(depth) * 1.4));
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
 * Where the water goes, as arrows: one every few feet, pointing down the plane.
 *
 * The core places them and points them — down the gradient of whichever plane
 * governs, which on a four-way layout is square to the boards and not at the
 * drain, because water runs down the fall and then along the valley. Flat ground
 * gets none: there is nothing to point at when the fall has run out, and the core
 * leaves those out.
 *
 * One mesh for every arrow, not one object each. `ArrowHelper` is an Object3D
 * carrying a line and a cone, so a field of them is two draw calls apiece — fine
 * on a demo, and on a real 36 by 24 sheet it is hundreds before the estimator has
 * done anything. The whole field is one instanced cone with a rotation per
 * instance. This came out of the prior 3D work on this machine rather than out of
 * a profiler, which is the cheaper way to learn it.
 */
function flowField(field: SolvedTaperField) {
  const flow = new THREE.Group();
  const arrowAt = field.flow.map((a) => ({
    x: a.x, y: field.elevation + inches(a.thicknessInches) + 0.08, z: a.y, angle: a.angle,
  }));
  if (arrowAt.length === 0) return flow;

  // Slim and long, so it reads as a dart pointing somewhere. The first pass at
  // instancing used the same proportions as the ArrowHelper's head without its
  // shaft, and a field of them read as a scatter of blobs — caught by opening the
  // screenshot rather than by any check, which is the only way a thing like this
  // ever gets caught.
  const cone = new THREE.ConeGeometry(ARROW_FEET * 0.06, ARROW_FEET * 0.55, 6);
  // A cone points up its own +Y; lay it on its side so it points along +X, which
  // is what the angle is measured from.
  cone.rotateZ(-Math.PI / 2);
  const arrows = new THREE.InstancedMesh(
    cone,
    new THREE.MeshLambertMaterial({ color: 0x1f7a8c }),
    arrowAt.length,
  );
  const placed = new THREE.Object3D();
  for (let i = 0; i < arrowAt.length; i += 1) {
    const a = arrowAt[i]!;
    placed.position.set(a.x, a.y, a.z);
    placed.rotation.set(0, -a.angle, 0);
    placed.updateMatrix();
    arrows.setMatrixAt(i, placed.matrix);
  }
  arrows.instanceMatrix.needsUpdate = true;
  // Not pickable: an arrow is the drawing explaining itself, not a thing on the
  // roof. Clicking one should select the field under it.
  arrows.raycast = () => {};
  flow.add(arrows);
  return flow;
}

/**
 * A parapet: the run, stood up off its base, at the thickness it is told.
 *
 * The traced line is the **reference line**, and by the convention every real
 * tool keeps, it is the *exterior* face — Bonsai's offset default is EXTERIOR
 * with `OffsetFromReferenceLine` at zero, and a parapet traced on a roof plan is
 * traced round the outside of the building. So the wall grows inboard from the
 * line, and up.
 *
 * Growing inboard is the one thing here that has to be worked out rather than
 * assumed, and it is worked out per segment rather than once for the run.
 *
 * The first attempt took the side off the ring's winding, which is right for a
 * closed ring and wrong for the thing an estimator actually traces: a parapet
 * round three sides of a building is an open polyline, and its two legs run in
 * opposite directions, so one leg's wall came out inboard and the other's
 * outboard. Caught by checking the drawn solid against its own trace rather than
 * by looking at it — from the outside it looked like a parapet.
 *
 * So each segment offsets toward the middle of the run. For a rectangle that is
 * the inside; for a three-sided run it is the building. Which is the same thing
 * said twice, and it is what the trace itself already knows.
 *
 * With no thickness stated it stays a ribbon, and the legend says the thickness
 * is not stated. That is not the same as inventing one.
 */
function standUp(
  ring: Point[], elevation: number, height: number, thick: number | undefined,
  colour: string, c: Condition,
): { mesh: THREE.Mesh; cap: THREE.Mesh | null } {
  const positions: number[] = [];
  /** The top of the wall, segment by segment, for the cap that goes on it. */
  const capPositions: number[] = [];
  const capWidth = c.properties?.['W'];
  const low = elevation;
  const high = elevation + height;

  const middle = {
    x: ring.reduce((sum, p) => sum + p.x, 0) / ring.length,
    y: ring.reduce((sum, p) => sum + p.y, 0) / ring.length,
  };
  const t = thick === undefined ? 0 : inches(thick);

  const quad = (
    a: { x: number; y: number }, b: { x: number; y: number }, yA: number, yB: number,
  ) => {
    positions.push(a.x, yA, a.y, b.x, yA, b.y, a.x, yB, a.y);
    positions.push(b.x, yA, b.y, b.x, yB, b.y, a.x, yB, a.y);
  };

  for (let i = 0; i < ring.length - 1; i += 1) {
    const a = ring[i]!;
    const b = ring[i + 1]!;
    quad(a, b, low, high);
    if (t > 0) {
      const dx = b.x - a.x;
      const dy = b.y - a.y;
      const len = Math.hypot(dx, dy) || 1;
      // Square to the run, by the wall's own thickness, on whichever side of
      // this segment faces the middle of the run.
      const ux = -dy / len;
      const uy = dx / len;
      const mx = (a.x + b.x) / 2;
      const my = (a.y + b.y) / 2;
      const inboard = (middle.x - mx) * ux + (middle.y - my) * uy >= 0 ? 1 : -1;
      const nx = ux * t * inboard;
      const ny = uy * t * inboard;
      const a2 = { x: a.x + nx, y: a.y + ny };
      const b2 = { x: b.x + nx, y: b.y + ny };
      quad(a2, b2, low, high);          // the inboard face
      quad(a, a2, low, high);           // and the two ends close it
      quad(b, b2, low, high);
      // The cap, which is where the coping goes.
      positions.push(a.x, high, a.y, b.x, high, b.y, a2.x, high, a2.y);
      positions.push(b.x, high, b.y, b2.x, high, b2.y, a2.x, high, a2.y);

      // And the formed metal over it, as wide as the detail says. It is drawn
      // as a surface and not a solid on purpose: 24 ga is twenty-eight
      // thousandths of an inch, and a plate thick enough to see would be a
      // thickness nobody stated.
      if (capWidth !== undefined && capWidth > 0) {
        // Centred on the wall, so it overhangs both faces the way a cap does.
        const mx2 = ux * (t / 2) * inboard;
        const my2 = uy * (t / 2) * inboard;
        const outX = ux * (capWidth / 2) * inboard;
        const outY = uy * (capWidth / 2) * inboard;
        const top = high + 0.02;
        const cA = { x: a.x + mx2 - outX, y: a.y + my2 - outY };
        const cB = { x: b.x + mx2 - outX, y: b.y + my2 - outY };
        const cC = { x: b.x + mx2 + outX, y: b.y + my2 + outY };
        const cD = { x: a.x + mx2 + outX, y: a.y + my2 + outY };
        capPositions.push(cA.x, top, cA.y, cB.x, top, cB.y, cC.x, top, cC.y);
        capPositions.push(cA.x, top, cA.y, cC.x, top, cC.y, cD.x, top, cD.y);
      }
    }
  }

  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
  geometry.computeVertexNormals();
  const mesh = new THREE.Mesh(geometry, new THREE.MeshLambertMaterial({
    color: new THREE.Color(colour), side: THREE.DoubleSide,
  }));
  // A curb is a run with a height, the same as a parapet is, and it is drawn by
  // the same arithmetic. What it is called here is what it is: a parapet is the
  // edge of the roof and a curb is a thing standing on it, and a check counting
  // one must not find the other.
  const kind = c.role === 'curb' ? 'curb' : (t > 0 ? 'parapet' : 'parapet-ribbon');
  mesh.userData = { conditionId: c.id, kind };

  let cap: THREE.Mesh | null = null;
  if (capPositions.length > 0) {
    const capGeometry = new THREE.BufferGeometry();
    capGeometry.setAttribute('position', new THREE.Float32BufferAttribute(capPositions, 3));
    capGeometry.computeVertexNormals();
    cap = new THREE.Mesh(capGeometry, new THREE.MeshLambertMaterial({
      color: new THREE.Color(GALVANIZED), side: THREE.DoubleSide,
    }));
    cap.userData = { conditionId: c.id, kind: 'coping' };
  }
  return { mesh, cap };
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
    color: new THREE.Color(WATER), dashSize: 2, gapSize: 1.4,
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

/** Inches as a roofer writes them: 1/2", 2 1/4", 4". */
function inchLabel(v: number): string {
  const whole = Math.floor(v);
  const frac = v - whole;
  const eighths = Math.round(frac * 8);
  const names = ['', '1/8', '1/4', '3/8', '1/2', '5/8', '3/4', '7/8'];
  if (eighths === 8) return `${whole + 1}"`;
  if (eighths === 0) return `${whole}"`;
  return whole === 0 ? `${names[eighths]}"` : `${whole} ${names[eighths]}"`;
}

/**
 * The thickness ramp, with depths written on it.
 *
 * "Thicker where it is lighter" is a sentence, not a scale — nobody can read a
 * depth off it, which is what an estimator actually wants from a shaded field.
 */
function thicknessScale(thin: number, thick: number, base: string): HTMLElement {
  const wrap = document.createElement('span');
  wrap.className = 'model-scale';

  const head = document.createElement('b');
  head.className = 'model-head';
  head.textContent = 'Insulation thickness';
  wrap.append(head);

  const ramp = document.createElement('i');
  ramp.className = 'model-ramp';
  // The same lerp the surface uses, so the ramp is the field's own colouring
  // rather than a second palette that happens to look like it.
  ramp.style.background = `linear-gradient(to right, ${base}, #ffffff)`;
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
    const el = document.createElement('span');
    el.className = 'model-key model-says';
    el.textContent = text;
    out.push(el);
  };

  if (r.taperedFields > 0 && Number.isFinite(r.thin) && Number.isFinite(r.thick)) {
    out.push(thicknessScale(r.thin, r.thick, r.ramp));
    // The surface is now the layout — planes off the drains, valleys where they
    // tie — and it is still not the board schedule: a real taper is lettered
    // boards on a four foot module, in panel repeats, with flat fill under them,
    // so the thickness steps at a board edge rather than sliding. Saying which of
    // the two this is, is the difference between a drawing that helps and one that
    // gets ordered from.
    say('the planes and the valleys are the layout; the lettered boards are not on it yet');
  }

  // The key: what the marks on the roof are, and only the marks that are on it.
  // A key that names a thing the drawing does not carry is a sentence that is
  // not true of this roof, which is the same defect as a wrong number.
  if (r.drains > 0) row(WATER, 'arrows follow the fall down the plane it is on');
  if (r.noFallSquareFeet > 0) row('#c98a1d', 'no fall');
  if (r.valleys > 0) row(WATER, 'dashed: the valleys, at 45° to the boards');
  if (r.ridges > 0) {
    row(WATER, r.ridges === 1
      ? 'dashed: the ridge, halfway between two drains'
      : `dashed: the ${r.ridges} ridges, halfway between two drains`);
  }
  // The count of them, in the words a roofer would use for it. "Region" is the
  // core's word for the ground one drain takes and it is not a word on a roof, so
  // the key says what happened to the field instead of naming the type.
  if (r.regions > 1) row(r.ramp, `the field is split between its ${r.regions} drains`);
  if (r.sumps > 0) {
    row(WATER, r.sumps === 1
      ? '1 drain, in the sump the layout cuts for it'
      : `${r.sumps} drains, each in the sump the layout cuts for it`);
  }
  // A drain nobody sized a sump for used to get a four foot one drawn anyway. NRCA
  // p.164 sizes it off the bowl — bowl plus about 24 in — and nothing in the job
  // says the bowl, so there is nothing to draw and this says which drains.
  if (r.unsumpedDrains > 0) {
    say(r.unsumpedDrains === 1
      ? 'a drain has no sump: nothing states its bowl or its width'
      : `${r.unsumpedDrains} drains have no sump: nothing states a bowl or a width`);
  }
  if (r.clippedSumps > 0) {
    say(r.clippedSumps === 1
      ? "a sump runs off the edge of the roof it is cut into"
      : `${r.clippedSumps} sumps run off the edge of the roof they are cut into`);
  }
  if (r.copings > 0) row(GALVANIZED, r.copings === 1 ? 'coping on the wall' : 'coping on the walls');
  if (r.curbs > 0) row(GALVANIZED, r.curbs === 1 ? '1 curb' : `${r.curbs} curbs`);
  if (r.crickets > 0) row('#8a5a1d', r.crickets === 1 ? '1 cricket' : `${r.crickets} crickets`);

  if (r.noFallSquareFeet > 0) {
    const why = r.capReason === 'flashing'
      ? 'the roof has reached 8" under the wall it flashes against'
      : 'the boards run out before the water gets anywhere';
    row('#c98a1d', `${Math.round(r.noFallSquareFeet).toLocaleString('en-US')} SF with no fall — ${why}`);
  }
  if (r.overOneLayer) say(`over ${SINGLE_LAYER_MAX_INCHES}" — this is a second layer of board, not a thicker one`);
  // NRCA p.166: two adjoining patterns should meet at one common thickness, and
  // where they cannot an area divider is what carries the step. With one slope over
  // the whole field they meet wherever the line joining two drains is square to
  // the boards, and they do not where it is slanted — so the drawing says by how
  // much rather than letting the mesh average it.
  if (r.ridgeUneven > 0) {
    say(`the two patterns meet ${inchLabel(r.ridgeUneven)} apart at a ridge`
      + ' — NRCA wants one thickness there, or an area divider to carry the step');
  }
  // And a tapered area the layout would not lay out at all says so in its own
  // words. The core is the only thing that knows why, so the sentence is the
  // core's; what would be wrong is silence, or a surface standing in for one.
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
      ? '1 wall has no cap width stated, so no coping is drawn on it'
      : `${r.unstatedCap} walls have no cap width stated, so no coping is drawn on them`);
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

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
//   the taper   thickness at the drain, plus the slope, over the distance to
//               the nearest drain — sampled on a grid and drawn as a surface
//   a cricket   two planes falling from a ridge the drains put there, not a
//               ridge somebody drew
//   no fall     where the build-up has hit its ceiling and the roof goes flat,
//               so the water stops moving
//
// The conventions above are not this file's opinion. They live in the engine at
// `roof.ts`, each with the manual or the tool it came from, and they are tested
// there against the source rather than against this drawing. This file draws
// what they say. That order matters: it was the other way round once, and the
// check passed fourteen out of fourteen against geometry nobody had checked.
//
// There is no editing in this window. Clicking selects, and selecting is not
// editing — it is the same one selection every other editor watches.

import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import {
  CRICKET_SLOPE_MULTIPLE, MIN_FLASHING_INCHES, SINGLE_LAYER_MAX_INCHES,
  buildUpInches, capInches, maxLengthToWidth, ridgeBetween, ridgeDisagreement,
  selfIntersects, type CapReason, type Spot,
} from '@roofnerd/engine';
import { at, doc, subscribe, type Doc } from '../doc.js';
import { select, selectedConditionId, watchSelection } from '../selection.js';
import { hueFor } from '../icons.js';

type Point = { x: number; y: number };
type Trace = { id: string; pageId: string; points: Point[]; arc?: boolean };
type Condition = {
  id: string; name: string; kind: 'area' | 'line' | 'count';
  traces?: Trace[]; properties?: Record<string, number>;
  role?: 'drain' | 'ridge'; between?: string[];
  /** The lines on this condition. Their thicknesses are the build-up. */
  items?: { thickness?: number }[];
  color?: string; hidden?: boolean;
};
type Page = { id: string; feetPerUnit?: number };

/** Inches to feet, because a property is in inches and the world is in feet. */
const inches = (n: number) => n / 12;

// Where a tapered field stops rising, and why, is `capInches` in the engine.
// It used to be four inches a board here, which is not a number from anywhere:
// Carlisle runs a single layer to 4.5 and GAF to 4.6, and past that you buy
// another layer rather than the roof going flat. The ceiling an estimator
// actually hits is flashing height against a wall the building already has.

/** How fine the taper is sampled, in feet. Small enough to see a sump. */
const GRID_FEET = 2;

let scene: THREE.Scene;
let camera: THREE.PerspectiveCamera;
let renderer: THREE.WebGLRenderer | null = null;
let controls: OrbitControls | null = null;
let world: THREE.Group;
let frameHandle = 0;
let stopWatching: (() => void) | null = null;
/** Has the estimator taken the camera? Until then the view follows the roof. */
let framed = false;
let stopSubscribing: (() => void) | null = null;

/**
 * The projection, in one place.
 *
 * §4.10 asks for an orbit camera and that is what this builds. The design work
 * argues for a fixed isometric with a vertical exaggeration instead, on the
 * grounds that a quarter inch in twelve is invisible at true scale and a camera
 * a person has to fight is worse than one they cannot move. That is an open
 * question, so the whole of the answer lives in this function: changing it is
 * changing this, and nothing else in the file knows which way it went.
 */
function projection(canvas: HTMLCanvasElement, target: THREE.Vector3, span: number) {
  camera = new THREE.PerspectiveCamera(45, 1, 0.5, 20_000);
  const back = Math.max(span * 1.4, 40);
  camera.position.set(target.x + back, target.y + back * 0.8, target.z + back);

  const orbit = new OrbitControls(camera, canvas);
  orbit.target.copy(target);
  orbit.enableDamping = true;
  orbit.dampingFactor = 0.08;
  // Under the roof there is nothing to see and it is easy to get lost there.
  orbit.maxPolarAngle = Math.PI * 0.49;
  orbit.update();
  return orbit;
}

/** Put the camera back around the roof, keeping the angle it is already at. */
function reframe(target: THREE.Vector3, span: number) {
  if (!controls || !camera) return;
  const back = Math.max(span * 1.4, 40);
  const direction = camera.position.clone().sub(controls.target).normalize();
  controls.target.copy(target);
  camera.position.copy(target).add(direction.multiplyScalar(back));
  controls.update();
}

/**
 * What the roof is doing under the pointer.
 *
 * Thickness, the fall, which drain the water reaches and how far it has to go.
 * All of it comes off the same arithmetic the surface is drawn from, so it can
 * never say something the picture does not.
 */
function readAt(canvas: HTMLCanvasElement, event: PointerEvent): string {
  if (!camera || fieldsNow.length === 0) return 'Point at the roof to read it.';
  const rect = canvas.getBoundingClientRect();
  raycaster.setFromCamera(new THREE.Vector2(
    ((event.clientX - rect.left) / rect.width) * 2 - 1,
    -((event.clientY - rect.top) / rect.height) * 2 + 1,
  ), camera);
  const hit = raycaster.intersectObjects(world.children, false)[0];
  if (!hit) return 'Point at the roof to read it.';

  const p = { x: hit.point.x, y: hit.point.z };
  const field = fieldsNow.find((f) => inside(p, f.ring));
  if (!field) return 'Off the tapered field.';

  const run = nearest(p, drainsNow);
  const raw = field.start + run * field.slope;
  const flat = raw > field.cap;
  const thickness = Math.min(raw, field.cap);
  const parts = [`${inchLabel(thickness)} thick`];
  if (flat) {
    parts.push(field.capReason === 'flashing'
      ? 'no fall — the roof is as high as the wall it flashes against allows'
      : 'no fall — the boards have run out here');
  } else {
    parts.push(`falls ${inchLabel(field.slope)} per foot`);
    parts.push(`${run.toFixed(1)} ft to the drain it reaches`);
  }
  return parts.join(' · ');
}

export function mountModel(host: HTMLElement): void {
  unmount();
  framed = false;
  host.replaceChildren();
  host.classList.add('model-editor');

  const canvas = document.createElement('canvas');
  canvas.className = 'model-canvas';
  canvas.setAttribute('aria-label', 'The roof in three dimensions');

  const legend = document.createElement('div');
  legend.className = 'model-legend';

  const note = document.createElement('p');
  note.className = 'model-note';

  // What the roof does where the pointer is. The water-migration view written
  // out in words — it costs no geometry and it answers the question the shading
  // can only hint at.
  const readout = document.createElement('p');
  readout.className = 'model-readout';
  readout.textContent = 'Point at the roof to read it.';

  host.append(canvas, readout, legend, note);

  renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));

  scene = new THREE.Scene();
  scene.background = new THREE.Color('#eceeef');

  // Flat light plus one sun. A tapered field reads by its shading, so the light
  // has to come from somewhere rather than being uniform.
  scene.add(new THREE.AmbientLight(0xffffff, 1.5));
  const sun = new THREE.DirectionalLight(0xffffff, 1.6);
  sun.position.set(-0.4, 1, 0.6);
  scene.add(sun);

  world = new THREE.Group();
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
      built = build(doc());
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
    if (!controls && built.span > 0) {
      controls = projection(canvas, built.centre, built.span);
      // Once they move the camera it is theirs. Until then the view follows the
      // roof, so tracing something new does not leave it off screen.
      controls.addEventListener('start', () => { framed = true; });
    } else if (controls) {
      if (!framed && built.span > 0) reframe(built.centre, built.span);
      controls.update();
    }
    resize(canvas);
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
  (window as unknown as Record<string, unknown>)['__roofnerdModel'] = () =>
    world.children.map((object) => {
      const mesh = object as THREE.Mesh;
      const box = new THREE.Box3().setFromObject(mesh);
      const kind = mesh.userData['kind'] ?? null;
      const position = mesh.geometry?.getAttribute('position');
      // The shapes whose construction is a trade convention travel whole, so the
      // check can work the convention out from the vertices instead of asking
      // the legend. A heightfield is thousands of points and is not one of them.
      const wholeShape = kind === 'cricket' || kind === 'parapet' || kind === 'parapet-ribbon';
      return {
        conditionId: mesh.userData['conditionId'] ?? null,
        kind,
        min: [box.min.x, box.min.y, box.min.z],
        max: [box.max.x, box.max.y, box.max.z],
        points: wholeShape && position ? Array.from(position.array as Float32Array) : null,
      };
    });

  stopSubscribing = subscribe(() => draw());
  stopWatching = watchSelection(() => highlight());

  canvas.addEventListener('click', (e) => pick(canvas, e));
  canvas.addEventListener('pointermove', (e) => { readout.textContent = readAt(canvas, e); });
  canvas.addEventListener('pointerleave', () => { readout.textContent = 'Point at the roof to read it.'; });
  window.addEventListener('resize', () => resize(canvas));

  const tick = () => {
    frameHandle = requestAnimationFrame(tick);
    controls?.update();
    if (renderer && camera) renderer.render(scene, camera);
  };
  tick();
}

function unmount() {
  if (frameHandle) cancelAnimationFrame(frameHandle);
  frameHandle = 0;
  stopWatching?.();
  stopSubscribing?.();
  stopWatching = null;
  stopSubscribing = null;
  controls?.dispose();
  controls = null;
  renderer?.dispose();
  renderer = null;
}

function resize(canvas: HTMLCanvasElement) {
  if (!renderer || !camera) return;
  const w = canvas.clientWidth || 1;
  const h = canvas.clientHeight || 1;
  renderer.setSize(w, h, false);
  camera.aspect = w / h;
  camera.updateProjectionMatrix();
}

// ── reading the job ────────────────────────────────────────────────────────

const conditionsOf = (d: Doc): Condition[] => (at('/conditions', d) as Condition[]) ?? [];

function feetPerUnit(d: Doc): Record<string, number> {
  const out: Record<string, number> = {};
  for (const p of ((at('/pages', d) as Page[]) ?? [])) {
    if (p.feetPerUnit) out[p.id] = p.feetPerUnit;
  }
  return out;
}

/** A trace's points in feet. Empty when its sheet has no scale — never zero. */
function feet(trace: Trace, scales: Record<string, number>): Point[] {
  const s = scales[trace.pageId];
  if (!s) return [];
  return trace.points.map((p) => ({ x: p.x * s, y: p.y * s }));
}

/** Every drain on the roof, in feet. A drain is a point, not a condition. */
/**
 * The drains named by a cricket's `between`, and only those.
 *
 * A cricket serves a pair of drainage points, not every drain on the roof, and
 * `between` is how the estimator says which. Falling back to all of them would
 * quietly draw a cricket between two drains at the far end of the building.
 */
function drainsNamedBy(
  list: Condition[], scales: Record<string, number>, names: readonly string[] | undefined,
): Point[] {
  if (!names || names.length === 0) return [];
  const wanted = new Set(names);
  const out: Point[] = [];
  for (const c of list) {
    if (!wanted.has(c.id) || c.hidden) continue;
    for (const t of c.traces ?? []) out.push(...feet(t, scales));
  }
  return out;
}

/**
 * The lowest thing the roof has to flash against, in inches above the deck.
 *
 * NRCA wants 8 in of flashing above the finished membrane, so this is what caps
 * the build-up long before any board does. A parapet the estimator has already
 * traced and given a height to is exactly this number, and it was sitting in the
 * document unused while the field was capped at an invented four inches a board.
 */
function perimeterInches(list: Condition[]): number | undefined {
  let lowest = Infinity;
  for (const c of list) {
    if (c.kind !== 'line' || c.hidden || c.role === 'ridge') continue;
    const h = c.properties?.['H'];
    if (h === undefined || h <= 0) continue;
    lowest = Math.min(lowest, h * 12);
  }
  return Number.isFinite(lowest) ? lowest : undefined;
}

function drainPoints(list: Condition[], scales: Record<string, number>): Point[] {
  const out: Point[] = [];
  for (const c of list) {
    if (c.role !== 'drain' || c.hidden) continue;
    for (const t of c.traces ?? []) out.push(...feet(t, scales));
  }
  return out;
}

/** Which drain this point falls to — the one it is closest to. */
const nearestDrain = (p: Point, drains: Point[]): Point | null => {
  let best: Point | null = null;
  let far = Infinity;
  for (const d of drains) {
    const r = Math.hypot(p.x - d.x, p.y - d.y);
    if (r < far) { far = r; best = d; }
  }
  return best;
};

const nearest = (p: Point, drains: Point[]): number => {
  let best = Infinity;
  for (const d of drains) {
    const dx = p.x - d.x;
    const dy = p.y - d.y;
    const r = Math.sqrt(dx * dx + dy * dy);
    if (r < best) best = r;
  }
  return best;
};

/** Ray casting. A textbook primitive, and three.js does not hand one out. */
function inside(p: Point, ring: Point[]): boolean {
  let hit = false;
  for (let i = 0, j = ring.length - 1; i < ring.length; j = i++) {
    const a = ring[i]!;
    const b = ring[j]!;
    if ((a.y > p.y) !== (b.y > p.y)
      && p.x < ((b.x - a.x) * (p.y - a.y)) / (b.y - a.y) + a.x) hit = !hit;
  }
  return hit;
}

// ── building the world ─────────────────────────────────────────────────────

type Built = {
  centre: THREE.Vector3; span: number; legend: HTMLElement[]; note: string;
  thin: number | null; thick: number | null;
};

/** A tapered facet, kept so the cursor can be asked what the roof does here. */
type Field = {
  ring: Point[]; start: number; slope: number; elevation: number;
  cap: number;
  /** Which constraint bound — they fail differently and the readout says which. */
  capReason: CapReason;
};

/** The tapered fields on screen, for the readout. Rebuilt with the world. */
let fieldsNow: Field[] = [];
let drainsNow: Point[] = [];

function build(d: Doc): Built {
  world.clear();

  const list = conditionsOf(d);
  const scales = feetPerUnit(d);
  const drains = drainPoints(list, scales);
  const perimeter = perimeterInches(list);

  const box = new THREE.Box3();
  let anything = false;
  let noFallSquares = 0;
  let taperedFacets = 0;
  let crickets = 0;
  let thin = Infinity;
  let thick = -Infinity;
  const fields: Field[] = [];
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

  list.forEach((c, index) => {
    if (c.hidden) return;
    const colour = c.color ?? hueFor(index);
    const props = c.properties ?? {};
    const elevation = props['ELEV'] ?? 0;

    for (const trace of c.traces ?? []) {
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
        const tapered = props['TAPER'] !== undefined && drains.length > 0;
        if (tapered) {
          const built = taperSurface(ring, props, elevation, drains, colour, c, perimeter);
          if (built) {
            world.add(built.mesh);
            world.add(built.flow);
            noFallSquares += built.noFallSquareFeet;
            taperedFacets += 1;
            if (built.thin !== null) thin = Math.min(thin, built.thin);
            if (built.thick !== null) thick = Math.max(thick, built.thick);
            if (built.thick !== null && built.thick > SINGLE_LAYER_MAX_INCHES) overOneLayer = true;
            if (built.capReason !== null) capReason = built.capReason;
            fields.push({
              ring: built.ring, start: built.start, slope: built.slope,
              cap: built.cap, capReason: built.capReason, elevation,
            });
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
          // The field this cricket sits in, for the slope it should be cut at.
          const fieldSlope = list.find((o) => o.kind === 'area' && o.properties?.['TAPER'] !== undefined)
            ?.properties?.['TAPER'] ?? 0;
          const pair = pairFor(ring, drainsNamedBy(list, scales, c.between));
          const built = pair
            ? cricketBetween(ring, props, fieldSlope, elevation, pair, colour, c)
            : null;
          if (built) {
            world.add(built.mesh);
            crickets += 1;
            if (built.lengthToWidth > built.maxLengthToWidth) outOfProportion += 1;
            if (built.offBy !== null && built.offBy > 15) ridgeOffSquare += 1;
            if (built.slopeDisagrees) cricketSlopeOff += 1;
          } else {
            // Nothing to derive it from. A ridge line on its own is a line, and
            // drawing a cricket off it would be inventing the shape again.
            world.add(runLine(ring, elevation, colour, c));
            unbuiltCrickets += 1;
          }
        } else if (height > 0) {
          const wall = props['WALL'];
          if (wall === undefined) unstatedWall += 1;
          world.add(parapet(ring, elevation, height, wall, colour, c));
        } else {
          world.add(runLine(ring, elevation, colour, c));
        }
        anything = true;
      } else if (c.kind === 'count') {
        for (const p of ring) world.add(marker(p, elevation, c, props));
        anything = true;
      }

      for (const p of ring) box.expandByPoint(new THREE.Vector3(p.x, elevation, p.y));
    }
  });

  highlight();

  const centre = anything ? box.getCenter(new THREE.Vector3()) : new THREE.Vector3();
  const span = anything ? Math.max(box.getSize(new THREE.Vector3()).length(), 20) : 0;

  fieldsNow = fields;
  drainsNow = drains;

  return {
    centre,
    span,
    thin: Number.isFinite(thin) ? thin : null,
    thick: Number.isFinite(thick) ? thick : null,
    legend: legendFor({
      taperedFacets, noFallSquareFeet: noFallSquares, drains: drains.length, crickets,
      thin, thick, capReason, unstatedBuildUp, outOfProportion, ridgeOffSquare,
      cricketSlopeOff, overOneLayer, unbuiltCrickets, crossedTraces, unstatedWall,
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
 * The tapered field.
 *
 * Thickness at any point is what the estimator put at the drain, plus the slope
 * times how far that point is from the nearest drain. That is the whole model,
 * it is the same arithmetic a formula on the sheet can write out by hand, and it
 * is why this view can never disagree with the takeoff.
 *
 * It stops rising when the boards run out. Where it stops, the roof is flat, and
 * flat is where water stays.
 */
function taperSurface(
  ring: Point[], props: Record<string, number>, elevation: number,
  drains: Point[], colour: string, c: Condition, perimeter: number | undefined,
) {
  const start = props['T'] ?? 0;
  const slope = props['TAPER'] ?? 0;
  // Whichever binds first, and which one it was. Boards mean buy another layer;
  // flashing means the wall will not have it. Not the same problem.
  const { cap, reason: capReason } = capInches(start, props['BOARDS'], perimeter);

  let minX = Infinity; let maxX = -Infinity; let minY = Infinity; let maxY = -Infinity;
  for (const p of ring) {
    minX = Math.min(minX, p.x); maxX = Math.max(maxX, p.x);
    minY = Math.min(minY, p.y); maxY = Math.max(maxY, p.y);
  }
  const cols = Math.max(2, Math.ceil((maxX - minX) / GRID_FEET) + 1);
  const rows = Math.max(2, Math.ceil((maxY - minY) / GRID_FEET) + 1);
  if (cols * rows > 40_000) return null;

  const stepX = (maxX - minX) / (cols - 1);
  const stepY = (maxY - minY) / (rows - 1);

  const thickness: number[] = [];
  const within: boolean[] = [];
  const flat: boolean[] = [];
  for (let r = 0; r < rows; r += 1) {
    for (let col = 0; col < cols; col += 1) {
      const p = { x: minX + col * stepX, y: minY + r * stepY };
      const isIn = inside(p, ring);
      const raw = start + nearest(p, drains) * slope;
      const capped = Math.min(raw, cap);
      within.push(isIn);
      thickness.push(capped);
      flat.push(isIn && raw > cap);
    }
  }

  const positions: number[] = [];
  const colours: number[] = [];
  const base = new THREE.Color(colour);
  const noFall = new THREE.Color('#c98a1d');
  let noFallCells = 0;

  const push = (col: number, r: number) => {
    const i = r * cols + col;
    positions.push(minX + col * stepX, elevation + inches(thickness[i]!), minY + r * stepY);
    const shade = flat[i] ? noFall : base.clone().lerp(new THREE.Color('#ffffff'), Math.min(0.65, inches(thickness[i]!) * 1.4));
    colours.push(shade.r, shade.g, shade.b);
  };

  for (let r = 0; r < rows - 1; r += 1) {
    for (let col = 0; col < cols - 1; col += 1) {
      const corners = [r * cols + col, r * cols + col + 1, (r + 1) * cols + col, (r + 1) * cols + col + 1];
      if (!corners.every((i) => within[i])) continue;
      push(col, r); push(col + 1, r); push(col, r + 1);
      push(col + 1, r); push(col + 1, r + 1); push(col, r + 1);
      if (corners.every((i) => flat[i])) noFallCells += 1;
    }
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
  // Where the water goes, as arrows. One every few feet, pointing at the drain
  // that part of the roof falls to — which is the nearest one, because that is
  // what the heightfield already says. Flat ground gets none: there is nothing
  // to point at when the fall has run out.
  // One mesh for every arrow, not one object each.
  //
  // `ArrowHelper` is an Object3D carrying a line and a cone, so a field of them
  // is two draw calls apiece — fine on a demo, and on a real 36 by 24 sheet it
  // is hundreds before the estimator has done anything. The direction is known
  // analytically at every sample (it points at the drain whose cell the sample
  // is in), so the whole field is one instanced cone with a rotation per
  // instance. This came out of the prior 3D work on this machine rather than
  // out of a profiler, which is the cheaper way to learn it.
  const flow = new THREE.Group();
  const ARROW_FEET = 8;
  const arrowAt: { x: number; y: number; z: number; angle: number }[] = [];
  const cols2 = Math.max(2, Math.round((maxX - minX) / ARROW_FEET));
  const rows2 = Math.max(2, Math.round((maxY - minY) / ARROW_FEET));
  for (let r = 1; r < rows2; r += 1) {
    for (let col = 1; col < cols2; col += 1) {
      const p = { x: minX + (col * (maxX - minX)) / cols2, y: minY + (r * (maxY - minY)) / rows2 };
      if (!inside(p, ring)) continue;
      const raw = start + nearest(p, drains) * slope;
      if (raw > cap) continue;
      const to = nearestDrain(p, drains);
      if (!to) continue;
      const dx = to.x - p.x;
      const dy = to.y - p.y;
      const len = Math.hypot(dx, dy);
      if (len < 1) continue;
      const y = elevation + inches(Math.min(raw, cap)) + 0.08;
      // The angle about the vertical that turns +X onto the fall direction.
      arrowAt.push({ x: p.x, y, z: p.y, angle: Math.atan2(dy, dx) });
    }
  }

  if (arrowAt.length > 0) {
    // Slim and long, so it reads as a dart pointing somewhere. The first pass
    // at instancing used the same proportions as the ArrowHelper's head without
    // its shaft, and a field of them read as a scatter of blobs — caught by
    // opening the screenshot rather than by any check, which is the only way a
    // thing like this ever gets caught.
    const cone = new THREE.ConeGeometry(ARROW_FEET * 0.06, ARROW_FEET * 0.55, 6);
    // A cone points up its own +Y; lay it on its side so it points along +X,
    // which is what the angle above is measured from.
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
    // Not pickable: an arrow is the drawing explaining itself, not a thing on
    // the roof. Clicking one should select the field under it.
    arrows.raycast = () => {};
    flow.add(arrows);
  }

  let thin = Infinity;
  let thick = -Infinity;
  for (let i = 0; i < thickness.length; i += 1) {
    if (!within[i]) continue;
    thin = Math.min(thin, thickness[i]!);
    thick = Math.max(thick, thickness[i]!);
  }

  return {
    mesh, flow,
    noFallSquareFeet: noFallCells * stepX * stepY,
    thin: Number.isFinite(thin) ? thin : null,
    thick: Number.isFinite(thick) ? thick : null,
    start, slope, cap, capReason, ring,
  };
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
function parapet(
  ring: Point[], elevation: number, height: number, thick: number | undefined,
  colour: string, c: Condition,
) {
  const positions: number[] = [];
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
    }
  }

  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
  geometry.computeVertexNormals();
  const mesh = new THREE.Mesh(geometry, new THREE.MeshLambertMaterial({
    color: new THREE.Color(colour), side: THREE.DoubleSide,
  }));
  mesh.userData = { conditionId: c.id, kind: t > 0 ? 'parapet' : 'parapet-ribbon' };
  return mesh;
}

/** What a cricket came out as, so the legend can say if it will not drain. */
type Cricket = {
  readonly mesh: THREE.Mesh;
  readonly lengthToWidth: number;
  readonly maxLengthToWidth: number;
  /** How far the traced ridge is off the one the drains imply, in degrees. */
  readonly offBy: number | null;
  /** True when the ridge slope is not twice the field it sits in. */
  readonly slopeDisagrees: boolean;
};

/**
 * A cricket: two planes falling from the ridge the drains put there.
 *
 * The ridge is derived. NRCA is explicit about it (pp.166–168): the ridge line
 * sits equidistant between the drainage points and runs perpendicular to the
 * line joining them. It is where two drainage fields meet, not a line somebody
 * draws and hangs planes off — and the old version of this function had that
 * backwards, taking a traced ridge and a `W` property I had invented, with a
 * default of four feet that came from nowhere.
 *
 * Width is derived too, and it is the single most load-bearing number here.
 * Water leaves the ridge and runs half the drain-to-drain span to get away, so
 * that half-span *is* the width, and NRCA's length-to-width table (Fig. 4-13)
 * bounds it — because widening a cricket is what steepens the valley, and
 * steepening the surface does nothing for it. A cricket long and thin enough
 * ponds at its own valley however sharp the boards are.
 *
 * What the trace is still worth is how far the cricket runs, which no amount of
 * drain geometry knows. So: position, direction, width and rise come off the
 * drains; extent comes off the trace; and if the two disagree about square, the
 * legend says so instead of the code quietly preferring one.
 */
function cricketBetween(
  traced: Point[], props: Record<string, number>, fieldSlope: number,
  elevation: number, pair: readonly [Spot, Spot], colour: string, c: Condition,
): Cricket | null {
  const derived = ridgeBetween(pair[0], pair[1]);
  if (!derived) return null;

  // Twice the field, unless the estimator said otherwise — in which case theirs
  // is drawn and the disagreement is reported. Their number may be right; a
  // manufacturer's cricket stock comes in the slopes it comes in.
  const stated = props['TAPER'];
  const wanted = fieldSlope * CRICKET_SLOPE_MULTIPLE;
  const slope = stated ?? wanted;
  const slopeDisagrees = stated !== undefined && fieldSlope > 0
    && Math.abs(stated - wanted) > 1e-6;

  // How far it runs: what was traced.
  let length = 0;
  for (let i = 0; i < traced.length - 1; i += 1) {
    length += Math.hypot(traced[i + 1]!.x - traced[i]!.x, traced[i + 1]!.y - traced[i]!.y);
  }
  if (length < 1e-6) return null;

  const width = derived.width;
  const rise = inches(slope * width);
  const half = length / 2;
  // Along the ridge, and square to it — square to the ridge is the direction the
  // water falls, which is the line joining the two drains.
  const ax = derived.along.x; const ay = derived.along.y;
  const fx = -ay; const fy = ax;

  const ridgeA = { x: derived.at.x - ax * half, y: derived.at.y - ay * half };
  const ridgeB = { x: derived.at.x + ax * half, y: derived.at.y + ay * half };

  const positions: number[] = [];
  for (const side of [1, -1]) {
    const footA = { x: ridgeA.x + fx * width * side, y: ridgeA.y + fy * width * side };
    const footB = { x: ridgeB.x + fx * width * side, y: ridgeB.y + fy * width * side };
    const top = elevation + rise;
    positions.push(ridgeA.x, top, ridgeA.y, ridgeB.x, top, ridgeB.y, footA.x, elevation, footA.y);
    positions.push(ridgeB.x, top, ridgeB.y, footB.x, elevation, footB.y, footA.x, elevation, footA.y);
  }

  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
  geometry.computeVertexNormals();
  const mesh = new THREE.Mesh(geometry, new THREE.MeshLambertMaterial({
    color: new THREE.Color(colour), side: THREE.DoubleSide,
  }));
  mesh.userData = { conditionId: c.id, kind: 'cricket' };

  return {
    mesh,
    lengthToWidth: length / width,
    maxLengthToWidth: maxLengthToWidth(fieldSlope || slope / CRICKET_SLOPE_MULTIPLE),
    offBy: ridgeDisagreement(traced, derived.along),
    slopeDisagrees,
  };
}

/**
 * The two drainage points a cricket sits between.
 *
 * `between` names conditions, and one count condition can hold every drain on
 * the roof, so the pair still has to be picked. The two nearest the ridge the
 * estimator drew is the honest reading of what they meant — their line says
 * which pair, and the drains say everything else about the shape.
 */
function pairFor(traced: Point[], candidates: Spot[]): readonly [Spot, Spot] | null {
  if (candidates.length < 2 || traced.length === 0) return null;
  const mid = traced[Math.floor(traced.length / 2)]!;
  const byRange = [...candidates].sort((a, b) =>
    Math.hypot(a.x - mid.x, a.y - mid.y) - Math.hypot(b.x - mid.x, b.y - mid.y));
  return [byRange[0]!, byRange[1]!];
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

/** A counted thing. A drain gets its sump; anything else gets a marker. */
function marker(p: Point, elevation: number, c: Condition, props: Record<string, number>) {
  const isDrain = c.role === 'drain';
  const sump = props['SUMP'] ?? 4;
  const geometry = isDrain
    ? new THREE.CylinderGeometry(sump / 2, sump / 4, 0.5, 16)
    : new THREE.SphereGeometry(0.75, 12, 8);
  const mesh = new THREE.Mesh(geometry, new THREE.MeshLambertMaterial({
    color: new THREE.Color(isDrain ? '#1f7a8c' : (c.color ?? '#555555')),
  }));
  // A drain sits at the thin end of the taper, where the surface is only an
  // inch or two off the deck — so a marker centred below the deck disappears
  // under the very field it drains. It sits proud instead: this is a mark on a
  // drawing, not a modelled bowl.
  mesh.position.set(p.x, elevation + (isDrain ? 0.2 : 0.75), p.y);
  mesh.userData = { conditionId: c.id, kind: isDrain ? 'drain' : 'count' };
  return mesh;
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
function thicknessScale(thin: number, thick: number): HTMLElement {
  const wrap = document.createElement('span');
  wrap.className = 'model-scale';

  const ramp = document.createElement('i');
  ramp.className = 'model-ramp';
  // The same lerp the surface uses, so the ramp is the field's own colouring.
  ramp.style.background = 'linear-gradient(to right, #4a6b8a, #ffffff)';
  wrap.append(ramp);

  const ticks = document.createElement('span');
  ticks.className = 'model-ticks';
  for (const v of [thin, (thin + thick) / 2, thick]) {
    const t = document.createElement('span');
    t.textContent = inchLabel(v);
    ticks.append(t);
  }
  wrap.append(ticks);
  return wrap;
}

/** Everything the legend has to be able to say. */
type Reading = {
  taperedFacets: number; noFallSquareFeet: number; drains: number; crickets: number;
  thin: number; thick: number;
  capReason: CapReason;
  unstatedBuildUp: number; outOfProportion: number; ridgeOffSquare: number;
  cricketSlopeOff: number; overOneLayer: boolean; unbuiltCrickets: number;
  crossedTraces: number; unstatedWall: number;
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

  if (r.drains > 0) row('#1f7a8c', r.drains === 1 ? '1 drain' : `${r.drains} drains`);
  if (r.taperedFacets > 0 && Number.isFinite(r.thin) && Number.isFinite(r.thick)) {
    out.push(thicknessScale(r.thin, r.thick));
    // The surface is the fall. It is not the layout, and it must not be read as
    // one: a real taper is a schedule of lettered boards on a four foot module,
    // in panel repeats, with flat fill under them — thickness steps at a board
    // edge rather than sliding. Saying so is the difference between a drawing
    // that helps and one that gets ordered from.
    say('the surface is the fall, not the board layout');
  }
  if (r.drains > 0) row('#1f7a8c', 'arrows follow the fall to the drain');
  if (r.crickets > 0) row('#8a5a1d', r.crickets === 1 ? '1 cricket' : `${r.crickets} crickets`);

  if (r.noFallSquareFeet > 0) {
    const why = r.capReason === 'flashing'
      ? 'the roof has reached 8" under the wall it flashes against'
      : 'the boards run out before the water gets anywhere';
    row('#c98a1d', `${Math.round(r.noFallSquareFeet).toLocaleString('en-US')} SF with no fall — ${why}`);
  }
  if (r.overOneLayer) say(`over ${SINGLE_LAYER_MAX_INCHES}" — this is a second layer of board, not a thicker one`);
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

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
//   a facet     an area condition, laid at its elevation
//   a parapet   a line condition with a height, stood up off its base
//   the taper   thickness at the drain, plus the slope, over the distance to
//               the nearest drain — sampled on a grid and drawn as a surface
//   a cricket   a traced ridge, with a plane falling away from each side
//   no fall     where the taper has run out of boards and the roof goes flat,
//               so the water stops moving
//
// There is no editing in this window. Clicking selects, and selecting is not
// editing — it is the same one selection every other editor watches.

import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { at, doc, subscribe, type Doc } from '../doc.js';
import { select, selectedConditionId, watchSelection } from '../selection.js';
import { hueFor } from '../icons.js';

type Point = { x: number; y: number };
type Trace = { id: string; pageId: string; points: Point[]; arc?: boolean };
type Condition = {
  id: string; name: string; kind: 'area' | 'line' | 'count';
  traces?: Trace[]; properties?: Record<string, number>;
  role?: 'drain' | 'ridge'; between?: string[];
  color?: string; hidden?: boolean;
};
type Page = { id: string; feetPerUnit?: number };

/** Inches to feet, because a property is in inches and the world is in feet. */
const inches = (n: number) => n / 12;

/**
 * A tapered board stack cannot build for ever. Four inches is the most a single
 * board carries, so the field can only rise so far before it goes flat — and
 * where it goes flat is where the water stops. That is the no-fall region, and
 * it is a real thing an estimator walks out to look at, not a rendering effect.
 */
const MAX_BOARD_INCHES = 4;

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
    parts.push('no fall — the boards have run out here');
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

  const draw = () => {
    const built = build(doc());
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
type Field = { ring: Point[]; start: number; slope: number; cap: number; elevation: number };

/** The tapered fields on screen, for the readout. Rebuilt with the world. */
let fieldsNow: Field[] = [];
let drainsNow: Point[] = [];

function build(d: Doc): Built {
  world.clear();

  const list = conditionsOf(d);
  const scales = feetPerUnit(d);
  const drains = drainPoints(list, scales);

  const box = new THREE.Box3();
  let anything = false;
  let noFallSquares = 0;
  let taperedFacets = 0;
  let crickets = 0;
  let thin = Infinity;
  let thick = -Infinity;
  const fields: Field[] = [];
  const unscaled = list.some((c) => (c.traces ?? []).some((t) => !scales[t.pageId]));

  list.forEach((c, index) => {
    if (c.hidden) return;
    const colour = c.color ?? hueFor(index);
    const props = c.properties ?? {};
    const elevation = props['ELEV'] ?? 0;

    for (const trace of c.traces ?? []) {
      const ring = feet(trace, scales);
      if (ring.length === 0) continue;

      if (c.kind === 'area') {
        const tapered = props['TAPER'] !== undefined && drains.length > 0;
        if (tapered) {
          const built = taperSurface(ring, props, elevation, drains, colour, c);
          if (built) {
            world.add(built.mesh);
            world.add(built.flow);
            noFallSquares += built.noFallSquareFeet;
            taperedFacets += 1;
            if (built.thin !== null) thin = Math.min(thin, built.thin);
            if (built.thick !== null) thick = Math.max(thick, built.thick);
            fields.push({ ring: built.ring, start: built.start, slope: built.slope, cap: built.cap, elevation });
          }
        }
        world.add(deck(ring, elevation, colour, c, tapered));
        anything = true;
      } else if (c.kind === 'line') {
        const height = props['H'] ?? (c.role === 'ridge' ? 0 : 0);
        if (c.role === 'ridge') {
          world.add(cricket(ring, props, elevation, colour, c));
          crickets += 1;
        } else if (height > 0) {
          world.add(parapet(ring, elevation, height, colour, c));
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
    legend: legendFor(taperedFacets, noFallSquares, drains.length, crickets, thin, thick),
    note: anything
      ? (unscaled ? 'Some of this roof is on a sheet nobody has scaled, so it is not drawn.' : '')
      : 'Nothing traced yet. Trace a roof on the Plan and it stands up here.',
  };
}

/** The deck itself — the plan polygon, laid flat at its elevation. */
/** How thick the deck reads as. Not a real assembly — enough to be a building. */
const DECK_FEET = 0.75;

function deck(ring: Point[], elevation: number, colour: string, c: Condition, tapered: boolean) {
  const shape = new THREE.Shape(ring.map((p) => new THREE.Vector2(p.x, p.y)));
  // Extruded rather than a bare plane. A facet drawn as a single surface floats
  // at its elevation with nothing under it and reads as a coloured shape in
  // space; give it a depth and it reads as a roof on a building, which is the
  // whole reason anyone asked for a 3D view.
  const geometry = new THREE.ExtrudeGeometry(shape, { depth: DECK_FEET, bevelEnabled: false });
  geometry.rotateX(Math.PI / 2);
  geometry.translate(0, elevation, 0);
  const mesh = new THREE.Mesh(geometry, new THREE.MeshLambertMaterial({
    color: new THREE.Color(colour),
    side: THREE.DoubleSide,
    transparent: tapered,
    opacity: tapered ? 0.35 : 1,
  }));
  mesh.userData = { conditionId: c.id, kind: 'deck' };
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
  drains: Point[], colour: string, c: Condition,
) {
  const start = props['T'] ?? 0;
  const slope = props['TAPER'] ?? 0;
  const boards = props['BOARDS'];
  const cap = boards === undefined ? Infinity : start + boards * MAX_BOARD_INCHES;

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
  const flow = new THREE.Group();
  const ARROW_FEET = 8;
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
      const arrow = new THREE.ArrowHelper(
        new THREE.Vector3(dx / len, 0, dy / len),
        new THREE.Vector3(p.x, y, p.y),
        Math.min(len * 0.5, ARROW_FEET * 0.55),
        0x1f7a8c, ARROW_FEET * 0.22, ARROW_FEET * 0.14,
      );
      flow.add(arrow);
    }
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
    start, slope, cap, ring,
  };
}

/** A parapet: the run, stood up off its base by its height. */
function parapet(ring: Point[], elevation: number, height: number, colour: string, c: Condition) {
  const positions: number[] = [];
  for (let i = 0; i < ring.length - 1; i += 1) {
    const a = ring[i]!;
    const b = ring[i + 1]!;
    const low = elevation;
    const high = elevation + height;
    positions.push(a.x, low, a.y, b.x, low, b.y, a.x, high, a.y);
    positions.push(b.x, low, b.y, b.x, high, b.y, a.x, high, a.y);
  }
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
  geometry.computeVertexNormals();
  const mesh = new THREE.Mesh(geometry, new THREE.MeshLambertMaterial({
    color: new THREE.Color(colour), side: THREE.DoubleSide,
  }));
  mesh.userData = { conditionId: c.id, kind: 'parapet' };
  return mesh;
}

/**
 * A cricket: the traced ridge, with a plane falling away from either side of it.
 *
 * The ridge is traced rather than solved for. §4.10 calls a cricket a ridge line
 * between drains and two planes; the line is a condition an estimator drew, and
 * `between` records which drains it serves rather than generating it — which
 * keeps this view derived, like everything else here.
 */
function cricket(ring: Point[], props: Record<string, number>, elevation: number, colour: string, c: Condition) {
  const slope = props['TAPER'] ?? 0.25;
  const width = props['W'] ?? 4;
  const rise = inches(slope * width);
  const positions: number[] = [];

  for (let i = 0; i < ring.length - 1; i += 1) {
    const a = ring[i]!;
    const b = ring[i + 1]!;
    const dx = b.x - a.x;
    const dy = b.y - a.y;
    const len = Math.hypot(dx, dy) || 1;
    // The fall is square to the ridge, one plane each side.
    const nx = (-dy / len) * width;
    const ny = (dx / len) * width;
    for (const s of [1, -1]) {
      const a2 = { x: a.x + nx * s, y: a.y + ny * s };
      const b2 = { x: b.x + nx * s, y: b.y + ny * s };
      positions.push(a.x, elevation + rise, a.y, b.x, elevation + rise, b.y, a2.x, elevation, a2.y);
      positions.push(b.x, elevation + rise, b.y, b2.x, elevation, b2.y, a2.x, elevation, a2.y);
    }
  }

  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
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

function legendFor(
  taperedFacets: number, noFallSquareFeet: number, drains: number, crickets: number,
  thin: number, thick: number,
): HTMLElement[] {
  const out: HTMLElement[] = [];
  const row = (swatch: string, text: string) => {
    const el = document.createElement('span');
    el.className = 'model-key';
    const dot = document.createElement('i');
    dot.style.background = swatch;
    el.append(dot, document.createTextNode(text));
    out.push(el);
  };
  if (drains > 0) row('#1f7a8c', drains === 1 ? '1 drain' : `${drains} drains`);
  if (taperedFacets > 0 && Number.isFinite(thin) && Number.isFinite(thick)) {
    out.push(thicknessScale(thin, thick));
  }
  if (drains > 0) row('#1f7a8c', 'arrows follow the fall to the drain');
  if (crickets > 0) row('#8a5a1d', crickets === 1 ? '1 cricket' : `${crickets} crickets`);
  if (noFallSquareFeet > 0) {
    row('#c98a1d', `${Math.round(noFallSquareFeet).toLocaleString('en-US')} SF with no fall — the boards run out before the water gets anywhere`);
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

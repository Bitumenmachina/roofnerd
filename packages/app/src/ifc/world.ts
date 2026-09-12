// ── The roof, stood up out of its own IFC model ────────────────────────────
// The same roof the Model editor derives, gone the long way round: written as
// IFC4 bytes, read back through the toolkit's own loader, and drawn from the
// fragments that come out. It is the long way round on purpose — what the
// estimator sees here is what an architect's program would see if it opened the
// exported file, so the two cannot quietly disagree.
//
// Nothing is fetched. The wasm and the worker are read out of the program's own
// bundle (`local-files.ts`), and the two places the toolkit would otherwise go
// to a CDN are both closed here:
//
//   `IfcLoader.setup({ autoSetWasm: false })` — left at its default of true it
//   fetches `unpkg.com/@thatopen/components/package.json` on every load, not
//   only at setup;
//   `FragmentsManager.init(WORKER_URL)` — never `FragmentsManager.getWorker()`,
//   which fetches the worker from unpkg and hands back a blob URL.
//
// The wordmark the toolkit paints into its own viewport is turned off and taken
// out of the page. The library offers the switch for exactly this case and asks
// politely that it be left on where it fits; it does not fit an estimator's roof
// — a third-party mark on a drawing that goes to an owner reads as that owner's
// consultant — so the attribution stays where attributions belong, in
// DEPENDENCIES.md, and the viewport stays the job's.

import * as OBC from '@thatopen/components';
import * as THREE from 'three';
import * as WEBIFC from 'web-ifc';
import { plural } from '../labels.js';
import type { Doc } from '../doc.js';
import { WASM_DIR, WORKER_URL } from './local-files.js';
import { METRES_PER_FOOT, roofPlan, writeRoof, type RoofPlan } from './write.js';

/** How long the roof is left alone after a change before it is stood up again. */
const SETTLE = 300;

export interface IfcRoofView {
  /** The region to put in the editor. */
  readonly root: HTMLElement;
  /** Stand this job up. Repeated calls settle, and an unchanged roof is left alone. */
  show(d: Doc): void;
  dispose(): void;
}

interface Standing {
  readonly components: OBC.Components;
  readonly world: OBC.SimpleWorld<OBC.SimpleScene, OBC.OrthoPerspectiveCamera, OBC.SimpleRenderer>;
  readonly fragments: OBC.FragmentsManager;
  readonly loader: OBC.IfcLoader;
  /**
   * The writer's own web-ifc, opened on the same module out of the same folder.
   *
   * Not the loader's: `IfcLoader.setup` sets a path and nothing else — the
   * module behind `loader.webIfc` has never been through `Init`, so asking it
   * to write says only that `CreateModel` is undefined. Reading and writing are
   * two jobs and they get one module each; the file is fetched once and
   * compiled twice, which is the price of the loader keeping its own.
   */
  readonly writer: WEBIFC.IfcAPI;
}

export function ifcRoof(): IfcRoofView {
  const root = document.createElement('div');
  root.className = 'model-ifc';

  const stage = document.createElement('div');
  stage.className = 'model-ifc-stage';

  const says = document.createElement('p');
  says.className = 'model-ifc-says';
  says.textContent = 'The roof, as an IFC model.';

  root.append(stage, says);

  let standing: Standing | null = null;
  let starting: Promise<Standing> | null = null;
  let modelId: string | null = null;
  let lastPlan = '';
  let settle = 0;
  let gone = false;
  const answers: Answers = blankAnswers();

  const trouble = (why: string) => {
    answers.trouble = why;
    says.textContent = why;
    says.classList.add('model-broke');
  };

  const settled = (text: string) => {
    answers.trouble = null;
    says.textContent = text;
    says.classList.remove('model-broke');
  };

  async function start(): Promise<Standing> {
    const components = new OBC.Components();
    const worlds = components.get(OBC.Worlds);
    const world = worlds.create<OBC.SimpleScene, OBC.OrthoPerspectiveCamera, OBC.SimpleRenderer>();
    world.scene = new OBC.SimpleScene(components);
    world.renderer = new OBC.SimpleRenderer(components, stage);
    world.renderer.showLogo = false;
    world.renderer.logo?.remove();
    world.camera = new OBC.OrthoPerspectiveCamera(components);
    components.init();
    world.scene.setup();
    world.scene.three.background = null;

    const fragments = components.get(OBC.FragmentsManager);
    fragments.init(WORKER_URL);

    const loader = components.get(OBC.IfcLoader);
    await loader.setup({ autoSetWasm: false, wasm: { path: WASM_DIR, absolute: true } });

    const writer = new WEBIFC.IfcAPI();
    writer.SetWasmPath(WASM_DIR, true);
    await writer.Init();

    return { components, world, fragments, loader, writer };
  }

  async function stand(d: Doc) {
    const plan = roofPlan(d);
    const signature = JSON.stringify(plan);
    if (signature === lastPlan && modelId !== null) return;

    if (plan.slabs.length === 0 && plan.walls.length === 0) {
      lastPlan = signature;
      trouble(nothingStood(plan));
      return;
    }

    if (!starting) starting = start();
    const now = await starting;
    if (gone) return;
    standing = now;

    const bytes = writeRoof(plan, now.writer);
    if (modelId !== null) {
      await now.fragments.core.disposeModel(modelId).catch(() => undefined);
      modelId = null;
    }

    const name = `${plan.job} — roof`;
    const model = await now.loader.load(bytes, true, name);
    if (gone) return;
    modelId = model.modelId;

    now.world.scene.three.add(model.object);
    model.useCamera(now.world.camera.three);
    await now.fragments.core.update(true);
    await frame(now, model.box);

    lastPlan = signature;
    const box = model.box;
    answers.model = {
      name,
      bytes: bytes.length,
      items: (await model.getLocalIds()).length,
      drawn: (await model.getItemsIdsWithGeometry()).length,
      kinds: await model.getCategories(),
      // How big the roof on screen is, as the model itself measures it, and
      // the same thing in feet. The fragments loader normalises every model to
      // metres whatever the file says, so these two differ by exactly the foot,
      // and the second is the one that can be held against the trace. A roof
      // that did not come off the trace is out by whatever it is out by, and
      // this is the only place that shows.
      size: box.isEmpty() ? null : sides(box.getSize(new THREE.Vector3())),
      feet: box.isEmpty() ? null : sides(box.getSize(new THREE.Vector3())
        .divideScalar(METRES_PER_FOOT)),
    };
    settled(stoodUp(plan));
  }

  const view: IfcRoofView = {
    root,
    show(d: Doc) {
      if (gone) return;
      window.clearTimeout(settle);
      settle = window.setTimeout(() => {
        stand(d).catch((e: unknown) => {
          trouble(`The roof did not stand up as an IFC model: ${message(e)}`);
        });
      }, SETTLE);
    },
    dispose() {
      gone = true;
      window.clearTimeout(settle);
      standing?.components.dispose();
      standing = null;
      starting = null;
      modelId = null;
      lastPlan = '';
      root.remove();
    },
  };

  // ── what the picture says about itself, for a check to read ─────────────
  //
  // Read-only, and nothing in the program reads it. The counts are the model's
  // own answers rather than a tally this file kept while it worked — a check
  // that reads its producer's bookkeeping measures the bookkeeping. What the
  // canvas and the scene report is read at the moment the question is asked,
  // because "it loaded" and "it is on screen" are two different claims and this
  // program has been caught believing the first was the second.
  (window as unknown as Record<string, unknown>)['__roofnerdIfcWorld'] = () => {
    const canvas = stage.querySelector('canvas');
    const gl = canvas?.getContext('webgl2') ?? canvas?.getContext('webgl') ?? null;
    const info = gl && 'getExtension' in gl
      ? (gl as WebGLRenderingContext).getExtension('WEBGL_debug_renderer_info')
      : null;
    const ask = (gl2: WebGLRenderingContext, name: number | undefined) =>
      (name === undefined ? null : String(gl2.getParameter(name)));

    let objects = 0;
    let meshes = 0;
    let triangles = 0;
    const box = new THREE.Box3();
    const scene = standing?.world.scene.three;
    scene?.traverse((o) => {
      objects += 1;
      const mesh = o as unknown as {
        isMesh?: boolean;
        geometry?: { index?: { count: number } | null; attributes?: Record<string, { count: number }> };
      };
      if (!mesh.isMesh) return;
      meshes += 1;
      box.expandByObject(o);
      const index = mesh.geometry?.index;
      const position = mesh.geometry?.attributes?.['position'];
      triangles += Math.floor(((index ? index.count : position?.count) ?? 0) / 3);
    });
    // How big the thing on screen actually is, in feet, read off the vertices.
    // A model in the wrong unit is off by a factor of three and a bit, and a
    // model that did not come off the trace is off by whatever it is off by;
    // both show up here and nowhere else.
    const size = meshes > 0 && !box.isEmpty() ? box.getSize(new THREE.Vector3()) : null;

    const here = window.location.origin;
    const fetched = (performance.getEntriesByType('resource') as { name: string }[]).map((r) => r.name);

    return {
      said: says.textContent,
      trouble: answers.trouble,
      stage: { width: canvas?.width ?? 0, height: canvas?.height ?? 0 },
      gl: {
        live: gl !== null && !(gl as WebGLRenderingContext).isContextLost(),
        maker: gl && info ? ask(gl as WebGLRenderingContext, info['UNMASKED_VENDOR_WEBGL']) : null,
        drawnBy: gl && info ? ask(gl as WebGLRenderingContext, info['UNMASKED_RENDERER_WEBGL']) : null,
        kind: gl ? (('drawBuffers' in gl) ? 'webgl2' : 'webgl') : null,
      },
      scene: {
        objects, meshes, triangles,
        size: size ? { x: size.x, y: size.y, z: size.z } : null,
        at: meshes > 0 && !box.isEmpty()
          ? { x: box.min.x, y: box.min.y, z: box.min.z }
          : null,
      },
      model: answers.model,
      wasm: {
        asked: standing?.loader.settings.wasm.path ?? null,
        absolute: standing?.loader.settings.wasm.absolute ?? null,
        onItsOwn: standing?.loader.settings.autoSetWasm ?? null,
      },
      // Where the two runtime files were actually read from, as the window
      // recorded it — not where this file meant to read them from. The worker
      // needs no second answer: a fragments model with anything in it exists
      // only because the worker is alive, so the count below is the worker's
      // own receipt.
      fetched: fetched.filter((u) => !u.startsWith(here)),
      readFrom: fetched.filter((u) => /web-ifc|fragments-worker/.test(u)),
    };
  };

  return view;
}

interface Answers {
  trouble: string | null;
  model: {
    name: string; bytes: number; items: number; drawn: number; kinds: string[];
    size: Sides | null;
    feet: Sides | null;
  } | null;
}

const blankAnswers = (): Answers => ({ trouble: null, model: null });

/**
 * A three.js size as three words rather than three axes.
 *
 * The toolkit stands an IFC model up the way three.js wants it — Y is up, and
 * the plan's north has gone to negative Z — so `x`, `y`, `z` here would each
 * mean something different from what they mean in the file. Across, up and
 * along say what they are in any frame.
 */
const sides = (v: { x: number; y: number; z: number }): Sides =>
  ({ across: v.x, up: v.y, along: v.z });

interface Sides { readonly across: number; readonly up: number; readonly along: number }

/** What stood up, in the words the job uses for it. */
function stoodUp(plan: RoofPlan): string {
  const pieces: string[] = [];
  if (plan.slabs.length > 0) pieces.push(plural(plan.slabs.length, 'roof area', 'roof areas'));
  if (plan.walls.length > 0) pieces.push(plural(plan.walls.length, 'length of parapet', 'lengths of parapet'));
  const left = plan.leftOut.map((l) => `${l.name} is not stood up: ${l.why}.`);
  return [`${pieces.join(' and ')}, off the same trace as the estimate.`, ...left].join(' ');
}

/** Why there is nothing to look at, which is never "nothing". */
function nothingStood(plan: RoofPlan): string {
  if (plan.leftOut.length === 0) return 'Nothing traced yet. Trace a roof on the Plan and it stands up here.';
  return plan.leftOut.map((l) => `${l.name} is not stood up: ${l.why}.`).join(' ');
}

/**
 * Put the whole roof on screen, from a corner.
 *
 * A roof is read from above and from one side at once — which is what an
 * estimator does standing at the parapet — so the camera sits off one corner
 * and above, at a distance the roof's own size sets. The four named corners and
 * the vertical exaggeration are the Model's own (D131) and are not here yet.
 */
async function frame(now: Standing, box: { min: Coordinate; max: Coordinate }) {
  const middle = {
    x: (box.min.x + box.max.x) / 2,
    y: (box.min.y + box.max.y) / 2,
    z: (box.min.z + box.max.z) / 2,
  };
  const span = Math.max(
    Math.hypot(box.max.x - box.min.x, box.max.y - box.min.y, box.max.z - box.min.z),
    1,
  );
  if (!Number.isFinite(span)) return;
  // Far enough back that the whole span fits across a sixty-degree lens —
  // a shade under one span away along a corner, which is where it lands.
  const away = span * 0.95;
  const corner = { x: 1, y: 0.8, z: 1 };
  const unit = Math.hypot(corner.x, corner.y, corner.z);
  await now.world.camera.controls.setLookAt(
    middle.x + (corner.x / unit) * away,
    middle.y + (corner.y / unit) * away,
    middle.z + (corner.z / unit) * away,
    middle.x, middle.y, middle.z,
    true,
  );
}

interface Coordinate { readonly x: number; readonly y: number; readonly z: number }

const message = (e: unknown) => (typeof e === 'string' ? e : e instanceof Error ? e.message : String(e));

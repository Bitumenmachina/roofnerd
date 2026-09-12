// ── Section 10b, step one: the 3D toolkit in the shipped runtime ───────────
// The risk step, and nothing else. Not "does the roof look right" — that is the
// rest of the section — but the four things that either work in WebKitGTK or
// stop the whole approach dead:
//
//   1. the WebAssembly module compiles at all, under this program's own
//      content security policy, out of this program's own bundle;
//   2. the fragments worker starts — a module worker from the app's origin;
//   3. an IFC4 model written in the window loads back through the toolkit and
//      arrives as something with items in it;
//   4. a second WebGL context lives beside the one the Model editor already
//      has, and something is actually drawn into it.
//
// Nothing is fetched, and that is read off the window rather than asserted: the
// list of addresses the page loaded from that were not its own has to be empty.
//
// Through the front door, like every check here: the start screen, the demo
// button, the editor picker. It reads back through `__roofnerdIfcWorld()`,
// which is in the program for checking and is read by nothing else.
//
//   pnpm build:app && node tools/probe/section10b-toolkit.mjs

import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import * as harness from './tauri-harness.mjs';

const {
  commitStamp, elementShot, errorsSoFar, launch, openDemoJob, until, wait, watchErrors,
} = harness;

/**
 * Get to the Model the way the window in front of you offers.
 *
 * Two ways, because there are two windows: the editor picker, which reloads
 * this page with a different editor named in its URL, and the surface's view
 * switch, which swaps Plan for Model in place. The harness gains `showView`
 * with the second one, so asking whether it is there is asking which window
 * this is — a named import of it would stop this file from loading at all on
 * the other. Which one was taken goes into the output, because a reload takes
 * every hook in the page with it and the watch has to go back in after one.
 */
async function showTheModel(session) {
  if (typeof harness.showView === 'function') {
    await harness.showView(session, 'Model');
    return 'the view switch, in place';
  }
  const switched = await session.execute(function () {
    const picker = document.querySelector('.editor-picker');
    if (!picker) return false;
    picker.value = 'model';
    picker.dispatchEvent(new Event('change', { bubbles: true }));
    return true;
  });
  if (!switched) throw new Error('this window offers no way through to the Model');
  return 'the editor picker, which reloads the window';
}

const ROOT = resolve(import.meta.dirname, '../..');
const EVIDENCE = join(ROOT, 'evidence');
const COMMIT = commitStamp();

/** How long the roof gets to be written, compiled and drawn before this gives up. */
const PATIENCE = 60000;

const results = [];
const check = (name, fn) => {
  try { fn(); results.push('PASS'); console.log(`  PASS  ${name}`); }
  catch (e) { results.push('FAIL'); console.log(`  FAIL  ${name}\n        ${e.message}`); }
};

const worldNow = (session) => session.execute(function () {
  const read = window.__roofnerdIfcWorld;
  return JSON.stringify(read ? read() : null);
});

/**
 * How big the demo roof is, in feet, worked out here from the job's own files.
 *
 * The arithmetic is deliberately this probe's rather than the program's: a
 * check that asks the writer how big the roof should be is asking the thing it
 * is checking. Points times the page's scale, over everything the writer stands
 * up — an area with a stated build-up, a run with a height and a wall thickness
 * — which is the same rule written out independently. The wall thickness never
 * widens the box: a parapet stands inboard of the line that was traced.
 */
async function tracedRoof() {
  const at = (f) => readFile(join(ROOT, 'jobs/demo-job', f), 'utf8').then(JSON.parse);
  const conditions = await at('conditions.json');
  const pages = await at('pages/pages.json');
  const scale = {};
  for (const p of pages) if (p.feetPerUnit) scale[p.id] = p.feetPerUnit;

  const low = { x: Infinity, y: Infinity, z: Infinity };
  const high = { x: -Infinity, y: -Infinity, z: -Infinity };
  let pieces = 0;

  for (const c of conditions) {
    if (c.hidden) continue;
    const props = c.properties ?? {};
    const base = props.ELEV ?? 0;
    let top = null;
    if (c.kind === 'area') {
      const layers = (c.items ?? []).filter((i) => i.thickness !== undefined);
      if (layers.length === 0) continue;
      top = base + layers.reduce((sum, i) => sum + Math.max(0, i.thickness), 0) / 12;
    } else if (c.kind === 'line' && c.role !== 'ridge'
      && (props.H ?? 0) > 0 && props.WALL !== undefined) {
      top = base + props.H;
    } else {
      continue;
    }
    let drawn = false;
    for (const t of c.traces ?? []) {
      const s = scale[t.pageId];
      if (!s) continue;
      drawn = true;
      for (const p of t.points) {
        low.x = Math.min(low.x, p.x * s); high.x = Math.max(high.x, p.x * s);
        low.y = Math.min(low.y, p.y * s); high.y = Math.max(high.y, p.y * s);
      }
    }
    if (!drawn) continue;
    pieces += 1;
    low.z = Math.min(low.z, base);
    high.z = Math.max(high.z, top);
  }

  return {
    pieces,
    across: high.x - low.x,
    along: high.y - low.y,
    up: high.z - low.z,
  };
}

const traced = await tracedRoof();
console.log(`  traced: ${traced.across.toFixed(3)} x ${traced.along.toFixed(3)} ft, `
  + `${traced.up.toFixed(3)} ft tall, over ${traced.pieces} of the job's conditions`);

await mkdir(EVIDENCE, { recursive: true });
const app = await launch();
const { session } = app;

try {
  await watchErrors(session);
  await openDemoJob(session);
  await wait(1200);

  // Whichever way through it was, the watch goes back in on the other side, as
  // early as the page will take it: a reload takes every hook in the page with
  // it, and installing twice over one page is harmless and says so. The
  // toolkit's own work starts a third of a second after the document arrives
  // and takes longer than that, so what it says goes into the watch either way.
  const wayIn = await showTheModel(session);
  console.log(`  through: ${wayIn}`);
  await wait(2500);
  await until(session, () => document.querySelector('.model-editor') !== null,
    { what: 'the Model', timeout: 30000 });
  await watchErrors(session);

  const origin = await session.execute(function () {
    return JSON.stringify({ origin: window.location.origin, protocol: window.location.protocol });
  });
  const { origin: home } = JSON.parse(origin);
  console.log(`  window: ${home}`);

  const shape = JSON.parse(await session.execute(function () {
    const stage = document.querySelector('.model-stage');
    const canvas = document.querySelector('.model-canvas');
    const ifc = document.querySelector('.model-ifc');
    const box = (el) => (el ? el.getBoundingClientRect() : null);
    return JSON.stringify({ stage: box(stage), derived: box(canvas), ifc: box(ifc) });
  }));

  check('the window is the program itself, not a page off a network', () => {
    // What "its own origin" is worth depends on what the origin is. The shell
    // serves this program over its own protocol; anything that is not that, and
    // not the loopback a harness uses, is a page somebody else is serving.
    assert.doesNotMatch(home, /^https?:\/\/(?!(tauri\.localhost|localhost|127\.0\.0\.1)([:/]|$))/, home);
  });

  check('the Model editor carries both roofs, each with room to be seen', () => {
    assert.ok(shape.derived, 'the derived roof has no canvas');
    assert.ok(shape.ifc, 'the IFC roof has no region');
    assert.ok(shape.derived.width > 200, `the derived roof is ${Math.round(shape.derived.width)}px wide`);
    assert.ok(shape.ifc.width > 200, `the IFC roof is ${Math.round(shape.ifc.width)}px wide`);
    assert.ok(shape.ifc.height > 150, `the IFC roof is ${Math.round(shape.ifc.height)}px tall`);
  });

  // Everything below waits on the same thing: the roof having been written,
  // compiled, loaded and drawn. A failure here is the answer this step exists
  // for, so the wait is long and what it was waiting for is named.
  let world = null;
  try {
    await until(session, function () {
      const read = window.__roofnerdIfcWorld;
      const now = read ? read() : null;
      return !!(now && (now.model || now.trouble));
    }, { what: 'the IFC roof to stand up or say why not', timeout: PATIENCE });
  } catch (e) {
    console.log(`  (nothing came back: ${e.message})`);
  }
  world = JSON.parse(await worldNow(session));

  check('the Model can be asked what it stood up', () => {
    assert.ok(world, '__roofnerdIfcWorld is not there at all');
  });

  if (world) {
    console.log(`  drawn by: ${world.gl.maker ?? 'unnamed'} — ${world.gl.drawnBy ?? 'unnamed'} (${world.gl.kind ?? 'no context'})`);
    console.log(`  said:     ${world.said}`);
    if (world.model?.feet) {
      const f = world.model.feet;
      console.log(`  on screen: ${f.across.toFixed(3)} x ${f.along.toFixed(3)} ft, ${f.up.toFixed(3)} ft tall`);
    }
    if (world.trouble) console.log(`  trouble:  ${world.trouble}`);
    console.log(`  read from: ${world.readFrom.length ? world.readFrom.join(', ') : '(the window recorded no addresses)'}`);

    check('the IFC roof stood up rather than saying why it could not', () => {
      assert.equal(world.trouble, null, world.trouble ?? '');
      assert.ok(world.model, 'no model');
    });

    check('a model with more than one item in it is loaded', () => {
      assert.ok(world.model, 'no model at all');
      assert.ok(world.model.items >= 2, `${world.model.items} item in the model`);
      assert.ok(world.model.drawn >= 2, `${world.model.drawn} of them carry geometry`);
    });

    check('and the items are a roof slab and a parapet wall, by their own names', () => {
      assert.ok(world.model, 'no model at all');
      const kinds = (world.model.kinds ?? []).join(' ');
      assert.match(kinds, /IFCSLAB/, kinds);
      assert.match(kinds, /IFCWALL/, kinds);
    });

    check('the WebGL context beside the derived roof is live', () => {
      assert.equal(world.gl.live, true, 'the context is gone or was never made');
      assert.ok(world.stage.width > 0 && world.stage.height > 0,
        `the canvas is ${world.stage.width}x${world.stage.height}`);
    });

    check('and something is actually drawn into it', () => {
      assert.ok(world.scene.meshes > 0, 'the scene holds no mesh');
      assert.ok(world.scene.triangles > 0, 'the scene holds no triangle');
    });

    check('the roof on screen is the roof that was traced, to the inch', () => {
      assert.ok(world.model, 'no model at all');
      assert.ok(world.model.feet, 'the model could not say how big it is');
      assert.ok(traced.pieces > 0, 'the demo job stands nothing up, so this proves nothing');
      const inch = 1 / 12;
      for (const side of ['across', 'along', 'up']) {
        const drawn = world.model.feet[side];
        assert.ok(Math.abs(drawn - traced[side]) < inch,
          `${side}: ${drawn.toFixed(4)} ft on screen against ${traced[side].toFixed(4)} ft traced`);
      }
    });

    check('and the toolkit holds it in metres, which is a conversion away', () => {
      assert.ok(world.model, 'no model at all');
      assert.ok(world.model.size && world.model.feet, 'the model could not say how big it is');
      // Worth a row of its own because it is the thing that will bite: the
      // fragments loader normalises every model to metres whatever the file
      // says, so a distance read off this world is not a foot. The file keeps
      // the trade's numbers; the view does not.
      assert.ok(Math.abs(world.model.size.across / world.model.feet.across - 0.3048) < 1e-6,
        `${world.model.size.across} drawn against ${world.model.feet.across} ft`);
    });

    check('the wasm was asked for out of the program, not off a network', () => {
      assert.ok(world.wasm.asked, 'the loader was given no path at all');
      assert.doesNotMatch(world.wasm.asked, /^https?:/, world.wasm.asked);
      assert.doesNotMatch(world.wasm.asked, /unpkg|jsdelivr/, world.wasm.asked);
      assert.equal(world.wasm.absolute, true, 'the path is not absolute');
      assert.equal(world.wasm.onItsOwn, false,
        'the loader is still allowed to go and find its own wasm, which it does over the network');
    });

    check('the single-thread module, never the multi-thread one', () => {
      assert.doesNotMatch(world.wasm.asked, /-mt/, world.wasm.asked);
      for (const address of world.readFrom) assert.doesNotMatch(address, /-mt/, address);
    });

    check('every address the window loaded from is its own', () => {
      assert.deepEqual(world.fetched, [],
        `the window fetched ${world.fetched.join(', ')}`);
    });

    check('and where the window recorded the wasm, it came from this program', () => {
      // Resource timing is the window's own account of what it loaded. WebKit
      // may have nothing to say about a fetch made from inside a WebAssembly
      // loader, and an empty list is not evidence either way — so this row
      // carries what there is and says which. The claim with teeth is the pair
      // above: the path the loader was given, and the empty list of addresses
      // that were not this program's own.
      if (world.readFrom.length === 0) {
        assert.ok(world.fetched.length === 0,
          'the window recorded no address for the wasm, and it did fetch from elsewhere');
        return;
      }
      for (const address of world.readFrom) {
        assert.ok(address.startsWith(home), `${address} is not under ${home}`);
      }
    });
  }

  const said = await session.execute(function () {
    const el = document.querySelector('.model-ifc-says');
    return el ? (el.innerText || el.textContent || '') : '';
  });

  check('the window says what it stood up, in trade words', () => {
    assert.ok(said.trim().length > 0, 'it says nothing');
    for (const word of ['agent', 'inference', 'token', 'prompt', 'embedding', 'pipeline', 'entity', 'component', 'render']) {
      assert.doesNotMatch(said, new RegExp(`\\b${word}\\b`, 'i'), `"${word}" is on screen: ${said}`);
    }
    assert.doesNotMatch(said, /\bAI\b/, said);
    assert.doesNotMatch(said, /undefined|NaN|\[object/, said);
  });

  check('and it names what it did not stand up, with the reason', () => {
    assert.match(said, /not stood up/, said);
  });

  const errors = await errorsSoFar(session);
  check('nothing in the window errored while the roof was standing up', () => {
    assert.deepEqual(errors, [], errors.join(' | '));
  });

  await writeFile(join(EVIDENCE, `section10b-toolkit-${COMMIT}.png`),
    Buffer.from(await session.screenshot(), 'base64'));
  await writeFile(join(EVIDENCE, `section10b-ifc-${COMMIT}.png`),
    Buffer.from(await elementShot(session, '.model-stage'), 'base64'));
} catch (e) {
  results.push('FAIL');
  console.log(`  FAIL  the check stopped: ${e.message}`);
} finally {
  await app.close();
}

const failed = results.filter((r) => r === 'FAIL').length;
console.log(failed ? `\nFAIL — ${failed} of ${results.length}` : `\nPASS — ${results.length}/${results.length}`);
process.exit(failed ? 1 : 0);

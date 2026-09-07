// ── Evidence from the shipped window ───────────────────────────────────────
// Screenshots for a section's done-check come from the Tauri window, never from
// a browser. This sets up a job with a drawing and some traces on it, drives the
// real application, and writes a labelled pair of shots per area.
//
//   node tools/probe/evidence-shots.mjs <label>
//
// Writes evidence/<area>-<label>-<commit>.png

import { execFileSync } from 'node:child_process';
import { cp, mkdir, readFile, writeFile, mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { launch, until, wait } from './tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '../..');
const EVIDENCE = join(ROOT, 'evidence');
const COMMIT = execFileSync('git', ['rev-parse', '--short', 'HEAD'], { cwd: ROOT }).toString().trim();
const label = process.argv[2] ?? 'shot';

/**
 * Prices come from the seeded synthetic set, never from anywhere else.
 *
 * A screenshot needs plausible money on it, and the only safe source of that is
 * one that was invented on purpose. Reaching for a real figure has gone wrong
 * three times; a generated set that is always closer to hand is the fix.
 */
const PRICES = JSON.parse(
  await readFile(resolve(import.meta.dirname, '../../jobs/demo-job/prices.json'), 'utf8'),
);
const priced = (id, over = {}) => {
  const found = PRICES.items.find((i) => i.id === id);
  if (!found) throw new Error(`no synthetic price called ${id}`);
  return { ...found, ...over };
};

/** A job with a drawing, three traces and a few priced lines to look at. */
async function demoJob() {
  const dir = await mkdtemp(join(tmpdir(), 'roofnerd-shots-'));
  await cp(join(ROOT, 'jobs/demo-job'), dir, { recursive: true });
  await mkdir(join(dir, 'pages'), { recursive: true });
  await cp(join(ROOT, 'tools/probe/test-sheet.pdf'), join(dir, 'pages/roof-plan.pdf'));

  const trace = (id, points) => ({ id, pageId: 'page-1', points });
  const write = (name, value) => writeFile(join(dir, name), `${JSON.stringify(value, null, 2)}\n`);

  await write('pages/pages.json', [{
    id: 'page-1', name: 'Roof Plan', source: 'pages/roof-plan.pdf', pageNumber: 1,
    feetPerUnit: 4 / 72, scaleNote: '1/4" = 1\'-0"',
  }]);

  await write('conditions.json', [
    {
      id: 'c-field', name: 'Main Roof Field', kind: 'area', color: '#2f6f4f',
      properties: { PITCH: 4 },
      traces: [trace('t1', [{ x: 110, y: 150 }, { x: 430, y: 150 }, { x: 430, y: 400 }, { x: 110, y: 400 }])],
      items: [
        priced('tpo-60', { formula: 'SQ' }),
        priced('iso-22', { formula: 'SQ' }),
        priced('install-membrane', { formula: 'SQ' }),
      ],
    },
    {
      id: 'c-parapet', name: 'Parapet Wall Flashing', kind: 'line', color: '#1d5f8a',
      properties: { H: 1.5, STRETCHOUT: 14 },
      traces: [trace('t2', [{ x: 110, y: 430 }, { x: 430, y: 430 }, { x: 430, y: 560 }])],
      items: [
        priced('counterflash', { formula: 'LF * H' }),
        priced('coping', { formula: 'LF' }),
        priced('mitre', { formula: 'VERTICES' }),
      ],
    },
    {
      id: 'c-drains', name: 'Roof Drains', kind: 'count', color: '#8a5a1d',
      properties: {},
      traces: [trace('t3', [{ x: 180, y: 610 }, { x: 300, y: 610 }, { x: 420, y: 610 }])],
      items: [
        priced('drain', { formula: 'EA' }),
      ],
    },
  ]);

  await write('job.json', {
    format: 1, name: 'Warehouse Reroof', activeScenarioId: 's1',
    scenarios: [{
      id: 's1', name: 'Scenario 1', prices: {},
      adders: {
        Material: { tax: 7, escalation: 3 }, Labor: { burden: 10 },
        Sub: { generalLiability: 5 }, Supervision: { burden: 50 },
      },
      overhead: 0, profit: 15, bond: 2,
    }],
  });

  return { dir, remove: () => rm(dir, { recursive: true, force: true }) };
}

await mkdir(EVIDENCE, { recursive: true });
const job = await demoJob();
const app = await launch();
const { session } = app;

try {
  await until(session, () => document.querySelector('#editor')?.children.length > 0,
    { what: 'the editor to mount' });
  await session.execute((folder) =>
    window.__TAURI_INTERNALS__.invoke('doc_open', { folder }), job.dir);
  await wait(1200);
  await until(session, () => document.querySelector('.surface-overlay')?.getAttribute('viewBox') !== null,
    { what: 'the drawing', timeout: 25000 });
  await wait(1500);

  const shot = async (name) => {
    const path = join(EVIDENCE, `${name}-${label}-${COMMIT}.png`);
    await writeFile(path, Buffer.from(await session.screenshot(), 'base64'));
    console.log(`  ${path.replace(`${ROOT}/`, '')}`);
  };

  await shot('plan');

  // The condition panel, with a condition selected.
  await session.execute(() => document.querySelector('.condition')?.click());
  await wait(600);
  await shot('condition-panel');

  const before = await session.handles();
  await session.execute(() => window.__TAURI_INTERNALS__.invoke('open_editor', { editor: 'estimate' }));
  await wait(2000);
  const after = await session.handles();
  const second = after.find((h) => !before.includes(h));
  if (second) {
    await session.switchTo(second);
    await until(session, () => document.querySelector('#editor')?.children.length > 0,
      { what: 'the sheet' });
    await wait(800);
    await shot('estimate-sheet');
  }
} finally {
  await app.close();
  await job.remove();
}

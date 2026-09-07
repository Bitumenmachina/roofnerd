// ── Evidence from the shipped window ───────────────────────────────────────
// Screenshots for a section's done-check come from the Tauri window, never from
// a browser. This sets up a job with a drawing and some traces on it, drives the
// real application, and writes a labelled pair of shots per area.
//
//   node tools/probe/evidence-shots.mjs <label>
//
// Writes evidence/<area>-<label>-<commit>.png

import { execFileSync } from 'node:child_process';
import { cp, mkdir, writeFile, mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { launch, until, wait } from './tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '../..');
const EVIDENCE = join(ROOT, 'evidence');
const COMMIT = execFileSync('git', ['rev-parse', '--short', 'HEAD'], { cwd: ROOT }).toString().trim();
const label = process.argv[2] ?? 'shot';

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
        { id: 'i-mem', description: 'TPO membrane, 60 mil', costCode: '07-100-100', unit: 'SQ',
          formula: 'SQ', waste: 10, order: { name: 'ROLL', per: 10, rule: 'ceil' },
          price: { name: 'SF', contains: 1000, rule: 'exact' }, unitCost: 1.95 },
        { id: 'i-iso', description: '2.2" polyiso, 4 x 8', costCode: '07-100-100', unit: 'SQ',
          formula: 'SQ', waste: 10, unitCost: 88.5 },
        { id: 'i-lab', description: 'Install membrane', costCode: '07-100-230', unit: 'SQ',
          formula: 'SQ', productionRate: 0.35, crewSize: 6, unitCost: 50 },
      ],
    },
    {
      id: 'c-parapet', name: 'Parapet Wall Flashing', kind: 'line', color: '#1d5f8a',
      properties: { H: 1.5, STRETCHOUT: 14 },
      traces: [trace('t2', [{ x: 110, y: 430 }, { x: 430, y: 430 }, { x: 430, y: 560 }])],
      items: [
        { id: 'i-wall', description: 'Wall flashing membrane', costCode: '07-100-100', unit: 'SF',
          formula: 'LF * H', unitCost: 1.95 },
        { id: 'i-cop', description: '24 ga coping, formed', costCode: '07-100-150', unit: 'LF',
          formula: 'LF', unitCost: 26.5 },
        { id: 'i-mitre', description: 'Coping mitres', costCode: '07-100-150', unit: 'EA',
          formula: 'VERTICES', unitCost: 52 },
      ],
    },
    {
      id: 'c-drains', name: 'Roof Drains', kind: 'count', color: '#8a5a1d',
      properties: {},
      traces: [trace('t3', [{ x: 180, y: 610 }, { x: 300, y: 610 }, { x: 420, y: 610 }])],
      items: [
        { id: 'i-drain', description: 'Retrofit drain assembly', costCode: '07-100-100', unit: 'EA',
          formula: 'EA', unitCost: 410 },
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

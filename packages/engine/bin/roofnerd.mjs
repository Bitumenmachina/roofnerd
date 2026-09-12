#!/usr/bin/env node
// The estimating engine, with no window open.
//
//   roofnerd recap <job-folder>      roll the job up to a selling price
//   roofnerd check <job-folder>      read it and say whether it round-trips clean
//   roofnerd normalize <job-folder>  write it back in canonical form
//
// This exists so the arithmetic can be run from a script, a test or a terminal,
// and so nothing about a window is ever able to change a number.

import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { readJob, writeJob, recap, formatRecap, plural, PRODUCT_NAME } from '../dist/index.js';

const [, , command, target] = process.argv;

function usage(code) {
  console.log(`${PRODUCT_NAME} — estimating engine

  ${PRODUCT_NAME} recap <job-folder>      roll the job up to a selling price
  ${PRODUCT_NAME} check <job-folder>      say whether the folder round-trips clean
  ${PRODUCT_NAME} normalize <job-folder>  write the folder back in canonical form`);
  process.exit(code);
}

if (!command || command === '-h' || command === '--help') usage(0);
if (!target) { console.error('which job folder?'); usage(1); }

const root = resolve(target);
const read = (rel) => {
  try { return readFileSync(join(root, rel), 'utf8'); }
  catch (e) { if (e.code === 'ENOENT') return null; throw e; }
};
const write = (rel, contents) => {
  const path = join(root, rel);
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, contents);
};

try {
  const doc = readJob(read);

  if (command === 'recap') {
    const id = doc.job.activeScenarioId;
    const scenario = doc.job.scenarios.find((s) => s.id === id) ?? doc.job.scenarios[0];
    if (!scenario) { console.error(`${doc.job.name} has no scenario to price against`); process.exit(1); }
    console.log(`${doc.job.name} — ${scenario.name}`);
    console.log('='.repeat(48));
    console.log(formatRecap(recap(doc, scenario)));
    process.exit(0);
  }

  if (command === 'check' || command === 'normalize') {
    // Serialize to memory first. A check that writes is not a check.
    const rewritten = new Map();
    writeJob(doc, (rel, contents) => rewritten.set(rel, contents));

    const moved = [...rewritten.entries()].filter(([rel, now]) => read(rel) !== now);

    if (command === 'check') {
      if (moved.length === 0) {
        console.log(`${doc.job.name}: round-trips clean (${rewritten.size} files)`);
        process.exit(0);
      }
      console.error(`${doc.job.name}: these files are not in canonical form:`);
      for (const [rel] of moved) console.error(`  ${rel}`);
      console.error(`\nrun: ${PRODUCT_NAME} normalize ${target}`);
      process.exit(1);
    }

    for (const [rel, contents] of rewritten) write(rel, contents);
    console.log(moved.length === 0
      ? `${doc.job.name}: already canonical, nothing moved`
      : `${doc.job.name}: rewrote ${plural(moved.length, 'file')} — ${moved.map(([r]) => r).join(', ')}`);
    process.exit(0);
  }

  console.error(`unknown command "${command}"`);
  usage(1);
} catch (e) {
  console.error(`${e.name === 'JobFolderError' ? '' : `${e.name}: `}${e.message}`);
  process.exit(1);
}

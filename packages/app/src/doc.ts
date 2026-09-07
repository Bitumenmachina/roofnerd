// ── The one document, seen from a window ───────────────────────────────────
// A window does not hold the job. It asks the shell for it once, then listens.
// Every change to the job — from this window or any other — arrives here and
// every subscriber redraws. That is the entire mechanism behind two windows
// never disagreeing, and it is deliberately this small.

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

/**
 * The open job as the shell holds it: keyed by the file each part came from,
 * so this layer needs no opinion about the job's shape. The shape belongs to
 * @roofnerd/engine.
 */
export type Doc = Record<string, unknown>;

type Subscriber = (doc: Doc) => void;

let current: Doc = {};
const subscribers = new Set<Subscriber>();

function publish(doc: Doc) {
  current = doc;
  for (const fn of subscribers) fn(doc);
}

/** Read what the shell has, and follow it from here on. */
export async function connect(): Promise<Doc> {
  publish(await invoke<Doc>('doc_get'));
  await listen<Doc>('doc:changed', (event) => publish(event.payload));
  return current;
}

export function subscribe(fn: Subscriber): () => void {
  subscribers.add(fn);
  fn(current);
  return () => subscribers.delete(fn);
}

export const doc = () => current;

/**
 * Change one value. The window does not update itself — it asks the shell to
 * change the document, and then hears about it like everyone else. A window
 * that edited its own copy first is a window that can drift.
 */
export const set = (pointer: string, value: unknown) =>
  invoke<void>('doc_set', { pointer, value });

export const open = (folder: string) => invoke<void>('doc_open', { folder });
export const save = () => invoke<string>('doc_save');
export const folder = () => invoke<string | null>('doc_folder');

/** Where the demo job sits, if this build can still find it. Null once installed. */
export const demoFolder = () => invoke<string | null>('demo_folder');
export const tearOff = (editor: string) => invoke<void>('open_editor', { editor });

/** Read a value out of the document by pointer. Returns undefined if absent. */
export function at(pointer: string, from: Doc = current): unknown {
  if (pointer === '') return from;
  let node: unknown = from;
  for (const raw of pointer.slice(1).split('/')) {
    const key = raw.replace(/~1/g, '/').replace(/~0/g, '~');
    if (node === null || typeof node !== 'object') return undefined;
    node = Array.isArray(node)
      ? node[Number(key)]
      : (node as Record<string, unknown>)[key];
  }
  return node;
}

// ── The roof, again, as IFCX (alpha) ───────────────────────────────────────
// The adapter pattern of `~/three-axis/tools/ifc5-bridge` — read, not copied:
// that tool is Python, reads an IFC4 file that already exists with
// `ifcopenshell.open()`, and this product ships no Python. What is reused is
// the SHAPE its `schema_map.py`/`authority.py` settled on, ported to
// TypeScript so the one writer that already has the solve in hand can put
// out both files without a second process reading its own output back in:
//
//   header       {id, ifcxVersion, dataVersion, author, timestamp} — not
//                description/sourceFile/generator, which hello-wall.ifcx
//                does not carry either
//   children     a parent's own dict of child name -> child UUID
//   path         a bare UUID per node, not an IFC GlobalId
//   attributes   `bsi::ifc::class` for what the element is, `pset::<Name>::
//                <Prop>` for everything this writer's own property sets carry
//                (none of them are a bSI `*Common` pset, so nothing here uses
//                the shorter `bsi::ifc::prop::` form)
//
// One simplification against the reference tool, recorded rather than
// disguised as the same shape: `convert_ifc4_to_ifcx.py` emits several
// same-path nodes per element — one per concern, matching how a real IFC4
// file's own relationships are shaped when read back out of `ifcopenshell`.
// This writer already holds the whole element as one `RoofPlan` entry with
// nothing left to reassemble, so it puts every concern on the one node the
// path names. And `authority.py`'s own same-path layered-override scheme
// keys on FIVE fields (`AuthoritySource`, `AuthorityBasis`, `AuthorityDate`,
// `AuthorityWeight`, …) that this program's `Pset_Authority` does not carry —
// the charter's own shape is `src`/`hier`/`constraint`/`overridable` (D159),
// the operator's cross-project shorthand, not ifc5-bridge's. The two are
// reconciled by neither pretending to be the other: `Pset_Authority` here
// goes through the same generic `pset::Pset_Authority::<field>` mapping as
// any other property set, and the same-path weighted override this bridge is
// built for is not attempted.
//
// Path UUIDs are not the compressed IFC GlobalId expanded back out — this
// writer makes both files from the same `RoofPlan`, so it derives the UUID
// from the same 128 bits `identityBits` hashes the GlobalId from, formatted as
// the dashed hex form IFCX wants instead of buildingSMART's base 64. Writing
// the same roof twice writes the same UUIDs, the same as the same GlobalIds.

import type { RoofPlan } from './write.js';
import { expectedOf, identityBits } from './write.js';

const IFCX_VERSION = 'ifcx_alpha';
const DATA_VERSION = '1.0.0';

/** The 128 bits an identity is built from, as the dashed hex form IFCX's own `path` wants. */
function uuidOf(key: string): string {
  const hex = identityBits(key).toString(16).padStart(32, '0').slice(-32);
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20, 32)}`;
}

/** A dict key a USD-style `children` map accepts: no spaces, no slashes. */
function childKey(name: string): string {
  return name.trim().replace(/\//g, '_').replace(/\s+/g, '_') || 'Element';
}

interface IfcxNode {
  readonly path: string;
  readonly children?: Readonly<Record<string, string>>;
  readonly attributes?: Readonly<Record<string, unknown>>;
}

/**
 * The plan as an `.ifcx` (alpha) document.
 *
 * Pure, like `roofPlan`/`writeRoof`: the `RoofPlan` and `expectedOf`'s own
 * reading of it are all this needs, so the file this writes can never
 * disagree with the IFC4 one about a quantity — both come off the one plan.
 */
export function writeIfcx(plan: RoofPlan): string {
  const expected = expectedOf(plan);
  const byKey = new Map<string, { readonly class: string; readonly attributes: Record<string, unknown> }>();
  const order: string[] = [];
  const keyFor = (cls: string, name: string) => `${cls}:${name}`;

  for (const e of expected.entries) {
    const k = keyFor(e.class, e.name);
    let entry = byKey.get(k);
    if (!entry) {
      entry = { class: e.class, attributes: {} };
      byKey.set(k, entry);
      order.push(k);
    }
    entry.attributes[`pset::${e.set}::${e.key}`] = e.value;
  }

  const projectId = uuidOf(`ifcx:project:${plan.job}`);
  const siteId = uuidOf(`ifcx:site:${plan.job}`);
  const buildingId = uuidOf(`ifcx:building:${plan.job}`);
  const storeyId = uuidOf(`ifcx:storey:${plan.job}`);

  const storeyChildren: Record<string, string> = {};
  const elementNodes: IfcxNode[] = [];
  for (const k of order) {
    const entry = byKey.get(k)!;
    const [, ...nameParts] = k.split(':');
    const name = nameParts.join(':');
    const id = uuidOf(`ifcx:element:${k}`);
    storeyChildren[childKey(name)] = id;
    elementNodes.push({
      path: id,
      attributes: {
        'bsi::ifc::class': { code: entry.class, uri: `https://identifier.buildingsmart.org/uri/buildingsmart/ifc/4.3/class/${entry.class}` },
        'nlsfb::name': name,
        ...entry.attributes,
      },
    });
  }

  const data: IfcxNode[] = [
    { path: projectId, children: { [childKey(plan.job)]: siteId } },
    { path: siteId, children: { [childKey('Site')]: buildingId } },
    { path: buildingId, children: { [childKey(plan.job)]: storeyId } },
    { path: storeyId, children: storeyChildren },
    ...elementNodes,
  ];

  const document = {
    header: {
      id: uuidOf(`ifcx:document:${plan.job}`),
      ifcxVersion: IFCX_VERSION,
      dataVersion: DATA_VERSION,
      author: 'roofnerd',
      // Fixed, not read off the clock: two exports of an unchanged job should
      // be the same file, and unlike the IFC4 file's STEP header — which
      // carries the hour and is stripped before the two are compared — a JSON
      // document has no line to strip it from later. So it never goes in.
      timestamp: new Date(0).toISOString(),
    },
    imports: [
      { uri: 'https://ifcx.dev/@standards.buildingsmart.org/ifc/core/ifc@v5a.ifcx' },
      { uri: 'https://ifcx.dev/@standards.buildingsmart.org/ifc/core/prop@v5a.ifcx' },
      { uri: 'https://ifcx.dev/@openusd.org/usd@v1.ifcx' },
    ],
    schemas: {},
    data,
  };

  return JSON.stringify(document, null, 2);
}

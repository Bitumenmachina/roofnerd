// ── The tree ───────────────────────────────────────────────────────────────
// A bid nests, so the thing on the left nests:
//
//   Job
//     Page — a drawing
//       Condition — a thing traced on it
//         Item — what that condition consumes
//
// It is the shape of the job, not a list of one kind of thing. Selecting a node
// here selects it everywhere, which is what makes it navigation rather than
// decoration.

import { at, type Doc } from './doc.js';
import { icon } from './icons.js';
import { KIND_LABELS, plural, quantity } from './labels.js';
import { select, selected, watchSelection, type Selected } from './selection.js';

type Condition = {
  id: string; name: string; kind: 'area' | 'line' | 'count';
  traces?: { id: string; pageId: string }[];
  items?: { id: string; description: string }[];
  color?: string; from?: string;
};
type Page = { id: string; name: string; feetPerUnit?: number };

const open = new Set<string>();

export function renderTree(host: HTMLElement, doc: Doc): void {
  host.replaceChildren();

  const pages = (at('/pages', doc) as Page[]) ?? [];
  const conditions = (at('/conditions', doc) as Condition[]) ?? [];
  const jobName = at('/job/name', doc);

  const root = document.createElement('ul');
  root.className = 'tree';
  root.setAttribute('role', 'tree');

  const job = node({
    label: typeof jobName === 'string' && jobName ? jobName : 'No job open',
    iconName: 'job',
    depth: 0,
    id: 'job',
    target: { kind: 'job' },
    hasChildren: pages.length > 0,
  });
  root.append(job.row);

  if (isOpen('job')) {
    for (const page of pages) {
      const onThisPage = conditions.filter((c) => (c.traces ?? []).some((t) => t.pageId === page.id));
      const key = `page:${page.id}`;
      const pageNode = node({
        label: page.name,
        detail: page.feetPerUnit ? undefined : 'not scaled',
        iconName: 'page',
        depth: 1,
        id: key,
        target: { kind: 'page', id: page.id },
        hasChildren: onThisPage.length > 0,
      });
      root.append(pageNode.row);
      if (!isOpen(key)) continue;

      for (const condition of onThisPage) {
        const ckey = `condition:${condition.id}`;
        const items = condition.items ?? [];
        const conditionNode = node({
          label: condition.name,
          detail: KIND_LABELS[condition.kind],
          swatch: condition.color,
          depth: 2,
          id: ckey,
          target: { kind: 'condition', id: condition.id },
          hasChildren: items.length > 0,
        });
        root.append(conditionNode.row);
        if (!isOpen(ckey)) continue;

        for (const item of items) {
          const itemNode = node({
            label: item.description || 'Untitled line',
            iconName: 'item',
            depth: 3,
            id: `item:${item.id}`,
            target: { kind: 'item', conditionId: condition.id, id: item.id },
            hasChildren: false,
          });
          root.append(itemNode.row);
        }
      }
    }
  }

  host.append(root);

  // Conditions traced on no page yet still have to be reachable.
  const orphans = conditions.filter((c) => !(c.traces ?? []).length);
  if (orphans.length) {
    const note = document.createElement('p');
    note.className = 'tree-note';
    note.textContent = `${plural(orphans.length, 'condition')} not on a drawing yet`;
    host.append(note);
  }
}

function isOpen(id: string): boolean {
  if (id === 'job') return !open.has('!job');
  return open.has(id);
}

interface NodeSpec {
  label: string;
  detail?: string | undefined;
  iconName?: 'job' | 'page' | 'item';
  swatch?: string | undefined;
  depth: number;
  id: string;
  target: Selected;
  hasChildren: boolean;
}

function node(spec: NodeSpec) {
  const row = document.createElement('li');
  row.className = 'tree-node';
  row.style.setProperty('--depth', String(spec.depth));
  row.setAttribute('role', 'treeitem');

  const twisty = document.createElement('button');
  twisty.className = 'twisty';
  twisty.type = 'button';
  if (spec.hasChildren) {
    const open_ = isOpen(spec.id);
    twisty.append(icon('chevron', 14));
    twisty.classList.toggle('open', open_);
    twisty.setAttribute('aria-label', open_ ? 'Collapse' : 'Expand');
    twisty.addEventListener('click', (e) => {
      e.stopPropagation();
      toggle(spec.id);
      row.dispatchEvent(new CustomEvent('tree:changed', { bubbles: true }));
    });
  } else {
    twisty.classList.add('empty');
    twisty.tabIndex = -1;
  }

  const button = document.createElement('button');
  button.className = 'tree-label';
  button.type = 'button';

  if (spec.swatch) {
    const dot = document.createElement('span');
    dot.className = 'swatch';
    dot.style.background = spec.swatch;
    button.append(dot);
  } else if (spec.iconName) {
    button.append(icon(spec.iconName, 15));
  }

  const text = document.createElement('span');
  text.className = 'tree-text';
  text.textContent = spec.label;
  button.append(text);

  if (spec.detail) {
    const detail = document.createElement('span');
    detail.className = 'tree-detail';
    detail.textContent = spec.detail;
    button.append(detail);
  }

  const now = selected();
  const isSelected =
    (now.kind === spec.target.kind) &&
    ('id' in now && 'id' in spec.target ? now.id === spec.target.id : now.kind === 'job');
  row.classList.toggle('on', isSelected);

  button.addEventListener('click', () => {
    if (spec.hasChildren && !isOpen(spec.id)) toggle(spec.id);
    select(spec.target);
    row.dispatchEvent(new CustomEvent('tree:changed', { bubbles: true }));
  });

  row.append(twisty, button);
  return { row };
}

function toggle(id: string): void {
  if (id === 'job') {
    if (open.has('!job')) open.delete('!job');
    else open.add('!job');
    return;
  }
  if (open.has(id)) open.delete(id);
  else open.add(id);
}

/** Everything under the job starts open — a bid with one page should show it. */
export function openByDefault(doc: Doc): void {
  for (const page of ((at('/pages', doc) as Page[]) ?? [])) open.add(`page:${page.id}`);
}

export { watchSelection, quantity };

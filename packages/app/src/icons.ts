// ── Icons ──────────────────────────────────────────────────────────────────
// Drawn inline, from Lucide's geometry (ISC). Inline because the content
// security policy allows this program no network at all, and because an icon
// that arrives as part of the page cannot fail to arrive.
//
// One weight, one grid, 16 px on screen. An icon never appears without its word
// beside it: a roofer should not have to learn a pictogram to find the line tool.

const SVG_NS = 'http://www.w3.org/2000/svg';

const PATHS: Readonly<Record<string, string[]>> = {
  select: ['M12.586 12.586 19 19', 'M3.688 3.037a.497.497 0 0 0-.651.651l6.5 15.999a.501.501 0 0 0 .947-.062l1.569-6.083a2 2 0 0 1 1.448-1.479l6.124-1.579a.5.5 0 0 0 .063-.947z'],
  area: ['M3 3h18v18H3z'],
  line: ['M3 17h4', 'M17 17h4', 'm7 17 5-10 5 10'],
  count: ['M12 12h.01', 'M20 12a8 8 0 1 1-16 0 8 8 0 0 1 16 0z'],
  scale: ['M21.3 8.7 8.7 21.3a1 1 0 0 1-1.4 0l-4.6-4.6a1 1 0 0 1 0-1.4L15.3 2.7a1 1 0 0 1 1.4 0l4.6 4.6a1 1 0 0 1 0 1.4Z', 'm7.5 10.5 2 2', 'm10.5 7.5 2 2', 'm13.5 4.5 2 2', 'm4.5 13.5 2 2'],
  zoomIn: ['M11 3a8 8 0 1 0 0 16 8 8 0 0 0 0-16z', 'm21 21-4.3-4.3', 'M11 8v6', 'M8 11h6'],
  zoomOut: ['M11 3a8 8 0 1 0 0 16 8 8 0 0 0 0-16z', 'm21 21-4.3-4.3', 'M8 11h6'],
  fit: ['M8 3H5a2 2 0 0 0-2 2v3', 'M21 8V5a2 2 0 0 0-2-2h-3', 'M3 16v3a2 2 0 0 0 2 2h3', 'M16 21h3a2 2 0 0 0 2-2v-3'],
  page: ['M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z', 'M14 2v4a2 2 0 0 0 2 2h4'],
  job: ['M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z'],
  item: ['M3 6h18', 'M3 12h18', 'M3 18h18'],
  tearOff: ['M3 5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z', 'M15 3v18'],
  help: ['M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20z', 'M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3', 'M12 17h.01'],
  close: ['M18 6 6 18', 'm6 6 12 12'],
  add: ['M5 12h14', 'M12 5v14'],
  chevron: ['m9 18 6-6-6-6'],
  addDrawing: ['M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z', 'M14 2v4a2 2 0 0 0 2 2h4', 'M12 11v6', 'M9 14h6'],
};

export type IconName = keyof typeof PATHS;

/** One icon, 16 px, currentColor, no fill. */
export function icon(name: IconName, size = 16): SVGSVGElement {
  const svg = document.createElementNS(SVG_NS, 'svg');
  svg.setAttribute('viewBox', '0 0 24 24');
  svg.setAttribute('width', String(size));
  svg.setAttribute('height', String(size));
  svg.setAttribute('fill', 'none');
  svg.setAttribute('stroke', 'currentColor');
  svg.setAttribute('stroke-width', '1.75');
  svg.setAttribute('stroke-linecap', 'round');
  svg.setAttribute('stroke-linejoin', 'round');
  svg.setAttribute('aria-hidden', 'true');
  svg.classList.add('icon');
  for (const d of PATHS[name] ?? []) {
    const path = document.createElementNS(SVG_NS, 'path');
    path.setAttribute('d', d);
    svg.append(path);
  }
  return svg;
}

/**
 * The twelve hues a condition is drawn in.
 *
 * They are for telling one traced thing from another and nothing else. The
 * interface has exactly one accent, and it is not in this list: a colour that
 * means "selected" on Tuesday must not mean "parapet" on Wednesday.
 */
export const CONDITION_HUES: readonly string[] = [
  '#2f6f4f', '#1d5f8a', '#8a3f2f', '#6b4f9e', '#8a6a1d', '#2f6f6f',
  '#a04a6f', '#4f6f2f', '#8a5a1d', '#3f5f9e', '#7a2f4f', '#5f5f5f',
] as const;

export const hueFor = (index: number): string => CONDITION_HUES[index % CONDITION_HUES.length]!;

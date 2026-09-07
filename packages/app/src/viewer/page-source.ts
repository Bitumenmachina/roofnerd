// ── Getting a drawing on screen ────────────────────────────────────────────
// A page is a PDF page or an image. Either way it ends up as something that can
// be painted into a canvas at whatever zoom the estimator is working at, plus
// the size of the sheet in page units.
//
// Page units are PDF points — seventy-two to the paper inch — so an
// architectural scale preset means the same thing on every sheet. An image has
// no such thing, so it gets 72 units per pixel-inch by convention and is scaled
// by two points like any un-dimensioned aerial.

import * as pdfjs from 'pdfjs-dist';
// The worker is bundled as a local file. Nothing is fetched: the security policy
// allows a worker from this program's own origin and a blob, and no more.
import PdfWorker from 'pdfjs-dist/build/pdf.worker.min.mjs?worker';

pdfjs.GlobalWorkerOptions.workerPort = new PdfWorker();

/** A sheet ready to be painted, at any zoom. */
export interface Sheet {
  /** Sheet size in page units. */
  readonly width: number;
  readonly height: number;
  /** Page units in one paper inch. 72 for a PDF. */
  readonly unitsPerInch: number;
  /** How many pages the source has, so a PDF can offer the rest of them. */
  readonly pageCount: number;
  render(canvas: HTMLCanvasElement, zoom: number): Promise<void>;
}

export async function loadPdf(data: ArrayBuffer, pageNumber = 1): Promise<Sheet> {
  const doc = await pdfjs.getDocument({ data }).promise;
  const page = await doc.getPage(pageNumber);
  const base = page.getViewport({ scale: 1 });

  // pdf.js will not paint the same canvas twice at once, and an estimator
  // leaning on the zoom key asks it to do exactly that. Whatever is still
  // painting gets cancelled; only the zoom they landed on gets drawn.
  let inFlight: pdfjs.RenderTask | null = null;

  return {
    width: base.width,
    height: base.height,
    unitsPerInch: 72,
    pageCount: doc.numPages,
    async render(canvas, zoom) {
      inFlight?.cancel();
      inFlight = null;

      const viewport = page.getViewport({ scale: zoom });
      canvas.width = Math.floor(viewport.width);
      canvas.height = Math.floor(viewport.height);
      const context = canvas.getContext('2d');
      if (!context) throw new Error('this window cannot paint a drawing');

      const task = page.render({ canvas, canvasContext: context, viewport });
      inFlight = task;
      try {
        await task.promise;
      } catch (e) {
        // A cancelled paint is not a failure — it is a newer zoom winning.
        if ((e as { name?: string })?.name !== 'RenderingCancelledException') throw e;
      } finally {
        if (inFlight === task) inFlight = null;
      }
    },
  };
}

export async function loadImage(data: ArrayBuffer, type: string): Promise<Sheet> {
  const blob = new Blob([data], { type });
  const bitmap = await createImageBitmap(blob);

  return {
    width: bitmap.width,
    height: bitmap.height,
    // An aerial has no paper size. This only matters for the scale presets,
    // which do not apply to a photograph anyway — it gets two points.
    unitsPerInch: 72,
    pageCount: 1,
    async render(canvas, zoom) {
      canvas.width = Math.floor(bitmap.width * zoom);
      canvas.height = Math.floor(bitmap.height * zoom);
      const context = canvas.getContext('2d');
      if (!context) throw new Error('this window cannot paint a drawing');
      context.drawImage(bitmap, 0, 0, canvas.width, canvas.height);
    },
  };
}

/** Read a drawing off disk and work out which kind it is. */
export async function loadSheet(data: ArrayBuffer, filename: string, pageNumber = 1): Promise<Sheet> {
  const lower = filename.toLowerCase();
  if (lower.endsWith('.pdf')) return loadPdf(data, pageNumber);
  if (/\.(png|jpe?g|webp)$/.test(lower)) {
    const type = lower.endsWith('.png') ? 'image/png' : lower.endsWith('.webp') ? 'image/webp' : 'image/jpeg';
    return loadImage(data, type);
  }
  throw new Error(`${filename} is not a drawing this program can open (PDF, PNG or JPG)`);
}

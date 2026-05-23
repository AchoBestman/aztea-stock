import { jsPDF } from 'jspdf';

export type Html2PdfWorkerOptions = {
  margin?: number | number[];
  filename: string;
  image?: { type?: string; quality?: number };
  html2canvas?: { scale?: number; useCORS?: boolean; logging?: boolean };
  jsPDF?: { unit?: string; format?: string | number[]; orientation?: string };
  pagebreak?: { mode?: string | string[] };
};

/** Document HTML propre (sans oklch). */
export function wrapPdfDocument(bodyHtml: string, title = 'Document'): string {
  const trimmed = bodyHtml.trim();
  if (/<html[\s>]/i.test(trimmed)) {
    return trimmed.replace(/oklch\([^)]+\)/gi, '#000000');
  }
  const safeBody = bodyHtml.replace(/oklch\([^)]+\)/gi, '#000000');
  return `<!DOCTYPE html><html><head><meta charset="utf-8"><title>${title}</title></head><body>${safeBody}</body></html>`;
}

export function stripTailwindFromHtml(html: string): string {
  return html
    .replace(/\sclass="[^"]*"/gi, '')
    .replace(/\sclass='[^']*'/gi, '')
    .replace(/style="[^"]*oklch[^"]*"/gi, '')
    .replace(/style='[^']*oklch[^']*'/gi, '')
    .replace(/oklch\([^)]+\)/gi, '#000000');
}

/** html2pdf laissait des overlays — nettoyage si ancienne session. */
export function cleanupHtml2PdfArtifacts(): void {
  document.querySelectorAll('[data-pdf-export-mount]').forEach((n) => n.remove());
}

/** Convertit le HTML en lignes de texte pour jsPDF (pas de html2canvas / oklch). */
function htmlToPlainLines(htmlContent: string): string[] {
  let html = htmlContent.replace(/oklch\([^)]+\)/gi, '#000000');
  const bodyMatch = html.match(/<body[^>]*>([\s\S]*)<\/body>/i);
  if (bodyMatch) html = bodyMatch[1];

  html = html
    .replace(/<script[\s\S]*?<\/script>/gi, '')
    .replace(/<style[\s\S]*?<\/style>/gi, '')
    .replace(/<br\s*\/?>/gi, '\n')
    .replace(/<\/tr>/gi, '\n')
    .replace(/<\/p>/gi, '\n')
    .replace(/<\/div>/gi, '\n')
    .replace(/<\/h[1-6]>/gi, '\n')
    .replace(/<\/td>/gi, '\t')
    .replace(/<\/th>/gi, '\t')
    .replace(/<[^>]+>/g, '')
    .replace(/&nbsp;/g, ' ')
    .replace(/&amp;/g, '&')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>');

  const lines = html
    .split('\n')
    .map((l) => l.replace(/\t+/g, ' | ').trim())
    .filter(Boolean);

  if (lines.length === 0) {
    lines.push('Document Aztea Stock');
  }
  return lines;
}

function parseHtmlTables(html: string): { title: string; subtitle: string; head: string[]; rows: string[][] } {
  let body = html;
  const bodyMatch = html.match(/<body[^>]*>([\s\S]*)<\/body>/i);
  if (bodyMatch) body = bodyMatch[1];

  const titleMatch = body.match(/<h1[^>]*>([\s\S]*?)<\/h1>/i);
  const title = titleMatch ? titleMatch[1].replace(/<[^>]+>/g, '').trim() : 'Rapport AzteaStock';

  const subMatch = body.match(/<p[^>]*>([\s\S]*?)<\/p>/i);
  const subtitle = subMatch ? subMatch[1].replace(/<[^>]+>/g, '').trim() : '';

  const rowMatches = [...body.matchAll(/<tr[^>]*>([\s\S]*?)<\/tr>/gi)];
  const cellsFromRow = (rowHtml: string) =>
    [...rowHtml.matchAll(/<t[hd][^>]*>([\s\S]*?)<\/t[hd]>/gi)].map((c) =>
      c[1].replace(/<[^>]+>/g, '').replace(/\s+/g, ' ').trim()
    );

  let head: string[] = [];
  const rows: string[][] = [];
  rowMatches.forEach((m, idx) => {
    const cells = cellsFromRow(m[1]);
    if (cells.length === 0) return;
    if (idx === 0) head = cells;
    else rows.push(cells);
  });

  return { title, subtitle, head, rows };
}

function buildReportPdfFromHtml(htmlContent: string, options: Html2PdfWorkerOptions): jsPDF {
  const format = options.jsPDF?.format ?? 'a4';
  const unit = (options.jsPDF?.unit ?? 'mm') as 'mm';
  const orientation = (options.jsPDF?.orientation ?? 'portrait') as 'portrait' | 'landscape';
  const margin = 12;

  const pdf = new jsPDF({ unit, format, orientation });
  const { title, subtitle, head, rows } = parseHtmlTables(htmlContent);

  pdf.setFont('helvetica', 'bold');
  pdf.setFontSize(16);
  pdf.setTextColor(20, 20, 20);
  pdf.text(title, margin, 18);

  if (subtitle) {
    pdf.setFont('helvetica', 'normal');
    pdf.setFontSize(9);
    pdf.setTextColor(90, 90, 90);
    const subLines = pdf.splitTextToSize(subtitle, pdf.internal.pageSize.getWidth() - margin * 2) as string[];
    pdf.text(subLines, margin, 26);
  }

  // Tableau stylé
  let y = subtitle ? 32 : 24;
  const colCount = head.length || (rows[0]?.length ?? 0);
  const pageW = pdf.internal.pageSize.getWidth() - margin * 2;
  const colW = colCount > 0 ? pageW / colCount : pageW;

  const drawRow = (cells: string[], header = false) => {
    const rowH = 7;
    if (y + rowH > pdf.internal.pageSize.getHeight() - margin) {
      pdf.addPage();
      y = margin;
    }
    pdf.setFillColor(header ? 240 : 255, header ? 240 : 255, header ? 240 : 255);
    pdf.setDrawColor(200, 200, 200);
    pdf.rect(margin, y, pageW, rowH, header ? 'FD' : 'D');
    pdf.setFont('helvetica', header ? 'bold' : 'normal');
    pdf.setFontSize(8);
    pdf.setTextColor(0, 0, 0);
    cells.forEach((cell, i) => {
      const text = pdf.splitTextToSize(cell.substring(0, 40), colW - 2) as string[];
      pdf.text(text[0] ?? '', margin + i * colW + 1.5, y + 4.5);
    });
    y += rowH;
  };

  if (head.length > 0) drawRow(head, true);
  rows.forEach((r) => drawRow(r));

  // Lignes hors tableau (totaux, etc.)
  const plainLines = htmlToPlainLines(htmlContent).filter(
    (l) => !head.some((h) => l.includes(h)) && !rows.flat().some((c) => l === c)
  );
  const totalLines = plainLines.filter((l) => l.toLowerCase().includes('chiffre') || l.toLowerCase().includes('total'));
  if (totalLines.length > 0) {
    y += 4;
    pdf.setFont('helvetica', 'bold');
    pdf.setFontSize(10);
    totalLines.forEach((line) => {
      if (y > pdf.internal.pageSize.getHeight() - margin) {
        pdf.addPage();
        y = margin;
      }
      pdf.text(line, margin, y);
      y += 6;
    });
  }

  return pdf;
}

function buildPdfFromHtml(htmlContent: string, options: Html2PdfWorkerOptions): jsPDF {
  const hasTable = /<table/i.test(htmlContent);
  if (hasTable) {
    return buildReportPdfFromHtml(htmlContent, options);
  }

  const lines = htmlToPlainLines(htmlContent);
  const format = options.jsPDF?.format ?? 'a4';
  const unit = (options.jsPDF?.unit ?? 'mm') as 'mm';
  const orientation = (options.jsPDF?.orientation ?? 'portrait') as 'portrait' | 'landscape';
  const margin = Array.isArray(options.margin)
    ? options.margin[0] ?? 5
    : typeof options.margin === 'number'
      ? options.margin
      : 5;

  const pdf = new jsPDF({ unit, format, orientation });
  const pageWidth = pdf.internal.pageSize.getWidth();
  const maxTextWidth = pageWidth - margin * 2;
  let y = margin + 4;
  const lineH = 4.2;
  const maxY = pdf.internal.pageSize.getHeight() - margin;

  pdf.setFont('helvetica', 'normal');
  pdf.setFontSize(9);
  pdf.setTextColor(0, 0, 0);

  for (const line of lines) {
    const wrapped = pdf.splitTextToSize(line, maxTextWidth) as string[];
    for (const part of wrapped) {
      if (y > maxY) {
        pdf.addPage();
        y = margin + 4;
      }
      pdf.text(part, margin, y);
      y += lineH;
    }
  }

  return pdf;
}

export function pdfToBlob(pdf: jsPDF): Blob {
  const out = pdf.output('blob');
  if (out instanceof Blob) return out;
  if (out instanceof ArrayBuffer) return new Blob([out], { type: 'application/pdf' });
  if (out instanceof Uint8Array) return new Blob([out], { type: 'application/pdf' });
  throw new Error('Format de sortie PDF non reconnu.');
}

/** Génère un PDF (jsPDF uniquement — fiable dans Tauri, sans html2canvas). */
export async function exportHtmlToPdfBlob(
  htmlContent: string,
  options: Html2PdfWorkerOptions
): Promise<Blob> {
  try {
    const pdf = buildPdfFromHtml(htmlContent, options);
    const blob = pdfToBlob(pdf);
    if (blob.size < 100) {
      throw new Error('Le fichier PDF généré est vide.');
    }
    return blob;
  } catch (err) {
    console.error('[PDF] Génération échouée:', err);
    throw err instanceof Error ? err : new Error('Échec de la génération du PDF.');
  }
}

/** Enregistre dans Téléchargements (Tauri) ou déclenche le téléchargement navigateur. */
export async function deliverPdfBlob(blob: Blob, filename: string): Promise<string> {
  const safeName = filename.trim() || 'document.pdf';
  const finalName = safeName.toLowerCase().endsWith('.pdf') ? safeName : `${safeName}.pdf`;

  const buffer = await blob.arrayBuffer();
  const bytes = new Uint8Array(buffer);
  let binary = '';
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  const pdfBase64 = btoa(binary);

  const { isTauriApp } = await import('./hardwareConfig');
  if (isTauriApp()) {
    const { invoke } = await import('@tauri-apps/api/core');
    const path = await invoke<string>('save_pdf_to_downloads', {
      pdfBase64,
      filename: finalName,
    });
    return path;
  }

  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = finalName;
  anchor.style.display = 'none';
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  URL.revokeObjectURL(url);
  return finalName;
}

export async function exportHtmlToPdf(
  htmlContent: string,
  options: Html2PdfWorkerOptions
): Promise<string> {
  const blob = await exportHtmlToPdfBlob(htmlContent, options);
  return deliverPdfBlob(blob, options.filename);
}

import { jsPDF } from 'jspdf';
import type { Sale } from '../services/api';
import {
  barcodeToDataUrl,
  computeReceiptTotals,
  getItemBarcode,
  paymentLabel,
} from './receipt';
import { TVA_FIXED_AMOUNT_FCFA, TVA_RATE_PERCENT } from './tva';
import { pdfToBlob } from './pdfExport';

export type TicketPdfMeta = {
  cashierName?: string;
  widthMm?: 58 | 80;
};

function tvaLabel(): string {
  if (TVA_FIXED_AMOUNT_FCFA > 0) {
    return `TVA (${TVA_RATE_PERCENT}% + ${TVA_FIXED_AMOUNT_FCFA} F)`;
  }
  return `TVA (${TVA_RATE_PERCENT}%)`;
}

function estimateTicketHeightMm(sale: Sale, widthMm: number): number {
  const barcodeCount = sale.items.filter((i) => getItemBarcode(i) !== 'N/A').length;
  const base = 52 + sale.items.length * 10 + barcodeCount * (widthMm === 58 ? 14 : 16);
  return Math.min(400, Math.max(90, base));
}

/** Ticket de caisse stylé avec codes-barres (jsPDF, sans html2canvas). */
export function buildSaleTicketPdf(sale: Sale, meta: TicketPdfMeta = {}): jsPDF {
  const widthMm = meta.widthMm === 58 ? 58 : 80;
  const heightMm = estimateTicketHeightMm(sale, widthMm);
  const pdf = new jsPDF({ unit: 'mm', format: [widthMm, heightMm], orientation: 'portrait' });
  const pageW = pdf.internal.pageSize.getWidth();
  const margin = 3;
  const contentW = pageW - margin * 2;
  let y = margin;

  const center = (text: string, size: number, bold = false) => {
    pdf.setFont('helvetica', bold ? 'bold' : 'normal');
    pdf.setFontSize(size);
    pdf.setTextColor(0, 0, 0);
    const tw = pdf.getTextWidth(text);
    pdf.text(text, (pageW - tw) / 2, y);
    y += size * 0.45 + 1;
  };

  const row = (left: string, right: string, bold = false) => {
    pdf.setFont('helvetica', bold ? 'bold' : 'normal');
    pdf.setFontSize(8);
    const rightW = pdf.getTextWidth(right);
    const maxLeft = contentW - rightW - 1;
    const leftLines = pdf.splitTextToSize(left, maxLeft) as string[];
    pdf.text(leftLines[0] ?? left, margin, y);
    pdf.text(right, margin + contentW - rightW, y);
    y += 3.8;
  };

  const dashed = () => {
    pdf.setDrawColor(80);
    pdf.setLineWidth(0.15);
    pdf.setLineDashPattern([1.5, 1.5], 0);
    pdf.line(margin, y, margin + contentW, y);
    pdf.setLineDashPattern([], 0);
    y += 2.5;
  };

  center('AZTEA PHARMACY & POS', 11, true);
  center('Brazzaville, Congo', 7);
  center('Tel: +242 05 656 0299', 7);
  y += 1;
  dashed();

  pdf.setFont('helvetica', 'normal');
  pdf.setFontSize(7.5);
  pdf.text(`Ticket: ${sale.receipt_number}`, margin, y);
  y += 3.2;
  pdf.text(`Date: ${new Date(sale.sold_at).toLocaleString('fr-FR')}`, margin, y);
  y += 3.2;
  if (meta.cashierName) {
    pdf.text(`Caissier: ${meta.cashierName}`, margin, y);
    y += 3.2;
  }
  if (sale.customer_name) {
    pdf.text(`Client: ${sale.customer_name}`, margin, y);
    y += 3.2;
  }
  if (sale.notes) {
    try {
      const parsed = JSON.parse(sale.notes);
      if (parsed.phone) {
        pdf.text(`Tel: ${parsed.phone}`, margin, y);
        y += 3.2;
      }
    } catch {
      /* ignore */
    }
  }
  dashed();

  for (const item of sale.items) {
    row(item.product_name || 'Article', `${item.quantity} x ${item.unit_price} F`, true);
    const code = getItemBarcode(item);
    const dataUrl = barcodeToDataUrl(code, widthMm === 58);
    if (dataUrl) {
      const imgW = widthMm === 58 ? 44 : 52;
      const imgH = widthMm === 58 ? 11 : 13;
      const imgX = margin + (contentW - imgW) / 2;
      pdf.addImage(dataUrl, 'PNG', imgX, y, imgW, imgH);
      y += imgH + 2;
    } else if (code !== 'N/A') {
      pdf.setFontSize(6.5);
      pdf.text(`Code: ${code}`, margin, y);
      y += 3;
    }
    y += 1;
  }

  dashed();

  const t = computeReceiptTotals(sale);
  row('Sous-total', `${t.subtotal} F`);
  if (t.discount > 0) row('Remise', `-${t.discount} F`);
  row('Montant HT', `${t.ht} F`);
  pdf.setFont('helvetica', 'bold');
  pdf.setFontSize(7);
  pdf.text('Taxes appliquées', margin, y);
  y += 3.5;
  pdf.setFont('helvetica', 'normal');
  if (t.articleTaxes > 0) row('  Taxes articles', `${t.articleTaxes} F`);
  row(`  ${tvaLabel()}`, `${t.tva} F`);
  row('Total taxes', `${t.totalTaxes} F`, true);
  dashed();
  row('NET A PAYER', `${t.netAPayer} F`, true);

  if (sale.payment_method === 'cash') {
    const paid = sale.amount_paid > 0 ? sale.amount_paid : t.netAPayer;
    row('Montant reçu', `${paid} F`);
    row('Monnaie rendue', `${sale.change_given} F`);
  }

  dashed();
  pdf.setFont('helvetica', 'bold');
  pdf.setFontSize(8);
  center(`Mode: ${paymentLabel(sale.payment_method)}`, 8, true);
  y += 2;
  dashed();
  center('*** MERCI DE VOTRE VISITE ***', 9, true);

  return pdf;
}

export async function exportSaleTicketPdfBlob(sale: Sale, meta: TicketPdfMeta = {}): Promise<Blob> {
  const cfgWidth = meta.widthMm === 58 ? 58 : 80;
  const pdf = buildSaleTicketPdf(sale, { ...meta, widthMm: cfgWidth });
  const blob = pdfToBlob(pdf);
  if (blob.size < 100) throw new Error('Ticket PDF vide.');
  return blob;
}

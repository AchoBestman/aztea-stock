import { jsPDF } from 'jspdf';
import { api } from '../services/api';
import type { Sale } from '../services/api';
import {
  getReportPrinterConfig,
  getTicketCharWidth,
  getTicketPrinterConfig,
  isTauriApp,
  requireReportPrinter,
  requireTicketPrinter,
} from './hardwareConfig';
import { deliverPdfBlob, exportHtmlToPdfBlob, pdfToBlob } from './pdfExport';
import {
  barcodeToDataUrl,
  computeReceiptTotals,
  paymentLabel,
} from './receipt';

async function printPdfBlobToPrinter(printerName: string, blob: Blob, filename: string): Promise<void> {
  const buffer = await blob.arrayBuffer();
  const bytes = new Uint8Array(buffer);
  let binary = '';
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  const base64 = btoa(binary);

  const { invoke } = await import('@tauri-apps/api/core');
  await invoke<string>('print_pdf_base64', {
    printerName,
    pdfBase64: base64,
    filename,
  });
}

export async function printTicketFromHtml(htmlContent: string, filename: string): Promise<string | void> {
  const configErr = requireTicketPrinter();
  if (configErr) throw new Error(configErr);

  const cfg = getTicketPrinterConfig();
  const pw = cfg.widthMm;

  if (cfg.isPdf) {
    const blob = await exportHtmlToPdfBlob(htmlContent, {
      margin: 0,
      filename,
      image: { type: 'jpeg', quality: 0.98 },
      jsPDF: { unit: 'mm', format: [pw, 200], orientation: 'portrait' },
    });
    return deliverPdfBlob(blob, filename);
  }

  if (isTauriApp()) {
    const blob = await exportHtmlToPdfBlob(htmlContent, {
      margin: 0,
      filename,
      image: { type: 'jpeg', quality: 0.98 },
      jsPDF: { unit: 'mm', format: [pw, 200], orientation: 'portrait' },
    });
    await printPdfBlobToPrinter(cfg.printerName, blob, filename);
    return;
  }

  const html = `<!DOCTYPE html><html><head><meta charset="utf-8"><title>Ticket</title></head><body style="margin:0">${htmlContent}</body></html>`;
  const iframe = document.createElement('iframe');
  iframe.style.cssText = 'position:fixed;width:0;height:0;border:none;opacity:0;pointer-events:none;';
  document.body.appendChild(iframe);
  const doc = iframe.contentWindow?.document;
  if (doc) {
    doc.open();
    doc.write(html);
    doc.close();
    iframe.onload = () => {
      setTimeout(() => {
        iframe.contentWindow?.focus();
        iframe.contentWindow?.print();
        setTimeout(() => document.body.removeChild(iframe), 1000);
      }, 500);
    };
  }
}

export async function printTicketText(content: string): Promise<void> {
  const configErr = requireTicketPrinter();
  if (configErr) throw new Error(configErr);

  const cfg = getTicketPrinterConfig();

  if (cfg.isPdf) {
    const pw = cfg.widthMm;
    const html = `<pre style="font-family:'Courier New',monospace;font-size:${pw === 58 ? '10px' : '12px'};white-space:pre-wrap;margin:0;padding:3mm">${content.replace(/</g, '&lt;')}</pre>`;
    const blob = await exportHtmlToPdfBlob(html, {
      margin: 0,
      filename: `ticket_${Date.now()}.pdf`,
      jsPDF: { unit: 'mm', format: [pw, 200], orientation: 'portrait' },
    });
    await deliverPdfBlob(blob, `ticket_${Date.now()}.pdf`);
    return;
  }

  if (isTauriApp()) {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('print_receipt', { printerName: cfg.printerName, content });
    return;
  }

  throw new Error(
    "Impression texte sur imprimante physique : utilisez l'application desktop (Tauri) ou choisissez « Enregistrer au format PDF »."
  );
}

export function getTicketLayout() {
  const cfg = getTicketPrinterConfig();
  return {
    widthMm: cfg.widthMm,
    charWidth: getTicketCharWidth(cfg.widthMm),
    printerLabel: cfg.printerName,
    isPdf: cfg.isPdf,
  };
}

export async function printTicketFromSale(
  sale: Sale,
  filename: string,
  options?: { cashierName?: string }
): Promise<string | void> {
  const fmtF = (v: number) => {
    const s = Math.round(v).toString();
    return s.replace(/\B(?=(\d{3})+(?!\d))/g, ' ') + ' F';
  };
  const fmtDate = (d: string) =>
    new Date(d).toLocaleString('fr-FR', { dateStyle: 'short', timeStyle: 'short' });
  const pmLabel = paymentLabel(sale.payment_method);

  // Parse client info from notes JSON
  let parsedClient: { full_name?: string; phone?: string; email?: string } | null = null;
  if (sale.notes) {
    try {
      const parsed = JSON.parse(sale.notes);
      if (parsed && typeof parsed === 'object' && parsed.full_name) {
        parsedClient = parsed;
      }
    } catch (_) {}
  }

  // Fetch tenant info (name, address, phone, email)
  let tenantName = '';
  let tenantAddress = '';
  let tenantPhone = '';
  let tenantEmail = '';
  try {
    const tenant = await api.tenants.get();
    tenantName = tenant.name || '';
    tenantAddress = [tenant.address, tenant.country].filter(Boolean).join(', ');
    tenantPhone = tenant.phone || '';
    tenantEmail = tenant.email || '';
  } catch (_) {
    try {
      const storedUser = localStorage.getItem('aztea_user');
      if (storedUser) {
        const parsed = JSON.parse(storedUser);
        tenantName = parsed.tenant_name || '';
      }
    } catch (_) {}
  }

  const t = computeReceiptTotals(sale);
  const cfg = getTicketPrinterConfig();
  const widthMm = cfg.widthMm;
  const layout = getTicketLayout();
  const narrow = widthMm <= 58;

  // Build PDF directly with jsPDF (HTML is stripped to plain text by pdfExport)
  const margin = 3;
  const pageW = widthMm - margin * 2;
  const centerX = margin + pageW / 2;
  const rightX = margin + pageW;

  const pdf = new jsPDF({ unit: 'mm', format: [widthMm, 280], orientation: 'portrait' });
  let y = margin + 5;

  const lh = (fs: number) => fs * 0.38 + 1.8;

  const sep = () => {
    y += 2;
    pdf.setDrawColor(160, 160, 160);
    pdf.setLineDashPattern([0.7, 0.7], 0);
    pdf.line(margin, y, rightX, y);
    pdf.setLineDashPattern([], 0);
    pdf.setDrawColor(0, 0, 0);
    y += 4;
  };

  const ctr = (text: string, fs: number, bold = false) => {
    pdf.setFont('helvetica', bold ? 'bold' : 'normal');
    pdf.setFontSize(fs);
    pdf.setTextColor(0, 0, 0);
    pdf.text(text, centerX, y, { align: 'center' });
    y += lh(fs);
  };

  const lft = (text: string, fs: number, bold = false) => {
    pdf.setFont('helvetica', bold ? 'bold' : 'normal');
    pdf.setFontSize(fs);
    pdf.setTextColor(0, 0, 0);
    pdf.text(text, margin, y);
    y += lh(fs);
  };

  const lr = (left: string, right: string, fs: number, bold = false, red = false) => {
    pdf.setFont('helvetica', bold ? 'bold' : 'normal');
    pdf.setFontSize(fs);
    pdf.setTextColor(red ? 190 : 0, 0, 0);
    pdf.text(left, margin, y);
    pdf.text(right, rightX, y, { align: 'right' });
    pdf.setTextColor(0, 0, 0);
    y += lh(fs);
  };

  // ── HEADER ──
  ctr(tenantName.toUpperCase(), narrow ? 11 : 13, true);
  if (tenantAddress) ctr(tenantAddress, 8);
  if (tenantPhone)   ctr('Tel: ' + tenantPhone, 8);
  if (tenantEmail)   ctr(tenantEmail, 8);

  sep();

  // ── TICKET INFO ──
  const infoLines = [
    'Ticket: ' + (sale.receipt_number || ''),
    'Date: ' + fmtDate(sale.sold_at),
    'Client: ' + (parsedClient?.full_name || sale.customer_name || 'Passage'),
    ...(options?.cashierName ? ['Caissier: ' + options.cashierName] : []),
    'Periph. : ' + (layout.printerLabel || ''),
  ];
  const fs8 = narrow ? 7.5 : 8.5;
  for (const line of infoLines) lft(line, fs8);

  sep();

  // ── ARTICLES ──
  const fs9 = narrow ? 8 : 9;
  for (const item of sale.items) {
    const qty = item.quantity + ' x ' + fmtF(item.unit_price);
    pdf.setFont('helvetica', 'bold');
    pdf.setFontSize(fs9);
    pdf.setTextColor(0, 0, 0);
    // Truncate name if too long
    const maxNameW = pageW - pdf.getTextWidth(qty) - 2;
    let name = item.product_name || '';
    while (name.length > 3 && pdf.getTextWidth(name) > maxNameW) {
      name = name.slice(0, -1);
    }
    pdf.text(name, margin, y);
    pdf.setFont('helvetica', 'normal');
    pdf.text(qty, rightX, y, { align: 'right' });
    y += lh(fs9);
  }

  sep();

  // ── TOTAUX ──
  const fsT = narrow ? 8 : 8.5;
  lr('Sous-total:', fmtF(t.subtotal), fsT);
  if (t.discount > 0) lr('Remise:', '-' + fmtF(t.discount), fsT, false, true);
  lr('Montant HT:', fmtF(t.ht), fsT);
  y += 0.5;
  lft('Taxes appliquees', fsT, true);
  lr('  TVA:', fmtF(t.tva), fsT);
  if (t.articleTaxes > 0) lr('  Autres taxes:', fmtF(t.articleTaxes), fsT);
  lr('Total taxes:', fmtF(t.totalTaxes), fsT, true);
  y += 2;
  lr('NET A PAYER:', fmtF(t.netAPayer), narrow ? 9.5 : 10.5, true);
  if (sale.payment_method === 'cash') {
    lr('Montant recu:', fmtF(sale.amount_paid || t.netAPayer), fsT);
    lr('Monnaie rendue:', fmtF(sale.change_given), fsT);
  }
  y += 1;
  lft('Mode: ' + pmLabel, fsT);

  sep();

  // ── MERCI ──
  y += 1;
  ctr('*** MERCI DE VOTRE VISITE ***', narrow ? 8.5 : 9, true);
  y += 4;

  // ── CODE-BARRES (après merci) — numéro de ticket uniquement ──
  const receiptCode = sale.receipt_number?.trim();
  if (receiptCode) {
    const dataUrl = barcodeToDataUrl(receiptCode, narrow);
    if (dataUrl) {
      const bw = narrow ? 34 : 46;
      const bh = narrow ? 14 : 18;
      const bx = margin + (pageW - bw) / 2;
      pdf.addImage(dataUrl, 'PNG', bx, y, bw, bh);
      y += bh + 3;
    }
  }

  y += 2;

  // ── MENTION LÉGALE ──
  pdf.setFont('helvetica', 'normal');
  pdf.setFontSize(7);
  pdf.setTextColor(80, 80, 80);
  const noRefund = "Aucun remboursement n'est permis apres encaissement.";
  const wrappedLines = pdf.splitTextToSize(noRefund, pageW) as string[];
  for (const line of wrappedLines) {
    pdf.text(line, centerX, y, { align: 'center' });
    y += 3.5;
  }
  pdf.setTextColor(0, 0, 0);

  // ── DELIVER ──
  const blob = pdfToBlob(pdf);

  if (cfg.isPdf) {
    return deliverPdfBlob(blob, filename);
  }

  if (isTauriApp()) {
    await printPdfBlobToPrinter(cfg.printerName, blob, filename);
    return;
  }

  // Browser fallback: iframe print
  const blobUrl = URL.createObjectURL(blob);
  const iframe = document.createElement('iframe');
  iframe.style.cssText = 'position:fixed;width:0;height:0;border:none;opacity:0;pointer-events:none;';
  iframe.src = blobUrl;
  document.body.appendChild(iframe);
  iframe.onload = () => {
    setTimeout(() => {
      iframe.contentWindow?.focus();
      iframe.contentWindow?.print();
      setTimeout(() => {
        document.body.removeChild(iframe);
        URL.revokeObjectURL(blobUrl);
      }, 1000);
    }, 500);
  };
}

export async function printReportHtml(
  htmlContent: string,
  filename: string
): Promise<{ mode: 'pdf' | 'printer' | 'browser'; savedPath?: string }> {
  const configErr = requireReportPrinter();
  if (configErr) throw new Error(configErr);

  const cfg = getReportPrinterConfig();
  const pdfOptions = {
    margin: 10,
    filename,
    image: { type: 'jpeg' as const, quality: 0.98 },
    jsPDF: { unit: 'mm' as const, format: cfg.jsPdfFormat, orientation: 'portrait' as const },
  };

  if (cfg.isPdf) {
    const blob = await exportHtmlToPdfBlob(htmlContent, pdfOptions);
    const savedPath = await deliverPdfBlob(blob, filename);
    return { mode: 'pdf', savedPath };
  }

  if (isTauriApp()) {
    const blob = await exportHtmlToPdfBlob(htmlContent, pdfOptions);
    await printPdfBlobToPrinter(cfg.printerName, blob, filename);
    return { mode: 'printer' };
  }

  return { mode: 'browser' };
}

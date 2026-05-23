import JsBarcode from 'jsbarcode';
import type { Sale, SaleItem } from '../services/api';
import {
  computeReceiptTotals,
  TVA_FIXED_AMOUNT_FCFA,
  TVA_RATE_PERCENT,
} from './tva';

export function escapeHtml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

export function getItemBarcode(item: SaleItem): string {
  const code = item.product_barcode?.trim();
  if (code) return code;
  return 'N/A';
}

export function paymentLabel(method: string): string {
  if (method === 'cash') return 'Espèces';
  if (method === 'mobile_money') return 'Mobile Money';
  if (method === 'card') return 'Carte';
  return method;
}

function tvaLabel(): string {
  if (TVA_FIXED_AMOUNT_FCFA > 0) {
    return `TVA (${TVA_RATE_PERCENT}% + ${TVA_FIXED_AMOUNT_FCFA} F)`;
  }
  return `TVA (${TVA_RATE_PERCENT}%)`;
}

/** Image PNG du code-barres pour PDF / impression (jsPDF). */
export function barcodeToDataUrl(code: string, narrowTicket = false): string | null {
  const trimmed = code.trim();
  if (!trimmed || trimmed === 'N/A') return null;
  try {
    const canvas = document.createElement('canvas');
    JsBarcode(canvas, trimmed, {
      format: 'CODE128',
      width: narrowTicket ? 1.4 : 1.8,
      height: narrowTicket ? 36 : 44,
      displayValue: true,
      fontSize: narrowTicket ? 13 : 14,
      fontOptions: 'bold',
      margin: 6,
      textMargin: 4,
    });
    return canvas.toDataURL('image/png');
  } catch {
    return null;
  }
}

/** Génère un code-barres SVG scannable (CODE128) ou retourne une ligne texte de secours. */
export function renderBarcodeSvg(code: string, height = 32): string {
  const trimmed = code.trim();
  if (!trimmed || trimmed === 'N/A') {
    return '';
  }
  try {
    const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    JsBarcode(svg, trimmed, {
      format: 'CODE128',
      width: 1.2,
      height,
      displayValue: true,
      fontSize: 13,
      fontOptions: 'bold',
      margin: 2,
      textMargin: 2,
    });
    return (
      `<div style="margin-top:6px;width:100%;text-align:center;display:flex;flex-direction:column;align-items:center;justify-content:center">` +
      `${svg.outerHTML}</div>`
    );
  } catch {
    return `<div style="font-size:9px;margin-top:4px;text-align:center;font-family:monospace;color:#000">Code: ${escapeHtml(trimmed)}</div>`;
  }
}

/** Bloc totaux : toutes les taxes visibles, puis net à payer = HT + taxes. */
export function buildReceiptSummaryHtml(sale: Sale): string {
  const t = computeReceiptTotals(sale);
  const lines: string[] = [
    `<div style="display:flex;justify-content:space-between"><span>Sous-total:</span><span>${t.subtotal} F</span></div>`,
  ];

  if (t.discount > 0) {
    lines.push(
      `<div style="display:flex;justify-content:space-between"><span>Remise:</span><span>-${t.discount} F</span></div>`
    );
  }

  lines.push(
    `<div style="display:flex;justify-content:space-between"><span>Montant HT:</span><span>${t.ht} F</span></div>`,
    `<div style="border-top:1px dotted #000;margin:4px 0"></div>`,
    `<div style="font-weight:bold;margin-bottom:2px">Taxes appliquées</div>`
  );

  if (t.articleTaxes > 0) {
    lines.push(
      `<div style="display:flex;justify-content:space-between;padding-left:4px"><span>Taxes articles:</span><span>${t.articleTaxes} F</span></div>`
    );
  }

  lines.push(
    `<div style="display:flex;justify-content:space-between;padding-left:4px"><span>${tvaLabel()}:</span><span>${t.tva} F</span></div>`,
    `<div style="display:flex;justify-content:space-between;font-weight:bold;margin-top:2px"><span>Total taxes:</span><span>${t.totalTaxes} F</span></div>`,
    `<div style="border-top:1px dashed #000;margin:3px 0"></div>`,
    `<div style="display:flex;justify-content:space-between;font-weight:bold;font-size:1.05em"><span>NET A PAYER:</span><span>${t.netAPayer} F</span></div>`
  );

  if (sale.payment_method === 'cash') {
    const paid = sale.amount_paid > 0 ? sale.amount_paid : t.netAPayer;
    lines.push(
      `<div style="display:flex;justify-content:space-between"><span>Montant reçu:</span><span>${paid} F</span></div>`,
      `<div style="display:flex;justify-content:space-between"><span>Monnaie rendue:</span><span>${sale.change_given} F</span></div>`
    );
  }

  return lines.join('');
}

export function buildReceiptItemsHtml(sale: Sale): string {
  return sale.items
    .map((item) => {
      const code = getItemBarcode(item);
      const barcodeBlock = renderBarcodeSvg(code);
      return (
        `<div style="margin-bottom:10px;padding-bottom:6px;border-bottom:1px dotted #ccc">` +
        `<div style="display:flex;justify-content:space-between;gap:8px;font-weight:bold">` +
        `<span>${escapeHtml(item.product_name || '')}</span>` +
        `<span style="white-space:nowrap">${item.quantity}x${item.unit_price}F</span>` +
        `</div>` +
        (barcodeBlock ? barcodeBlock : '') +
        `</div>`
      );
    })
    .join('');
}

export function buildReceiptItemsText(
  sale: Sale,
  line: (left: string, right: string) => string,
  maxNameWidth: number
): string[] {
  const lines: string[] = [];
  sale.items.forEach((item) => {
    const name = (item.product_name || '').substring(0, maxNameWidth);
    const code = getItemBarcode(item);
    lines.push(line(name, `${item.quantity}x${item.unit_price}F`));
    if (code !== 'N/A') lines.push(`  Code: ${code}`);
  });
  return lines;
}

export { computeReceiptTotals } from './tva';

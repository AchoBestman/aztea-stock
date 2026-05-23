/**
 * Calcul TVA provisoire pour les reçus.
 * Modifier uniquement computeTva() (et les constantes) quand le calcul métier réel sera prêt.
 */

/** Taux en pourcentage appliqué sur le montant taxable. */
export const TVA_RATE_PERCENT = 2;

/** Montant fixe ajouté (FCFA), en plus du pourcentage. */
export const TVA_FIXED_AMOUNT_FCFA = 20;

/**
 * TVA = (montant × taux %) + montant fixe
 */
export function computeTva(taxableAmount: number): number {
  const base = Math.max(0, taxableAmount);
  const percentPart = (base * TVA_RATE_PERCENT) / 100;
  return Math.round(percentPart + TVA_FIXED_AMOUNT_FCFA);
}

/** Montant HT (sous-total − remise). */
export function computeTaxableAmount(subtotal: number, discountTotal: number): number {
  return Math.max(0, subtotal - discountTotal);
}

export type ReceiptTotals = {
  subtotal: number;
  discount: number;
  ht: number;
  articleTaxes: number;
  tva: number;
  totalTaxes: number;
  netAPayer: number;
};

/** Détail des montants du ticket : toutes les taxes sont explicites avant le net à payer. */
export function computeReceiptTotals(sale: {
  subtotal: number;
  discount_total: number;
  tax_total: number;
}): ReceiptTotals {
  const discount = sale.discount_total ?? 0;
  const ht = computeTaxableAmount(sale.subtotal, discount);
  const articleTaxes = sale.tax_total ?? 0;
  const tva = computeTva(ht);
  const totalTaxes = articleTaxes + tva;
  const netAPayer = ht + totalTaxes;
  return {
    subtotal: sale.subtotal,
    discount,
    ht,
    articleTaxes,
    tva,
    totalTaxes,
    netAPayer,
  };
}

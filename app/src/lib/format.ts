/** Libellé affiché pour le type d'activité du tenant (sidebar, etc.). */
export function getBusinessTypeLabel(businessType: string): string {
  switch (businessType) {
    case "pharmacy":
      return "Pharmacie";
    case "supermarket":
      return "Supermarché";
    case "both":
      return "Pharmacie & Supermarché";
    default:
      return businessType;
  }
}

/** Libellé pour « votre … » selon le type d'activité du tenant. */
export function getBusinessEstablishmentLabel(businessType: string): string {
  switch (businessType) {
    case "pharmacy":
      return "officine";
    case "supermarket":
      return "magasin";
    case "both":
      return "pharmacie et magasin";
    default:
      return "établissement";
  }
}

export function getDashboardPerformanceSubtitle(businessType: string): string {
  if (businessType === "both") {
    return "Voici les performances de votre pharmacie et de votre magasin sur la période sélectionnée.";
  }
  const establishment = getBusinessEstablishmentLabel(businessType);
  return `Voici les performances de votre ${establishment} sur la période sélectionnée.`;
}

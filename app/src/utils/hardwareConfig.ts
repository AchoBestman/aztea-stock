/** Option virtuelle : enregistrement PDF dans Téléchargements */
export const PDF_PRINTER_OPTION = 'Enregistrer au format PDF';

export const STORAGE_KEYS = {
  TICKET_PRINTER: 'aztea_ticket_printer',
  TICKET_WIDTH: 'aztea_ticket_printer_width',
  REPORT_PRINTER: 'aztea_report_printer',
  REPORT_FORMAT: 'aztea_report_printer_format',
  SCANNER: 'aztea_default_scanner',
  /** @deprecated Migré vers TICKET_PRINTER */
  LEGACY_PRINTER: 'aztea_default_printer',
  LEGACY_WIDTH: 'aztea_printer_width',
} as const;

export const TICKET_WIDTH_OPTIONS = [
  { value: '80', label: 'Standard 80 mm (recommandé)' },
  { value: '58', label: 'Compact 58 mm' },
] as const;

/** Formats papier pour rapports / documents */
export const REPORT_FORMAT_OPTIONS = [
  { value: 'a4', label: 'A4 (210 × 297 mm)', jsPdf: 'a4' },
  { value: 'a3', label: 'A3 (297 × 420 mm)', jsPdf: 'a3' },
  { value: 'a2', label: 'A2 (420 × 594 mm)', jsPdf: 'a2' },
  { value: 'a1', label: 'A1 (594 × 841 mm)', jsPdf: 'a1' },
  { value: 'a0', label: 'A0 (841 × 1189 mm)', jsPdf: 'a0' },
  { value: 'letter', label: 'Letter US (216 × 279 mm)', jsPdf: 'letter' },
  { value: 'legal', label: 'Legal US (216 × 356 mm)', jsPdf: 'legal' },
  { value: 'tabloid', label: 'Tabloid (279 × 432 mm)', jsPdf: 'tabloid' },
  { value: 'b4', label: 'B4 (250 × 353 mm)', jsPdf: 'b4' },
  { value: 'b3', label: 'B3 (353 × 500 mm)', jsPdf: 'b3' },
  { value: 'b2', label: 'B2 (500 × 707 mm)', jsPdf: 'b2' },
  { value: 'b1', label: 'B1 (707 × 1000 mm)', jsPdf: 'b1' },
  { value: 'b0', label: 'B0 (1000 × 1414 mm)', jsPdf: 'b0' },
] as const;

export type ReportFormatId = (typeof REPORT_FORMAT_OPTIONS)[number]['value'];

export function isTauriApp(): boolean {
  return typeof window !== 'undefined' && (window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ !== undefined;
}

export function isPdfOutput(printerName: string): boolean {
  if (!printerName.trim()) return false;
  const lower = printerName.toLowerCase();
  return lower.includes('pdf') || printerName === PDF_PRINTER_OPTION;
}

export function migrateLegacyPrinterConfig(): void {
  if (!localStorage.getItem(STORAGE_KEYS.TICKET_PRINTER)) {
    const legacy = localStorage.getItem(STORAGE_KEYS.LEGACY_PRINTER);
    if (legacy) localStorage.setItem(STORAGE_KEYS.TICKET_PRINTER, legacy);
  }
  if (!localStorage.getItem(STORAGE_KEYS.TICKET_WIDTH)) {
    const legacyW = localStorage.getItem(STORAGE_KEYS.LEGACY_WIDTH);
    if (legacyW) localStorage.setItem(STORAGE_KEYS.TICKET_WIDTH, legacyW);
  }
}

export function getTicketPrinterConfig() {
  migrateLegacyPrinterConfig();
  const printerName = localStorage.getItem(STORAGE_KEYS.TICKET_PRINTER) || '';
  const widthMm = parseInt(localStorage.getItem(STORAGE_KEYS.TICKET_WIDTH) || '80', 10);
  return {
    printerName,
    widthMm: widthMm === 58 ? 58 : 80,
    isPdf: isPdfOutput(printerName),
  };
}

export function getReportPrinterConfig() {
  const printerName = localStorage.getItem(STORAGE_KEYS.REPORT_PRINTER) || '';
  const format = (localStorage.getItem(STORAGE_KEYS.REPORT_FORMAT) || 'a4') as ReportFormatId;
  const formatOpt = REPORT_FORMAT_OPTIONS.find((o) => o.value === format) ?? REPORT_FORMAT_OPTIONS[0];
  return {
    printerName,
    format,
    formatLabel: formatOpt.label,
    jsPdfFormat: formatOpt.jsPdf,
    isPdf: isPdfOutput(printerName),
  };
}

export function getScannerName(): string {
  return localStorage.getItem(STORAGE_KEYS.SCANNER) || '';
}

export function requireTicketPrinter(): string | null {
  const { printerName } = getTicketPrinterConfig();
  if (!printerName.trim()) {
    return "Veuillez configurer votre imprimante de ticket dans Paramètres → Périphériques.";
  }
  return null;
}

export function requireReportPrinter(): string | null {
  const { printerName } = getReportPrinterConfig();
  if (!printerName.trim()) {
    return "Veuillez configurer votre imprimante de rapport dans Paramètres → Périphériques.";
  }
  return null;
}

export function requireScanner(): string | null {
  if (!getScannerName().trim()) {
    return "Veuillez configurer votre scanner de code-barres dans Paramètres → Périphériques.";
  }
  return null;
}

export function getTicketCharWidth(widthMm: number): number {
  return widthMm === 58 ? 32 : 42;
}

export function withPdfPrinterOption<T extends { name: string }>(devices: T[]): T[] {
  if (devices.some((d) => d.name === PDF_PRINTER_OPTION)) return devices;
  return [{ name: PDF_PRINTER_OPTION, connected: true, is_default: false } as T, ...devices];
}

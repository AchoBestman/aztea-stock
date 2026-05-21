import { useState, useEffect } from 'react';
import {
  Search, Calendar, FileText, Download, Printer, Eye, X,
  ChevronLeft, ChevronRight, Filter, Receipt
} from 'lucide-react';
import { api, Sale, TenantResponse } from '../services/api';
import { useAuthStore } from '../store/authStore';
import toast from 'react-hot-toast';

export default function SalesHistory() {
  const { user } = useAuthStore();
  const isSystem = !!user && (user.role === 'Super Admin' || user.role === 'admin');

  // Permissions from localStorage user profile
  const storedUser = localStorage.getItem('aztea_user');
  const permissions: string[] = storedUser ? (JSON.parse(storedUser).permissions || []) : [];
  const canRead = permissions.includes('can_read_sale') || user?.role === 'Super Admin';
  const canExportPdf = permissions.includes('can_export_sale_pdf') || user?.role === 'Super Admin';
  const canExportExcel = permissions.includes('can_export_sale_excel') || user?.role === 'Super Admin';
  const canPrint = permissions.includes('can_print_sale_receipt') || user?.role === 'Super Admin';

  const [sales, setSales] = useState<Sale[]>([]);
  const [loading, setLoading] = useState(true);
  const [page, setPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const [total, setTotal] = useState(0);
  const perPage = 20;

  // Filters
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState('');
  const [startDate, setStartDate] = useState('');
  const [endDate, setEndDate] = useState('');

  // Cross-tenant (system only)
  const [tenants, setTenants] = useState<TenantResponse[]>([]);
  const [selectedTenant, setSelectedTenant] = useState('');

  // Detail modal
  const [detailSale, setDetailSale] = useState<Sale | null>(null);

  useEffect(() => {
    if (isSystem) {
      api.admin.tenants.list().then(setTenants).catch(() => {});
    }
  }, [isSystem]);

  const loadSales = async () => {
    if (!canRead) return;
    setLoading(true);
    try {
      const res = await api.sales.list(search, statusFilter, page, perPage, startDate, endDate, selectedTenant);
      setSales(res.data || []);
      setTotalPages(res.total_pages || 1);
      setTotal(res.total || 0);
    } catch (e: any) {
      toast.error(e.message || 'Erreur lors du chargement');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { loadSales(); }, [page, statusFilter, startDate, endDate, selectedTenant]);

  // Debounced search: reloads 500ms after user stops typing
  useEffect(() => {
    const timer = setTimeout(() => {
      setPage(1);
      loadSales();
    }, 500);
    return () => clearTimeout(timer);
  }, [search]);

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    setPage(1);
    loadSales();
  };

  const fmt = (val: number) =>
    new Intl.NumberFormat('fr-FR', { style: 'currency', currency: 'XAF', minimumFractionDigits: 0 }).format(val).replace('FCFA', 'F');

  const fmtDate = (d: string) =>
    new Date(d).toLocaleString('fr-FR', { dateStyle: 'short', timeStyle: 'short' });

  const statusLabel = (s: string) => {
    if (s === 'completed') return { text: 'Validé', cls: 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400' };
    if (s === 'voided') return { text: 'Annulé', cls: 'bg-rose-500/10 text-rose-600 dark:text-rose-400' };
    if (s === 'refunded') return { text: 'Remboursé', cls: 'bg-amber-500/10 text-amber-600 dark:text-amber-400' };
    return { text: s, cls: 'bg-muted text-muted-foreground' };
  };

  const paymentLabel = (m: string) =>
    m === 'cash' ? 'Espèces' : m === 'mobile_money' ? 'Mobile Money' : m === 'card' ? 'Carte' : m;

  // Export handlers
  const handleExportPdf = async () => {
    if (!canExportPdf) { toast.error('Permission insuffisante'); return; }
    try {
      const data = await api.sales.export('pdf', startDate, endDate, selectedTenant);
      const htmlContent = buildPrintableHtml(data);
      
      toast.success('Génération du rapport PDF en cours...');
      const loadHtml2Pdf = () => new Promise<any>((resolve) => {
        if ((window as any).html2pdf) return resolve((window as any).html2pdf);
        const script = document.createElement('script');
        script.src = 'https://cdnjs.cloudflare.com/ajax/libs/html2pdf.js/0.10.1/html2pdf.bundle.min.js';
        script.onload = () => resolve((window as any).html2pdf);
        document.head.appendChild(script);
      });

      const html2pdf = await loadHtml2Pdf();
      const element = document.createElement('div');
      element.innerHTML = htmlContent;
      
      const opt = {
        margin: 10,
        filename: `rapport_ventes_${new Date().toISOString().split('T')[0]}.pdf`,
        image: { type: 'jpeg', quality: 0.98 },
        html2canvas: { scale: 2 },
        jsPDF: { unit: 'mm', format: 'a4', orientation: 'portrait' }
      };
      
      html2pdf().set(opt).from(element).save();
    } catch (e: any) { toast.error(e.message || 'Erreur export PDF'); }
  };

  const handleExportCSV = async () => {
    if (!canExportExcel) { toast.error('Permission insuffisante'); return; }
    try {
      const data = await api.sales.export('csv', startDate, endDate, selectedTenant);
      const headers = ['N° Reçu', 'Date', 'Client', 'Sous-total', 'Taxes', 'Remise', 'Total', 'Paiement', 'Statut'];
      const rows = data.map(s => [
        s.receipt_number, fmtDate(s.sold_at), s.customer_name || 'Passage',
        s.subtotal, s.tax_total, s.discount_total, s.total,
        paymentLabel(s.payment_method), s.status
      ]);
      const csv = "data:text/csv;charset=utf-8," + [headers.join(';'), ...rows.map(r => r.join(';'))].join('\n');
      const link = document.createElement('a');
      link.setAttribute('href', encodeURI(csv));
      link.setAttribute('download', `historique_ventes_${new Date().toISOString().split('T')[0]}.csv`);
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      toast.success('Export CSV téléchargé');
    } catch (e: any) { toast.error(e.message || 'Erreur export CSV'); }
  };

  const handleExportXLS = async () => {
    if (!canExportExcel) { toast.error('Permission insuffisante'); return; }
    try {
      const data = await api.sales.export('excel', startDate, endDate, selectedTenant);
      const headers = ['N° Reçu', 'Date', 'Client', 'Sous-total', 'Taxes', 'Remise', 'Total', 'Paiement', 'Statut'];
      const rows = data.map(s => [
        s.receipt_number, fmtDate(s.sold_at), s.customer_name || 'Passage',
        s.subtotal, s.tax_total, s.discount_total, s.total,
        paymentLabel(s.payment_method), s.status
      ]);
      const tsv = [headers.join('\t'), ...rows.map(r => r.join('\t'))].join('\n');
      const blob = new Blob([tsv], { type: 'application/vnd.ms-excel' });
      const link = document.createElement('a');
      link.href = URL.createObjectURL(blob);
      link.download = `historique_ventes_${new Date().toISOString().split('T')[0]}.xls`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      toast.success('Export Excel téléchargé');
    } catch (e: any) { toast.error(e.message || 'Erreur export Excel'); }
  };

  const handlePrintReceipt = async (sale: Sale) => {
    if (!canPrint) { toast.error('Permission insuffisante pour imprimer'); return; }
    const printerName = localStorage.getItem('aztea_default_printer') || '';
    const isTauri = typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__ !== undefined;
    const isVirtualPdf = printerName.toLowerCase().includes('pdf');

    if (isTauri && !isVirtualPdf) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const w = parseInt(localStorage.getItem('aztea_printer_width') || '80') === 58 ? 32 : 42;
        const center = (s: string) => { const pad = Math.max(0, Math.floor((w - s.length) / 2)); return ' '.repeat(pad) + s; };
        const line = (l: string, r: string) => { const space = Math.max(1, w - l.length - r.length); return l + ' '.repeat(space) + r; };
        const sep = '-'.repeat(w);
        const lines = [center('AZTEA PHARMACY & POS'), center('Brazzaville, Congo'), sep,
          `Ticket: ${sale.receipt_number}`, `Date: ${fmtDate(sale.sold_at)}`, sep];
          
        if (sale.customer_name) lines.push(`Client: ${sale.customer_name}`);
        if (sale.notes) {
          try {
            const parsed = JSON.parse(sale.notes);
            if (parsed.phone) lines.push(`Tel: ${parsed.phone}`);
          } catch(e) {}
        }
        
        lines.push(sep, ...sale.items.map(i => line(i.product_name.substring(0, w - 16), `${i.quantity}x${i.unit_price}F`)),
          sep, line('Total:', `${sale.total} F`), sep, center('*** MERCI ***'), '', '');
          
        if (!printerName) {
           toast.error('Aucune imprimante configurée. Veuillez la définir dans les Paramètres.');
           return;
        }
        await invoke('print_receipt', { printerName, content: lines.join('\n') });
        toast.success('Reçu imprimé');
      } catch (e: any) { toast.error(e?.toString() || 'Erreur impression'); }
    } else {
      const pw = parseInt(localStorage.getItem('aztea_printer_width') || '80');
      
      let clientInfoHtml = '';
      if (sale.customer_name) clientInfoHtml += `<div>Client: ${sale.customer_name}</div>`;
      if (sale.notes) {
        try {
          const parsed = JSON.parse(sale.notes);
          if (parsed.phone) clientInfoHtml += `<div>Tel: ${parsed.phone}</div>`;
        } catch (e) {}
      }

      const htmlContent = `<div style="font-family:'Courier New',monospace;font-size:${pw===58?'10px':'12px'};width:${pw}mm;margin:0 auto;padding:3mm;color:#000">`
        + `<div style="text-align:center;font-weight:bold">AZTEA PHARMACY & POS</div>`
        + `<div style="text-align:center;font-size:9px">Brazzaville, Congo</div>`
        + `<div style="text-align:center;font-size:9px">Tel: +242 05 656 0299</div>`
        + `<div style="border-top:1px dashed #000;margin:3px 0"></div>`
        + `<div>Ticket: ${sale.receipt_number}</div>`
        + `<div>Date: ${fmtDate(sale.sold_at)}</div>`
        + clientInfoHtml
        + `<div style="border-top:1px dashed #000;margin:3px 0"></div>`
        + sale.items.map(it => `<div style="display:flex;justify-content:space-between"><span>${it.product_name}</span><span>${it.quantity}x${it.unit_price}F</span></div>`).join('')
        + `<div style="border-top:1px dashed #000;margin:3px 0"></div>`
        + `<div style="display:flex;justify-content:space-between;font-weight:bold"><span>Total:</span><span>${sale.total} F</span></div>`
        + `<div style="border-top:1px dashed #000;margin:3px 0"></div>`
        + `<div>Mode: ${paymentLabel(sale.payment_method)}</div>`
        + `<div style="border-top:1px dashed #000;margin:3px 0"></div>`
        + `<div style="text-align:center;font-weight:bold">*** MERCI DE VOTRE VISITE ***</div>`
        + `</div>`;

      if (isVirtualPdf) {
        toast.success('Génération du PDF en cours...');
        const loadHtml2Pdf = () => new Promise<any>((resolve) => {
          if ((window as any).html2pdf) return resolve((window as any).html2pdf);
          const script = document.createElement('script');
          script.src = 'https://cdnjs.cloudflare.com/ajax/libs/html2pdf.js/0.10.1/html2pdf.bundle.min.js';
          script.onload = () => resolve((window as any).html2pdf);
          document.head.appendChild(script);
        });

        try {
          const html2pdf = await loadHtml2Pdf();
          const element = document.createElement('div');
          element.innerHTML = htmlContent;
          
          const opt = {
            margin: 0,
            filename: `ticket_${sale.receipt_number}.pdf`,
            image: { type: 'jpeg', quality: 0.98 },
            html2canvas: { scale: 2 },
            jsPDF: { unit: 'mm', format: [pw, 200], orientation: 'portrait' }
          };
          
          html2pdf().set(opt).from(element).save();
          toast.success('Ticket PDF sauvegardé dans les téléchargements.');
        } catch (e) {
          toast.error('Erreur lors de la génération PDF.');
        }
      } else {
        const html = `<!DOCTYPE html><html><head><meta charset="utf-8"><title>Ticket</title></head><body style="margin:0">${htmlContent}</body></html>`;
        const iframe = document.createElement('iframe');
        iframe.style.position = 'fixed';
        iframe.style.width = '0';
        iframe.style.height = '0';
        iframe.style.border = 'none';
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
    }
  };

  const buildReceiptHtml = (sale: Sale) => {
    const pw = parseInt(localStorage.getItem('aztea_printer_width') || '80');
    return `<!DOCTYPE html><html><head><meta charset="utf-8"><title>Reçu</title>
<style>@page{size:${pw}mm auto;margin:0}body{font-family:'Courier New',monospace;font-size:${pw===58?'10px':'12px'};width:${pw}mm;margin:0 auto;padding:3mm;color:#000}
.c{text-align:center}.b{font-weight:bold}.d{border-top:1px dashed #000;margin:3px 0}.r{display:flex;justify-content:space-between}</style></head><body>
<div class="c b">AZTEA PHARMACY & POS</div><div class="c" style="font-size:9px">Brazzaville, Congo</div><div class="d"></div>
<div>Ticket: ${sale.receipt_number}</div><div>Date: ${fmtDate(sale.sold_at)}</div><div class="d"></div>
${sale.items.map(i => `<div class="r"><span>${i.product_name}</span><span>${i.quantity}x${i.unit_price}F</span></div>`).join('')}
<div class="d"></div><div class="r b"><span>Total:</span><span>${sale.total} F</span></div><div class="d"></div>
<div>Mode: ${paymentLabel(sale.payment_method)}</div><div class="d"></div>
<div class="c b">*** MERCI DE VOTRE VISITE ***</div>
</body></html>`;
  };

  const buildPrintableHtml = (data: Sale[]) => {
    const totalRevenue = data.filter(s => s.status === 'completed').reduce((a, s) => a + s.total, 0);
    return `<!DOCTYPE html><html><head><meta charset="utf-8"><title>Historique des Ventes</title>
<style>@page{margin:15mm}body{font-family:Arial,sans-serif;font-size:11px;color:#000}
h1{font-size:16px;margin-bottom:4px}table{width:100%;border-collapse:collapse;margin-top:10px}
th,td{border:1px solid #ccc;padding:5px 8px;text-align:left}th{background:#f5f5f5;font-weight:bold}
.r{text-align:right}.s{margin-top:12px;font-size:12px;font-weight:bold}</style></head><body>
<h1>Historique des Ventes — AzteaStock</h1>
<p style="font-size:10px;color:#666">Généré le ${new Date().toLocaleString('fr-FR')} | ${data.length} ventes</p>
<table><thead><tr><th>N° Reçu</th><th>Date</th><th>Client</th><th>Articles</th><th class="r">Total</th><th>Paiement</th><th>Statut</th></tr></thead><tbody>
${data.map(s => `<tr><td>${s.receipt_number}</td><td>${fmtDate(s.sold_at)}</td><td>${s.customer_name || 'Passage'}</td>
<td>${s.items.length}</td><td class="r">${fmt(s.total)}</td><td>${paymentLabel(s.payment_method)}</td><td>${statusLabel(s.status).text}</td></tr>`).join('')}
</tbody></table><div class="s">Chiffre d'Affaires Total (complétées): ${fmt(totalRevenue)}</div>
</body></html>`;
  };

  if (!canRead) {
    return (
      <div className="flex flex-col items-center justify-center h-96 text-muted-foreground animate-slide-up">
        <Receipt className="w-16 h-16 mb-4 opacity-20" />
        <p className="font-bold text-lg">Accès Refusé</p>
        <p className="text-sm mt-1">Vous n'avez pas la permission de consulter l'historique des ventes.</p>
      </div>
    );
  }

  return (
    <div className="space-y-6 animate-slide-up select-none">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold text-foreground flex items-center gap-2">
            <Receipt className="w-6 h-6 text-primary" />
            Historique des Ventes
          </h1>
          <p className="text-xs text-muted-foreground font-semibold mt-0.5">
            {total} vente{total > 1 ? 's' : ''} trouvée{total > 1 ? 's' : ''}
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          {canExportPdf && (
            <button onClick={handleExportPdf}
              className="flex items-center gap-1.5 px-4 py-2 rounded-xl bg-primary text-primary-foreground text-xs font-bold hover:opacity-90 transition-all shadow-md cursor-pointer">
              <FileText className="w-4 h-4" /><span>Export PDF</span>
            </button>
          )}
          {canExportExcel && (
            <div className="flex gap-2">
              <button onClick={handleExportCSV}
                className="flex items-center gap-1.5 px-4 py-2 rounded-xl bg-secondary text-foreground text-xs font-bold hover:opacity-90 transition-all shadow-md cursor-pointer border border-border">
                <Download className="w-4 h-4" /><span>Export CSV</span>
              </button>
              <button onClick={handleExportXLS}
                className="flex items-center gap-1.5 px-4 py-2 rounded-xl bg-emerald-600 text-white text-xs font-bold hover:opacity-90 transition-all shadow-md cursor-pointer">
                <Download className="w-4 h-4" /><span>Export Excel</span>
              </button>
            </div>
          )}
        </div>
      </div>

      {/* Filters */}
      <div className="p-4 bg-card border border-border rounded-2xl shadow-sm flex flex-wrap items-end gap-3">
        <form onSubmit={handleSearch} className="relative flex-1 min-w-[180px]">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <input type="text" placeholder="Rechercher client..." value={search}
            onChange={e => setSearch(e.target.value)}
            className="w-full pl-9 pr-3 py-2 bg-accent/20 border border-border rounded-xl text-xs font-semibold focus:outline-none focus:ring-1 focus:ring-primary" />
        </form>

        <div className="flex items-center gap-2 h-[34px]">
          <Filter className="w-4 h-4 text-muted-foreground shrink-0" />
          <select value={statusFilter} onChange={e => { setStatusFilter(e.target.value); setPage(1); }}
            className="h-full px-3 bg-accent/20 border border-border rounded-xl text-xs font-bold focus:outline-none min-w-[130px]">
            <option value="">Tous statuts</option>
            <option value="completed">Validé</option>
            <option value="voided">Annulé</option>
            <option value="refunded">Remboursé</option>
          </select>
        </div>

        <div className="flex items-center gap-2 h-[34px]">
          <Calendar className="w-4 h-4 text-primary shrink-0" />
          <input type="date" value={startDate} onChange={e => { setStartDate(e.target.value); setPage(1); }}
            className="h-full px-3 bg-accent/20 border border-border rounded-xl text-xs font-bold focus:outline-none" />
          <span className="text-xs text-muted-foreground font-bold">au</span>
          <input type="date" value={endDate} onChange={e => { setEndDate(e.target.value); setPage(1); }}
            className="h-full px-3 bg-accent/20 border border-border rounded-xl text-xs font-bold focus:outline-none" />
        </div>

        {isSystem && tenants.length > 0 && (
          <select value={selectedTenant} onChange={e => { setSelectedTenant(e.target.value); setPage(1); }}
            className="h-[34px] px-3 bg-accent/20 border border-border rounded-xl text-xs font-bold focus:outline-none">
            <option value="">Mon tenant</option>
            {tenants.map(t => <option key={t.id} value={t.id}>{t.name}</option>)}
          </select>
        )}
      </div>

      {/* Table */}
      <div className="bg-card border border-border rounded-2xl shadow-sm overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs border-collapse">
            <thead>
              <tr className="border-b border-border text-muted-foreground font-bold bg-muted/10">
                <th className="py-3 px-4">N° Reçu</th>
                <th className="py-3 px-3">Date</th>
                <th className="py-3 px-3">Client</th>
                <th className="py-3 px-3 text-center">Articles</th>
                <th className="py-3 px-3 text-right">Total</th>
                <th className="py-3 px-3 text-center">Paiement</th>
                <th className="py-3 px-3 text-center">Statut</th>
                <th className="py-3 px-4 text-center">Actions</th>
              </tr>
            </thead>
            <tbody>
              {loading ? (
                <tr><td colSpan={8} className="py-12 text-center text-muted-foreground font-semibold">Chargement...</td></tr>
              ) : sales.length === 0 ? (
                <tr><td colSpan={8} className="py-12 text-center text-muted-foreground font-semibold">Aucune vente trouvée.</td></tr>
              ) : (
                sales.map(sale => {
                  const st = statusLabel(sale.status);
                  return (
                    <tr key={sale.id} className="border-b border-border/50 hover:bg-accent/20 transition-colors font-medium">
                      <td className="py-3.5 px-4 font-bold text-foreground">{sale.receipt_number}</td>
                      <td className="py-3.5 px-3 text-muted-foreground">{fmtDate(sale.sold_at)}</td>
                      <td className="py-3.5 px-3">{sale.customer_name || 'Client de passage'}</td>
                      <td className="py-3.5 px-3 text-center font-semibold">{sale.items.length}</td>
                      <td className="py-3.5 px-3 text-right font-bold text-foreground">{fmt(sale.total)}</td>
                      <td className="py-3.5 px-3 text-center">
                        <span className="px-2 py-0.5 rounded-full text-[10px] font-bold bg-primary/5 text-primary">
                          {paymentLabel(sale.payment_method)}
                        </span>
                      </td>
                      <td className="py-3.5 px-3 text-center">
                        <span className={`px-2 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wider ${st.cls}`}>
                          {st.text}
                        </span>
                      </td>
                      <td className="py-3.5 px-4">
                        <div className="flex items-center justify-center gap-1.5">
                          <button onClick={() => setDetailSale(sale)} title="Détail"
                            className="w-7 h-7 rounded-lg bg-accent/50 text-foreground flex items-center justify-center hover:bg-primary hover:text-white transition-all cursor-pointer">
                            <Eye className="w-3.5 h-3.5" />
                          </button>
                          {canPrint && (
                            <button onClick={() => handlePrintReceipt(sale)} title="Imprimer reçu"
                              className="w-7 h-7 rounded-lg bg-accent/50 text-foreground flex items-center justify-center hover:bg-emerald-500 hover:text-white transition-all cursor-pointer">
                              <Printer className="w-3.5 h-3.5" />
                            </button>
                          )}
                        </div>
                      </td>
                    </tr>
                  );
                })
              )}
            </tbody>
          </table>
        </div>

        {/* Pagination */}
        {totalPages > 1 && (
          <div className="flex items-center justify-between px-5 py-3 border-t border-border bg-muted/10">
            <span className="text-[10px] font-bold text-muted-foreground">
              Page {page} sur {totalPages}
            </span>
            <div className="flex items-center gap-2">
              <button disabled={page <= 1} onClick={() => setPage(p => p - 1)}
                className="w-8 h-8 rounded-lg bg-card border border-border flex items-center justify-center text-foreground hover:bg-accent disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer transition-all">
                <ChevronLeft className="w-4 h-4" />
              </button>
              <button disabled={page >= totalPages} onClick={() => setPage(p => p + 1)}
                className="w-8 h-8 rounded-lg bg-card border border-border flex items-center justify-center text-foreground hover:bg-accent disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer transition-all">
                <ChevronRight className="w-4 h-4" />
              </button>
            </div>
          </div>
        )}
      </div>

      {/* Detail Modal */}
      {detailSale && (() => {
        let parsedClient = null;
        if (detailSale.notes) {
          try {
            const parsed = JSON.parse(detailSale.notes);
            if (parsed && typeof parsed === 'object' && parsed.full_name) {
              parsedClient = parsed;
            }
          } catch(e) {}
        }
        
        return (
        <div className="fixed inset-0 bg-background/80 backdrop-blur-sm flex flex-col items-center justify-center p-4 z-50 animate-scale-in select-none">
          <div className="bg-card border border-border w-full max-w-lg rounded-3xl shadow-2xl flex flex-col mt-16 max-h-[calc(100vh-6rem)]">
            <div className="flex items-center justify-between p-5 border-b border-border">
              <div>
                <h3 className="font-bold text-base text-foreground">Détail Vente</h3>
                <p className="text-[10px] text-muted-foreground font-semibold mt-0.5">{detailSale.receipt_number}</p>
              </div>
              <button onClick={() => setDetailSale(null)}
                className="w-8 h-8 rounded-lg bg-accent/50 flex items-center justify-center text-muted-foreground hover:text-foreground cursor-pointer transition-colors">
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="flex-1 overflow-y-auto p-5 space-y-4">
              <div className="grid grid-cols-2 gap-3">
                {[
                  ['Date', fmtDate(detailSale.sold_at)],
                  ['Client', detailSale.customer_name || 'Passage'],
                  ['Paiement', paymentLabel(detailSale.payment_method)],
                  ['Statut', statusLabel(detailSale.status).text],
                ].map(([l, v], i) => (
                  <div key={i} className="p-3 bg-accent/30 rounded-xl">
                    <span className="text-[10px] font-bold text-muted-foreground uppercase block mb-0.5">{l}</span>
                    <span className="text-sm font-bold text-foreground">{v}</span>
                  </div>
                ))}
              </div>

              <div>
                <h4 className="text-xs font-extrabold text-foreground uppercase mb-2">Articles</h4>
                <div className="space-y-2">
                  {detailSale.items.map((item, i) => (
                    <div key={i} className="flex justify-between items-center text-xs p-2.5 bg-muted/20 rounded-xl border border-border/50">
                      <div>
                        <p className="font-bold text-foreground">{item.product_name}</p>
                        <p className="text-[10px] text-muted-foreground">{item.quantity} × {fmt(item.unit_price)}</p>
                      </div>
                      <span className="font-extrabold text-foreground">{fmt(item.line_total)}</span>
                    </div>
                  ))}
                </div>
              </div>

              <div className="space-y-2 pt-3 border-t border-border">
                {[
                  ['Sous-total', fmt(detailSale.subtotal)],
                  ['Taxes', fmt(detailSale.tax_total)],
                  ['Remise', detailSale.discount_total > 0 ? `-${fmt(detailSale.discount_total)}` : fmt(0)],
                ].map(([l, v], i) => (
                  <div key={i} className="flex justify-between text-xs font-semibold text-muted-foreground">
                    <span>{l}</span><span className="text-foreground">{v}</span>
                  </div>
                ))}
                <div className="flex justify-between text-sm font-extrabold text-foreground pt-2 border-t border-border/50">
                  <span>Total</span><span className="text-primary">{fmt(detailSale.total)}</span>
                </div>
              </div>
            </div>

            {parsedClient && (
              <div className="px-5 py-3 border-t border-border bg-emerald-500/5">
                <h4 className="text-[10px] font-extrabold text-emerald-600 dark:text-emerald-400 uppercase mb-1.5">Informations Client</h4>
                <div className="grid grid-cols-2 gap-2 text-xs">
                  <div>
                    <span className="text-muted-foreground font-semibold">Nom:</span> <span className="font-bold text-foreground">{parsedClient.full_name}</span>
                  </div>
                  {parsedClient.phone && (
                    <div>
                      <span className="text-muted-foreground font-semibold">Tél:</span> <span className="font-bold text-foreground">{parsedClient.phone}</span>
                    </div>
                  )}
                  {parsedClient.email && (
                    <div className="col-span-2">
                      <span className="text-muted-foreground font-semibold">Email:</span> <span className="font-bold text-foreground">{parsedClient.email}</span>
                    </div>
                  )}
                </div>
              </div>
            )}

            <div className="flex gap-3 p-5 border-t border-border">
              <button onClick={() => setDetailSale(null)}
                className="flex-1 py-2.5 rounded-xl border border-border bg-card hover:bg-accent text-foreground text-xs font-bold transition-all cursor-pointer">
                Fermer
              </button>
              {canPrint && (
                <button onClick={() => handlePrintReceipt(detailSale)}
                  className="flex-1 py-2.5 rounded-xl bg-primary text-primary-foreground text-xs font-bold shadow-sm hover:opacity-90 transition-all cursor-pointer flex items-center justify-center gap-1.5">
                  <Printer className="w-3.5 h-3.5" /><span>Imprimer</span>
                </button>
              )}
            </div>
          </div>
        </div>
        );
      })()}
    </div>
  );
}

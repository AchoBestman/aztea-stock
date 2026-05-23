import { useState, useEffect } from 'react';
import { 
  TrendingUp, 
  Download,
  Calendar,
  FileText,
  AlertTriangle,
  ArrowUpRight,
  ArrowDownRight
} from 'lucide-react';
import { api, Sale, Product, StockItem } from '../services/api';
import { stripTailwindFromHtml, wrapPdfDocument } from '../utils/pdfExport';
import { printReportHtml } from '../utils/printService';
import toast from 'react-hot-toast';

export default function Reports() {
  const [range, setRange] = useState<'30' | '90' | '365' | 'custom'>('30');
  
  // Custom date range states
  const [startDate, setStartDate] = useState(() => {
    const d = new Date();
    d.setDate(d.getDate() - 30);
    return d.toISOString().split('T')[0];
  });
  const [endDate, setEndDate] = useState(() => new Date().toISOString().split('T')[0]);

  const [sales, setSales] = useState<Sale[]>([]);
  const [products, setProducts] = useState<Product[]>([]);
  const [stockItems, setStockItems] = useState<StockItem[]>([]);
  const [loading, setLoading] = useState(true);

  const loadData = async () => {
    setLoading(true);
    try {
      const [salesRes, prodRes, stockRes] = await Promise.all([
        api.sales.list('', '', 1, 1000),
        api.products.list('', '', 1, 1000),
        api.stock.listItems('', false, '', 1, 1000)
      ]);
      setSales(salesRes.data || []);
      setProducts(prodRes.data || []);
      setStockItems(stockRes.data || []);
    } catch (e) {
      console.error("Failed to load report data:", e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();
  }, []);

  // Filter sales based on selected range of days or custom dates
  const getFilteredSales = () => {
    const completedSales = sales.filter(s => s.status === 'completed');
    if (range === 'custom') {
      const start = new Date(startDate);
      start.setHours(0, 0, 0, 0);
      const end = new Date(endDate);
      end.setHours(23, 59, 59, 999);
      
      return completedSales.filter(s => {
        const d = new Date(s.sold_at);
        return d >= start && d <= end;
      });
    } else {
      const cutoffDate = new Date();
      cutoffDate.setDate(cutoffDate.getDate() - parseInt(range, 10));
      cutoffDate.setHours(0, 0, 0, 0);
      return completedSales.filter(s => new Date(s.sold_at) >= cutoffDate);
    }
  };

  const filteredSales = getFilteredSales();

  // Create lookup map for products
  const productMap: { [id: string]: Product } = {};
  products.forEach(p => {
    productMap[p.id] = p;
  });

  // Calculate statistics
  let totalRevenue = 0;
  let totalCost = 0;
  let transactionCount = filteredSales.length;

  const categorySalesMap: { [name: string]: number } = {};
  const paymentMethodMap: { [method: string]: number } = { cash: 0, mobile_money: 0, card: 0 };
  
  // Aggregate sales by product to calculate Top 10 most/least sold
  const productQtySoldMap: { [id: string]: number } = {};
  
  // Initialize map with all products at 0 quantity sold
  products.forEach(p => {
    productQtySoldMap[p.id] = 0;
  });

  filteredSales.forEach(sale => {
    totalRevenue += sale.total;
    
    // Group by payment method
    const method = sale.payment_method;
    if (paymentMethodMap[method] !== undefined) {
      paymentMethodMap[method] += 1;
    } else {
      paymentMethodMap.cash += 1; // fallback
    }

    sale.items.forEach(item => {
      productQtySoldMap[item.product_id] = (productQtySoldMap[item.product_id] || 0) + item.quantity;
      
      const prod = productMap[item.product_id];
      if (prod) {
        totalCost += item.quantity * prod.purchase_price;
        const catName = prod.category_name || 'Général';
        categorySalesMap[catName] = (categorySalesMap[catName] || 0) + item.line_total;
      } else {
        totalCost += item.quantity * (item.unit_price * 0.6); // assume 40% margin fallback
        categorySalesMap['Général'] = (categorySalesMap['Général'] || 0) + item.line_total;
      }
    });
  });

  const profit = totalRevenue - totalCost;
  const marginRate = totalRevenue > 0 ? Math.round((profit / totalRevenue) * 100) : 0;

  // Top 10 Most Sold and Least Sold Products
  const sortedProductSales = Object.entries(productQtySoldMap)
    .map(([id, qty]) => ({
      product: productMap[id],
      quantity: qty
    }))
    .filter(item => item.product !== undefined); // Exclude deleted products

  const sortedDesc = [...sortedProductSales].sort((a, b) => b.quantity - a.quantity);
  const totalProducts = sortedDesc.length;

  let top10MostSold: typeof sortedProductSales = [];
  let top10LeastSold: typeof sortedProductSales = [];

  if (totalProducts < 20) {
    const half = Math.floor(totalProducts / 2);
    top10MostSold = sortedDesc.slice(0, half);
    top10LeastSold = sortedDesc.slice(half).reverse();
  } else {
    top10MostSold = sortedDesc.slice(0, 10);
    top10LeastSold = sortedDesc.slice(-10).reverse();
  }

  // Average Sale Calculation (Chiffre d'Affaires Moyen par Transaction)
  const averageSaleAmount = transactionCount > 0 ? Math.round(totalRevenue / transactionCount) : 0;

  // Average quantity sold per product (for deviation calculations)
  const uniqueProductsSold = sortedProductSales.filter(p => p.quantity > 0);
  const totalQtySold = uniqueProductsSold.reduce((sum, p) => sum + p.quantity, 0);
  const avgQtyPerProduct = uniqueProductsSold.length > 0 ? totalQtySold / uniqueProductsSold.length : 0;

  // Max and Min quantities sold
  const maxQty = top10MostSold[0]?.quantity || 0;
  const minQty = uniqueProductsSold.length > 0 
    ? [...uniqueProductsSold].sort((a, b) => a.quantity - b.quantity)[0]?.quantity || 0
    : 0;

  // Deviations:
  // "l'ecart entre moins vendus a la moyenne"
  const deviationMinToAvg = Math.max(0, avgQtyPerProduct - minQty);
  // "l'ecart entre moyenne et plus vendu"
  const deviationAvgToMax = Math.max(0, maxQty - avgQtyPerProduct);

  // Ruptures de Stock
  const outOfStockProducts = stockItems
    .filter(item => item.quantity <= 0)
    .map(item => ({
      id: item.product_id,
      name: item.product_name,
      barcode: products.find(p => p.id === item.product_id)?.barcode || 'N/A',
      location: item.unit_location || 'Non spécifié'
    }));

  const formatCurrency = (val: number) => {
    return new Intl.NumberFormat('fr-FR', { style: 'currency', currency: 'XAF', minimumFractionDigits: 0 }).format(val).replace('FCFA', 'F');
  };

  // Category percentage calculation
  const categoryPercentages = Object.entries(categorySalesMap).map(([name, val]) => ({
    name,
    amount: val,
    percentage: totalRevenue > 0 ? Math.round((val / totalRevenue) * 100) : 0
  })).sort((a, b) => b.amount - a.amount);

  // Payment percentage calculation
  const totalPayments = Object.values(paymentMethodMap).reduce((a, b) => a + b, 0);
  const paymentPercentages = {
    cash: totalPayments > 0 ? Math.round((paymentMethodMap.cash / totalPayments) * 100) : 0,
    momo: totalPayments > 0 ? Math.round((paymentMethodMap.mobile_money / totalPayments) * 100) : 0,
    card: totalPayments > 0 ? Math.round((paymentMethodMap.card / totalPayments) * 100) : 0,
  };

  // Generate sales curve daily aggregates
  const getSalesCurveData = () => {
    const dailyMap: { [dateStr: string]: number } = {};
    
    // Fill all days in range with 0 to show complete timeline
    const daysToGenerate = range === 'custom' 
      ? Math.max(1, Math.round((new Date(endDate).getTime() - new Date(startDate).getTime()) / (1000 * 3600 * 24)))
      : parseInt(range, 10);

    const start = new Date(range === 'custom' ? startDate : new Date().setDate(new Date().getDate() - daysToGenerate));

    for (let i = 0; i <= daysToGenerate; i++) {
      const d = new Date(start);
      d.setDate(d.getDate() + i);
      dailyMap[d.toISOString().split('T')[0]] = 0;
    }

    filteredSales.forEach(s => {
      const dateStr = s.sold_at.split('T')[0];
      if (dailyMap[dateStr] !== undefined) {
        dailyMap[dateStr] += s.total;
      }
    });

    return Object.entries(dailyMap).sort((a, b) => a[0].localeCompare(b[0]));
  };

  const curveData = getSalesCurveData();

  // Export to CSV
  const handleExportCSV = () => {
    const headers = ['Facture', 'Date', 'Client', 'Total Revenue', 'Cost', 'Profit', 'Mode Paiement'];
    const rows = filteredSales.map(sale => {
      let saleCost = 0;
      sale.items.forEach(item => {
        const prod = productMap[item.product_id];
        saleCost += item.quantity * (prod?.purchase_price || item.unit_price * 0.6);
      });
      return [
        sale.receipt_number,
        new Date(sale.sold_at).toLocaleString('fr-FR'),
        sale.customer_name || 'Passage',
        sale.total,
        saleCost,
        sale.total - saleCost,
        sale.payment_method
      ];
    });

    const csvContent = "data:text/csv;charset=utf-8," 
      + [headers.join(','), ...rows.map(e => e.join(','))].join('\n');
      
    const encodedUri = encodeURI(csvContent);
    const link = document.createElement("a");
    link.setAttribute("href", encodedUri);
    link.setAttribute("download", `rapport_financier_${range === 'custom' ? 'custom' : range + 'J'}.csv`);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  };

  const [exportingPdf, setExportingPdf] = useState(false);

  const handleExportPDF = async () => {
    if (exportingPdf) return;
    setExportingPdf(true);
    const toastId = 'reports-pdf';
    toast.loading('Génération du rapport...', { id: toastId });

    try {
      const root = document.querySelector('.print-full-width');
      const htmlContent = root
        ? wrapPdfDocument(
            `<h1>Statistiques et Analyses — AzteaStock</h1>
<p style="font-size:10px;color:#666">Période: ${range === 'custom' ? `${startDate} → ${endDate}` : `${range} jours`} · Généré le ${new Date().toLocaleString('fr-FR')}</p>
${stripTailwindFromHtml(root.innerHTML)}`,
            'Rapport Aztea'
          )
        : wrapPdfDocument('<p>Aucun contenu à exporter</p>', 'Rapport Aztea');

      const result = await printReportHtml(
        htmlContent,
        `rapport_financier_${range === 'custom' ? 'custom' : range + 'J'}_${new Date().toISOString().split('T')[0]}.pdf`
      );

      if (result.mode === 'pdf') {
        toast.success(
          result.savedPath
            ? `PDF enregistré : ${result.savedPath}`
            : 'Rapport PDF enregistré dans Téléchargements',
          { id: toastId }
        );
      } else if (result.mode === 'printer') {
        toast.success('Rapport envoyé à l\'imprimante', { id: toastId });
      } else {
        window.print();
        toast.success('Utilisez la boîte de dialogue système pour imprimer', { id: toastId });
      }
    } catch (e: unknown) {
      toast.error(e instanceof Error ? e.message : 'Erreur export PDF', { id: toastId });
    } finally {
      setExportingPdf(false);
    }
  };

  // Render SVG Path line coordinates
  const svgWidth = 600;
  const svgHeight = 200;
  const padding = 30;
  
  const maxRevenueInCurve = Math.max(...curveData.map(d => d[1]), 1);
  
  const points = curveData.map((d, index) => {
    const x = padding + (index / Math.max(1, curveData.length - 1)) * (svgWidth - 2 * padding);
    const y = svgHeight - padding - (d[1] / maxRevenueInCurve) * (svgHeight - 2 * padding);
    return `${x},${y}`;
  });

  const pathD = points.length > 0 ? `M ${points.join(' L ')}` : '';

  return (
    <div className="space-y-8 animate-slide-up select-none print:p-0 print:m-0 print:bg-white print:text-black">
      
      {/* Print PDF Custom Styles injection */}
      <style>{`
        @media print {
          body {
            background-color: white !important;
            color: black !important;
          }
          aside, header, nav, button, input, select, .no-print {
            display: none !important;
          }
          .print-full-width {
            width: 100% !important;
            max-width: 100% !important;
            margin: 0 !important;
            padding: 0 !important;
            border: none !important;
            box-shadow: none !important;
          }
          .bg-card {
            background: white !important;
            border: 1px solid #ddd !important;
            color: black !important;
          }
        }
      `}</style>

      {/* Page Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 no-print">
        <div>
          <h1 className="text-2xl font-bold text-foreground">Statistiques et Analyses</h1>
          <p className="text-xs text-muted-foreground font-semibold mt-0.5">Explorez l'évolution financière de votre activité.</p>
        </div>

        <div className="flex flex-wrap items-center gap-3">
          {/* Quick ranges */}
          <div className="flex bg-card border border-border p-1 rounded-xl shadow-sm">
            {(['30', '90', '365', 'custom'] as const).map((days) => (
              <button
                key={days}
                onClick={() => setRange(days)}
                className={`px-3 py-1.5 rounded-lg text-[10px] font-extrabold uppercase transition-all cursor-pointer ${
                  range === days
                    ? 'bg-primary dark:bg-blue-600 text-primary-foreground shadow-sm'
                    : 'text-muted-foreground hover:text-foreground'
                }`}
              >
                {days === 'custom' ? 'Période' : `${days} Jours`}
              </button>
            ))}
          </div>

          <button 
            onClick={handleExportCSV}
            className="flex items-center gap-1 px-4 py-2 rounded-xl bg-secondary text-foreground text-xs font-bold hover:bg-opacity-95 transition-all shadow-md cursor-pointer border border-border"
          >
            <Download className="w-4 h-4" />
            <span>CSV</span>
          </button>

          <button 
            onClick={handleExportPDF}
            disabled={exportingPdf}
            className="flex items-center gap-1 px-4 py-2 rounded-xl bg-primary dark:bg-blue-600 text-primary-foreground text-xs font-bold hover:bg-opacity-95 transition-all shadow-md cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <FileText className="w-4 h-4" />
            <span>{exportingPdf ? 'Génération…' : 'Exporter PDF'}</span>
          </button>
        </div>
      </div>

      {/* Custom Date Picker inputs */}
      {range === 'custom' && (
        <div className="p-4 bg-card border border-border rounded-2xl flex flex-wrap items-center gap-4 no-print animate-fade-in">
          <div className="flex items-center gap-2">
            <Calendar className="w-4 h-4 text-primary dark:text-blue-600" />
            <span className="text-xs font-bold text-muted-foreground">Intervalle de date :</span>
          </div>
          <div className="flex items-center gap-2">
            <input 
              type="date" 
              value={startDate} 
              onChange={(e) => setStartDate(e.target.value)}
              className="px-3 py-1.5 bg-accent/30 border border-border rounded-xl text-xs font-bold focus:outline-none"
            />
            <span className="text-xs text-muted-foreground font-bold">au</span>
            <input 
              type="date" 
              value={endDate} 
              onChange={(e) => setEndDate(e.target.value)}
              className="px-3 py-1.5 bg-accent/30 border border-border rounded-xl text-xs font-bold focus:outline-none"
            />
          </div>
        </div>
      )}

      {loading ? (
        <div className="py-20 text-center text-muted-foreground font-semibold">
          Calcul des analyses...
        </div>
      ) : (
        <div className="space-y-6 print-full-width">
          {/* Overview summary grids */}
          <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
            <div className="bg-card border border-border p-5 rounded-2xl shadow-sm">
              <span className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Chiffre d'Affaires</span>
              <h3 className="text-xl font-black text-foreground">{formatCurrency(totalRevenue)}</h3>
              <span className="text-[9px] text-emerald-500 font-bold flex items-center gap-0.5 mt-1">
                <TrendingUp className="w-3 h-3" /> Période active
              </span>
            </div>

            <div className="bg-card border border-border p-5 rounded-2xl shadow-sm">
              <span className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Marge Brute</span>
              <h3 className="text-xl font-black text-foreground">{formatCurrency(profit)}</h3>
              <span className="text-[9px] text-emerald-500 font-bold flex items-center gap-0.5 mt-1">
                <TrendingUp className="w-3 h-3" /> Gain net estimé
              </span>
            </div>

            <div className="bg-card border border-border p-5 rounded-2xl shadow-sm">
              <span className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Moyenne de Vente / Panier</span>
              <h3 className="text-xl font-black text-blue-600 dark:text-blue-400">{formatCurrency(averageSaleAmount)}</h3>
              <span className="text-[9px] text-muted-foreground font-semibold block mt-1">
                Valeur moyenne facturée
              </span>
            </div>

            <div className="bg-card border border-border p-5 rounded-2xl shadow-sm">
              <span className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Taux de Marge</span>
              <h3 className="text-xl font-black text-foreground">{marginRate}%</h3>
              <span className="text-[9px] text-muted-foreground font-semibold block mt-1">
                Rentabilité globale
              </span>
            </div>
          </div>

          {/* Premium Sales Evolution SVG Line Chart */}
          <div className="bg-card border border-border p-6 rounded-2xl shadow-sm">
            <h3 className="font-bold text-sm text-foreground mb-1">Évolution des Ventes</h3>
            <p className="text-[10px] text-muted-foreground font-medium mb-6">Courbe temporelle du chiffre d'affaires cumulé par jour</p>
            
            <div className="w-full overflow-x-auto">
              <svg viewBox={`0 0 ${svgWidth} ${svgHeight}`} className="w-full max-h-[220px]">
                {/* Horizontal Grid lines */}
                {[0, 0.25, 0.5, 0.75, 1].map((ratio, i) => {
                  const y = padding + ratio * (svgHeight - 2 * padding);
                  return (
                    <line 
                      key={i} 
                      x1={padding} 
                      y1={y} 
                      x2={svgWidth - padding} 
                      y2={y} 
                      stroke="var(--border)" 
                      strokeDasharray="4 4" 
                      strokeWidth={1}
                      opacity={0.3}
                    />
                  );
                })}

                {/* Main Curve Path */}
                {pathD && (
                  <>
                    <path
                      d={pathD}
                      fill="none"
                      className="stroke-blue-600 dark:stroke-blue-400"
                      strokeWidth={3}
                      strokeLinecap="round"
                      strokeLinejoin="round"
                    />
                    {/* Shadow area beneath curve */}
                    <path
                      d={`${pathD} L ${svgWidth - padding},${svgHeight - padding} L ${padding},${svgHeight - padding} Z`}
                      fill="url(#gradient)"
                      opacity={0.1}
                    />
                  </>
                )}

                {/* Gradients */}
                <defs>
                  <linearGradient id="gradient" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stopColor="currentColor" className="text-blue-600 dark:text-blue-400" />
                    <stop offset="100%" stopColor="currentColor" stopOpacity="0" className="text-blue-600 dark:text-blue-400" />
                  </linearGradient>
                </defs>

                {/* Point markers */}
                {curveData.map((d, index) => {
                  const x = padding + (index / Math.max(1, curveData.length - 1)) * (svgWidth - 2 * padding);
                  const y = svgHeight - padding - (d[1] / maxRevenueInCurve) * (svgHeight - 2 * padding);
                  return (
                    <circle 
                      key={index} 
                      cx={x} 
                      cy={y} 
                      r={3.5} 
                      style={{ fill: 'var(--color-card)' }}
                      strokeWidth={2}
                      className="stroke-blue-600 dark:stroke-blue-400 cursor-pointer hover:r-5 transition-all"
                    >
                      <title>{`${d[0]}: ${d[1]} F`}</title>
                    </circle>
                  );
                })}
              </svg>
            </div>
            
            <div className="flex justify-between text-[9px] font-bold text-muted-foreground px-4 mt-2">
              <span>{curveData[0]?.[0]}</span>
              <span>{curveData[Math.floor(curveData.length / 2)]?.[0]}</span>
              <span>{curveData[curveData.length - 1]?.[0]}</span>
            </div>
          </div>

          {/* Best / Worst Sellers & Deviations */}
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            
            {/* Top 10 Best Sellers */}
            <div className="bg-card border border-border p-5 rounded-2xl shadow-sm">
              <h4 className="font-extrabold text-xs text-foreground mb-3 uppercase tracking-wider flex items-center gap-1.5">
                <ArrowUpRight className="w-4 h-4 text-emerald-500" />
                Top 10 Produits Plus Vendus
              </h4>
              <div className="space-y-2">
                {top10MostSold.length === 0 ? (
                  <p className="text-xs text-muted-foreground">Aucune vente enregistrée.</p>
                ) : (
                  top10MostSold.map((item, index) => (
                    <div key={index} className="flex justify-between items-center text-xs py-1.5 border-b border-border/40 font-semibold">
                      <span className="truncate max-w-[200px] text-foreground">{index + 1}. {item.product.name}</span>
                      <span className="text-emerald-500 font-extrabold">{item.quantity} {item.product.unit}s</span>
                    </div>
                  ))
                )}
              </div>
            </div>

            {/* Top 10 Least Sold */}
            <div className="bg-card border border-border p-5 rounded-2xl shadow-sm">
              <h4 className="font-extrabold text-xs text-foreground mb-3 uppercase tracking-wider flex items-center gap-1.5">
                <ArrowDownRight className="w-4 h-4 text-amber-500" />
                Top 10 Produits Moins Vendus
              </h4>
              <div className="space-y-2">
                {top10LeastSold.length === 0 ? (
                  <p className="text-xs text-muted-foreground">Aucune vente enregistrée.</p>
                ) : (
                  top10LeastSold.map((item, index) => (
                    <div key={index} className="flex justify-between items-center text-xs py-1.5 border-b border-border/40 font-semibold">
                      <span className="truncate max-w-[200px] text-foreground">{index + 1}. {item.product.name}</span>
                      <span className="text-amber-500 font-extrabold">{item.quantity} {item.product.unit}s</span>
                    </div>
                  ))
                )}
              </div>
            </div>
          </div>

          {/* Deviation Stats and Rupture de Stocks */}
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            
            {/* Deviation Stats Card */}
            <div className="bg-card border border-border p-5 rounded-2xl shadow-sm lg:col-span-1 space-y-4">
              <h4 className="font-extrabold text-xs text-foreground uppercase tracking-wider">Statistiques d'Écarts</h4>
              
              <div className="space-y-3 font-semibold text-xs text-muted-foreground">
                <div className="p-3 bg-accent/30 rounded-xl">
                  <span className="text-[10px] font-bold block uppercase mb-0.5">Moyenne Quantités Vendu / Produit</span>
                  <span className="text-sm font-black text-foreground">{Math.round(avgQtyPerProduct)} unités</span>
                </div>

                <div className="p-3 bg-accent/30 rounded-xl">
                  <span className="text-[10px] font-bold block uppercase mb-0.5">Écart Moins Vendus à la Moyenne</span>
                  <span className="text-sm font-black text-amber-500">{Math.round(deviationMinToAvg)} unités</span>
                </div>

                <div className="p-3 bg-accent/30 rounded-xl">
                  <span className="text-[10px] font-bold block uppercase mb-0.5">Écart Moyenne au Plus Vendu</span>
                  <span className="text-sm font-black text-emerald-500">{Math.round(deviationAvgToMax)} unités</span>
                </div>
              </div>
            </div>

            {/* Out of Stock listing */}
            <div className="bg-card border border-border p-5 rounded-2xl shadow-sm lg:col-span-2">
              <h4 className="font-extrabold text-xs text-foreground uppercase tracking-wider flex items-center gap-1.5 text-rose-500 mb-3">
                <AlertTriangle className="w-4 h-4" />
                Ruptures de Stocks Actuelles ({outOfStockProducts.length})
              </h4>
              
              <div className="max-h-[220px] overflow-y-auto space-y-2">
                {outOfStockProducts.length === 0 ? (
                  <p className="text-xs text-emerald-500 font-semibold py-4 text-center">Aucun produit en rupture de stock. Félicitations !</p>
                ) : (
                  outOfStockProducts.map((prod, i) => (
                    <div key={i} className="flex justify-between items-center text-xs p-2.5 bg-rose-500/5 border border-rose-500/10 rounded-xl font-semibold">
                      <div>
                        <p className="font-bold text-foreground">{prod.name}</p>
                        <p className="text-[9px] text-muted-foreground font-mono">Code: {prod.barcode}</p>
                      </div>
                      <span className="px-2 py-0.5 rounded bg-rose-500/10 text-rose-500 text-[10px] font-extrabold uppercase">
                        Rupture
                      </span>
                    </div>
                  ))
                )}
              </div>
            </div>

          </div>

          {/* Category distribution and Payment methods */}
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            
            {/* Category distribution */}
            <div className="bg-card border border-border p-6 rounded-2xl shadow-sm flex flex-col justify-between">
              <div>
                <h3 className="font-bold text-sm text-foreground">Répartition des Ventes par Catégorie</h3>
                <p className="text-[10px] text-muted-foreground font-medium mb-6">Volume de vente comparé en XAF</p>
              </div>

              <div className="space-y-4">
                {categoryPercentages.length === 0 ? (
                  <div className="py-8 text-center text-muted-foreground text-xs font-semibold">
                    Aucune vente enregistrée.
                  </div>
                ) : (
                  categoryPercentages.map((cat, index) => {
                    const colors = ['bg-blue-600 dark:bg-blue-400', 'bg-violet-500', 'bg-emerald-500', 'bg-amber-500'];
                    const colorClass = colors[index % colors.length];
                    return (
                      <div key={index}>
                        <div className="flex justify-between text-xs font-bold mb-1">
                          <span>{cat.name}</span>
                          <span>{cat.percentage}% ({formatCurrency(cat.amount)})</span>
                        </div>
                        <div className="w-full bg-accent h-2.5 rounded-full overflow-hidden">
                          <div className={`${colorClass} h-full rounded-full`} style={{ width: `${cat.percentage}%` }}></div>
                        </div>
                      </div>
                    );
                  })
                )}
              </div>
            </div>

            {/* Payment mode chart */}
            <div className="bg-card border border-border p-6 rounded-2xl shadow-sm flex flex-col justify-between">
              <div>
                <h3 className="font-bold text-sm text-foreground">Modes de Paiement Préférés</h3>
                <p className="text-[10px] text-muted-foreground font-medium mb-6">Répartition par transaction émise</p>
              </div>

              <div className="flex items-center justify-around gap-6 py-4">
                <div className="text-center">
                  <div className="w-16 h-16 rounded-full border-4 border-blue-600 dark:border-blue-400 flex items-center justify-center font-extrabold text-sm text-blue-600 dark:text-blue-400">
                    {paymentPercentages.cash}%
                  </div>
                  <span className="text-[10px] font-bold text-muted-foreground block mt-2">ESPÈCES</span>
                </div>

                <div className="text-center">
                  <div className="w-16 h-16 rounded-full border-4 border-violet-500 flex items-center justify-center font-extrabold text-sm text-violet-500">
                    {paymentPercentages.momo}%
                  </div>
                  <span className="text-[10px] font-bold text-muted-foreground block mt-2">MOBILE MONEY</span>
                </div>

                <div className="text-center">
                  <div className="w-16 h-16 rounded-full border-4 border-emerald-500 flex items-center justify-center font-extrabold text-sm text-emerald-500">
                    {paymentPercentages.card}%
                  </div>
                  <span className="text-[10px] font-bold text-muted-foreground block mt-2">CARTE</span>
                </div>
              </div>
            </div>

          </div>

        </div>
      )}
    </div>
  );
}

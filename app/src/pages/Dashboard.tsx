import { 
  TrendingUp, 
  ShoppingCart, 
  AlertTriangle, 
  PackageCheck
} from 'lucide-react';
import { useSyncStore } from '../store/syncStore';
import { useAuthStore } from '../store/authStore';
import { useState, useEffect, useMemo } from 'react';
import { api, Sale, StockItem } from '../services/api';
import { getDashboardPerformanceSubtitle } from '../lib/format';
import {
  buildRevenueChartSeries,
  buildChartPaths,
  formatChartAxisValue,
  getChartMaxValue,
} from '../lib/dashboardChart';

export default function Dashboard() {
  const { isOnline, lastSyncAt } = useSyncStore();
  const { user } = useAuthStore();

  const [selectedPeriod, setSelectedPeriod] = useState<'day' | 'week' | 'month' | 'interval'>('day');
  const [dateRange, setDateRange] = useState({ 
    start: new Date().toISOString().split('T')[0], 
    end: new Date().toISOString().split('T')[0] 
  });
  const [sales, setSales] = useState<Sale[]>([]);
  const [stockItems, setStockItems] = useState<StockItem[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function loadData() {
      setLoading(true);
      try {
        const [salesRes, stockRes] = await Promise.all([
          api.sales.list('', '', 1, 1000),
          api.stock.listItems('', false, '', 1, 1000),
        ]);
        setSales(salesRes.data || []);
        setStockItems(stockRes.data || []);
      } catch (error) {
        console.error("Failed to load dashboard data:", error);
      } finally {
        setLoading(false);
      }
    }
    loadData();
  }, []);

  // Compute stats based on the selected period
  const getFilteredSales = () => {
    const now = new Date();
    const startOfToday = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    
    // Fix startOfWeek calculation by creating a clone of now
    const weekTemp = new Date(now);
    const startOfWeek = new Date(weekTemp.setDate(weekTemp.getDate() - weekTemp.getDay()));
    
    const startOfMonth = new Date(now.getFullYear(), now.getMonth(), 1);

    return sales.filter(s => {
      const soldDate = new Date(s.sold_at);
      if (selectedPeriod === 'day') {
        return soldDate >= startOfToday;
      } else if (selectedPeriod === 'week') {
        return soldDate >= startOfWeek;
      } else if (selectedPeriod === 'month') {
        return soldDate >= startOfMonth;
      } else {
        const startStr = dateRange.start;
        const endStr = dateRange.end;
        const soldStr = s.sold_at.split('T')[0];
        return soldStr >= startStr && soldStr <= endStr;
      }
    });
  };

  const filteredSales = getFilteredSales();

  // Chiffre d'affaires
  const totalRevenue = filteredSales.reduce((acc, s) => acc + (s.status === 'completed' ? s.total : 0), 0);
  
  // Transactions Count
  const transactionsCount = filteredSales.length;

  // Alerts: Stock items with quantity <= threshold
  const lowStockItems = stockItems.filter(item => item.quantity <= item.low_stock_threshold);
  const criticalStockItems = stockItems.filter(item => item.quantity <= 0);

  // Top Products: Group by product name & sum quantity + revenue
  const topProductsMap: { [key: string]: { name: string; qty: number; sales: number } } = {};
  sales.forEach(sale => {
    if (sale.status !== 'completed') return;
    sale.items.forEach(item => {
      if (!topProductsMap[item.product_id]) {
        topProductsMap[item.product_id] = {
          name: item.product_name,
          qty: 0,
          sales: 0
        };
      }
      topProductsMap[item.product_id].qty += item.quantity;
      topProductsMap[item.product_id].sales += item.line_total;
    });
  });

  const topProducts = Object.values(topProductsMap)
    .sort((a, b) => b.qty - a.qty)
    .slice(0, 5);

  const formatCurrency = (val: number) => {
    return new Intl.NumberFormat('fr-FR', { style: 'currency', currency: 'XAF', minimumFractionDigits: 0 }).format(val).replace('FCFA', 'F');
  };

  const chartSeries = useMemo(
    () => buildRevenueChartSeries(filteredSales, selectedPeriod, dateRange),
    [filteredSales, selectedPeriod, dateRange]
  );
  const chartMax = getChartMaxValue(chartSeries.values);
  const chartPaths = useMemo(
    () => buildChartPaths(chartSeries.values),
    [chartSeries.values]
  );
  const hasChartData = chartMax > 0;
  const yAxisTicks = hasChartData
    ? [chartMax, chartMax * (2 / 3), chartMax * (1 / 3), 0]
    : [0, 0, 0, 0];

  return (
    <div className="space-y-8 animate-slide-up select-none">
      {/* Welcome banner with time & Quick status */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 className="text-3xl font-extrabold text-foreground tracking-tight">
            Bonjour, <span className="gradient-text">{user?.name || 'Gérant'}</span> 👋
          </h1>
          <p className="text-muted-foreground mt-1 font-medium">
            {getDashboardPerformanceSubtitle(user?.tenantBusinessType ?? 'pharmacy')}
          </p>
        </div>

        {/* Quick Period Selector */}
        <div className="flex flex-wrap items-center gap-3 self-start">
          <div className="flex bg-card border border-border p-1 rounded-xl shadow-sm">
            {(['day', 'week', 'month', 'interval'] as const).map((period) => (
              <button
                key={period}
                onClick={() => setSelectedPeriod(period)}
                className={`px-4 py-2 rounded-lg text-xs font-bold transition-all capitalize cursor-pointer ${
                  selectedPeriod === period
                    ? 'bg-primary dark:bg-blue-600 text-primary-foreground shadow-sm'
                    : 'text-muted-foreground hover:text-foreground'
                }`}
              >
                {period === 'day' ? 'Journée' : period === 'week' ? 'Semaine' : period === 'month' ? 'Mois' : 'Intervalle'}
              </button>
            ))}
          </div>

          {selectedPeriod === 'interval' && (
            <div className="flex items-center gap-2 bg-card p-1 rounded-xl shadow-sm border border-border">
              <input
                type="date"
                value={dateRange.start}
                onChange={(e) => setDateRange(prev => ({ ...prev, start: e.target.value }))}
                max={dateRange.end}
                className="px-3 py-1.5 rounded-lg text-xs font-bold bg-transparent text-foreground focus:outline-none"
              />
              <span className="text-muted-foreground text-xs font-bold">-</span>
              <input
                type="date"
                value={dateRange.end}
                onChange={(e) => setDateRange(prev => ({ ...prev, end: e.target.value }))}
                min={dateRange.start}
                max={new Date().toISOString().split('T')[0]}
                className="px-3 py-1.5 rounded-lg text-xs font-bold bg-transparent text-foreground focus:outline-none"
              />
            </div>
          )}
        </div>
      </div>

      {loading ? (
        <div className="py-20 text-center text-muted-foreground font-semibold">
          Chargement des données du tableau de bord...
        </div>
      ) : (
        <>
          {/* Stats Cards Grid */}
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            {/* Card: CA */}
            <div className="bg-card border border-border rounded-2xl p-6 card-hover shadow-sm flex flex-col justify-between">
              <div className="flex items-center justify-between">
                <span className="text-sm font-semibold text-muted-foreground">Chiffre d'Affaires</span>
                <div className="w-10 h-10 rounded-xl bg-primary/10 flex items-center justify-center text-primary">
                  <TrendingUp className="w-5 h-5 dark:text-blue-600" />
                </div>
              </div>
              <div className="mt-4">
                <h3 className="text-2xl font-extrabold tracking-tight text-foreground">
                  {formatCurrency(totalRevenue)}
                </h3>
                <div className="flex items-center gap-1 mt-1 text-xs text-muted-foreground font-medium">
                  <span>Période sélectionnée</span>
                </div>
              </div>
            </div>

            {/* Card: Transactions count */}
            <div className="bg-card border border-border rounded-2xl p-6 card-hover shadow-sm flex flex-col justify-between">
              <div className="flex items-center justify-between">
                <span className="text-sm font-semibold text-muted-foreground">Transactions</span>
                <div className="w-10 h-10 rounded-xl bg-blue-500/10 flex items-center justify-center text-blue-500">
                  <ShoppingCart className="w-5 h-5" />
                </div>
              </div>
              <div className="mt-4">
                <h3 className="text-2xl font-extrabold tracking-tight text-foreground">{transactionsCount}</h3>
                <p className="text-xs text-muted-foreground font-medium mt-1">Factures éditées sur la période</p>
              </div>
            </div>

            {/* Card: Shortages */}
            <div className="bg-card border border-border rounded-2xl p-6 card-hover shadow-sm flex flex-col justify-between">
              <div className="flex items-center justify-between">
                <span className="text-sm font-semibold text-muted-foreground">Alertes de Stock</span>
                <div className={`w-10 h-10 rounded-xl flex items-center justify-center ${
                  lowStockItems.length > 0 ? 'bg-rose-500/10 text-rose-500' : 'bg-emerald-500/10 text-emerald-500'
                }`}>
                  <AlertTriangle className="w-5 h-5" />
                </div>
              </div>
              <div className="mt-4">
                <h3 className="text-2xl font-extrabold tracking-tight text-foreground">
                  {lowStockItems.length} produit{lowStockItems.length > 1 ? 's' : ''}
                </h3>
                <p className={`text-xs font-semibold mt-1 ${
                  lowStockItems.length > 0 ? 'text-rose-500 animate-pulse' : 'text-emerald-500'
                }`}>
                  {lowStockItems.length > 0 ? `${criticalStockItems.length} rupture(s) critique(s)` :  lowStockItems.length > 0 ? 'Tous les stocks sont OK' : 'Aucun stock en alerte'}
                </p>
              </div>
            </div>

            {/* Card: Sync log */}
            <div className="bg-card border border-border rounded-2xl p-6 card-hover shadow-sm flex flex-col justify-between">
              <div className="flex items-center justify-between">
                <span className="text-sm font-semibold text-muted-foreground">Statut Sync</span>
                <div className={`w-10 h-10 rounded-xl flex items-center justify-center ${
                  isOnline ? 'bg-emerald-500/10 text-emerald-500' : 'bg-amber-500/10 text-amber-500'
                }`}>
                  <PackageCheck className="w-5 h-5" />
                </div>
              </div>
              <div className="mt-4">
                <div className="flex items-center gap-1.5">
                  <h3 className="text-base font-bold text-foreground truncate">
                    {isOnline ? 'Synchronisé' : 'Stockage local'}
                  </h3>
                </div>
                <p className="text-[11px] text-muted-foreground font-medium mt-1 truncate">
                  Dernière sync : {lastSyncAt ? lastSyncAt.toLocaleTimeString('fr-FR') : 'Jamais'}
                </p>
              </div>
            </div>
          </div>

          {/* Main Charts & Lists Grid */}
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            {/* Sales Chart Card */}
            <div className="bg-card border border-border rounded-2xl p-6 shadow-sm lg:col-span-2 flex flex-col">
              <div className="flex items-center justify-between mb-6">
                <div>
                  <h3 className="font-bold text-lg text-foreground">Évolution Financière</h3>
                  <p className="text-xs text-muted-foreground">Représentation du volume des transactions récentes</p>
                </div>
              </div>

              <div className="flex-1 min-h-[220px] flex flex-col relative pt-6 px-2">
                {!hasChartData && (
                  <p className="absolute inset-x-0 top-1/3 z-20 text-center text-xs font-semibold text-muted-foreground">
                    Aucune vente sur cette période
                  </p>
                )}

                <div className="absolute inset-0 flex flex-col justify-between pointer-events-none text-[10px] text-muted-foreground/40 font-medium pb-10">
                  {yAxisTicks.map((tick, i) => (
                    <div
                      key={i}
                      className={`w-full text-right ${i < yAxisTicks.length - 1 ? 'border-b border-border/50 pb-1' : ''}`}
                    >
                      {formatChartAxisValue(tick)}
                    </div>
                  ))}
                </div>

                <svg
                  className="absolute inset-0 w-full h-[85%] mt-4 overflow-visible"
                  viewBox="0 0 640 200"
                  preserveAspectRatio="none"
                >
                  <defs>
                    <linearGradient id="chartGradient" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="0%" stopColor="currentColor" stopOpacity="0.2" className="text-blue-600 dark:text-blue-400" />
                      <stop offset="100%" stopColor="currentColor" stopOpacity="0.0" className="text-blue-600 dark:text-blue-400" />
                    </linearGradient>
                  </defs>
                  {chartPaths.linePath && (
                    <>
                      <path
                        d={chartPaths.linePath}
                        fill="none"
                        className="stroke-blue-600 dark:stroke-blue-400"
                        strokeWidth="3.5"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        vectorEffect="non-scaling-stroke"
                      />
                      <path d={chartPaths.areaPath} fill="url(#chartGradient)" />
                      {hasChartData &&
                        chartPaths.points.map((p, i) => (
                          <circle
                            key={i}
                            cx={p.x}
                            cy={p.y}
                            r="5"
                            className="fill-blue-600 dark:fill-blue-400"
                            style={{ stroke: 'var(--color-card)' }}
                            strokeWidth="2"
                            vectorEffect="non-scaling-stroke"
                          />
                        ))}
                    </>
                  )}
                </svg>

                <div className="w-full flex justify-between text-[11px] font-bold text-muted-foreground pt-4 border-t border-border mt-auto z-10 gap-1">
                  {chartSeries.labels.map((label) => (
                    <span key={label} className="truncate text-center flex-1 min-w-0">
                      {label}
                    </span>
                  ))}
                </div>
              </div>
            </div>

            {/* Top Products Card */}
            <div className="bg-card border border-border rounded-2xl p-6 shadow-sm flex flex-col justify-between">
              <div>
                <h3 className="font-bold text-lg text-foreground mb-4">Top 5 Produits Ventes</h3>
                <div className="space-y-4">
                  {topProducts.length === 0 ? (
                    <div className="py-8 text-center text-muted-foreground text-xs font-semibold">
                      Aucun produit vendu pour le moment.
                    </div>
                  ) : (
                    topProducts.map((prod, index) => (
                      <div key={index} className="flex items-center justify-between p-2 rounded-xl hover:bg-accent/40 transition-colors">
                        <div className="flex items-center gap-3">
                          <span className="w-6 h-6 rounded-lg bg-primary/10 text-primary dark:bg-blue-600 flex items-center justify-center font-bold text-xs">
                            {index + 1}
                          </span>
                          <div className="max-w-[150px]">
                            <p className="text-xs font-bold text-foreground truncate">{prod.name}</p>
                          </div>
                        </div>
                        <div className="text-right">
                          <p className="text-xs font-bold text-foreground">{formatCurrency(prod.sales)}</p>
                          <span className="text-[10px] text-muted-foreground font-medium">{prod.qty} vendus</span>
                        </div>
                      </div>
                    ))
                  )}
                </div>
              </div>
            </div>
          </div>

          {/* Bottom Grid: Recent Sales & Stock Alerts */}
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            {/* Recent Transactions list */}
            <div className="bg-card border border-border rounded-2xl p-6 shadow-sm lg:col-span-2 flex flex-col">
              <div className="flex items-center justify-between mb-4">
                <h3 className="font-bold text-lg text-foreground">Dernières Factures</h3>
              </div>

              <div className="overflow-x-auto">
                <table className="w-full text-left text-xs border-collapse">
                  <thead>
                    <tr className="border-b border-border text-muted-foreground font-bold">
                      <th className="py-3 px-2">N° Facture</th>
                      <th className="py-3 px-2">Date/Heure</th>
                      <th className="py-3 px-2">Client</th>
                      <th className="py-3 px-2 text-center">Articles</th>
                      <th className="py-3 px-2 text-right">Total</th>
                      <th className="py-3 px-2 text-center">Statut</th>
                    </tr>
                  </thead>
                  <tbody>
                    {sales.length === 0 ? (
                      <tr>
                        <td colSpan={6} className="py-8 text-center text-muted-foreground font-semibold">
                          Aucune facture disponible.
                        </td>
                      </tr>
                    ) : (
                      sales.slice(0, 5).map((sale, i) => (
                        <tr key={i} className="border-b border-border/50 hover:bg-accent/20 transition-colors font-medium">
                          <td className="py-3.5 px-2 font-bold text-foreground">{sale.receipt_number}</td>
                          <td className="py-3.5 px-2 text-muted-foreground">
                            {new Date(sale.sold_at).toLocaleString('fr-FR', { dateStyle: 'short', timeStyle: 'short' })}
                          </td>
                          <td className="py-3.5 px-2">{sale.customer_name || 'Client de passage'}</td>
                          <td className="py-3.5 px-2 text-center font-semibold">{sale.items.length}</td>
                          <td className="py-3.5 px-2 text-right font-bold text-foreground">
                            {formatCurrency(sale.total)}
                          </td>
                          <td className="py-3.5 px-2 text-center">
                            <span className={`px-2 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wider ${
                              sale.status === 'completed' 
                                ? 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400' 
                                : 'bg-rose-500/10 text-rose-600 dark:text-rose-400'
                            }`}>
                              {sale.status === 'completed' ? 'Validé' : 'Annulé'}
                            </span>
                          </td>
                        </tr>
                      ))
                    )}
                  </tbody>
                </table>
              </div>
            </div>

            {/* Stock warnings */}
            <div className="bg-card border border-border rounded-2xl p-6 shadow-sm flex flex-col justify-between">
              <div>
                <h3 className="font-bold text-lg text-foreground mb-4">Alerte Ruptures & Seuils</h3>
                <div className="space-y-4 max-h-[300px] overflow-y-auto pr-1">
                  {lowStockItems.length === 0 ? (
                    <div className="py-8 text-center text-muted-foreground text-xs font-semibold">
                      Aucune alerte de stock en cours.
                    </div>
                  ) : (
                    lowStockItems.map((item, idx) => {
                      const isCritical = item.quantity <= 0;
                      return (
                        <div 
                          key={idx} 
                          className={`flex items-center gap-3 p-3 rounded-xl border ${
                            isCritical 
                              ? 'bg-rose-500/10 border-rose-500/20' 
                              : 'bg-amber-500/10 border-amber-500/20'
                          }`}
                        >
                          <AlertTriangle className={`w-5 h-5 shrink-0 ${isCritical ? 'text-rose-500' : 'text-amber-500'}`} />
                          <div>
                            <p className={`text-xs font-bold ${isCritical ? 'text-rose-950 dark:text-rose-400' : 'text-amber-950 dark:text-amber-400'}`}>
                              {isCritical ? 'Rupture Critique' : 'Seuil de Sécurité Atteint'}
                            </p>
                            <p className={`text-[11px] font-semibold ${isCritical ? 'text-rose-700 dark:text-rose-500' : 'text-amber-700 dark:text-amber-500'}`}>
                              {item.product_name} ({item.quantity} restant{item.quantity > 1 ? 's' : ''}, seuil: {item.low_stock_threshold})
                            </p>
                          </div>
                        </div>
                      );
                    })
                  )}
                </div>
              </div>
            </div>
          </div>
        </>
      )}
    </div>
  );
}

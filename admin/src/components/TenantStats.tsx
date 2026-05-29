import { useState, useEffect, useMemo } from "react";
import { 
  TrendingUp, 
  ShoppingCart, 
  AlertTriangle, 
  Package, 
  PieChart,
  CreditCard
} from "lucide-react";
import { api, type Sale, type StockItem, type Product, type Category } from "../lib/api";
import Badge from "./Badge";

interface TenantStatsProps {
  tenantId: string;
}

export function TenantStats({ tenantId }: TenantStatsProps) {
  const [range, setRange] = useState<'30' | '90' | '365' | 'custom'>('30');
  const [startDate, setStartDate] = useState(() => {
    const d = new Date();
    d.setDate(d.getDate() - 30);
    return d.toISOString().split('T')[0];
  });
  const [endDate, setEndDate] = useState(() => new Date().toISOString().split('T')[0]);

  const [sales, setSales] = useState<Sale[]>([]);
  const [stockItems, setStockItems] = useState<StockItem[]>([]);
  const [products, setProducts] = useState<Product[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function loadStats() {
      setLoading(true);
      try {
        // We fetch a larger set for stats, but backend export might be capped or take time.
        // For admin dashboard, we'll try to fetch up to 1000 items if allowed.
        const [salesRes, stockRes, prodRes, catRes] = await Promise.all([
          api.sales.export({ 
            tenant_id: tenantId, 
            start_date: range === 'custom' ? startDate : undefined,
            end_date: range === 'custom' ? endDate : undefined,
            format: 'json' 
          }).catch(() => []), // Fallback to empty if export not available or fails
          api.stock.listItems({ tenant_id: tenantId, per_page: 1000 }),
          api.products.list({ tenant_id: tenantId, per_page: 1000 }),
          api.categories.list({ tenant_id: tenantId, per_page: 100 })
        ]);

        setSales(Array.isArray(salesRes) ? salesRes : []);
        setStockItems(stockRes.data || []);
        setProducts(prodRes.data || []);
        setCategories(catRes.data || []);
      } catch (error) {
        console.error("Failed to load tenant stats:", error);
      } finally {
        setLoading(false);
      }
    }
    loadStats();
  }, [tenantId, range, startDate, endDate]);

  const filteredSales = useMemo(() => {
    const completed = sales.filter(s => s.status === 'completed');
    if (range === 'custom') return completed; // Already filtered by API start/end
    
    const cutoff = new Date();
    cutoff.setDate(cutoff.getDate() - parseInt(range));
    return completed.filter(s => new Date(s.sold_at) >= cutoff);
  }, [sales, range]);

  // --- Calculations ---
  
  const totalRevenue = filteredSales.reduce((acc: number, s: Sale) => acc + s.total, 0);
  const transactionCount = filteredSales.length;
  
  const lowStockItems = stockItems.filter((item: StockItem) => item.quantity <= item.low_stock_threshold);
  const outOfStockItems = stockItems.filter((item: StockItem) => item.quantity <= 0);

  // Financial CurveData
  const curveData = useMemo(() => {
    const dailyMap: Record<string, number> = {};
    const days = range === 'custom' 
      ? Math.ceil((new Date(endDate).getTime() - new Date(startDate).getTime()) / 86400000)
      : parseInt(range);
    
    const start = range === 'custom' ? new Date(startDate) : new Date();
    if (range !== 'custom') start.setDate(start.getDate() - days);

    for (let i = 0; i <= days; i++) {
      const d = new Date(start);
      d.setDate(d.getDate() + i);
      dailyMap[d.toISOString().split('T')[0]] = 0;
    }

    filteredSales.forEach((s: Sale) => {
      const date = s.sold_at.split('T')[0];
      if (dailyMap[date] !== undefined) dailyMap[date] += s.total;
    });

    return Object.entries(dailyMap).sort((a,b) => a[0].localeCompare(b[0]));
  }, [filteredSales, range, startDate, endDate]);

  // Categories Distribution
  const categoryStats = useMemo(() => {
    const map: Record<string, number> = {};
    const productToCat: Record<string, string> = {};
    products.forEach((p: Product) => {
      const cat = categories.find((c: Category) => c.id === p.category_id)?.name || "Général";
      productToCat[p.id] = cat;
    });

    filteredSales.forEach((sale: Sale) => {
      sale.items.forEach((item: any) => {
        const cat = productToCat[item.product_id] || "Général";
        map[cat] = (map[cat] || 0) + item.line_total;
      });
    });

    return Object.entries(map)
      .map(([name, total]) => ({ name, total, percent: totalRevenue > 0 ? (total / totalRevenue) * 100 : 0 }))
      .sort((a,b) => b.total - a.total)
      .slice(0, 5);
  }, [filteredSales, products, categories, totalRevenue]);

  // Payment Methods
  const paymentStats = useMemo(() => {
    const methods: Record<string, number> = { cash: 0, card: 0, mobile_money: 0 };
    filteredSales.forEach((s: Sale) => {
      methods[s.payment_method] = (methods[s.payment_method] || 0) + 1;
    });
    const total = Object.values(methods).reduce((a,b) => a+b, 0);
    return Object.entries(methods).map(([name, count]) => ({
      name: name === 'cash' ? 'Espèces' : name === 'card' ? 'Carte' : 'Mobile Money',
      count,
      percent: total > 0 ? (count / total) * 100 : 0
    }));
  }, [filteredSales]);

  const formatCurrency = (val: number) => {
    return new Intl.NumberFormat('fr-FR', { style: 'currency', currency: 'XAF', minimumFractionDigits: 0 }).format(val).replace('FCFA', 'F');
  };

  if (loading) {
    return (
      <div className="py-12 flex flex-col items-center justify-center text-muted-foreground">
        <div className="w-8 h-8 border-4 border-primary border-t-transparent rounded-full animate-spin mb-4" />
        <p className="text-sm font-medium">Analyse des données du tenant...</p>
      </div>
    );
  }

  // Chart Helpers
  const svgWidth = 600;
  const svgHeight = 160;
  const padding = 20;
  const maxVal = Math.max(...curveData.map(d => d[1]), 1);
  const points = curveData.map((d: [string, number], i: number) => {
    const x = padding + (i / Math.max(1, curveData.length - 1)) * (svgWidth - 2 * padding);
    const y = svgHeight - padding - (d[1] / maxVal) * (svgHeight - 2 * padding);
    return { x, y };
  });
  const pathD = points.length > 0 ? `M ${points.map((p: {x: number, y: number}) => `${p.x},${p.y}`).join(' L ')}` : '';

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-bold flex items-center gap-2">
          <TrendingUp className="w-5 h-5 text-primary" />
          Statistiques Opérationnelles
        </h2>
        <div className="flex items-center gap-2">
          <select 
            value={range} 
            onChange={(e) => setRange(e.target.value as any)}
            className="text-xs border border-border rounded-lg bg-background px-2 py-1.5 font-semibold focus:outline-none"
          >
            <option value="30">30 derniers jours</option>
            <option value="90">90 derniers jours</option>
            <option value="365">12 derniers mois</option>
            <option value="custom">Période personnalisée</option>
          </select>
          {range === 'custom' && (
            <div className="flex items-center gap-1">
              <input type="date" value={startDate} onChange={e => setStartDate(e.target.value)} className="text-[10px] border border-border rounded-lg p-1" />
              <span className="text-[10px] text-muted-foreground">-</span>
              <input type="date" value={endDate} onChange={e => setEndDate(e.target.value)} className="text-[10px] border border-border rounded-lg p-1" />
            </div>
          )}
        </div>
      </div>

      {/* Primary Metrics */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <StatCard 
          label="Chiffre d'Affaires" 
          value={formatCurrency(totalRevenue)} 
          icon={<TrendingUp className="w-4 h-4" />} 
          trend="Période sélectionnée" 
          color="primary"
        />
        <StatCard 
          label="Transactions" 
          value={transactionCount.toString()} 
          icon={<ShoppingCart className="w-4 h-4" />} 
          trend="Ventes validées" 
          color="blue"
        />
        <StatCard 
          label="Alerte Stocks" 
          value={lowStockItems.length.toString()} 
          icon={<AlertTriangle className="w-4 h-4" />} 
          trend={`${outOfStockItems.length} ruptures critiques`} 
          color={lowStockItems.length > 0 ? "amber" : "emerald"}
        />
        <StatCard 
          label="Produits" 
          value={products.length.toString()} 
          icon={<Package className="w-4 h-4" />} 
          trend="Catalogue actif" 
          color="zinc"
        />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Financial Curve */}
        <div className="lg:col-span-2 bg-card border border-border rounded-2xl p-5 shadow-sm">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-sm font-bold uppercase tracking-wider text-muted-foreground">Évolution Financière</h3>
            <span className="text-[10px] font-bold text-muted-foreground">{curveData[0]?.[0]} &rarr; {curveData[curveData.length-1]?.[0]}</span>
          </div>
          <div className="h-[180px] w-full relative">
            <svg viewBox={`0 0 ${svgWidth} ${svgHeight}`} className="w-full h-full">
              <defs>
                <linearGradient id="adminChartGrad" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stopColor="var(--primary)" stopOpacity="0.2" />
                  <stop offset="100%" stopColor="var(--primary)" stopOpacity="0" />
                </linearGradient>
              </defs>
              {pathD && (
                <>
                  <path d={pathD} fill="none" stroke="var(--primary)" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round" />
                  <path d={`${pathD} L ${points[points.length-1].x},${svgHeight-padding} L ${points[0].x},${svgHeight-padding} Z`} fill="url(#adminChartGrad)" />
                  {points.map((p, i) => (
                    <circle key={i} cx={p.x} cy={p.y} r="3" fill="var(--card)" stroke="var(--primary)" strokeWidth="2" />
                  ))}
                </>
              )}
            </svg>
          </div>
        </div>

        {/* Shortages List */}
        <div className="bg-card border border-border rounded-2xl p-5 shadow-sm">
          <h3 className="text-sm font-bold uppercase tracking-wider text-muted-foreground mb-4">Ruptures & Alertes</h3>
          <div className="space-y-3 max-h-[180px] overflow-y-auto pr-1">
            {lowStockItems.length === 0 ? (
              <p className="text-xs text-center py-8 text-muted-foreground">Aucune alerte de stock.</p>
            ) : (
              lowStockItems.map((item: StockItem, i: number) => (
                <div key={i} className="flex items-center justify-between p-2 rounded-xl border border-border/50 bg-muted/20">
                  <div className="min-w-0">
                    <p className="text-[11px] font-bold truncate">{item.product_name}</p>
                    <p className="text-[9px] text-muted-foreground">Stock: {item.quantity} (Seuil: {item.low_stock_threshold})</p>
                  </div>
                  <Badge 
                    label={item.quantity <= 0 ? "Rupture" : "Critique"} 
                    tone={item.quantity <= 0 ? "red" : "amber"} 
                  />
                </div>
              ))
            )}
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Categories Distribution */}
        <div className="bg-card border border-border rounded-2xl p-5 shadow-sm">
          <h3 className="text-sm font-bold uppercase tracking-wider text-muted-foreground mb-4 flex items-center gap-2">
            <PieChart className="w-4 h-4" />
            Répartition par Catégorie
          </h3>
          <div className="space-y-4">
            {categoryStats.length === 0 ? (
              <p className="text-xs text-center py-6 text-muted-foreground">Aucune donnée disponible.</p>
            ) : (
              categoryStats.map((cat: {name: string, percent: number, total: number}, i: number) => (
                <div key={i} className="space-y-1">
                  <div className="flex justify-between text-xs font-bold">
                    <span>{cat.name}</span>
                    <span>{Math.round(cat.percent)}% ({formatCurrency(cat.total)})</span>
                  </div>
                  <div className="h-1.5 w-full bg-muted rounded-full overflow-hidden">
                    <div 
                      className="h-full bg-primary rounded-full transition-all duration-500" 
                      style={{ width: `${cat.percent}%`, opacity: 1 - i * 0.15 }} 
                    />
                  </div>
                </div>
              ))
            )}
          </div>
        </div>

        {/* Payment Methods */}
        <div className="bg-card border border-border rounded-2xl p-5 shadow-sm">
          <h3 className="text-sm font-bold uppercase tracking-wider text-muted-foreground mb-4 flex items-center gap-2">
            <CreditCard className="w-4 h-4" />
            Modes de Paiement
          </h3>
          <div className="flex items-center justify-around h-full py-2">
            {paymentStats.map((p: {name: string, count: number, percent: number}, i: number) => (
              <div key={i} className="text-center group">
                <div className="relative w-16 h-16 mb-2 mx-auto">
                  <svg className="w-full h-full rotate-[-90deg]">
                    <circle cx="32" cy="32" r="28" fill="none" stroke="currentColor" strokeWidth="4" className="text-muted/30" />
                    <circle 
                      cx="32" cy="32" r="28" fill="none" stroke="currentColor" strokeWidth="4" 
                      strokeDasharray={`${28 * 2 * Math.PI}`}
                      strokeDashoffset={`${28 * 2 * Math.PI * (1 - p.percent / 100)}`}
                      className={p.name === 'Espèces' ? 'text-primary' : p.name === 'Carte' ? 'text-blue-500' : 'text-amber-500'}
                    />
                  </svg>
                  <div className="absolute inset-0 flex items-center justify-center text-[11px] font-black">
                    {Math.round(p.percent)}%
                  </div>
                </div>
                <p className="text-[10px] font-bold uppercase text-muted-foreground group-hover:text-foreground transition-colors">{p.name}</p>
                <p className="text-[9px] font-semibold text-muted-foreground/60">{p.count} trans.</p>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

function StatCard({ label, value, icon, trend, color }: { 
  label: string; 
  value: string; 
  icon: React.ReactNode; 
  trend: string;
  color: "primary" | "blue" | "amber" | "emerald" | "zinc";
}) {
  const colors = {
    primary: "bg-primary/10 text-primary",
    blue: "bg-blue-500/10 text-blue-500",
    amber: "bg-amber-500/10 text-amber-500",
    emerald: "bg-emerald-500/10 text-emerald-500",
    zinc: "bg-zinc-500/10 text-zinc-500",
  };

  return (
    <div className="bg-card border border-border rounded-2xl p-4 shadow-sm hover:shadow-md transition-shadow">
      <div className="flex items-center justify-between mb-2">
        <span className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground">{label}</span>
        <div className={`p-2 rounded-xl ${colors[color]}`}>
          {icon}
        </div>
      </div>
      <div>
        <h3 className="text-xl font-black tracking-tight">{value}</h3>
        <p className="text-[10px] font-semibold text-muted-foreground mt-1 truncate">{trend}</p>
      </div>
    </div>
  );
}

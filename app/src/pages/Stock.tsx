import { useState, useEffect } from 'react';
import { 
  Search, 
  Download
} from 'lucide-react';
import { api, StockItem, Category } from '../services/api';

export default function Stock() {
  const [stockItems, setStockItems] = useState<StockItem[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [search, setSearch] = useState('');
  const [categoryFilter, setCategoryFilter] = useState('all');
  const [statusFilter, setStatusFilter] = useState<'all' | 'ok' | 'low' | 'out'>('all');
  const [loading, setLoading] = useState(true);

  const [adjustmentModal, setAdjustmentModal] = useState<{ 
    open: boolean; 
    item: StockItem | null; 
    qty: string; 
    reason: string; 
    type: 'add' | 'remove' 
  }>({
    open: false,
    item: null,
    qty: '',
    reason: '',
    type: 'add',
  });

  const loadData = async () => {
    setLoading(true);
    try {
      const filterCat = categoryFilter === 'all' ? '' : categoryFilter;
      const [stockRes, catRes] = await Promise.all([
        api.stock.listItems('', false, filterCat, 1, 1000),
        api.categories.list('', 1, 100)
      ]);
      setStockItems(stockRes.data || []);
      setCategories(catRes.data || []);
    } catch (e) {
      console.error("Failed to load stock data:", e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();
  }, [categoryFilter]);

  const getStockStatus = (item: StockItem) => {
    if (item.quantity <= 0) return 'out'; // Out of stock
    if (item.quantity <= item.low_stock_threshold) return 'low'; // Low stock threshold
    return 'ok';
  };

  const handleAdjustStock = async () => {
    const { item, qty, reason, type } = adjustmentModal;
    if (!item || !qty || !reason) return;

    const amount = parseFloat(qty) || 0;
    if (amount <= 0) {
      alert("La quantité doit être supérieure à zéro.");
      return;
    }

    try {
      // Determine movement type
      let movementType: 'purchase' | 'adjustment' | 'return' | 'loss' = 'adjustment';
      if (type === 'add') {
        if (reason === 'Livraison fournisseur') movementType = 'purchase';
        else if (reason === 'Retour client') movementType = 'return';
      } else {
        if (reason === 'Périssable / Expiré' || reason === 'Produit défectueux / endommagé') movementType = 'loss';
      }

      const qtyChange = type === 'add' ? amount : -amount;

      await api.stock.createMovement({
        product_id: item.product_id,
        movement_type: movementType,
        quantity_change: qtyChange,
        note: reason,
      });

      setAdjustmentModal({ open: false, item: null, qty: '', reason: '', type: 'add' });
      loadData();
    } catch (err: any) {
      console.error(err);
      alert(err.message || "Erreur lors de l'ajustement du stock.");
    }
  };

  const handleExportCSV = () => {
    const headers = ['ID Fiche', 'Nom Produit', 'Quantité', 'Seuil Alerte', 'Batch', 'Péremption', 'Dernière MAJ'];
    const rows = stockItems.map(item => [
      item.id, 
      item.product_name, 
      item.quantity, 
      item.low_stock_threshold, 
      item.batch_number || '', 
      item.expiry_date || '', 
      new Date(item.updated_at).toLocaleString('fr-FR')
    ]);
    
    const csvContent = "data:text/csv;charset=utf-8," 
      + [headers.join(','), ...rows.map(e => e.join(','))].join('\n');
      
    const encodedUri = encodeURI(csvContent);
    const link = document.createElement("a");
    link.setAttribute("href", encodedUri);
    link.setAttribute("download", `rapport_stock_${new Date().toISOString().slice(0,10)}.csv`);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  };

  // Filter items
  const filteredItems = stockItems.filter(item => {
    const matchesSearch = item.product_name.toLowerCase().includes(search.toLowerCase()) || 
                          (item.batch_number && item.batch_number.includes(search));
    const status = getStockStatus(item);
    const matchesStatus = statusFilter === 'all' || status === statusFilter;
    return matchesSearch && matchesStatus;
  });

  return (
    <div className="space-y-6 animate-slide-up select-none">
      
      {/* Top action grid & Filters */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <input
            type="text"
            placeholder="Rechercher par nom ou numéro de lot..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full pl-11 pr-4 py-2.5 bg-card border border-border rounded-xl focus:outline-none focus:ring-1 focus:ring-primary text-xs font-semibold text-foreground"
          />
        </div>

        <div className="flex flex-wrap items-center gap-3">
          {/* Category Filter */}
          <select 
            value={categoryFilter}
            onChange={(e) => setCategoryFilter(e.target.value)}
            className="px-3.5 py-2.5 rounded-xl border border-border bg-card text-xs font-bold text-foreground focus:outline-none"
          >
            <option value="all">Toutes Catégories</option>
            {categories.map(cat => (
              <option key={cat.id} value={cat.id}>{cat.name}</option>
            ))}
          </select>

          {/* Status Filter */}
          <div className="flex bg-card border border-border p-1 rounded-xl shadow-sm">
            {(['all', 'ok', 'low', 'out'] as const).map((status) => (
              <button
                key={status}
                onClick={() => setStatusFilter(status)}
                className={`px-3 py-1.5 rounded-lg text-[10px] font-extrabold uppercase transition-all cursor-pointer ${
                  statusFilter === status
                    ? 'bg-primary dark:bg-blue-600 text-primary-foreground shadow-sm'
                    : 'text-muted-foreground hover:text-foreground'
                }`}
              >
                {status === 'all' ? 'Tous' : status === 'ok' ? 'Dispo' : status === 'low' ? 'Seuil' : 'Rupture'}
              </button>
            ))}
          </div>

          {/* Export Button */}
          <button
            onClick={handleExportCSV}
            className="flex items-center gap-1.5 px-4 py-2.5 rounded-xl bg-card border border-border text-xs font-bold hover:bg-accent text-foreground transition-all cursor-pointer shadow-sm"
          >
            <Download className="w-4 h-4 text-primary dark:text-blue-600" />
            <span>Exporter</span>
          </button>
        </div>
      </div>

      {/* Main Stock Table */}
      {loading ? (
        <div className="py-20 text-center text-muted-foreground font-semibold">
          Chargement du stock...
        </div>
      ) : (
        <div className="bg-card border border-border rounded-2xl shadow-sm overflow-hidden">
          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs border-collapse">
              <thead>
                <tr className="border-b border-border text-muted-foreground font-bold bg-muted/10">
                  <th className="py-3.5 px-6">Produit</th>
                  <th className="py-3.5 px-4 text-center">Quantité</th>
                  <th className="py-3.5 px-4 text-center">Seuil Alerte</th>
                  <th className="py-3.5 px-4">Lot / Batch</th>
                  <th className="py-3.5 px-4">Péremption</th>
                  <th className="py-3.5 px-4">Statut</th>
                  <th className="py-3.5 px-4">Mise à jour</th>
                  <th className="py-3.5 px-6 text-right">Actions</th>
                </tr>
              </thead>
              <tbody>
                {filteredItems.length === 0 ? (
                  <tr>
                    <td colSpan={8} className="py-8 text-center text-muted-foreground font-semibold">
                      Aucun article en stock.
                    </td>
                  </tr>
                ) : (
                  filteredItems.map((item) => {
                    const status = getStockStatus(item);
                    return (
                      <tr key={item.id} className="border-b border-border/50 hover:bg-accent/20 transition-colors font-medium">
                        <td className="py-4 px-6 font-bold text-foreground">{item.product_name}</td>
                        <td className="py-4 px-4 text-center font-extrabold text-foreground">{item.quantity}</td>
                        <td className="py-4 px-4 text-center font-semibold text-muted-foreground">{item.low_stock_threshold}</td>
                        <td className="py-4 px-4 font-mono font-bold text-muted-foreground">{item.batch_number || 'N/A'}</td>
                        <td className="py-4 px-4 font-semibold text-muted-foreground">
                          {item.expiry_date ? new Date(item.expiry_date).toLocaleDateString('fr-FR') : 'N/A'}
                        </td>
                        <td className="py-4 px-4">
                          <span className={`inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wider ${
                            status === 'ok' && 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400'
                          } ${
                            status === 'low' && 'bg-amber-500/10 text-amber-600 dark:text-amber-400'
                          } ${
                            status === 'out' && 'bg-rose-500/10 text-rose-600 dark:text-rose-400'
                          }`}>
                            {status === 'ok' ? 'En Stock' : status === 'low' ? 'Alerte' : 'Rupture'}
                          </span>
                        </td>
                        <td className="py-4 px-4 text-muted-foreground font-semibold">
                          {new Date(item.updated_at).toLocaleString('fr-FR', { dateStyle: 'short', timeStyle: 'short' })}
                        </td>
                        <td className="py-4 px-6 text-right space-x-1.5">
                          <button
                            onClick={() => setAdjustmentModal({ open: true, item, qty: '', reason: '', type: 'add' })}
                            className="px-2 py-1 bg-emerald-500/10 hover:bg-emerald-500 hover:text-white rounded-lg text-emerald-600 text-[10px] font-bold transition-all cursor-pointer"
                          >
                            Ajuster (+)
                          </button>
                          <button
                            onClick={() => setAdjustmentModal({ open: true, item, qty: '', reason: '', type: 'remove' })}
                            className="px-2 py-1 bg-rose-500/10 hover:bg-rose-500 hover:text-white rounded-lg text-rose-600 text-[10px] font-bold transition-all cursor-pointer"
                          >
                            Ajuster (-)
                          </button>
                        </td>
                      </tr>
                    );
                  })
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Manual Stock Adjustment Dialog */}
      {adjustmentModal.open && adjustmentModal.item && (
        <div className="fixed inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-scale-in">
          <div className="bg-card border border-border w-full max-w-sm rounded-3xl shadow-2xl p-6 relative">
            <h3 className="font-extrabold text-base text-foreground mb-1">
              Ajustement Manuel de Stock
            </h3>
            <p className="text-[11px] text-muted-foreground font-semibold mb-4">
              Produit : {adjustmentModal.item.product_name}
            </p>

            <div className="space-y-4">
              {/* Type Switcher */}
              <div className="flex bg-accent/40 border border-border p-1 rounded-xl">
                <button
                  onClick={() => setAdjustmentModal(m => ({ ...m, type: 'add' }))}
                  className={`flex-1 py-1.5 rounded-lg text-xs font-bold transition-all cursor-pointer ${
                    adjustmentModal.type === 'add'
                      ? 'bg-emerald-500 text-white shadow-sm'
                      : 'text-muted-foreground'
                  }`}
                >
                  Entrée (+)
                </button>
                <button
                  onClick={() => setAdjustmentModal(m => ({ ...m, type: 'remove' }))}
                  className={`flex-1 py-1.5 rounded-lg text-xs font-bold transition-all cursor-pointer ${
                    adjustmentModal.type === 'remove'
                      ? 'bg-rose-500 text-white shadow-sm'
                      : 'text-muted-foreground'
                  }`}
                >
                  Sortie (-)
                </button>
              </div>

              {/* Quantity Input */}
              <div>
                <label className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">QUANTITÉ</label>
                <input
                  type="number"
                  placeholder="0"
                  value={adjustmentModal.qty}
                  onChange={(e) => setAdjustmentModal(m => ({ ...m, qty: e.target.value }))}
                  className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-bold focus:outline-none focus:ring-1 focus:ring-primary text-foreground"
                />
              </div>

              {/* Reason Selection */}
              <div>
                <label className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">MOTIF DE L'AJUSTEMENT</label>
                <select
                  value={adjustmentModal.reason}
                  onChange={(e) => setAdjustmentModal(m => ({ ...m, reason: e.target.value }))}
                  className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-bold focus:outline-none text-foreground"
                >
                  <option value="">Sélectionnez un motif...</option>
                  <option value="Livraison fournisseur">Livraison fournisseur</option>
                  <option value="Inventaire physique">Inventaire physique</option>
                  <option value="Périssable / Expiré">Périssable / Expiré</option>
                  <option value="Produit défectueux / endommagé">Produit défectueux / endommagé</option>
                  <option value="Retour client">Retour client</option>
                </select>
              </div>
            </div>

            {/* Modal actions */}
            <div className="flex gap-3 mt-6">
              <button
                onClick={() => setAdjustmentModal({ open: false, item: null, qty: '', reason: '', type: 'add' })}
                className="flex-1 py-2.5 rounded-xl border border-border bg-card hover:bg-accent text-foreground text-xs font-bold cursor-pointer"
              >
                Annuler
              </button>
              <button
                onClick={handleAdjustStock}
                disabled={!adjustmentModal.qty || !adjustmentModal.reason}
                className={`flex-1 py-2.5 rounded-xl text-xs font-bold cursor-pointer ${
                  !adjustmentModal.qty || !adjustmentModal.reason
                    ? 'bg-muted text-muted-foreground cursor-not-allowed'
                    : adjustmentModal.type === 'add'
                      ? 'bg-emerald-500 text-white hover:bg-emerald-600'
                      : 'bg-rose-500 text-white hover:bg-rose-600'
                }`}
              >
                Confirmer
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

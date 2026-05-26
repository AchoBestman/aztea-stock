import { useState, useEffect } from 'react';
import { 
  Plus, 
  Search, 
  Edit, 
  Trash2, 
  Barcode, 
  AlertCircle,
  PackageCheck
} from 'lucide-react';
import { api, Category, Product } from '../services/api';
import { usePermissions } from '../hooks/usePermissions';
import { toast } from 'react-hot-toast';
import { ConfirmModal } from '../components/ConfirmModal';

export default function Products() {
  const { canManageProduct, has } = usePermissions();
  const canEdit = canManageProduct();
  const canRead = has('can_read_product');

  const [products, setProducts] = useState<Product[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState('');
  const [categoryFilter, setCategoryFilter] = useState('all');

  // Modals state
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);
  const [editingProduct, setEditingProduct] = useState<Product | null>(null);

  // Delete modal state
  const [deleteModalOpen, setDeleteModalOpen] = useState(false);
  const [productToDelete, setProductToDelete] = useState<Product | null>(null);

  // Form states
  const [newProduct, setNewProduct] = useState({
    name: '',
    barcode: '',
    category_id: '',
    buyingPrice: '',
    sellingPrice: '',
    stockThreshold: '10',
    unit: 'boite',
  });

  const loadData = async () => {
    setLoading(true);
    try {
      const [prodRes, catRes] = await Promise.all([
        api.products.list('', '', 1, 1000),
        api.categories.list('', 1, 1000)
      ]);
      
      let fetchedCategories = catRes.data || [];
      
      // Auto seed default categories if database is completely empty
      if (fetchedCategories.length === 0 && canEdit) {
        const defaults = ["Pharmacie", "Alimentation", "Hygiène"];
        await Promise.all(defaults.map(name => api.categories.create(name)));
        const reloadedCats = await api.categories.list('', 1, 1000);
        fetchedCategories = reloadedCats.data || [];
      }

      setCategories(fetchedCategories);
      setProducts(prodRes.data || []);
      
      // Set default category in form if available
      if (fetchedCategories.length > 0) {
        setNewProduct(m => ({ ...m, category_id: fetchedCategories[0].id }));
      }
    } catch (e) {
      console.error("Failed to load products/categories:", e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();
  }, []);

  const handleAddProduct = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!canEdit) return;
    if (!newProduct.name.trim() || !newProduct.barcode.trim()) return;

    try {
      const p = await api.products.create({
        name: newProduct.name,
        barcode: newProduct.barcode,
        category_id: newProduct.category_id || undefined,
        unit: newProduct.unit,
        purchase_price: parseFloat(newProduct.buyingPrice) || 0,
        selling_price: parseFloat(newProduct.sellingPrice) || 0,
        tax_rate: 0,
      });

      // Auto-create initial stock card for the product
      await api.stock.createItem({
        product_id: p.id,
        quantity: 0,
        low_stock_threshold: parseInt(newProduct.stockThreshold, 10) || 10,
      });

      setIsAddModalOpen(false);
      setNewProduct({
        name: '',
        barcode: '',
        category_id: categories[0]?.id || '',
        buyingPrice: '',
        sellingPrice: '',
        stockThreshold: '10',
        unit: 'boite',
      });
      loadData();
      toast.success("Produit ajouté avec succès.");
    } catch (err: any) {
      console.error(err);
      toast.error(err.message || "Erreur lors de la création du produit.");
    }
  };

  const handleEditProduct = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!canEdit || !editingProduct) return;

    try {
      await api.products.update(editingProduct.id, {
        name: editingProduct.name,
        barcode: editingProduct.barcode || null,
        category_id: editingProduct.category_id || null,
        unit: editingProduct.unit,
        purchase_price: editingProduct.purchase_price,
        selling_price: editingProduct.selling_price,
      });

      setEditingProduct(null);
      loadData();
      toast.success("Produit mis à jour avec succès.");
    } catch (err: any) {
      console.error(err);
      toast.error(err.message || "Erreur lors de la modification du produit.");
    }
  };

  const confirmDeleteProduct = (prod: Product) => {
    if (!canEdit) return;
    setProductToDelete(prod);
    setDeleteModalOpen(true);
  };

  const handleDeleteProduct = async () => {
    if (!productToDelete) return;
    try {
      await api.products.delete(productToDelete.id);
      setDeleteModalOpen(false);
      setProductToDelete(null);
      loadData();
      toast.success("Le produit a été supprimé.");
    } catch (err: any) {
      console.error(err);
      toast.error(err.message || "Erreur lors de la suppression du produit.");
    }
  };

  const filteredProducts = products.filter(p => {
    const matchesSearch = p.name.toLowerCase().includes(search.toLowerCase()) || (p.barcode && p.barcode.includes(search));
    const matchesCategory = categoryFilter === 'all' || p.category_id === categoryFilter;
    return matchesSearch && matchesCategory;
  });

  return (
    <div className="space-y-6 animate-slide-up select-none">
      
      {/* Top search & Add layout */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <input
            type="text"
            placeholder="Rechercher un produit par nom ou code-barres..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full pl-11 pr-4 py-2.5 bg-card border border-border rounded-xl focus:outline-none focus:ring-1 focus:ring-primary text-xs font-semibold text-foreground"
          />
        </div>

        <div className="flex items-center gap-3">
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

          <button
            onClick={() => setIsAddModalOpen(true)}
            disabled={!canEdit}
            className={`flex items-center gap-1.5 px-4 py-2.5 rounded-xl font-bold text-xs shadow-md transition-all ${
              !canEdit
                ? 'bg-muted text-muted-foreground cursor-not-allowed opacity-60'
                : 'bg-primary dark:bg-blue-600 text-primary-foreground hover:bg-opacity-95 cursor-pointer'
            }`}
          >
            <Plus className="w-4 h-4" />
            <span>Nouveau Produit</span>
          </button>
        </div>
      </div>

      {canRead && !canEdit && (
        <div className="p-3.5 rounded-xl bg-amber-500/10 text-amber-600 dark:text-amber-400 border border-amber-500/20 text-xs font-semibold flex items-center gap-2">
          <AlertCircle className="w-4 h-4" />
          <span>Mode lecture seule : vous n&apos;avez pas la permission de modifier le catalogue produits.</span>
        </div>
      )}

      {/* Main product listing (as a table list) */}
      {loading ? (
        <div className="py-20 text-center text-muted-foreground font-semibold">
          Chargement du catalogue...
        </div>
      ) : (
        <div className="bg-card border border-border rounded-2xl shadow-sm overflow-hidden">
          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs border-collapse">
              <thead>
                <tr className="border-b border-border text-muted-foreground font-bold bg-muted/10">
                  <th className="py-3.5 px-6">Désignation</th>
                  <th className="py-3.5 px-4">Code-barres</th>
                  <th className="py-3.5 px-4">Catégorie</th>
                  <th className="py-3.5 px-4">Format / Unité</th>
                  <th className="py-3.5 px-4 text-right">P. Achat (F)</th>
                  <th className="py-3.5 px-4 text-right text-primary dark:text-blue-600">P. Vente (F)</th>
                  <th className="py-3.5 px-6 text-right">Actions</th>
                </tr>
              </thead>
              <tbody>
                {filteredProducts.length === 0 ? (
                  <tr>
                    <td colSpan={7} className="py-8 text-center text-muted-foreground font-semibold">
                      Aucun produit trouvé.
                    </td>
                  </tr>
                ) : (
                  filteredProducts.map((prod) => (
                    <tr key={prod.id} className="border-b border-border/50 hover:bg-accent/20 transition-colors font-medium">
                      <td className="py-4 px-6 font-bold text-foreground flex items-center gap-2">
                        <PackageCheck className="w-4 h-4 text-primary dark:text-blue-600 shrink-0" />
                        <span>{prod.name}</span>
                      </td>
                      <td className="py-4 px-4 font-mono text-muted-foreground font-semibold">
                        <span className="flex items-center gap-1">
                          <Barcode className="w-3.5 h-3.5 opacity-60" />
                          {prod.barcode || 'N/A'}
                        </span>
                      </td>
                      <td className="py-4 px-4">
                        <span className="px-2 py-0.5 rounded bg-primary/10 dark:text-blue-600 text-primary text-[10px] font-bold">
                          {prod.category_name || 'Général'}
                        </span>
                      </td>
                      <td className="py-4 px-4 text-muted-foreground font-semibold capitalize">
                        {prod.unit}
                      </td>
                      <td className="py-4 px-4 text-right font-bold text-foreground">
                        {prod.purchase_price.toLocaleString('fr-FR')} F
                      </td>
                      <td className="py-4 px-4 text-right font-extrabold text-primary dark:text-blue-600">
                        {prod.selling_price.toLocaleString('fr-FR')} F
                      </td>
                      <td className="py-4 px-6 text-right">
                        <div className="flex justify-end gap-1.5">
                          <button
                            onClick={() => setEditingProduct(prod)}
                            disabled={!canEdit}
                            className={`w-7 h-7 rounded-lg border flex items-center justify-center transition-colors ${
                              !canEdit
                                ? 'border-border text-muted-foreground/40 cursor-not-allowed'
                                : 'border-border hover:bg-accent text-foreground cursor-pointer'
                            }`}
                            title="Modifier"
                          >
                            <Edit className="w-3.5 h-3.5" />
                          </button>
                          <button
                            onClick={() => confirmDeleteProduct(prod)}
                            disabled={!canEdit}
                            className={`w-7 h-7 rounded-lg flex items-center justify-center transition-colors ${
                              !canEdit
                                ? 'bg-muted text-muted-foreground/30 cursor-not-allowed'
                                : 'bg-rose-500/10 hover:bg-rose-500 hover:text-white text-rose-500 cursor-pointer'
                            }`}
                            title="Supprimer"
                          >
                            <Trash2 className="w-3.5 h-3.5" />
                          </button>
                        </div>
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Add Product Modal - Centered and Styled */}
      {isAddModalOpen && (
        <div className="fixed inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-scale-in">
          <div className="bg-card border border-border w-full max-w-md rounded-3xl shadow-2xl p-6 relative">
            <h3 className="font-extrabold text-base text-foreground mb-4">Créer un Nouveau Produit</h3>
            
            <form onSubmit={handleAddProduct} className="space-y-4">
              <div>
                <label className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Désignation *</label>
                <input
                  type="text"
                  required
                  placeholder="ex. Doliprane 500mg..."
                  value={newProduct.name}
                  onChange={(e) => setNewProduct(m => ({ ...m, name: e.target.value }))}
                  className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-semibold focus:outline-none focus:ring-1 focus:ring-primary text-foreground"
                />
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Catégorie</label>
                  <select
                    value={newProduct.category_id}
                    onChange={(e) => setNewProduct(m => ({ ...m, category_id: e.target.value }))}
                    className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-bold focus:outline-none text-foreground"
                  >
                    {categories.map(cat => (
                      <option key={cat.id} value={cat.id}>{cat.name}</option>
                    ))}
                  </select>
                </div>

                <div>
                  <label className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Code-barres *</label>
                  <input
                    type="text"
                    required
                    placeholder="34009300..."
                    value={newProduct.barcode}
                    onChange={(e) => setNewProduct(m => ({ ...m, barcode: e.target.value }))}
                    className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-semibold focus:outline-none focus:ring-1 focus:ring-primary text-foreground"
                  />
                </div>
              </div>

              <div className="grid grid-cols-3 gap-4">
                <div>
                  <label className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Achat (F) *</label>
                  <input
                    type="number"
                    required
                    placeholder="ex. 800"
                    value={newProduct.buyingPrice}
                    onChange={(e) => setNewProduct(m => ({ ...m, buyingPrice: e.target.value }))}
                    className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-bold focus:outline-none focus:ring-1 focus:ring-primary text-foreground"
                  />
                </div>

                <div>
                  <label className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Vente (F) *</label>
                  <input
                    type="number"
                    required
                    placeholder="ex. 1200"
                    value={newProduct.sellingPrice}
                    onChange={(e) => setNewProduct(m => ({ ...m, sellingPrice: e.target.value }))}
                    className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-bold focus:outline-none focus:ring-1 focus:ring-primary text-foreground"
                  />
                </div>

                <div>
                  <label className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Seuil Alerte</label>
                  <input
                    type="number"
                    placeholder="10"
                    value={newProduct.stockThreshold}
                    onChange={(e) => setNewProduct(m => ({ ...m, stockThreshold: e.target.value }))}
                    className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-bold focus:outline-none focus:ring-1 focus:ring-primary text-foreground"
                  />
                </div>
              </div>

              <div>
                <label className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Format / Unité</label>
                <select
                  value={newProduct.unit}
                  onChange={(e) => setNewProduct(m => ({ ...m, unit: e.target.value }))}
                  className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-bold focus:outline-none text-foreground font-semibold"
                >
                  <option value="boite">Boite</option>
                  <option value="flacon">Flacon</option>
                  <option value="ampoule">Ampoule</option>
                  <option value="unité">Unité individuelle</option>
                </select>
              </div>

              {/* Form buttons */}
              <div className="flex gap-3 pt-4">
                <button
                  type="button"
                  onClick={() => setIsAddModalOpen(false)}
                  className="flex-1 py-2.5 rounded-xl border border-border bg-card hover:bg-accent text-foreground text-xs font-bold cursor-pointer"
                >
                  Annuler
                </button>
                <button
                  type="submit"
                  className="flex-1 py-2.5 rounded-xl bg-primary dark:bg-blue-600 text-primary-foreground text-xs font-bold shadow-sm hover:bg-opacity-95 cursor-pointer"
                >
                  Ajouter Produit
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Edit Product Modal - Centered and Styled */}
      {editingProduct && (
        <div className="fixed inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-scale-in">
          <div className="bg-card border border-border w-full max-w-md rounded-3xl shadow-2xl p-6 relative">
            <h3 className="font-extrabold text-base text-foreground mb-4">Modifier le Produit</h3>
            
            <form onSubmit={handleEditProduct} className="space-y-4">
              <div>
                <label className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Désignation</label>
                <input
                  type="text"
                  required
                  value={editingProduct.name}
                  onChange={(e) => setEditingProduct(m => m ? ({ ...m, name: e.target.value }) : null)}
                  className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-semibold focus:outline-none focus:ring-1 focus:ring-primary text-foreground"
                />
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Catégorie</label>
                  <select
                    value={editingProduct.category_id || ''}
                    onChange={(e) => setEditingProduct(m => m ? ({ ...m, category_id: e.target.value }) : null)}
                    className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-bold focus:outline-none text-foreground"
                  >
                    <option value="">Aucune</option>
                    {categories.map(cat => (
                      <option key={cat.id} value={cat.id}>{cat.name}</option>
                    ))}
                  </select>
                </div>

                <div>
                  <label className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Code-barres</label>
                  <input
                    type="text"
                    required
                    value={editingProduct.barcode || ''}
                    onChange={(e) => setEditingProduct(m => m ? ({ ...m, barcode: e.target.value }) : null)}
                    className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-semibold focus:outline-none focus:ring-1 focus:ring-primary text-foreground font-mono"
                  />
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Achat (F)</label>
                  <input
                    type="number"
                    required
                    value={editingProduct.purchase_price}
                    onChange={(e) => setEditingProduct(m => m ? ({ ...m, purchase_price: parseFloat(e.target.value) || 0 }) : null)}
                    className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-bold focus:outline-none focus:ring-1 focus:ring-primary text-foreground"
                  />
                </div>

                <div>
                  <label className="text-[10px] font-extrabold text-primary dark:text-blue-600 uppercase block mb-1">Vente (F)</label>
                  <input
                    type="number"
                    required
                    value={editingProduct.selling_price}
                    onChange={(e) => setEditingProduct(m => m ? ({ ...m, selling_price: parseFloat(e.target.value) || 0 }) : null)}
                    className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-bold focus:outline-none focus:ring-1 focus:ring-primary text-foreground"
                  />
                </div>
              </div>

              <div>
                <label className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Format / Unité</label>
                <select
                  value={editingProduct.unit}
                  onChange={(e) => setEditingProduct(m => m ? ({ ...m, unit: e.target.value }) : null)}
                  className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-bold focus:outline-none text-foreground font-semibold"
                >
                  <option value="boite">Boite</option>
                  <option value="flacon">Flacon</option>
                  <option value="ampoule">Ampoule</option>
                  <option value="unité">Unité individuelle</option>
                </select>
              </div>

              {/* Form buttons */}
              <div className="flex gap-3 pt-4">
                <button
                  type="button"
                  onClick={() => setEditingProduct(null)}
                  className="flex-1 py-2.5 rounded-xl border border-border bg-card hover:bg-accent text-foreground text-xs font-bold cursor-pointer"
                >
                  Annuler
                </button>
                <button
                  type="submit"
                  className="flex-1 py-2.5 rounded-xl bg-primary dark:bg-blue-600 text-primary-foreground text-xs font-bold shadow-sm hover:bg-opacity-95 cursor-pointer"
                >
                  Enregistrer
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Delete Confirmation Modal */}
      <ConfirmModal
        isOpen={deleteModalOpen}
        title="Supprimer ce produit ?"
        message={`Voulez-vous vraiment supprimer le produit "${productToDelete?.name}" ? Cette action supprimera également sa fiche de stock.`}
        confirmText="Oui, Supprimer"
        cancelText="Annuler"
        onConfirm={handleDeleteProduct}
        onCancel={() => {
          setDeleteModalOpen(false);
          setProductToDelete(null);
        }}
      />
    </div>
  );
}

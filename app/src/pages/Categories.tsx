import { useState, useEffect } from 'react';
import { 
  Plus, 
  Search, 
  Edit, 
  Trash2,
  Tag,
  AlertCircle
} from 'lucide-react';
import { api, Category, Product } from '../services/api';
import { usePermissions } from '../hooks/usePermissions';
import { toast } from 'react-hot-toast';
import { ConfirmModal } from '../components/ConfirmModal';

export default function Categories() {
  const { hasAny, has } = usePermissions();
  const canEdit = hasAny(
    'can_create_category',
    'can_update_category',
    'can_delete_category'
  );
  const canRead = has('can_read_category');

  const [categories, setCategories] = useState<Category[]>([]);
  const [products, setProducts] = useState<Product[]>([]);
  const [search, setSearch] = useState('');
  const [loading, setLoading] = useState(true);

  // Modals state
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);
  const [editingCategory, setEditingCategory] = useState<Category | null>(null);

  // Delete modal state
  const [deleteModalOpen, setDeleteModalOpen] = useState(false);
  const [categoryToDelete, setCategoryToDelete] = useState<Category | null>(null);

  // Form states
  const [newCat, setNewCat] = useState({ name: '', description: '' });

  const loadData = async () => {
    setLoading(true);
    try {
      const [catRes, prodRes] = await Promise.all([
        api.categories.list('', 1, 1000),
        api.products.list('', '', 1, 1000)
      ]);
      setCategories(catRes.data || []);
      setProducts(prodRes.data || []);
    } catch (e) {
      console.error("Failed to load categories:", e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();
  }, []);

  const handleAddCategory = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!canEdit) return;
    if (!newCat.name.trim()) return;

    try {
      await api.categories.create(newCat.name, newCat.description || undefined);
      setIsAddModalOpen(false);
      setNewCat({ name: '', description: '' });
      loadData();
      toast.success("La catégorie a été créée avec succès.");
    } catch (err: any) {
      console.error(err);
      toast.error(err.message || "Erreur lors de la création de la catégorie.");
    }
  };

  const handleEditCategory = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!canEdit || !editingCategory) return;

    try {
      await api.categories.update(
        editingCategory.id, 
        editingCategory.name, 
        editingCategory.description
      );
      setEditingCategory(null);
      loadData();
      toast.success("Catégorie mise à jour avec succès.");
    } catch (err: any) {
      console.error(err);
      toast.error(err.message || "Erreur lors de la modification de la catégorie.");
    }
  };

  const confirmDeleteCategory = (cat: Category) => {
    if (!canEdit) return;
    setCategoryToDelete(cat);
    setDeleteModalOpen(true);
  };

  const handleDeleteCategory = async () => {
    if (!categoryToDelete) return;
    try {
      await api.categories.delete(categoryToDelete.id);
      setDeleteModalOpen(false);
      setCategoryToDelete(null);
      loadData();
      toast.success("La catégorie a été supprimée.");
    } catch (err: any) {
      console.error(err);
      toast.error(err.message || "Erreur lors de la suppression de la catégorie.");
    }
  };

  // Calculate products count per category
  const getProductCount = (catId: string) => {
    return products.filter(p => p.category_id === catId).length;
  };

  const filteredCategories = categories.filter(cat => 
    cat.name.toLowerCase().includes(search.toLowerCase()) || 
    (cat.description && cat.description.toLowerCase().includes(search.toLowerCase()))
  );

  return (
    <div className="space-y-6 animate-slide-up select-none">
      
      {/* Top Header & Search */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <input
            type="text"
            placeholder="Rechercher une catégorie..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full pl-11 pr-4 py-2.5 bg-card border border-border rounded-xl focus:outline-none focus:ring-1 focus:ring-primary text-xs font-semibold text-foreground"
          />
        </div>

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
          <span>Nouvelle Catégorie</span>
        </button>
      </div>

      {canRead && !canEdit && (
        <div className="p-3.5 rounded-xl bg-amber-500/10 text-amber-600 dark:text-amber-400 border border-amber-500/20 text-xs font-semibold flex items-center gap-2">
          <AlertCircle className="w-4 h-4" />
          <span>Mode lecture seule : vous n&apos;avez pas la permission de modifier les catégories.</span>
        </div>
      )}

      {/* Main categories listing */}
      {loading ? (
        <div className="py-20 text-center text-muted-foreground font-semibold">
          Chargement des catégories...
        </div>
      ) : (
        <div className="bg-card border border-border rounded-2xl shadow-sm overflow-hidden">
          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs border-collapse">
              <thead>
                <tr className="border-b border-border text-muted-foreground font-bold bg-muted/10">
                  <th className="py-3.5 px-6">Nom</th>
                  <th className="py-3.5 px-4">Description</th>
                  <th className="py-3.5 px-4 text-center">Nombre de Produits</th>
                  <th className="py-3.5 px-6 text-right">Actions</th>
                </tr>
              </thead>
              <tbody>
                {filteredCategories.length === 0 ? (
                  <tr>
                    <td colSpan={4} className="py-8 text-center text-muted-foreground font-semibold">
                      Aucune catégorie trouvée.
                    </td>
                  </tr>
                ) : (
                  filteredCategories.map((cat) => (
                    <tr key={cat.id} className="border-b border-border/50 hover:bg-accent/20 transition-colors font-medium">
                      <td className="py-4 px-6 font-bold text-foreground flex items-center gap-2">
                        <Tag className="w-4 h-4 text-primary dark:text-blue-600 shrink-0" />
                        <span>{cat.name}</span>
                      </td>
                      <td className="py-4 px-4 text-muted-foreground font-semibold">
                        {cat.description || 'Aucune description'}
                      </td>
                      <td className="py-4 px-4 text-center font-extrabold text-foreground">
                        {getProductCount(cat.id)}
                      </td>
                      <td className="py-4 px-6 text-right">
                        <div className="flex justify-end gap-1.5">
                          <button
                            onClick={() => setEditingCategory(cat)}
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
                            onClick={() => confirmDeleteCategory(cat)}
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

      {/* Add Category Modal - Centered and Styled */}
      {isAddModalOpen && (
        <div className="fixed inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-scale-in">
          <div className="bg-card border border-border w-full max-w-md rounded-3xl shadow-2xl p-6 relative">
            <h3 className="font-extrabold text-base text-foreground mb-4">Créer une Nouvelle Catégorie</h3>
            
            <form onSubmit={handleAddCategory} className="space-y-4">
              <div>
                <label className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Désignation *</label>
                <input
                  type="text"
                  required
                  placeholder="ex. Antibiotiques, Laits Infantis..."
                  value={newCat.name}
                  onChange={(e) => setNewCat(m => ({ ...m, name: e.target.value }))}
                  className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-semibold focus:outline-none focus:ring-1 focus:ring-primary text-foreground"
                />
              </div>

              <div>
                <label className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Description (optionnel)</label>
                <textarea
                  placeholder="Notes ou détails sur la catégorie..."
                  value={newCat.description}
                  onChange={(e) => setNewCat(m => ({ ...m, description: e.target.value }))}
                  rows={3}
                  className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-semibold focus:outline-none focus:ring-1 focus:ring-primary text-foreground"
                />
              </div>

              {/* Form buttons */}
              <div className="flex gap-3 pt-2">
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
                  Ajouter Catégorie
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Edit Category Modal - Centered and Styled */}
      {editingCategory && (
        <div className="fixed inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-scale-in">
          <div className="bg-card border border-border w-full max-w-md rounded-3xl shadow-2xl p-6 relative">
            <h3 className="font-extrabold text-base text-foreground mb-4">Modifier la Catégorie</h3>
            
            <form onSubmit={handleEditCategory} className="space-y-4">
              <div>
                <label className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Désignation *</label>
                <input
                  type="text"
                  required
                  value={editingCategory.name}
                  onChange={(e) => setEditingCategory(m => m ? ({ ...m, name: e.target.value }) : null)}
                  className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-semibold focus:outline-none focus:ring-1 focus:ring-primary text-foreground"
                />
              </div>

              <div>
                <label className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Description</label>
                <textarea
                  value={editingCategory.description || ''}
                  onChange={(e) => setEditingCategory(m => m ? ({ ...m, description: e.target.value }) : null)}
                  rows={3}
                  className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-semibold focus:outline-none focus:ring-1 focus:ring-primary text-foreground"
                />
              </div>

              {/* Form buttons */}
              <div className="flex gap-3 pt-2">
                <button
                  type="button"
                  onClick={() => setEditingCategory(null)}
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
        title="Supprimer cette catégorie ?"
        message={`Voulez-vous vraiment supprimer la catégorie "${categoryToDelete?.name}" ? Cette action est irréversible.`}
        confirmText="Oui, Supprimer"
        cancelText="Annuler"
        onConfirm={handleDeleteCategory}
        onCancel={() => {
          setDeleteModalOpen(false);
          setCategoryToDelete(null);
        }}
      />
    </div>
  );
}

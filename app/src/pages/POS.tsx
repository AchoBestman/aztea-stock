import { useState, useEffect } from 'react';
import { 
  Search, 
  ShoppingCart, 
  Trash2, 
  Printer, 
  Plus, 
  Minus, 
  CheckCircle,
  Barcode,
  Tag
} from 'lucide-react';
import { useAuthStore } from '../store/authStore';
import { api, Sale, Category } from '../services/api';

interface POSProduct {
  id: string;
  name: string;
  barcode: string;
  price: number; // in F
  stock: number;
  category: string;
  categoryId: string;
  taxRate: number;
  unit: string;
}

interface CartItem {
  product: POSProduct;
  quantity: number;
}

export default function POS() {
  const { user } = useAuthStore();
  
  const [products, setProducts] = useState<POSProduct[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [categoryFilter, setCategoryFilter] = useState('all');
  const [barcodeInput, setBarcodeInput] = useState('');
  const [cart, setCart] = useState<CartItem[]>([]);
  const [discount, setDiscount] = useState(0); // flat discount in F
  const [paymentMethod, setPaymentMethod] = useState<'cash' | 'momo' | 'card'>('cash');
  const [amountReceived, setAmountReceived] = useState('');
  const [showReceiptModal, setShowReceiptModal] = useState(false);
  const [receiptData, setReceiptData] = useState<Sale | null>(null);
  const [loading, setLoading] = useState(true);
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Load products, stock, and categories from API
  const loadData = async () => {
    setLoading(true);
    try {
      const [prodRes, stockRes, catRes] = await Promise.all([
        api.products.list('', '', 1, 1000),
        api.stock.listItems('', false, '', 1, 1000),
        api.categories.list('', 1, 1000)
      ]);

      const stockMap: { [prodId: string]: number } = {};
      stockRes.data.forEach(item => {
        stockMap[item.product_id] = item.quantity;
      });

      const posProducts: POSProduct[] = prodRes.data.map(p => ({
        id: p.id,
        name: p.name,
        barcode: p.barcode || '',
        price: p.selling_price,
        stock: stockMap[p.id] || 0,
        category: p.category_name || 'Général',
        categoryId: p.category_id || '',
        taxRate: p.tax_rate,
        unit: p.unit || 'boite',
      }));

      setProducts(posProducts);
      setCategories(catRes.data || []);
    } catch (e) {
      console.error("Failed to load POS data:", e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();
  }, []);

  const handleBarcodeSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!barcodeInput.trim()) return;

    const matched = products.find(p => p.barcode === barcodeInput.trim());
    if (matched) {
      addToCart(matched);
      setBarcodeInput('');
    } else {
      alert(`Produit introuvable pour le code-barres: ${barcodeInput}`);
    }
  };

  const addToCart = (product: POSProduct) => {
    setCart(currentCart => {
      const existing = currentCart.find(item => item.product.id === product.id);
      
      if (existing) {
        if (existing.quantity >= product.stock) {
          alert(`Stock insuffisant. Max disponible : ${product.stock}`);
          return currentCart;
        }
        return currentCart.map(item => 
          item.product.id === product.id 
            ? { ...item, quantity: item.quantity + 1 }
            : item
        );
      }
      
      if (product.stock <= 0) {
        alert("Ce produit est en rupture de stock.");
        return currentCart;
      }
      
      return [...currentCart, { product, quantity: 1 }];
    });
  };

  const updateQuantity = (productId: string, delta: number) => {
    setCart(currentCart => {
      return currentCart.map(item => {
        if (item.product.id === productId) {
          const newQty = item.quantity + delta;
          if (newQty <= 0) return null;
          if (newQty > item.product.stock) {
            alert(`Stock insuffisant. Max disponible : ${item.product.stock}`);
            return item;
          }
          return { ...item, quantity: newQty };
        }
        return item;
      }).filter(Boolean) as CartItem[];
    });
  };

  const removeFromCart = (productId: string) => {
    setCart(currentCart => currentCart.filter(item => item.product.id !== productId));
  };

  // Compute pricing totals
  const subtotal = cart.reduce((sum, item) => sum + (item.product.price * item.quantity), 0);
  const total = Math.max(0, subtotal - discount);
  
  const numericAmountReceived = parseFloat(amountReceived) || 0;
  const change = Math.max(0, numericAmountReceived - total);

  const handleCheckout = async () => {
    if (cart.length === 0 || isSubmitting) return;

    if (paymentMethod === 'cash' && amountReceived && numericAmountReceived < total) {
      alert("Le montant reçu est insuffisant.");
      return;
    }

    setIsSubmitting(true);
    try {
      // Map payment methods to backend expectations: 'cash' -> 'cash', 'momo' -> 'mobile_money', 'card' -> 'card'
      const apiPaymentMethod = paymentMethod === 'momo' ? 'mobile_money' : paymentMethod;

      const createdSale = await api.sales.create({
        payment_method: apiPaymentMethod,
        notes: `Achat POS - Caissier ${user?.name || 'Inconnu'}`,
        items: cart.map(item => ({
          product_id: item.product.id,
          quantity: item.quantity,
          unit_price: item.product.price,
          tax_rate: item.product.taxRate,
          discount: 0,
        })),
      });

      // Update receipt preview
      setReceiptData(createdSale);
      setShowReceiptModal(true);

      // Reset checkout states
      setCart([]);
      setDiscount(0);
      setAmountReceived('');
      
      // Reload products to get latest stock levels after the sale
      loadData();
    } catch (e: any) {
      console.error(e);
      alert(e.message || "Erreur lors de la validation de la vente.");
    } finally {
      setIsSubmitting(false);
    }
  };

  // Filter products by search text and category selection
  const filteredProducts = products.filter(p => {
    const matchesSearch = p.name.toLowerCase().includes(searchQuery.toLowerCase()) || p.barcode.includes(searchQuery);
    const matchesCategory = categoryFilter === 'all' || p.categoryId === categoryFilter;
    return matchesSearch && matchesCategory;
  });

  return (
    <div className="h-[calc(100vh-10rem)] grid grid-cols-1 lg:grid-cols-12 gap-8 animate-slide-up select-none">
      
      {/* Product Selection Pane (Left) */}
      <div className="lg:col-span-7 flex flex-col h-full space-y-4">
        
        {/* Filters and Scanner Input */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
          
          {/* Text Search */}
          <div className="relative md:col-span-1">
            <Search className="absolute left-3.5 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
            <input
              type="text"
              placeholder="Rechercher nom..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full pl-9 pr-3 py-2 bg-card border border-border rounded-xl focus:outline-none focus:ring-1 focus:ring-primary text-xs font-semibold text-foreground"
            />
          </div>

          {/* Category Filter */}
          <div className="relative">
            <Tag className="absolute left-3.5 top-1/2 -translate-y-1/2 w-4 h-4 text-primary shrink-0" />
            <select
              value={categoryFilter}
              onChange={(e) => setCategoryFilter(e.target.value)}
              className="w-full pl-9 pr-3 py-2 bg-card border border-border rounded-xl focus:outline-none text-xs font-bold text-foreground"
            >
              <option value="all">Toutes Catégories</option>
              {categories.map(cat => (
                <option key={cat.id} value={cat.id}>{cat.name}</option>
              ))}
            </select>
          </div>

          {/* Barcode scanner emulation input */}
          <form onSubmit={handleBarcodeSubmit} className="relative">
            <Barcode className="absolute left-3.5 top-1/2 -translate-y-1/2 w-4 h-4 text-primary" />
            <input
              type="text"
              placeholder="Scanner / Saisir code..."
              value={barcodeInput}
              onChange={(e) => setBarcodeInput(e.target.value)}
              className="w-full pl-9 pr-3 py-2 bg-accent/20 border border-border rounded-xl focus:outline-none focus:ring-1 focus:ring-primary text-xs font-semibold text-foreground"
            />
          </form>
        </div>

        {/* Catalog Table List */}
        {loading ? (
          <div className="flex-1 flex items-center justify-center text-muted-foreground font-semibold">
            Chargement du catalogue...
          </div>
        ) : (
          <div className="flex-1 bg-card border border-border rounded-2xl shadow-sm overflow-hidden flex flex-col">
            <div className="overflow-y-auto flex-1">
              <table className="w-full text-left text-xs border-collapse">
                <thead>
                  <tr className="border-b border-border text-muted-foreground font-bold bg-muted/10 sticky top-0 bg-card z-10">
                    <th className="py-3 px-4">Désignation</th>
                    <th className="py-3 px-3">Catégorie</th>
                    <th className="py-3 px-3">Code-barres</th>
                    <th className="py-3 px-3 text-center">Dispo</th>
                    <th className="py-3 px-3 text-right">Prix (F)</th>
                    <th className="py-3 px-4 text-right">Ajouter</th>
                  </tr>
                </thead>
                <tbody>
                  {filteredProducts.length === 0 ? (
                    <tr>
                      <td colSpan={6} className="py-8 text-center text-muted-foreground font-semibold">
                        Aucun produit disponible.
                      </td>
                    </tr>
                  ) : (
                    filteredProducts.map((product) => {
                      const isOutOfStock = product.stock <= 0;
                      const isLowStock = product.stock > 0 && product.stock < 10;
                      return (
                        <tr 
                          key={product.id} 
                          onClick={() => !isOutOfStock && addToCart(product)}
                          className={`border-b border-border/50 hover:bg-accent/20 transition-colors font-medium cursor-pointer ${
                            isOutOfStock ? 'opacity-55 hover:bg-transparent cursor-not-allowed' : ''
                          }`}
                        >
                          <td className="py-3 px-4 font-bold text-foreground truncate max-w-[180px]">
                            {product.name}
                          </td>
                          <td className="py-3 px-3 text-muted-foreground font-semibold">
                            {product.category}
                          </td>
                          <td className="py-3 px-3 font-mono text-[10px] text-muted-foreground">
                            {product.barcode || 'N/A'}
                          </td>
                          <td className="py-3 px-3 text-center">
                            <span className={`px-2 py-0.5 rounded text-[10px] font-extrabold ${
                              isOutOfStock 
                                ? 'bg-rose-500/10 text-rose-500' 
                                : isLowStock 
                                  ? 'bg-amber-500/10 text-amber-500' 
                                  : 'bg-emerald-500/10 text-emerald-500'
                            }`}>
                              {isOutOfStock ? 'Rupture' : `${product.stock} ${product.unit}`}
                            </span>
                          </td>
                          <td className="py-3 px-3 text-right font-extrabold text-foreground">
                            {product.price.toLocaleString('fr-FR')} F
                          </td>
                          <td className="py-3 px-4 text-right">
                            <button
                              disabled={isOutOfStock}
                              onClick={(e) => {
                                e.stopPropagation();
                                addToCart(product);
                              }}
                              className="w-6.5 h-6.5 rounded-lg bg-primary/10 text-primary flex items-center justify-center hover:bg-primary hover:text-white transition-all ml-auto cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed"
                            >
                              <Plus className="w-3.5 h-3.5" />
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
      </div>

      {/* Cart & Checkout Panel (Right) */}
      <div className="lg:col-span-5 bg-card border border-border rounded-3xl shadow-md flex flex-col h-full overflow-hidden">
        
        {/* Cart header */}
        <div className="p-5 border-b border-border flex items-center justify-between">
          <div className="flex items-center gap-2">
            <ShoppingCart className="w-5 h-5 text-primary" />
            <h3 className="font-bold text-base text-foreground">Panier en cours</h3>
          </div>
          <span className="bg-primary/10 text-primary text-xs px-2.5 py-1 rounded-full font-extrabold">
            {cart.reduce((sum, item) => sum + item.quantity, 0)} articles
          </span>
        </div>

        {/* Cart Item lines */}
        <div className="flex-1 overflow-y-auto p-5 space-y-4">
          {cart.length === 0 ? (
            <div className="h-full flex flex-col items-center justify-center text-center text-muted-foreground">
              <ShoppingCart className="w-12 h-12 text-muted-foreground/30 mb-3" />
              <p className="text-sm font-semibold">Le panier est vide</p>
              <p className="text-xs max-w-xs mt-1">Sélectionnez des produits à gauche ou scannez un code-barres pour commencer.</p>
            </div>
          ) : (
            cart.map((item) => (
              <div key={item.product.id} className="flex items-center justify-between gap-4 p-3 rounded-2xl bg-accent/30 border border-border/50">
                <div className="flex-1 min-w-0">
                  <h5 className="text-xs font-bold text-foreground truncate">{item.product.name}</h5>
                  <p className="text-[10px] text-muted-foreground font-semibold mt-0.5">
                    {item.product.price.toLocaleString('fr-FR')} F x {item.quantity}
                  </p>
                </div>
                
                {/* Quantity adjustments */}
                <div className="flex items-center gap-2.5">
                  <button 
                    onClick={() => updateQuantity(item.product.id, -1)}
                    className="w-6 h-6 rounded-md border border-border bg-card flex items-center justify-center hover:bg-accent text-foreground transition-all cursor-pointer"
                  >
                    <Minus className="w-3 h-3" />
                  </button>
                  <span className="text-xs font-extrabold tabular-nums w-4 text-center">{item.quantity}</span>
                  <button 
                    onClick={() => updateQuantity(item.product.id, 1)}
                    className="w-6 h-6 rounded-md border border-border bg-card flex items-center justify-center hover:bg-accent text-foreground transition-all cursor-pointer"
                  >
                    <Plus className="w-3 h-3" />
                  </button>
                  <button 
                    onClick={() => removeFromCart(item.product.id)}
                    className="w-6 h-6 rounded-md bg-rose-500/10 text-rose-500 hover:bg-rose-500 hover:text-white flex items-center justify-center transition-all ml-1.5 cursor-pointer"
                  >
                    <Trash2 className="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>
            ))
          )}
        </div>

        {/* Pricing calculations & Payment type select */}
        <div className="p-5 border-t border-border bg-muted/10 space-y-4">
          <div className="space-y-2.5">
            <div className="flex justify-between text-xs font-semibold text-muted-foreground">
              <span>Sous-total</span>
              <span className="text-foreground">{subtotal.toLocaleString('fr-FR')} F</span>
            </div>
            
            <div className="flex justify-between items-center text-xs font-semibold text-muted-foreground">
              <span>Remise (F)</span>
              <input
                type="number"
                placeholder="0"
                value={discount || ''}
                onChange={(e) => setDiscount(Math.max(0, parseFloat(e.target.value) || 0))}
                className="w-20 px-2 py-1 bg-card border border-border rounded-lg text-right text-xs font-bold text-foreground focus:outline-none focus:ring-1 focus:ring-primary"
              />
            </div>

            <div className="flex justify-between text-sm font-extrabold pt-2.5 border-t border-border/50 text-foreground">
              <span>NET A PAYER</span>
              <span className="text-primary text-base">{total.toLocaleString('fr-FR')} F</span>
            </div>
          </div>

          {/* Payment Method Option buttons */}
          <div className="grid grid-cols-3 gap-2">
            {(['cash', 'momo', 'card'] as const).map((method) => (
              <button
                key={method}
                onClick={() => setPaymentMethod(method)}
                className={`py-2 rounded-xl text-[10px] font-bold uppercase transition-all cursor-pointer border ${
                  paymentMethod === method
                    ? 'bg-primary border-primary text-primary-foreground shadow-sm'
                    : 'bg-card border-border text-foreground hover:bg-accent'
                }`}
              >
                {method === 'cash' ? 'Espèces' : method === 'momo' ? 'Mobile Money' : 'Carte'}
              </button>
            ))}
          </div>

          {/* Cash input calculations */}
          {paymentMethod === 'cash' && cart.length > 0 && (
            <div className="grid grid-cols-2 gap-3 p-3 bg-accent/40 rounded-2xl border border-border/50">
              <div>
                <label className="text-[10px] font-bold text-muted-foreground block mb-1">MONTANT REÇU</label>
                <input
                  type="number"
                  placeholder="0"
                  value={amountReceived}
                  onChange={(e) => setAmountReceived(e.target.value)}
                  className="w-full px-2 py-1.5 bg-card border border-border rounded-lg text-xs font-bold text-foreground focus:outline-none focus:ring-1 focus:ring-primary"
                />
              </div>
              <div>
                <span className="text-[10px] font-bold text-muted-foreground block mb-1">MONNAIE À RENDRE</span>
                <span className={`text-sm font-extrabold block py-1.5 ${
                  change > 0 ? 'text-emerald-500' : 'text-foreground'
                }`}>
                  {change.toLocaleString('fr-FR')} F
                </span>
              </div>
            </div>
          )}

          {/* Checkout CTA */}
          <button
            onClick={handleCheckout}
            disabled={cart.length === 0 || isSubmitting}
            className={`w-full py-3.5 rounded-2xl font-bold flex items-center justify-center gap-2 cursor-pointer shadow-md transition-all ${
              cart.length === 0 || isSubmitting
                ? 'bg-muted text-muted-foreground cursor-not-allowed shadow-none'
                : 'bg-primary text-primary-foreground hover:bg-opacity-95'
            }`}
          >
            <Printer className="w-4 h-4" />
            <span>{isSubmitting ? 'Validation...' : 'Valider & Imprimer le Ticket'}</span>
          </button>
        </div>
      </div>

      {/* Gorgeous Receipt Preview Modal */}
      {showReceiptModal && receiptData && (
        <div className="fixed inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-scale-in">
          <div className="bg-card border border-border w-full max-w-sm rounded-3xl shadow-2xl p-6 relative flex flex-col">
            
            {/* Modal header & Success icon */}
            <div className="flex flex-col items-center text-center pb-4 border-b border-border/50">
              <div className="w-12 h-12 rounded-full bg-emerald-500/10 text-emerald-500 flex items-center justify-center mb-2 animate-bounce">
                <CheckCircle className="w-6 h-6" />
              </div>
              <h4 className="font-extrabold text-base text-foreground">Transaction Validée</h4>
              <p className="text-[10px] text-muted-foreground mt-0.5">Le reçu a été envoyé à l'imprimante thermique.</p>
            </div>

            {/* Thermal ticket simulator */}
            <div className="bg-muted/30 border border-dashed border-border rounded-xl p-4 my-4 font-mono text-[11px] space-y-3 max-h-[300px] overflow-y-auto">
              <div className="text-center space-y-1">
                <p className="font-bold text-xs">AZTEA PHARMACY & POS</p>
                <p className="text-[9px] text-muted-foreground">Brazzaville, Congo</p>
                <p className="text-[9px] text-muted-foreground">Tel: +242 05 656 0299</p>
              </div>
              
              <div className="border-t border-dashed border-border/50 pt-2 space-y-0.5">
                <p>Ticket: {receiptData.receipt_number}</p>
                <p>Date: {new Date(receiptData.sold_at).toLocaleString('fr-FR')}</p>
                <p>Caissier: {user?.name || 'Inconnu'}</p>
                <p>Périphérique : {localStorage.getItem('aztea_default_printer') || 'Imprimante système par défaut'}</p>
              </div>

              <div className="border-t border-dashed border-border/50 pt-2 space-y-1">
                {receiptData.items.map((item, i) => (
                  <div key={i} className="flex justify-between">
                    <span className="truncate max-w-[150px]">{item.product_name}</span>
                    <span>{item.quantity} x {item.unit_price}F</span>
                  </div>
                ))}
              </div>

              <div className="border-t border-dashed border-border/50 pt-2 space-y-0.5">
                <div className="flex justify-between font-bold">
                  <span>Sous-total:</span>
                  <span>{receiptData.subtotal} F</span>
                </div>
                {receiptData.discount_total > 0 && (
                  <div className="flex justify-between text-rose-500 font-semibold">
                    <span>Remise:</span>
                    <span>-{receiptData.discount_total} F</span>
                  </div>
                )}
                <div className="flex justify-between font-bold text-xs pt-1 border-t border-dotted border-border/30">
                  <span>NET A PAYER:</span>
                  <span>{receiptData.total} F</span>
                </div>
              </div>

              <div className="border-t border-dashed border-border/50 pt-2 space-y-0.5">
                <p className="capitalize">Mode: {receiptData.payment_method === 'cash' ? 'Espèces' : receiptData.payment_method === 'mobile_money' ? 'Mobile Money' : 'Carte'}</p>
                {receiptData.payment_method === 'cash' && (
                  <>
                    <p>Reçu: {amountReceived || receiptData.total} F</p>
                    <p>Rendu: {change} F</p>
                  </>
                )}
              </div>

              <p className="text-center font-bold pt-2 border-t border-dotted border-border/30">*** MERCI DE VOTRE VISITE ***</p>
            </div>

            {/* Modal actions */}
            <div className="flex gap-3">
              <button 
                onClick={() => setShowReceiptModal(false)}
                className="flex-1 py-2.5 rounded-xl border border-border bg-card hover:bg-accent text-foreground text-xs font-bold transition-all cursor-pointer"
              >
                Fermer
              </button>
              <button 
                onClick={() => {
                  alert(`Imprimer à nouveau sur : ${localStorage.getItem('aztea_default_printer') || 'Imprimante par défaut'}`);
                }}
                className="flex-1 py-2.5 rounded-xl bg-primary text-primary-foreground text-xs font-bold shadow-sm hover:bg-opacity-95 transition-all cursor-pointer flex items-center justify-center gap-1.5"
              >
                <Printer className="w-3.5 h-3.5" />
                <span>Réimprimer</span>
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

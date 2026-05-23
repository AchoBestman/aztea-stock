import { useState, useEffect, useCallback, useRef } from 'react';
import { 
  Search, 
  ShoppingCart, 
  Trash2, 
  Printer, 
  Plus, 
  Minus, 
  CheckCircle,
  Barcode,
  Tag,
  ChevronDown,
  ChevronRight,
  UserPlus,
  AlertTriangle,
  X,
  Camera
} from 'lucide-react';
import { useAuthStore } from '../store/authStore';
import { api, Sale, Category } from '../services/api';
import { ConfirmModal } from '../components/ConfirmModal';
import { CameraBarcodeScanner } from '../components/CameraBarcodeScanner';
import { getTicketPrinterConfig, isTauriApp } from '../utils/hardwareConfig';
import { computeReceiptTotals, getItemBarcode, renderBarcodeSvg } from '../utils/receipt';
import { getTicketLayout, printTicketFromSale } from '../utils/printService';
import React from 'react';

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

type BarcodeErrorReason = 'not_found' | 'out_of_stock' | 'insufficient_stock';

export default function POS() {
  const { user } = useAuthStore();
  
  const [products, setProducts] = useState<POSProduct[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [categoryFilter, setCategoryFilter] = useState('all');
  const [barcodeInput, setBarcodeInput] = useState('');
  const [isManualBarcode, setIsManualBarcode] = useState(false);
  const [barcodeErrorModal, setBarcodeErrorModal] = useState<{
    isOpen: boolean;
    title: string;
    message: string;
    code: string;
    reason: BarcodeErrorReason | null;
  }>({
    isOpen: false,
    title: '',
    message: '',
    code: '',
    reason: null,
  });
  const dismissedBarcodeErrorRef = useRef<{ code: string; reason: BarcodeErrorReason } | null>(null);
  const barcodeTimestampsRef = useRef<number[]>([]);
  const SCAN_CHAR_INTERVAL_MS = 50;
  const [cart, setCart] = useState<CartItem[]>([]);
  const [discount, setDiscount] = useState(0); // flat discount in F
  const [paymentMethod, setPaymentMethod] = useState<'cash' | 'momo' | 'card'>('cash');
  const [amountReceived, setAmountReceived] = useState('');
  const [showReceiptModal, setShowReceiptModal] = useState(false);
  const [receiptData, setReceiptData] = useState<Sale | null>(null);
  const [loading, setLoading] = useState(true);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [notification, setNotification] = useState<{ message: string; type: 'success' | 'error' | 'warning' } | null>(null);

  // Client info state
  const [showClientModal, setShowClientModal] = useState(false);
  const [clientInfo, setClientInfo] = useState({ full_name: '', phone: '', email: '' });
  const [expandedProducts, setExpandedProducts] = useState<Record<string, boolean>>({});
  const [isCartDetailsExpanded, setIsCartDetailsExpanded] = useState(true);
  const [deleteConfirm, setDeleteConfirm] = useState<{ isOpen: boolean; productId: string; productName: string }>({ isOpen: false, productId: '', productName: '' });
  const [showCameraScanner, setShowCameraScanner] = useState(false);
  const productTableRef = useRef<HTMLDivElement>(null);

  const toggleExpand = (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setExpandedProducts(prev => ({ ...prev, [id]: !prev[id] }));
  };

  // Auto-dismiss notification
  useEffect(() => {
    if (notification) {
      const t = setTimeout(() => setNotification(null), 4000);
      return () => clearTimeout(t);
    }
  }, [notification]);

  const notify = useCallback((message: string, type: 'success' | 'error' | 'warning') => {
    setNotification({ message, type });
  }, []);

  const printReceipt = async (sale: Sale) => {
    try {
      const cfg = getTicketPrinterConfig();
      notify('Génération du ticket en cours...', 'success');
      const savedPath = await printTicketFromSale(sale, `ticket_${sale.receipt_number}.pdf`, {
        cashierName: user?.name,
      });
      if (cfg.isPdf) {
        notify(
          typeof savedPath === 'string' && savedPath
            ? `PDF enregistré : ${savedPath}`
            : 'Ticket PDF enregistré dans Téléchargements.',
          'success'
        );
      } else if (isTauriApp()) {
        notify('Ticket imprimé avec succès', 'success');
      } else {
        notify('Ticket envoyé à l\'impression', 'success');
      }
      return;
    } catch (e: unknown) {
      const message = e instanceof Error ? e.message : 'Erreur impression';
      notify(message, 'error');
    }
  };

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

  const resetBarcodeSession = () => {
    barcodeTimestampsRef.current = [];
    setIsManualBarcode(false);
  };

  const handleBarcodeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    const now = Date.now();

    if (value.length === 0) {
      resetBarcodeSession();
      setBarcodeInput('');
      return;
    }

    if (value.length < barcodeInput.length) {
      barcodeTimestampsRef.current = [];
      setIsManualBarcode(true);
      setBarcodeInput(value);
      return;
    }

    barcodeTimestampsRef.current.push(now);
    const stamps = barcodeTimestampsRef.current;

    if (stamps.length >= 2) {
      const intervals = stamps.slice(1).map((t, i) => t - stamps[i]);
      const hasSlowInterval = intervals.some((gap) => gap >= SCAN_CHAR_INTERVAL_MS);
      const fastRatio = intervals.filter((gap) => gap < SCAN_CHAR_INTERVAL_MS).length / intervals.length;
      if (hasSlowInterval) {
        setIsManualBarcode(true);
      } else if (fastRatio >= 0.8 && value.length >= 4) {
        setIsManualBarcode(false);
      }
    }

    setBarcodeInput(value);
  };

  const handleBarcodePaste = () => {
    requestAnimationFrame(() => setIsManualBarcode(true));
  };

  const closeBarcodeErrorModal = useCallback(() => {
    setBarcodeErrorModal((prev) => {
      if (prev.isOpen && prev.code && prev.reason) {
        dismissedBarcodeErrorRef.current = { code: prev.code, reason: prev.reason };
      }
      return { isOpen: false, title: '', message: '', code: '', reason: null };
    });
  }, []);

  const showBarcodeError = useCallback(
    (code: string, reason: BarcodeErrorReason, title: string, message: string) => {
      const dismissed = dismissedBarcodeErrorRef.current;
      if (dismissed?.code === code && dismissed.reason === reason) {
        return;
      }
      setBarcodeErrorModal((prev) => {
        if (prev.isOpen && prev.code === code && prev.reason === reason) {
          return prev;
        }
        return { isOpen: true, title, message, code, reason };
      });
    },
    []
  );

  const processBarcodeCode = useCallback(
    (rawCode: string) => {
      const code = rawCode.trim();
      console.log('[POS] processBarcodeCode:', code);
      if (!code) return;

      const normalized = code.replace(/\s/g, '');
      const matched = products.find((p) => {
        const bc = (p.barcode || '').trim().replace(/\s/g, '');
        return bc && (bc === normalized || bc === code);
      });

      console.log('[POS] Produit trouvé:', matched ? matched.name : 'AUCUN', '| produits chargés:', products.length);

      if (!matched) {
        showBarcodeError(
          code,
          'not_found',
          'Produit introuvable',
          `Aucun produit ne correspond au code-barres « ${code} ».`
        );
        return;
      }

      if (matched.stock <= 0) {
        showBarcodeError(
          code,
          'out_of_stock',
          'Rupture de stock',
          `« ${matched.name} » est en rupture de stock et ne peut pas être ajouté au panier.`
        );
        return;
      }

      const existing = cart.find((item) => item.product.id === matched.id);
      if (existing && existing.quantity >= matched.stock) {
        showBarcodeError(
          code,
          'insufficient_stock',
          'Stock insuffisant',
          `Stock maximum atteint pour « ${matched.name} » (${matched.stock} disponible${matched.stock > 1 ? 's' : ''}).`
        );
        return;
      }

      if (dismissedBarcodeErrorRef.current?.code === code) {
        dismissedBarcodeErrorRef.current = null;
      }

      setCart((currentCart) => {
        const existingItem = currentCart.find((item) => item.product.id === matched.id);
        if (existingItem) {
          return currentCart.map((item) =>
            item.product.id === matched.id ? { ...item, quantity: item.quantity + 1 } : item
          );
        }
        return [...currentCart, { product: matched, quantity: 1 }];
      });
      setBarcodeInput('');
      resetBarcodeSession();
      notify(`${matched.name} ajouté au panier`, 'success');
    },
    [products, cart, showBarcodeError, notify]
  );

  const attemptBarcodeAdd = () => {
    processBarcodeCode(barcodeInput);
  };

  const handleCameraScan = useCallback(
    (code: string) => {
      console.log('[POS] Scan caméra reçu:', code);
      resetBarcodeSession();
      setIsManualBarcode(false);
      setBarcodeInput(code);
      processBarcodeCode(code);
    },
    [processBarcodeCode]
  );

  const handleBarcodeSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!barcodeInput.trim() || isManualBarcode) return;
    attemptBarcodeAdd();
  };

  const handleBarcodeKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      if (!barcodeInput.trim() || isManualBarcode) return;
      attemptBarcodeAdd();
    }
  };

  const handleManualBarcodeAdd = () => {
    attemptBarcodeAdd();
  };

  const addToCart = (product: POSProduct) => {
    if (product.stock <= 0) {
      notify('Ce produit est en rupture de stock.', 'warning');
      return;
    }
    setCart(currentCart => {
      const existing = currentCart.find(item => item.product.id === product.id);
      if (existing) {
        if (existing.quantity >= product.stock) {
          setTimeout(() => notify(`Stock insuffisant. Max disponible : ${product.stock}`, 'warning'), 0);
          return currentCart;
        }
        return currentCart.map(item => 
          item.product.id === product.id 
            ? { ...item, quantity: item.quantity + 1 }
            : item
        );
      }
      return [...currentCart, { product, quantity: 1 }];
    });
  };

  const updateQuantity = (productId: string, delta: number) => {
    setCart(currentCart => {
      return currentCart.map(item => {
        if (item.product.id === productId) {
          let newQty = (typeof item.quantity === 'number' ? item.quantity : 1) + delta;
          if (newQty < 1) newQty = 1;
          if (newQty > item.product.stock) {
            setTimeout(() => notify(`Stock insuffisant. Max disponible : ${item.product.stock}`, 'warning'), 0);
            return item;
          }
          return { ...item, quantity: newQty };
        }
        return item;
      });
    });
  };

  const setQuantity = (productId: string, value: string | number) => {
    setCart(currentCart => {
      return currentCart.map(item => {
        if (item.product.id === productId) {
          if (value === '') return { ...item, quantity: '' as any };
          
          let cleanValue = typeof value === 'string' ? value.replace(/\D/g, '') : value;
          if (cleanValue === '') return { ...item, quantity: '' as any };
          
          let parsed = typeof cleanValue === 'string' ? parseInt(cleanValue, 10) : cleanValue;
          if (isNaN(parsed) || parsed < 1) parsed = 1;
          if (parsed > item.product.stock) {
            setTimeout(() => notify(`Stock insuffisant. Max disponible : ${item.product.stock}`, 'warning'), 0);
            parsed = item.product.stock;
          }
          return { ...item, quantity: parsed };
        }
        return item;
      });
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

  const isPaymentValid = paymentMethod !== 'cash' || (numericAmountReceived >= total && numericAmountReceived > 0);

  const handleCheckout = async (withPrint: boolean) => {
    if (cart.length === 0 || isSubmitting) return;

    if (paymentMethod === 'cash' && numericAmountReceived < total) {
      notify('Veuillez saisir un montant reçu suffisant.', 'warning');
      return;
    }

    setIsSubmitting(true);
    try {
      const apiPaymentMethod = paymentMethod === 'momo' ? 'mobile_money' : paymentMethod;
      const clientJson = clientInfo.full_name ? JSON.stringify(clientInfo) : '';

      const cartSnapshot = cart.map((item) => ({ ...item }));

      const createdSale = await api.sales.create({
        customer_name: clientInfo.full_name || undefined,
        customer_phone: clientInfo.phone || undefined,
        payment_method: apiPaymentMethod,
        notes: clientJson ? clientJson : `Achat POS - Caissier ${user?.name || 'Inconnu'}`,
        amount_paid: paymentMethod === 'cash' ? numericAmountReceived : total,
        change_given: paymentMethod === 'cash' ? change : 0,
        items: cartSnapshot.map((item, idx) => ({
          product_id: item.product.id,
          quantity: item.quantity,
          unit_price: item.product.price,
          tax_rate: item.product.taxRate,
          discount: idx === 0 ? discount : 0,
        })),
      });

      const barcodeByProduct = Object.fromEntries(
        cartSnapshot.map((item) => [item.product.id, item.product.barcode || ''])
      );
      const saleForReceipt: Sale = {
        ...createdSale,
        items: createdSale.items.map((line) => ({
          ...line,
          product_barcode: line.product_barcode || barcodeByProduct[line.product_id] || null,
        })),
      };
      setReceiptData(saleForReceipt);

      // Reset cart state IMMEDIATELY so the user is not blocked
      setCart([]);
      setDiscount(0);
      setAmountReceived('');
      setClientInfo({ full_name: '', phone: '', email: '' });
      loadData();

      if (withPrint) {
        setShowReceiptModal(true);
        // Do not await the print so the UI remains interactive
        printReceipt(saleForReceipt).catch(err => {
          console.error(err);
          notify(err?.message || "Erreur lors de l'impression", 'error');
        });
      } else {
        notify('Vente validée avec succès.', 'success');
      }
    } catch (e: any) {
      console.error(e);
      notify(e.message || 'Erreur lors de la validation de la vente.', 'error');
    } finally {
      setIsSubmitting(false);
    }
  };

  // Filter products by search text and category selection
  const filteredProducts = products.filter(p => {
    const matchesSearch = p.name.toLowerCase().includes(searchQuery.toLowerCase()) || (p.barcode && p.barcode.includes(searchQuery));
    const matchesCategory = categoryFilter === 'all' || p.categoryId === categoryFilter;
    return matchesSearch && matchesCategory;
  });

  return (
    <>
    {/* Toast notification */}
    {notification && (
      <div className={`fixed top-5 right-5 z-[200] px-5 py-3 rounded-2xl shadow-2xl text-xs font-bold flex items-center gap-3 max-w-xs animate-slide-up ${
        notification.type === 'success' ? 'bg-emerald-500 text-white' :
        notification.type === 'error' ? 'bg-rose-500 text-white' :
        'bg-amber-500 text-white'
      }`}>
        <span className="flex-1">{notification.message}</span>
        <button onClick={() => setNotification(null)} className="opacity-70 hover:opacity-100 text-sm cursor-pointer">✕</button>
      </div>
    )}
    {showCameraScanner && (
      <CameraBarcodeScanner
        anchorRef={productTableRef}
        onScan={handleCameraScan}
        onClose={() => setShowCameraScanner(false)}
      />
    )}
    <div className="h-[calc(100vh-10rem)] grid grid-cols-1 lg:grid-cols-12 gap-8 animate-slide-up select-none">
      
      {/* Product Selection Pane (Left) */}
      <div className="lg:col-span-7 flex flex-col h-full space-y-4">
        
        {/* Filters and Scanner Input */}
        <div className="flex flex-col gap-3">
          {/* Text Search Full Width */}
          <div className="relative w-full">
            <Search className="absolute left-3.5 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
            <input
              type="text"
              placeholder="Rechercher nom..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full pl-9 pr-3 py-2 bg-card border border-border rounded-xl focus:outline-none focus:ring-1 focus:ring-primary text-xs font-semibold text-foreground"
            />
          </div>

          <div className="flex gap-3 h-[38px]">
            <button
              type="button"
              onClick={() => setShowCameraScanner((v) => !v)}
              className={`shrink-0 flex items-center gap-1.5 px-3 rounded-xl border text-[10px] font-extrabold uppercase tracking-wide transition-all cursor-pointer ${
                showCameraScanner
                  ? 'bg-primary dark:bg-blue-600 text-primary-foreground border-primary dark:border-blue-600'
                  : 'bg-card border-border text-foreground hover:bg-accent'
              }`}
              title="Scanner avec la caméra du terminal"
            >
              <Camera className="w-4 h-4" />
              <span className="hidden sm:inline">Caméra</span>
            </button>

            {/* Category Filter */}
            <div className="flex-1 flex items-center bg-card border border-border rounded-xl px-3 focus-within:ring-1 focus-within:ring-primary overflow-hidden">
              <Tag className="w-4 h-4 text-primary dark:text-blue-600 shrink-0 mr-2 pointer-events-none" />
              <select
                value={categoryFilter}
                onChange={(e) => setCategoryFilter(e.target.value)}
                className="flex-1 h-full w-full bg-transparent focus:outline-none text-xs font-bold text-foreground truncate"
              >
                <option value="all">Toutes Catégories</option>
                {categories.map(cat => (
                  <option key={cat.id} value={cat.id}>{cat.name}</option>
                ))}
              </select>
            </div>

            {/* Barcode scanner / manual code input */}
            <form onSubmit={handleBarcodeSubmit} className="flex-1 flex items-center gap-2 bg-accent/20 border border-border rounded-xl px-3 focus-within:ring-1 focus-within:ring-primary overflow-hidden">
              <Barcode className="w-4 h-4 text-primary dark:text-blue-600 shrink-0 pointer-events-none" />
              <input
                type="text"
                placeholder="Scanner / Saisir code..."
                value={barcodeInput}
                onChange={handleBarcodeChange}
                onKeyDown={handleBarcodeKeyDown}
                onPaste={handleBarcodePaste}
                className="flex-1 min-w-0 h-full bg-transparent focus:outline-none text-xs font-semibold text-foreground placeholder:text-muted-foreground/70"
              />
              {isManualBarcode && barcodeInput.trim() && (
                <button
                  type="button"
                  onClick={handleManualBarcodeAdd}
                  className="shrink-0 px-3 py-1 rounded-lg bg-primary dark:bg-blue-600 text-primary-foreground text-[10px] font-extrabold uppercase tracking-wide hover:opacity-95 transition-opacity cursor-pointer"
                >
                  Ajouter
                </button>
              )}
            </form>
          </div>
        </div>

        {/* Catalog Table List */}
        <div ref={productTableRef} className="flex-1 min-h-0 flex flex-col">
        {loading ? (
          <div className="flex-1 flex items-center justify-center text-muted-foreground font-semibold bg-card border border-border rounded-2xl">
            Chargement du catalogue...
          </div>
        ) : (
          <div className="flex-1 bg-card border border-border rounded-2xl shadow-sm overflow-hidden flex flex-col">
            <div className="overflow-y-auto flex-1">
              <table className="w-full text-left text-xs border-collapse">
                <thead>
                  <tr className="border-b border-border text-muted-foreground font-bold bg-muted/10 sticky top-0 bg-card z-10">
                    <th className="py-3 px-4 w-[10px]"></th>
                    <th className="py-3 px-2">Désignation</th>
                    <th className="py-3 px-3 text-right">Prix (F)</th>
                    <th className="py-3 px-4 text-right">Ajouter</th>
                  </tr>
                </thead>
                <tbody>
                  {filteredProducts.length === 0 ? (
                    <tr>
                      <td colSpan={4} className="py-8 text-center text-muted-foreground font-semibold">
                        Aucun produit disponible.
                      </td>
                    </tr>
                  ) : (
                    filteredProducts.map((product) => {
                      const isOutOfStock = product.stock <= 0;
                      const isLowStock = product.stock > 0 && product.stock < 10;
                      const isExpanded = expandedProducts[product.id];
                      return (
                        <React.Fragment key={product.id}>
                          <tr 
                            onClick={() => !isOutOfStock && addToCart(product)}
                            className={`border-b border-border/50 hover:bg-accent/20 transition-colors font-medium cursor-pointer ${
                              isOutOfStock ? 'opacity-55 hover:bg-transparent cursor-not-allowed' : ''
                            }`}
                          >
                            <td className="py-3 px-4" onClick={(e) => toggleExpand(product.id, e)}>
                              <button className="w-5 h-5 flex items-center justify-center rounded hover:bg-accent text-muted-foreground transition-colors cursor-pointer">
                                {isExpanded ? <ChevronDown className="w-4 h-4" /> : <ChevronRight className="w-4 h-4" />}
                              </button>
                            </td>
                            <td className="py-3 px-2 font-bold text-foreground truncate max-w-[200px]">
                              {product.name}
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
                                className="w-6.5 h-6.5 rounded-lg bg-primary text-white dark:bg-blue-600 flex items-center justify-center hover:bg-amber-500 hover:text-white transition-all ml-auto cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed"
                              >
                                <Plus className="w-3.5 h-3.5 dark:text-white" />
                              </button>
                            </td>
                          </tr>
                          {isExpanded && (
                            <tr className="bg-muted/10 border-b border-border/50">
                              <td colSpan={4} className="px-4 py-3 text-xs">
                                <div className="grid grid-cols-3 gap-4">
                                  <div>
                                    <p className="text-[10px] text-muted-foreground uppercase font-bold">Catégorie</p>
                                    <p className="font-semibold text-foreground mt-0.5">{product.category}</p>
                                  </div>
                                  <div>
                                    <p className="text-[10px] text-muted-foreground uppercase font-bold">Code-barres</p>
                                    <p className="font-mono text-foreground mt-0.5">{product.barcode || 'N/A'}</p>
                                  </div>
                                  <div>
                                    <p className="text-[10px] text-muted-foreground uppercase font-bold">Disponibilité</p>
                                    <span className={`inline-block mt-0.5 px-2 py-0.5 rounded text-[10px] font-extrabold ${
                                      isOutOfStock 
                                        ? 'bg-rose-500/10 text-rose-500' 
                                        : isLowStock 
                                          ? 'bg-amber-500/10 text-amber-500' 
                                          : 'bg-emerald-500/10 text-emerald-500'
                                    }`}>
                                      {isOutOfStock ? 'Rupture' : `${product.stock} ${product.unit}`}
                                    </span>
                                  </div>
                                </div>
                              </td>
                            </tr>
                          )}
                        </React.Fragment>
                      );
                    })
                  )}
                </tbody>
              </table>
            </div>
          </div>
        )}
        </div>
      </div>

      {/* Cart & Checkout Panel (Right) */}
      <div className="lg:col-span-5 bg-card border border-border rounded-3xl shadow-md flex flex-col h-full overflow-hidden">
        
        {/* Cart header */}
        <div className="p-4 border-b border-border flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="relative">
              <ShoppingCart className="w-5 h-5 text-primary dark:text-blue-600" />
              {cart.length > 0 && (
                <span className="absolute -top-3 left-4 bg-rose-500 text-white text-[9px] font-extrabold px-1.5 py-0.5 rounded-full shadow-sm whitespace-nowrap z-10">
                  {total.toLocaleString('fr-FR')} F
                </span>
              )}
            </div>
            <h3 className="font-bold text-base text-foreground">Panier</h3>
          </div>
          <div className="flex items-center gap-2">
            {clientInfo.full_name && (
              <span className="bg-emerald-500/10 text-emerald-600 text-[10px] px-2 py-1 rounded-full font-bold truncate max-w-[100px]">
                {clientInfo.full_name}
              </span>
            )}
            <button
              onClick={() => setShowClientModal(true)}
              className="bg-accent/50 text-foreground hover:bg-accent text-xs px-2.5 py-1 rounded-full font-extrabold flex items-center gap-1 cursor-pointer transition-colors"
            >
              <UserPlus className="w-3 h-3" />
              Client
            </button>
          </div>
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
                    className="w-6 h-6 rounded-md border border-border bg-card flex items-center justify-center hover:bg-accent text-foreground transition-all cursor-pointer shrink-0"
                  >
                    <Minus className="w-3 h-3" />
                  </button>
                  <input 
                    type="number"
                    min="1"
                    value={item.quantity}
                    onChange={(e) => setQuantity(item.product.id, e.target.value)}
                    onBlur={() => {
                      if (!item.quantity || item.quantity <= 0) {
                        setQuantity(item.product.id, 1);
                      }
                    }}
                    className="w-14 h-6 px-1 text-xs font-extrabold tabular-nums text-center bg-card border border-border rounded focus:outline-none focus:ring-1 focus:ring-primary appearance-none m-0"
                  />
                  <button 
                    onClick={() => updateQuantity(item.product.id, 1)}
                    className="w-6 h-6 rounded-md border border-border bg-card flex items-center justify-center hover:bg-accent text-foreground transition-all cursor-pointer"
                  >
                    <Plus className="w-3 h-3" />
                  </button>
                  <button 
                    onClick={() => setDeleteConfirm({ isOpen: true, productId: item.product.id, productName: item.product.name })}
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
          
          {/* Header toggle for calculations */}
          <div 
            className="flex items-center justify-between cursor-pointer group"
            onClick={() => setIsCartDetailsExpanded(!isCartDetailsExpanded)}
          >
            <span className="text-xs font-bold text-muted-foreground uppercase tracking-wider group-hover:text-foreground transition-colors">Détails de Paiement</span>
            <button className="w-6 h-6 rounded-lg bg-card border border-border flex items-center justify-center text-muted-foreground group-hover:text-foreground transition-colors cursor-pointer">
              {isCartDetailsExpanded ? <ChevronDown className="w-3.5 h-3.5" /> : <ChevronRight className="w-3.5 h-3.5" />}
            </button>
          </div>

          {isCartDetailsExpanded && (
            <div className="space-y-4 animate-slide-up">
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
                  <span className="text-amber-600 dark:text-amber-400 text-base">{total.toLocaleString('fr-FR')} F</span>
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
            </div>
          )}

          {/* Checkout CTAs */}
          <div className="flex flex-col gap-2">
            <button
              onClick={() => handleCheckout(true)}
              disabled={cart.length === 0 || isSubmitting || !isPaymentValid}
              className={`py-2.5 rounded-xl text-xs font-bold flex items-center justify-center gap-2 cursor-pointer shadow-md transition-all ${
                cart.length === 0 || isSubmitting || !isPaymentValid
                  ? 'bg-muted text-muted-foreground cursor-not-allowed shadow-none'
                  : 'bg-primary text-primary-foreground hover:bg-opacity-95'
              }`}
            >
              <Printer className="w-3.5 h-3.5" />
              <span>Valider & Imprimer Ticket</span>
            </button>
            <button
              onClick={() => handleCheckout(false)}
              disabled={cart.length === 0 || isSubmitting || !isPaymentValid}
              className={`py-2.5 rounded-xl text-xs font-bold flex items-center justify-center gap-2 cursor-pointer shadow-sm transition-all border border-border ${
                cart.length === 0 || isSubmitting || !isPaymentValid
                  ? 'bg-muted/50 text-muted-foreground cursor-not-allowed shadow-none'
                  : 'bg-card text-foreground hover:bg-accent'
              }`}
            >
              <span>Valider sans impression</span>
            </button>
          </div>
        </div>
      </div>

      {/* Barcode scan / manual entry error modal */}
      {barcodeErrorModal.isOpen && (
        <div className="fixed inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center p-4 z-[100] animate-scale-in select-none">
          <div className="bg-card border border-border w-full max-w-sm rounded-3xl shadow-2xl p-8 relative flex flex-col items-center text-center">
            <button
              onClick={closeBarcodeErrorModal}
              className="absolute top-4 right-4 p-2 rounded-full hover:bg-muted text-muted-foreground transition-colors cursor-pointer"
            >
              <X className="w-5 h-5" />
            </button>
            <div className="w-20 h-20 bg-amber-500/10 rounded-full flex items-center justify-center mb-5 text-amber-600 shadow-inner">
              <AlertTriangle className="w-10 h-10" />
            </div>
            <h3 className="text-xl font-extrabold text-foreground mb-2">{barcodeErrorModal.title}</h3>
            <p className="text-sm font-semibold text-muted-foreground mb-8">{barcodeErrorModal.message}</p>
            <button
              onClick={closeBarcodeErrorModal}
              className="w-full py-3.5 rounded-2xl bg-primary dark:bg-blue-600 text-primary-foreground text-xs font-bold shadow-md transition-colors cursor-pointer"
            >
              Fermer
            </button>
          </div>
        </div>
      )}

      {/* Delete Confirmation Modal */}
      <ConfirmModal
        isOpen={deleteConfirm.isOpen}
        title="Retirer du panier"
        message={`Voulez-vous retirer "${deleteConfirm.productName}" du panier ?`}
        confirmText="Retirer"
        cancelText="Annuler"
        onConfirm={() => {
          removeFromCart(deleteConfirm.productId);
          setDeleteConfirm({ isOpen: false, productId: '', productName: '' });
        }}
        onCancel={() => setDeleteConfirm({ isOpen: false, productId: '', productName: '' })}
      />

      {/* Client Registration Modal */}
      {showClientModal && (
        <div className="fixed inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-scale-in">
          <div className="bg-card border border-border w-full max-w-sm rounded-3xl shadow-2xl p-6">
            <h3 className="font-bold text-lg text-foreground mb-4 flex items-center gap-2">
              <UserPlus className="w-5 h-5 text-primary dark:text-blue-600" />
              Enregistrer Client
            </h3>
            <div className="space-y-4">
              <div>
                <label className="text-[10px] font-bold text-muted-foreground uppercase block mb-1">Nom Complet *</label>
                <input
                  type="text"
                  value={clientInfo.full_name}
                  onChange={(e) => setClientInfo(prev => ({ ...prev, full_name: e.target.value }))}
                  className="w-full px-3 py-2 bg-accent/20 border border-border rounded-xl text-xs font-semibold focus:outline-none focus:ring-1 focus:ring-primary"
                  placeholder="Jean Dupont"
                />
              </div>
              <div>
                <label className="text-[10px] font-bold text-muted-foreground uppercase block mb-1">Téléphone</label>
                <input
                  type="tel"
                  value={clientInfo.phone}
                  onChange={(e) => setClientInfo(prev => ({ ...prev, phone: e.target.value }))}
                  className="w-full px-3 py-2 bg-accent/20 border border-border rounded-xl text-xs font-semibold focus:outline-none focus:ring-1 focus:ring-primary"
                  placeholder="+242 00 000 0000"
                />
              </div>
              <div>
                <label className="text-[10px] font-bold text-muted-foreground uppercase block mb-1">Email</label>
                <input
                  type="email"
                  value={clientInfo.email}
                  onChange={(e) => setClientInfo(prev => ({ ...prev, email: e.target.value }))}
                  className="w-full px-3 py-2 bg-accent/20 border border-border rounded-xl text-xs font-semibold focus:outline-none focus:ring-1 focus:ring-primary"
                  placeholder="jean@example.com"
                />
              </div>
            </div>
            <div className="flex gap-3 mt-6">
              <button 
                onClick={() => setShowClientModal(false)}
                className="flex-1 py-2.5 rounded-xl border border-border bg-card hover:bg-accent text-foreground text-xs font-bold transition-all cursor-pointer"
              >
                Annuler
              </button>
              <button 
                onClick={() => {
                  if (!clientInfo.full_name) {
                    notify('Le nom complet est obligatoire', 'warning');
                    return;
                  }
                  setShowClientModal(false);
                }}
                className="flex-1 py-2.5 rounded-xl bg-primary dark:bg-blue-600 text-primary-foreground text-xs font-bold shadow-sm hover:bg-opacity-95 transition-all cursor-pointer"
              >
                Confirmer
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Gorgeous Receipt Preview Modal */}
      {showReceiptModal && receiptData && (
        <div className="fixed inset-0 bg-background/80 backdrop-blur-sm z-50 animate-scale-in flex flex-col">
          {/* Fixed top bar with close */}
          <div className="flex items-center justify-between px-6 py-3 bg-card/90 backdrop-blur-md border-b border-border shrink-0">
            <h4 className="font-extrabold text-sm text-foreground flex items-center gap-2">
              <CheckCircle className="w-5 h-5 text-emerald-500" />
              Transaction Validée
            </h4>
            <button
              onClick={() => setShowReceiptModal(false)}
              className="w-8 h-8 rounded-full bg-accent hover:bg-destructive hover:text-white flex items-center justify-center text-foreground text-sm font-bold cursor-pointer transition-all"
            >
              ✕
            </button>
          </div>

          {/* Scrollable content area */}
          <div className="flex-1 overflow-y-auto p-4 flex justify-center">
            <div className="bg-card border border-border w-full max-w-sm rounded-3xl shadow-2xl p-6 h-fit">
              
              {/* Success header */}
              <div className="flex flex-col items-center text-center pb-4 border-b border-border/50">
                <div className="w-12 h-12 rounded-full bg-emerald-500/10 text-emerald-500 flex items-center justify-center mb-2">
                  <CheckCircle className="w-6 h-6" />
                </div>
                <p className="text-[10px] text-muted-foreground mt-0.5">Le reçu a été envoyé à l'imprimante.</p>
              </div>

              {/* Thermal ticket simulator - fully visible */}
              <div className="bg-muted/30 border border-dashed border-border rounded-xl p-4 my-4 font-mono text-[11px] space-y-3">
                <div className="text-center space-y-1">
                  <p className="font-bold text-xs">AZTEA PHARMACY & POS</p>
                  <p className="text-[9px] text-muted-foreground">Brazzaville, Congo</p>
                  <p className="text-[9px] text-muted-foreground">Tel: +242 05 656 0299</p>
                </div>
                
                <div className="border-t border-dashed border-border/50 pt-2 space-y-0.5">
                  <p>Ticket: {receiptData.receipt_number}</p>
                  <p>Date: {new Date(receiptData.sold_at).toLocaleString('fr-FR')}</p>
                  <p>Caissier: {user?.name || 'Inconnu'}</p>
                  {receiptData.customer_name && <p>Client: {receiptData.customer_name}</p>}
                  <p>Périphérique : {getTicketLayout().printerLabel || 'Non configuré'}</p>
                </div>

                <div className="border-t border-dashed border-border/50 pt-2 space-y-2">
                  {receiptData.items.map((item, i) => {
                    const barcodeHtml = renderBarcodeSvg(getItemBarcode(item), 26);
                    return (
                      <div key={i} className="pb-2 border-b border-dotted border-border/40 last:border-0 space-y-1.5">
                        <div className="flex justify-between gap-2">
                          <span className="truncate max-w-[180px] font-semibold">{item.product_name}</span>
                          <span className="shrink-0 ml-2">{item.quantity} x {item.unit_price}F</span>
                        </div>
                        {barcodeHtml ? (
                          <div
                            className="flex justify-center pt-1 border-t border-dotted border-border/30 overflow-hidden"
                            dangerouslySetInnerHTML={{ __html: barcodeHtml }}
                          />
                        ) : null}
                      </div>
                    );
                  })}
                </div>

                {(() => {
                  const t = computeReceiptTotals(receiptData);
                  return (
                    <div className="border-t border-dashed border-border/50 pt-2 space-y-0.5">
                      <div className="flex justify-between font-bold">
                        <span>Sous-total:</span>
                        <span>{t.subtotal} F</span>
                      </div>
                      {t.discount > 0 && (
                        <div className="flex justify-between text-rose-500 font-semibold">
                          <span>Remise:</span>
                          <span>-{t.discount} F</span>
                        </div>
                      )}
                      <div className="flex justify-between">
                        <span>Montant HT:</span>
                        <span>{t.ht} F</span>
                      </div>
                      <p className="text-[10px] font-bold pt-1">Taxes appliquées</p>
                      {t.articleTaxes > 0 && (
                        <div className="flex justify-between pl-2 text-[10px]">
                          <span>Taxes articles:</span>
                          <span>{t.articleTaxes} F</span>
                        </div>
                      )}
                      <div className="flex justify-between pl-2 text-[10px]">
                        <span>TVA:</span>
                        <span>{t.tva} F</span>
                      </div>
                      <div className="flex justify-between font-semibold">
                        <span>Total taxes:</span>
                        <span>{t.totalTaxes} F</span>
                      </div>
                      <div className="flex justify-between font-bold text-xs pt-1 border-t border-dotted border-border/30">
                        <span>NET A PAYER:</span>
                        <span>{t.netAPayer} F</span>
                      </div>
                      {receiptData.payment_method === 'cash' && (
                        <>
                          <div className="flex justify-between">
                            <span>Montant reçu:</span>
                            <span>{receiptData.amount_paid || t.netAPayer} F</span>
                          </div>
                          <div className="flex justify-between">
                            <span>Monnaie rendue:</span>
                            <span>{receiptData.change_given} F</span>
                          </div>
                        </>
                      )}
                    </div>
                  );
                })()}

                <div className="border-t border-dashed border-border/50 pt-2 space-y-0.5">
                  <p className="capitalize">Mode: {receiptData.payment_method === 'cash' ? 'Espèces' : receiptData.payment_method === 'mobile_money' ? 'Mobile Money' : 'Carte'}</p>
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
                  onClick={() => receiptData && printReceipt(receiptData)}
                  className="flex-1 py-2.5 rounded-xl bg-primary text-primary-foreground text-xs font-bold shadow-sm hover:bg-opacity-95 transition-all cursor-pointer flex items-center justify-center gap-1.5"
                >
                  <Printer className="w-3.5 h-3.5" />
                  <span>Réimprimer</span>
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
    </>
  );
}

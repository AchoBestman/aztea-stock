import { useState, useEffect, useCallback } from "react";
import { Search, ChevronLeft, ChevronRight, Eye } from "lucide-react";
import { api, type Paginated } from "../lib/api";
import { formatDate, formatCurrency } from "../lib/format";
import Badge from "./Badge";
import Modal from "./Modal";

interface AdminDataViewProps {
  tenantId: string;
  type: "Alerts" | "Categories" | "Products" | "SyncLogs" | "Sales" | "Purchases" | "Stock";
}

export default function AdminDataView({ tenantId, type }: AdminDataViewProps) {
  const [data, setData] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [page, setPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const [total, setTotal] = useState(0);
  const [search, setSearch] = useState("");
  const [selectedItem, setSelectedItem] = useState<any | null>(null);
  const perPage = 10;

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      let res: Paginated<any>;
      const params = { tenant_id: tenantId, page, per_page: perPage, search: search || undefined };

      switch (type) {
        case "Alerts":
          res = await api.alerts.list(params);
          break;
        case "Categories":
          res = await api.categories.list(params);
          break;
        case "Products":
          res = await api.products.list(params);
          break;
        case "SyncLogs":
          res = await api.sync.logs(params);
          break;
        case "Sales":
          res = await api.sales.list(params);
          break;
        case "Purchases":
          res = await api.purchases.list(params);
          break;
        case "Stock":
          res = await api.stock.list(params);
          break;
        default:
          return;
      }
      setData(res.data);
      setTotalPages(res.total_pages);
      setTotal(res.total);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  }, [tenantId, type, page, search]);

  useEffect(() => {
    setPage(1);
  }, [type, search]);

  useEffect(() => {
    void loadData();
  }, [loadData]);

  const renderTable = () => {
    if (loading) return <div className="py-8 text-center text-muted-foreground">Chargement...</div>;
    if (data.length === 0) return <div className="py-8 text-center text-muted-foreground">Aucune donnée trouvée.</div>;

    const headers = {
      Alerts: ["Message", "Type", "Lue", "Date"],
      Categories: ["Nom", "Description", "Statut", "Date"],
      Products: ["Produit", "SKU", "Prix Sell", "Stock", "Statut"],
      SyncLogs: ["Appareil", "Type", "Status", "Push/Pull", "Date"],
      Sales: ["Client", "Total", "Net", "Méthode", "Statut", "Date"],
      Purchases: ["Fournisseur", "Total", "Statut", "Date"],
      Stock: ["Produit ID", "Type", "Quantité", "Précédent", "Nouveau", "Date"],
    };

    const currentHeaders = headers[type];

    return (
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead className="bg-muted/50 text-left">
            <tr>
              {currentHeaders.map((h) => (
                <th key={h} className="px-4 py-3 font-semibold">{h}</th>
              ))}
              <th className="px-4 py-3 font-semibold text-right">Actions</th>
            </tr>
          </thead>
          <tbody>
            {data.map((item) => (
              <tr key={item.id} className="border-t border-border hover:bg-accent/30">
                {renderRow(item)}
                <td className="px-4 py-3 text-right">
                  <button
                    onClick={() => setSelectedItem(item)}
                    className="p-1.5 hover:bg-muted rounded-lg text-muted-foreground hover:text-primary transition-colors cursor-pointer"
                  >
                    <Eye className="w-4 h-4" />
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    );
  };

  const renderRow = (item: any) => {
    switch (type) {
      case "Alerts":
        return (
          <>
            <td className="px-4 py-3 max-w-xs truncate">{item.message}</td>
            <td className="px-4 py-3 text-xs">{item.alert_type}</td>
            <td className="px-4 py-3">
              <Badge label={item.is_read ? "Oui" : "Non"} tone={item.is_read ? "active" : "suspended"} />
            </td>
            <td className="px-4 py-3 text-xs text-muted-foreground">{formatDate(item.created_at)}</td>
          </>
        );
      case "Categories":
        return (
          <>
            <td className="px-4 py-3 font-medium">{item.name}</td>
            <td className="px-4 py-3 text-xs text-muted-foreground max-w-xs truncate">{item.description || "—"}</td>
            <td className="px-4 py-3">
              <Badge label={item.is_active ? "Actif" : "Suspendu"} tone={item.is_active ? "active" : "suspended"} />
            </td>
            <td className="px-4 py-3 text-xs text-muted-foreground">{formatDate(item.created_at)}</td>
          </>
        );
      case "Products":
        return (
          <>
            <td className="px-4 py-3 font-medium">{item.name}</td>
            <td className="px-4 py-3 text-xs">{item.sku || "—"}</td>
            <td className="px-4 py-3 font-semibold">{formatCurrency(item.price_sell)}</td>
            <td className="px-4 py-3">
              <span className={`font-mono ${item.stock_quantity <= item.stock_min ? "text-destructive font-bold" : ""}`}>
                {item.stock_quantity}
              </span>
            </td>
            <td className="px-4 py-3">
              <Badge label={item.is_active ? "Actif" : "Suspendu"} tone={item.is_active ? "active" : "suspended"} />
            </td>
          </>
        );
      case "SyncLogs":
        return (
          <>
            <td className="px-4 py-3 font-mono text-[10px]">{item.device_id}</td>
            <td className="px-4 py-3 text-xs">{item.sync_type || "—"}</td>
            <td className="px-4 py-3">
              <Badge label={item.status || "—"} tone={item.status === "success" ? "active" : "suspended"} />
            </td>
            <td className="px-4 py-3 text-xs">
              <span className="text-primary">↑{item.records_pushed}</span> / <span className="text-orange-500">↓{item.records_pulled}</span>
            </td>
            <td className="px-4 py-3 text-xs text-muted-foreground">{formatDate(item.started_at)}</td>
          </>
        );
      case "Sales":
        return (
          <>
            <td className="px-4 py-3 truncate">{item.customer_name || "Client de passage"}</td>
            <td className="px-4 py-3 font-semibold">{formatCurrency(item.total_amount)}</td>
            <td className="px-4 py-3 font-semibold text-primary">{formatCurrency(item.net_amount)}</td>
            <td className="px-4 py-3 text-[10px] uppercase">{item.payment_method}</td>
            <td className="px-4 py-3">
              <Badge label={item.status} tone={item.status === "completed" ? "active" : item.status === "voided" ? "suspended" : "default"} />
            </td>
            <td className="px-4 py-3 text-xs text-muted-foreground">{formatDate(item.created_at)}</td>
          </>
        );
      case "Purchases":
        return (
          <>
            <td className="px-4 py-3">{item.supplier_name || "—"}</td>
            <td className="px-4 py-3 font-semibold">{formatCurrency(item.total_amount)}</td>
            <td className="px-4 py-3">
              <Badge label={item.status} tone={item.status === "completed" ? "active" : "default"} />
            </td>
            <td className="px-4 py-3 text-xs text-muted-foreground">{formatDate(item.created_at)}</td>
          </>
        );
      case "Stock":
        return (
          <>
            <td className="px-4 py-3 font-mono text-[10px] truncate max-w-[80px]">{item.product_id}</td>
            <td className="px-4 py-3">
              <Badge label={item.operation_type} tone={item.quantity > 0 ? "default" : "suspended"} />
            </td>
            <td className="px-4 py-3 font-bold">{item.quantity > 0 ? `+${item.quantity}` : item.quantity}</td>
            <td className="px-4 py-3 text-muted-foreground">{item.previous_stock}</td>
            <td className="px-4 py-3 font-semibold">{item.new_stock}</td>
            <td className="px-4 py-3 text-xs text-muted-foreground">{formatDate(item.created_at)}</td>
          </>
        );
      default:
        return null;
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div className="relative flex-1 max-w-sm">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <input
            className="w-full pl-10 pr-4 py-2 rounded-xl border border-input bg-background/50 focus:bg-background transition-all outline-none"
            placeholder="Rechercher..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>
        <div className="text-xs text-muted-foreground font-medium">
          {total} résultat(s)
        </div>
      </div>

      <div className="bg-background/40 border border-border/50 rounded-2xl overflow-hidden">
        {renderTable()}
      </div>

      {totalPages > 1 && (
        <div className="flex items-center justify-center gap-4 pt-2">
          <button
            disabled={page <= 1}
            onClick={() => setPage(p => p - 1)}
            className="p-2 rounded-lg border border-border disabled:opacity-40 hover:bg-accent transition-colors cursor-pointer"
          >
            <ChevronLeft className="w-4 h-4" />
          </button>
          <span className="text-sm font-medium">
            Page {page} / {totalPages}
          </span>
          <button
            disabled={page >= totalPages}
            onClick={() => setPage(p => p + 1)}
            className="p-2 rounded-lg border border-border disabled:opacity-40 hover:bg-accent transition-colors cursor-pointer"
          >
            <ChevronRight className="w-4 h-4" />
          </button>
        </div>
      )}

      {selectedItem && (
        <Modal title={`Détail : ${type}`} onClose={() => setSelectedItem(null)} wide>
          <div className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {Object.entries(selectedItem).map(([key, value]) => (
                <div key={key} className="space-y-1 border-b border-border/50 pb-2">
                  <p className="text-[10px] uppercase font-bold text-muted-foreground">{key.replace(/_/g, " ")}</p>
                  <p className="text-sm font-medium break-words">
                    {typeof value === "boolean" ? (value ? "Oui" : "Non") : 
                     (key.includes("price") || key.includes("amount")) && typeof value === "number" ? formatCurrency(value) :
                     (key.includes("_at") || key === "date") && typeof value === "string" ? formatDate(value) :
                     String(value ?? "—")}
                  </p>
                </div>
              ))}
            </div>
            <div className="flex justify-end pt-4">
              <button
                onClick={() => setSelectedItem(null)}
                className="px-6 py-2 rounded-xl bg-muted font-semibold hover:bg-accent transition-colors cursor-pointer"
              >
                Fermer
              </button>
            </div>
          </div>
        </Modal>
      )}
    </div>
  );
}

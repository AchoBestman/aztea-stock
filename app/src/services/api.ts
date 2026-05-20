// API Client Service for Aztea Stock
// Integrates the React frontend with the Rust Axum backend

export function getApiBaseUrl(): string {
  return localStorage.getItem('aztea_api_base_url') || '';
}

export function setApiBaseUrl(url: string) {
  localStorage.setItem('aztea_api_base_url', url);
}

export interface UserProfile {
  id: string;
  name: string;
  email: string;
  role: string;
  tenant_id: string;
  tenant_name: string;
  roles: string[];
  permissions: string[];
}

export interface LoginResponse {
  requires_two_factor: boolean;
  message: string | null;
  access_token: string | null;
  refresh_token: string | null;
  expires_in: number | null;
  user: UserProfile | null;
}

export interface Category {
  id: string;
  tenant_id: string;
  parent_id: string | null;
  name: string;
  description: string | null;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface PaginatedCategories {
  data: Category[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

export interface Product {
  id: string;
  tenant_id: string;
  category_id: string | null;
  category_name: string | null;
  barcode: string | null;
  name: string;
  description: string | null;
  brand: string | null;
  unit: string;
  purchase_price: number;
  selling_price: number;
  tax_rate: number;
  image_url: string | null;
  is_active: boolean;
  requires_prescription: boolean;
  created_at: string;
  updated_at: string;
}

export interface PaginatedProducts {
  data: Product[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

export interface StockItem {
  id: string;
  tenant_id: string;
  product_id: string;
  product_name: string;
  quantity: number;
  quantity_reserved: number;
  low_stock_threshold: number;
  unit_location: string | null;
  batch_number: string | null;
  expiry_date: string | null;
  updated_at: string;
}

export interface PaginatedStockItems {
  data: StockItem[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

export interface StockMovement {
  id: string;
  tenant_id: string;
  product_id: string;
  product_name: string;
  user_id: string | null;
  user_name: string | null;
  movement_type: 'sale' | 'purchase' | 'adjustment' | 'return' | 'loss' | 'initial';
  quantity_before: number;
  quantity_change: number;
  quantity_after: number;
  reference_id: string | null;
  note: string | null;
  occurred_at: string;
}

export interface PaginatedStockMovements {
  data: StockMovement[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

export interface SaleItem {
  id: string;
  product_id: string;
  product_name: string;
  product_barcode: string | null;
  quantity: number;
  unit_price: number;
  tax_rate: number;
  discount: number;
  line_total: number;
}

export interface Sale {
  id: string;
  tenant_id: string;
  user_id: string | null;
  receipt_number: string;
  customer_name: string | null;
  customer_phone: string | null;
  subtotal: number;
  tax_total: number;
  discount_total: number;
  total: number;
  amount_paid: number;
  change_given: number;
  payment_method: string;
  status: string;
  notes: string | null;
  sold_at: string;
  created_at: string;
  items: SaleItem[];
}

export interface PaginatedSales {
  data: Sale[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

export interface SyncLog {
  id: string;
  tenant_id: string;
  user_id?: string;
  device_id?: string;
  status: 'success' | 'failed' | 'partial';
  records_synced: number;
  errors?: any;
  created_at: string;
}

export interface PaginatedSyncLogs {
  data: SyncLog[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

export interface Permission {
  id: string;
  name: string;
  description?: string;
}

export interface GroupedPermission {
  group: string;
  permissions: Permission[];
}

export interface Role {
  id: string;
  name: string;
  description?: string;
  tenant_id: string;
  is_system?: boolean;
}

export interface AdminUser {
  id: string;
  name: string;
  email: string;
  roles: string[];
  is_active: boolean;
  tenant_id: string;
  created_at: string;
}

export interface TenantResponse {
  id: string;
  name: string;
  business_type: string;
  email: string;
  phone: string | null;
  address: string | null;
  country: string | null;
  timezone: string | null;
  logo_url: string | null;
  is_active: boolean | null;
  is_system: boolean;
  two_factor_enabled: boolean;
  sender_email: string | null;
  created_at: string;
  updated_at: string;
}

export interface LicenseStatusResponse {
  has_active_license: boolean;
  status: 'active' | 'trial' | 'expired' | 'suspended' | 'revoked';
  license_id: string | null;
  subscription_plan: string | null;
  expires_at: string | null;
  days_remaining: number | null;
  renewal_alert: boolean;
}

let cachedFingerprint: string | null = null;

async function getDeviceFingerprint(): Promise<string> {
  if (cachedFingerprint) return cachedFingerprint;

  try {
    const isTauri = typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__ !== undefined;
    if (isTauri) {
      const { invoke } = await import('@tauri-apps/api/core');
      const fp = await invoke<string>('get_device_fingerprint');
      cachedFingerprint = fp;
      return fp;
    }
  } catch (e) {
    console.error("Tauri get_device_fingerprint failed:", e);
  }

  // Fallback valid encrypted mock fingerprint for browser
  return 'AAAAAAAAAAAAAAAAAAAAAMKuRLPzNfGMEejIg4eDQgmz1w80ljy5t1GqcdX03uvIZXLMrxZMlH3hmJq5l0wRkQ==';
}

// Request Helper
async function request<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<T> {
  const token = localStorage.getItem('aztea_access_token');
  const headers = new Headers(options.headers || {});

  if (token) {
    headers.set('Authorization', `Bearer ${token}`);
  }

  // Securely inject the encrypted device fingerprint header for license gate middleware
  const fingerprint = await getDeviceFingerprint();
  headers.set('x-device-fingerprint', fingerprint);

  if (!headers.has('Content-Type') && !(options.body instanceof FormData)) {
    headers.set('Content-Type', 'application/json');
  }

  const response = await fetch(`${getApiBaseUrl()}${endpoint}`, {
    ...options,
    headers,
  });

  if (!response.ok) {
    if (response.status === 401) {
      localStorage.removeItem('aztea_access_token');
      localStorage.removeItem('aztea_user');
      window.dispatchEvent(new Event('auth-logout'));
    }
    const errorData = await response.json().catch(() => ({}));
    throw new Error(errorData.message || `API Error: ${response.statusText}`);
  }

  return response.json() as Promise<T>;
}

export const api = {
  // Authentication
  auth: {
    login: (email: string, password: string) =>
      request<LoginResponse>('/auth/login', {
        method: 'POST',
        body: JSON.stringify({ email, password }),
      }),
    
    getProfile: () => request<UserProfile>('/auth/profile'),
  },

  // Tenants management
  tenants: {
    get: () => request<TenantResponse>('/admin/tenant'),
  },

  // Licenses
  licenses: {
    getStatus: () => request<LicenseStatusResponse>('/licenses/status'),
    activate: async (licenseKey: string) => {
      let device_fingerprint: string | undefined;
      let device_name: string | undefined;

      try {
        const isTauri = typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__ !== undefined;
        if (isTauri) {
          const { invoke } = await import('@tauri-apps/api/core');
          const info = await invoke<{ name: string; fingerprint: string }>('get_device_info');
          device_fingerprint = info.fingerprint;
          device_name = info.name;
        } else {
          device_fingerprint = 'AAAAAAAAAAAAAAAAAAAAAMKuRLPzNfGMEejIg4eDQgmz1w80ljy5t1GqcdX03uvIZXLMrxZMlH3hmJq5l0wRkQ==';
          device_name = 'Navigateur Web (Simulé)';
        }
      } catch (e) {
        console.error("Tauri get_device_info failed during activation:", e);
      }

      return request<any>('/licenses/activate', {
        method: 'POST',
        body: JSON.stringify({
          license_key: licenseKey,
          device_name,
          device_fingerprint,
        }),
      });
    },
  },

  // Categories
  categories: {
    list: (search = '', page = 1, perPage = 100) => {
      const params = new URLSearchParams({
        page: page.toString(),
        per_page: perPage.toString(),
      });
      if (search) params.append('search', search);
      return request<PaginatedCategories>(`/categories?${params.toString()}`);
    },
    
    create: (name: string, description?: string, parentId?: string) =>
      request<Category>('/categories', {
        method: 'POST',
        body: JSON.stringify({
          name,
          description: description || null,
          parent_id: parentId || null,
          is_active: true,
        }),
      }),

    update: (id: string, name: string, description?: string | null) =>
      request<Category>(`/categories/${id}`, {
        method: 'PUT',
        body: JSON.stringify({ name, description: description || null, is_active: true }),
      }),

    delete: (id: string) =>
      request<{ id: string }>(`/categories/${id}`, {
        method: 'DELETE',
      }),
  },

  // Products
  products: {
    list: (search = '', categoryId = '', page = 1, perPage = 100) => {
      const params = new URLSearchParams({
        page: page.toString(),
        per_page: perPage.toString(),
      });
      if (search) params.append('search', search);
      if (categoryId) params.append('category_id', categoryId);
      return request<PaginatedProducts>(`/products?${params.toString()}`);
    },

    create: (payload: {
      name: string;
      barcode?: string;
      category_id?: string;
      description?: string;
      brand?: string;
      unit: string;
      purchase_price: number;
      selling_price: number;
      tax_rate: number;
      image_url?: string;
      requires_prescription?: boolean;
    }) =>
      request<Product>('/products', {
        method: 'POST',
        body: JSON.stringify({
          ...payload,
          is_active: true,
        }),
      }),

    update: (
      id: string,
      payload: Partial<{
        name: string;
        barcode: string | null;
        category_id: string | null;
        description: string | null;
        brand: string | null;
        unit: string;
        purchase_price: number;
        selling_price: number;
        tax_rate: number;
        image_url: string | null;
        is_active: boolean;
        requires_prescription: boolean;
      }>
    ) =>
      request<Product>(`/products/${id}`, {
        method: 'PUT',
        body: JSON.stringify(payload),
      }),

    delete: (id: string) =>
      request<{ id: string }>(`/products/${id}`, {
        method: 'DELETE',
      }),
  },

  // Stock Items & Movements
  stock: {
    listItems: (search = '', isLowStock = false, categoryId = '', page = 1, perPage = 100) => {
      const params = new URLSearchParams({
        page: page.toString(),
        per_page: perPage.toString(),
      });
      if (search) params.append('search', search);
      if (isLowStock) params.append('is_low_stock', 'true');
      if (categoryId) params.append('category_id', categoryId);
      return request<PaginatedStockItems>(`/stock/items?${params.toString()}`);
    },

    createItem: (payload: {
      product_id: string;
      quantity?: number;
      low_stock_threshold?: number;
      unit_location?: string;
      batch_number?: string;
      expiry_date?: string;
    }) =>
      request<StockItem>('/stock/items', {
        method: 'POST',
        body: JSON.stringify(payload),
      }),

    updateItem: (
      id: string,
      payload: Partial<{
        quantity: number;
        low_stock_threshold: number;
        unit_location: string | null;
        batch_number: string | null;
        expiry_date: string | null;
      }>
    ) =>
      request<StockItem>(`/stock/items/${id}`, {
        method: 'PUT',
        body: JSON.stringify(payload),
      }),

    createMovement: (payload: {
      product_id: string;
      movement_type: 'sale' | 'purchase' | 'adjustment' | 'return' | 'loss' | 'initial';
      quantity_change: number;
      reference_id?: string;
      note?: string;
    }) =>
      request<StockMovement>('/stock/movements', {
        method: 'POST',
        body: JSON.stringify(payload),
      }),

    listMovements: (productId = '', movementType = '', page = 1, perPage = 100) => {
      const params = new URLSearchParams({
        page: page.toString(),
        per_page: perPage.toString(),
      });
      if (productId) params.append('product_id', productId);
      if (movementType) params.append('movement_type', movementType);
      return request<PaginatedStockMovements>(`/stock/movements?${params.toString()}`);
    },
  },

  // Sales (POS)
  sales: {
    create: (payload: {
      customer_name?: string;
      customer_phone?: string;
      payment_method: string;
      notes?: string;
      items: {
        product_id: string;
        quantity: number;
        unit_price: number;
        tax_rate?: number;
        discount?: number;
      }[];
    }) =>
      request<Sale>('/sales', {
        method: 'POST',
        body: JSON.stringify(payload),
      }),

    list: (search = '', status = '', page = 1, perPage = 100) => {
      const params = new URLSearchParams({
        page: page.toString(),
        per_page: perPage.toString(),
      });
      if (search) params.append('customer_name', search);
      if (status) params.append('status', status);
      return request<PaginatedSales>(`/sales?${params.toString()}`);
    },
  },

  // Sync
  sync: {
    logs: (startDate?: string, endDate?: string, page = 1, perPage = 100) => {
      const params = new URLSearchParams({
        page: page.toString(),
        per_page: perPage.toString(),
      });
      if (startDate) params.append('start_date', startDate);
      if (endDate) params.append('end_date', endDate);
      return request<PaginatedSyncLogs>(`/sync/logs?${params.toString()}`);
    },
  },

  // Admin: Users & Roles & Tenants
  admin: {
    tenants: {
      list: () => request<TenantResponse[]>('/admin/tenants'),
    },
    users: {
      list: () => request<AdminUser[]>('/admin/users'),
      create: (payload: { name: string; email: string; role_id: string; tenant_id?: string }) => 
        request<AdminUser>('/admin/users', {
          method: 'POST',
          body: JSON.stringify(payload),
        }),
      delete: (id: string) =>
        request<{success: boolean; message: string}>(`/admin/users/${id}`, {
          method: 'DELETE',
        }),
    },
    roles: {
      list: () => request<Role[]>('/admin/roles'),
      create: (payload: { name: string; description: string; tenant_id?: string }) => 
        request<Role>('/admin/roles', {
          method: 'POST',
          body: JSON.stringify(payload),
        }),
      delete: (id: string) =>
        request<{success: boolean; message: string}>(`/admin/roles/${id}`, {
          method: 'DELETE',
        }),
      listPermissions: (roleId: string) => request<Permission[]>(`/admin/roles/${roleId}/permissions`),
      assignPermissions: (roleId: string, permissionIds: string[]) => 
        request<{success: boolean; message: string}>(`/admin/roles/${roleId}/permissions`, {
          method: 'POST',
          body: JSON.stringify({ permission_ids: permissionIds }),
        }),
    },
    permissions: {
      listGrouped: () => request<GroupedPermission[]>('/admin/permissions'),
    }
  }
};

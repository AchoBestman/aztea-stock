import { getApiBaseUrl } from "./env";

const TOKEN_KEY = "aztea_admin_access_token";
const USER_KEY = "aztea_admin_user";

/** Browser fallback fingerprint (same as POS web mode) for license middleware */
const WEB_DEVICE_FINGERPRINT =
  "AAAAAAAAAAAAAAAAAAAAAMKuRLPzNfGMEejIg4eDQgmz1w80ljy5t1GqcdX03uvIZXLMrxZMlH3hmJq5l0wRkQ==";

let cachedFingerprint: string | null = null;

async function getDeviceFingerprint(): Promise<string> {
  if (!cachedFingerprint) cachedFingerprint = WEB_DEVICE_FINGERPRINT;
  return cachedFingerprint;
}

export class ApiError extends Error {
  status: number;
  constructor(message: string, status: number) {
    super(message);
    this.name = "ApiError";
    this.status = status;
  }
}

export interface UserProfileTenant {
  name: string;
  email: string;
  phone: string | null;
  country: string | null;
  address: string | null;
  business_type: string;
  created_at: string;
  is_active: boolean | null;
}

export interface UserProfile {
  id: string;
  name: string;
  email: string;
  role: string;
  tenant_id: string;
  tenant_name: string;
  tenant: UserProfileTenant;
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

export interface Tenant {
  id: string;
  name: string;
  business_type: string;
  email: string;
  phone: string | null;
  address: string | null;
  city: string | null;
  country: string | null;
  country_code: string | null;
  timezone: string | null;
  logo_url: string | null;
  is_active: boolean | null;
  is_system: boolean;
  two_factor_enabled: boolean;
  sender_email: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateTenantPayload {
  name: string;
  business_type: string;
  email: string;
  phone?: string;
  address?: string;
  country: string;
  country_code?: string;
  city: string;
  timezone: string;
  logo_url?: string;
}

export interface UpdateTenantPayload {
  name?: string;
  business_type?: string;
  email?: string;
  phone?: string | null;
  address?: string | null;
  country?: string;
  country_code?: string;
  city?: string;
  timezone?: string;
  logo_url?: string | null;
  is_active?: boolean | null;
  two_factor_enabled?: boolean;
}

export interface Subscription {
  id: string;
  tenant_id: string;
  plan: string;
  status: string;
  price_monthly: string | number;
  currency: string;
  started_at: string;
  expires_at: string;
  trial_ends_at: string | null;
  cancelled_at: string | null;
  notes: string | null;
  created_at: string;
}

export interface CreateSubscriptionPayload {
  tenant_id: string;
  plan: string;
  status: string;
  price_monthly: number;
  currency?: string;
  expires_at: string;
  trial_ends_at?: string;
  notes?: string;
}

export interface License {
  id: string;
  tenant_id: string;
  subscription_id: string;
  license_key_masked: string;
  is_active: boolean;
  device_name: string | null;
  device_fingerprint: string | null;
  last_verified_at: string | null;
  activated_at: string | null;
  revoked_at: string | null;
  created_at: string;
}

export interface FullLicense {
  id: string;
  tenant_id: string;
  subscription_id: string;
  license_key_plain: string;
  is_active: boolean;
  created_at: string;
}

export interface Paginated<T> {
  data: T[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

export interface SyncLog {
  id: string;
  tenant_id: string;
  device_id: string;
  sync_type: string | null;
  status: string | null;
  records_pushed: number;
  records_pulled: number;
  error_message: string | null;
  started_at: string;
  finished_at: string | null;
}

export function getStoredToken(): string | null {
  return localStorage.getItem(TOKEN_KEY);
}

export function getStoredUser(): UserProfile | null {
  const raw = localStorage.getItem(USER_KEY);
  if (!raw) return null;
  try {
    return JSON.parse(raw) as UserProfile;
  } catch {
    return null;
  }
}

export function clearSession(): void {
  localStorage.removeItem(TOKEN_KEY);
  localStorage.removeItem(USER_KEY);
  window.dispatchEvent(new Event("admin-auth-logout"));
}

async function request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
  const token = getStoredToken();
  const headers = new Headers(options.headers || {});

  if (token) headers.set("Authorization", `Bearer ${token}`);
  headers.set("x-device-fingerprint", await getDeviceFingerprint());

  if (!headers.has("Content-Type") && !(options.body instanceof FormData)) {
    headers.set("Content-Type", "application/json");
  }

  const base = getApiBaseUrl().replace(/\/$/, "");
  const path = endpoint.startsWith("/") ? endpoint : `/${endpoint}`;
  const response = await fetch(`${base}${path}`, { ...options, headers });

  if (!response.ok) {
    if (response.status === 401) clearSession();
    const body = await response.json().catch(() => ({}));
    const errPayload = (body as { error?: { message?: string } | string }).error;
    const message =
      (typeof errPayload === "object" && errPayload?.message) ||
      (typeof errPayload === "string" ? errPayload : undefined) ||
      (body as { message?: string }).message ||
      response.statusText;
    throw new ApiError(message, response.status);
  }

  if (response.status === 204) return undefined as T;
  return response.json() as Promise<T>;
}

function qs(params: Record<string, string | number | undefined | null>): string {
  const sp = new URLSearchParams();
  for (const [k, v] of Object.entries(params)) {
    if (v !== undefined && v !== null && v !== "") sp.set(k, String(v));
  }
  const s = sp.toString();
  return s ? `?${s}` : "";
}

export const api = {
  auth: {
    login: (email: string, password: string) =>
      request<LoginResponse>("/auth/login", {
        method: "POST",
        body: JSON.stringify({ email, password }),
      }),

    verifyOtp: (email: string, otp_code: string) =>
      request<LoginResponse>("/auth/verify-otp", {
        method: "POST",
        body: JSON.stringify({ email, otp_code }),
      }),

    profile: () => request<UserProfile>("/auth/profile"),
  },

  tenant: {
    me: () => request<Tenant>("/admin/tenant"),
  },

  tenants: {
    list: (params?: {
      search?: string;
      business_type?: string;
      is_active?: string;
      page?: number;
      per_page?: number;
    }) => request<Paginated<Tenant>>(`/admin/tenants${qs(params || {})}`),

    create: (payload: CreateTenantPayload) =>
      request<Tenant>("/admin/tenants", {
        method: "POST",
        body: JSON.stringify(payload),
      }),

    update: (tenantId: string, payload: UpdateTenantPayload) =>
      request<Tenant>(`/admin/tenant${qs({ tenant_id: tenantId })}`, {
        method: "PUT",
        body: JSON.stringify(payload),
      }),

    get: (id: string) => request<Tenant>(`/admin/tenants/${id}`),

    remove: (id: string) =>
      request<Tenant>(`/admin/tenants/${id}`, { method: "DELETE" }),
  },

  subscriptions: {
    list: (params?: { page?: number; per_page?: number; tenant_id?: string }) =>
      request<Paginated<Subscription>>(`/admin/subscriptions${qs(params || {})}`),

    create: (payload: CreateSubscriptionPayload) =>
      request<Subscription>("/admin/subscriptions", {
        method: "POST",
        body: JSON.stringify({
          ...payload,
          price_monthly: payload.price_monthly,
        }),
      }),

    delete: (id: string) =>
      request<{ success: boolean; message: string }>(`/admin/subscriptions/${id}`, {
        method: "DELETE",
      }),

    updateStatus: (id: string, status: string) =>
      request<Subscription>(`/admin/subscriptions/${id}/status`, {
        method: "PATCH",
        body: JSON.stringify({ status }),
      }),
  },

  licenses: {
    list: (params?: { page?: number; per_page?: number; tenant_id?: string }) =>
      request<Paginated<License>>(`/admin/licenses${qs(params || {})}`),

    generate: (tenant_id: string, subscription_id: string) =>
      request<FullLicense>("/admin/licenses", {
        method: "POST",
        body: JSON.stringify({ tenant_id, subscription_id }),
      }),

    reveal: (id: string) =>
      request<{ id: string; tenant_id: string; license_key_plain: string }>(`/admin/licenses/${id}/reveal`),

    sendKey: (id: string) =>
      request<{ success: boolean; message: string }>(`/admin/licenses/${id}/send-key`, {
        method: "POST",
      }),
  },

  sync: {
    logs: (params: {
      tenant_id: string;
      page?: number;
      per_page?: number;
      start_date?: string;
      end_date?: string;
      device_id?: string;
    }) => request<Paginated<SyncLog>>(`/sync/logs${qs(params)}`),
  },

  users: {
    list: (tenant_id: string) =>
      request<AdminUser[]>(`/admin/users${qs({ tenant_id })}`),

    create: (payload: {
      name: string;
      email: string;
      role_id: string;
      tenant_id: string;
    }) =>
      request<AdminUser>("/admin/users", {
        method: "POST",
        body: JSON.stringify(payload),
      }),

    updateStatus: (user_id: string, is_active: boolean) =>
      request<{ id: string; is_active: boolean }>("/auth/profile", {
        method: "PUT",
        body: JSON.stringify({ user_id, is_active }),
      }),

    sendReset: (email: string) =>
      request<{ success: boolean; message: string }>("/admin/users/send-reset", {
        method: "POST",
        body: JSON.stringify({ email }),
      }),
  },

  roles: {
    list: (tenant_id: string) =>
      request<Role[]>(`/admin/roles${qs({ tenant_id })}`),

    create: (payload: CreateRolePayload) =>
      request<Role>("/admin/roles", {
        method: "POST",
        body: JSON.stringify(payload),
      }),

    getPermissions: (roleId: string) =>
      request<Permission[]>(`/admin/roles/${roleId}/permissions`),

    setPermissions: (roleId: string, permission_ids: string[]) =>
      request<{ success: boolean; message: string }>(
        `/admin/roles/${roleId}/permissions`,
        { method: "POST", body: JSON.stringify({ permission_ids }) }
      ),

    update: (roleId: string, payload: { name: string; description?: string }) =>
      request<Role>(`/admin/roles/${roleId}`, {
        method: "PUT",
        body: JSON.stringify(payload),
      }),
  },

  permissions: {
    listGrouped: () =>
      request<GroupedPermission[]>("/admin/permissions"),
  },
};

export interface Role {
  id: string;
  name: string;
  description?: string;
  tenant_id: string;
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

export interface CreateRolePayload {
  name: string;
  description?: string;
  tenant_id?: string;
}

export interface AdminUser {
  id: string;
  tenant_id: string;
  name: string;
  email: string;
  roles: string[];
  is_active: boolean | null;
  two_factor_enabled: boolean;
  created_at: string;
  updated_at: string;
}

export function persistSession(login: LoginResponse): UserProfile | null {
  if (!login.access_token || !login.user) return null;
  localStorage.setItem(TOKEN_KEY, login.access_token);
  localStorage.setItem(USER_KEY, JSON.stringify(login.user));
  return login.user;
}

import { create } from 'zustand';
import { api, UserProfile } from '../services/api';

export interface User {
  id: string;
  name: string;
  role: 'cashier' | 'manager' | 'admin' | 'Super Admin';
  tenantId: string;
  tenantName: string;
  tenantLogoUrl: string | null;
  tenantBusinessType: string;
}

interface AuthState {
  user: User | null;
  permissions: string[];
  roles: string[];
  licenseKey: string | null;
  licenseStatus: 'active' | 'trial' | 'expired' | 'suspended' | 'revoked';
  trialDaysLeft: number;
  isAuthenticated: boolean;
  login: (email: string, password: string) => Promise<'success' | '2fa' | false>;
  verifyOtp: (email: string, otpCode: string) => Promise<boolean>;
  logout: () => void;
  hydrateSession: () => Promise<void>;
  activateLicense: (key: string) => Promise<boolean>;
  checkLicenseStatus: () => Promise<void>;
}

// Helper to map UserProfile to store User
const mapProfileToUser = (profile: UserProfile): User => {
  let role: 'cashier' | 'manager' | 'admin' | 'Super Admin' = 'cashier';
  if (profile.role.toLowerCase().includes('super admin')) {
    role = 'Super Admin';
  } else if (profile.role.toLowerCase().includes('admin')) {
    role = 'admin';
  } else if (profile.role.toLowerCase().includes('manager')) {
    role = 'manager';
  }
  return {
    id: profile.id,
    name: profile.name,
    role,
    tenantId: profile.tenant_id,
    tenantName: profile.tenant?.name ?? profile.tenant_name ?? 'Établissement',
    tenantLogoUrl: profile.tenant?.logo_url ?? null,
    tenantBusinessType: profile.tenant?.business_type ?? 'pharmacy',
  };
};

function readStoredAuth(): {
  user: User | null;
  permissions: string[];
  roles: string[];
  isAuthenticated: boolean;
} {
  const savedToken = localStorage.getItem('aztea_access_token');
  const savedUser = localStorage.getItem('aztea_user');
  if (!savedToken || !savedUser) {
    return { user: null, permissions: [], roles: [], isAuthenticated: false };
  }
  try {
    const profile = JSON.parse(savedUser) as UserProfile;
    return {
      user: mapProfileToUser(profile),
      permissions: profile.permissions ?? [],
      roles: profile.roles ?? [],
      isAuthenticated: true,
    };
  } catch (e) {
    console.error('Failed to parse saved user', e);
    return { user: null, permissions: [], roles: [], isAuthenticated: false };
  }
}

const initialAuth = readStoredAuth();

export const useAuthStore = create<AuthState>((set, get) => {
  // Listen to logout events triggered by API client on 401 errors
  window.addEventListener('auth-logout', () => {
    set({
      user: null,
      permissions: [],
      roles: [],
      isAuthenticated: false,
      licenseKey: null,
    });
  });

  const applyUserProfile = (profile: UserProfile) => {
    localStorage.setItem('aztea_user', JSON.stringify(profile));
    set({
      user: mapProfileToUser(profile),
      permissions: profile.permissions ?? [],
      roles: profile.roles ?? [],
    });
  };

  return {
    user: initialAuth.user,
    permissions: initialAuth.permissions,
    roles: initialAuth.roles,
    licenseKey: localStorage.getItem('aztea_license_key') || null,
    licenseStatus: (localStorage.getItem('aztea_license_status') as any) || 'active',
    trialDaysLeft: parseInt(localStorage.getItem('aztea_trial_days') || '14', 10),
    isAuthenticated: initialAuth.isAuthenticated,

    login: async (email, password) => {
      try {
        const response = await api.auth.login(email, password);

        // 2FA required — no token yet
        if (response.requires_two_factor) {
          return '2fa';
        }

        if (response.access_token && response.user) {
          localStorage.setItem('aztea_access_token', response.access_token);
          applyUserProfile(response.user);
          set({ isAuthenticated: true });

          // Fetch license status directly after login
          await get().checkLicenseStatus();
          return 'success';
        }
        return false;
      } catch (error) {
        console.error('Login error:', error);
        throw error;
      }
    },

    verifyOtp: async (email, otpCode) => {
      try {
        const response = await api.auth.verifyOtp(email, otpCode);
        if (response.access_token && response.user) {
          localStorage.setItem('aztea_access_token', response.access_token);
          applyUserProfile(response.user);
          set({ isAuthenticated: true });
          await get().checkLicenseStatus();
          return true;
        }
        return false;
      } catch (error) {
        console.error('Verify OTP error:', error);
        throw error;
      }
    },

    logout: () => {
      localStorage.removeItem('aztea_access_token');
      localStorage.removeItem('aztea_user');
      localStorage.removeItem('aztea_license_key');
      localStorage.removeItem('aztea_license_status');
      localStorage.removeItem('aztea_trial_days');
      set({
        user: null,
        permissions: [],
        roles: [],
        isAuthenticated: false,
        licenseKey: null,
      });
    },

    hydrateSession: async () => {
      const token = localStorage.getItem('aztea_access_token');
      const raw = localStorage.getItem('aztea_user');
      if (!token || !raw) return;

      try {
        const stored = JSON.parse(raw) as UserProfile;
        const profile = await api.auth.getProfile();
        const merged: UserProfile = {
          ...stored,
          id: profile.id,
          name: profile.name,
          email: profile.email,
          tenant: profile.tenant,
          tenant_name: profile.tenant.name,
          roles: profile.roles,
          permissions: profile.permissions,
        };
        applyUserProfile(merged);
      } catch (error) {
        console.error('Failed to hydrate session:', error);
      }
    },

    activateLicense: async (key) => {
      try {
        const response = await api.licenses.activate(key);
        if (response && (response.id || response.is_active)) {
          await get().checkLicenseStatus();
          return true;
        }
        return false;
      } catch (error) {
        console.error('Activation error:', error);
        return false;
      }
    },

    checkLicenseStatus: async () => {
      try {
        const statusResponse = await api.licenses.getStatus();
        const licenseKey = statusResponse.license_id || '';
        const status = statusResponse.status || 'expired';
        const trialDays = typeof statusResponse.days_remaining === 'number'
          ? statusResponse.days_remaining
          : 14;

        localStorage.setItem('aztea_license_key', licenseKey);
        localStorage.setItem('aztea_license_status', status);
        localStorage.setItem('aztea_trial_days', trialDays.toString());

        set({
          licenseKey,
          licenseStatus: status as any,
          trialDaysLeft: trialDays,
        });
      } catch (error) {
        console.error('Failed to retrieve license status:', error);
      }
    },
  };
});

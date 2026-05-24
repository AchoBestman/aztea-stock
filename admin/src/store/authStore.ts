import { create } from "zustand";
import {
  api,
  clearSession,
  getStoredToken,
  getStoredUser,
  persistSession,
  type Tenant,
  type UserProfile,
} from "../lib/api";

interface AuthState {
  user: UserProfile | null;
  systemTenant: Tenant | null;
  isAuthenticated: boolean;
  pending2faEmail: string | null;
  login: (email: string, password: string) => Promise<"ok" | "2fa" | "fail">;
  verifyOtp: (code: string) => Promise<boolean>;
  logout: () => void;
  ensureSystemAccess: () => Promise<boolean>;
}

function readInitialAuth(): Pick<AuthState, "user" | "isAuthenticated"> {
  const token = getStoredToken();
  const user = getStoredUser();
  if (token && user) return { user, isAuthenticated: true };
  return { user: null, isAuthenticated: false };
}

export const useAuthStore = create<AuthState>((set, get) => {
  if (typeof window !== "undefined") {
    window.addEventListener("admin-auth-logout", () => {
      set({ user: null, isAuthenticated: false, systemTenant: null, pending2faEmail: null });
    });
  }

  return {
    ...readInitialAuth(),
    systemTenant: null,
    pending2faEmail: null,

    login: async (email, password) => {
      const res = await api.auth.login(email, password);
      if (res.requires_two_factor) {
        set({ pending2faEmail: email });
        return "2fa";
      }
      if (!res.access_token || !res.user) return "fail";
      persistSession(res);
      set({ user: res.user, isAuthenticated: true, pending2faEmail: null });
      const ok = await get().ensureSystemAccess();
      if (!ok) {
        get().logout();
        throw new Error(
          "Accès refusé : ce compte n'appartient pas au tenant système AzteaStock."
        );
      }
      return "ok";
    },

    verifyOtp: async (code) => {
      const email = get().pending2faEmail;
      if (!email) return false;
      const res = await api.auth.verifyOtp(email, code);
      if (!res.access_token || !res.user) return false;
      persistSession(res);
      set({ user: res.user, isAuthenticated: true, pending2faEmail: null });
      const ok = await get().ensureSystemAccess();
      if (!ok) {
        get().logout();
        throw new Error(
          "Accès refusé : ce compte n'appartient pas au tenant système AzteaStock."
        );
      }
      return true;
    },

    ensureSystemAccess: async () => {
      try {
        const tenant = await api.tenant.me();
        if (!tenant.is_system) {
          set({ systemTenant: null });
          return false;
        }
        set({ systemTenant: tenant });
        return true;
      } catch {
        set({ systemTenant: null });
        return false;
      }
    },

    logout: () => {
      clearSession();
      set({
        user: null,
        isAuthenticated: false,
        systemTenant: null,
        pending2faEmail: null,
      });
    },
  };
});

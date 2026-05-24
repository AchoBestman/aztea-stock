import { HashRouter, Navigate, Route, Routes } from "react-router-dom";
import { Toaster } from "react-hot-toast";
import Layout from "./components/Layout";
import Login from "./pages/Login";
import Dashboard from "./pages/Dashboard";
import Tenants from "./pages/Tenants";
import TenantDetail from "./pages/TenantDetail";
import Subscriptions from "./pages/Subscriptions";
import Licenses from "./pages/Licenses";
import SyncLogs from "./pages/SyncLogs";
import Settings from "./pages/Settings";

/**
 * HashRouter: works in static hosting and future Tauri WebView (file://).
 * No SSR — entire app is client-side for easy Tauri wrap later.
 */
export default function App() {
  return (
    <>
      <Toaster
        position="top-right"
        toastOptions={{
          style: {
            background: "var(--card)",
            color: "var(--foreground)",
            border: "1px solid var(--border)",
            borderRadius: "0.75rem",
            fontSize: "0.875rem",
            fontWeight: 600,
          },
        }}
      />
      <HashRouter>
        <Routes>
          <Route path="/login" element={<Login />} />
          <Route path="/" element={<Layout />}>
            <Route index element={<Dashboard />} />
            <Route path="tenants" element={<Tenants />} />
            <Route path="tenants/:id" element={<TenantDetail />} />
            <Route path="subscriptions" element={<Subscriptions />} />
            <Route path="licenses" element={<Licenses />} />
            <Route path="sync-logs" element={<SyncLogs />} />
            <Route path="settings" element={<Settings />} />
          </Route>
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </HashRouter>
    </>
  );
}

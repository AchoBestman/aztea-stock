import { HashRouter as Router, Routes, Route } from 'react-router-dom';
import { Toaster } from 'react-hot-toast';
import Layout from './components/Layout';
import RequirePermission from './components/RequirePermission';
import Dashboard from './pages/Dashboard';
import POS from './pages/POS';
import Stock from './pages/Stock';
import Products from './pages/Products';
import Categories from './pages/Categories';
import Reports from './pages/Reports';
import Settings from './pages/Settings';
import Login from './pages/Login';
import ForgotPassword from './pages/ForgotPassword';
import ResetPassword from './pages/ResetPassword';
import Users from './pages/Users';
import Roles from './pages/Roles';
import Sync from './pages/Sync';
import SalesHistory from './pages/SalesHistory';

function App() {
  return (
    <>
      <Toaster 
        position="top-right" 
        toastOptions={{
          style: {
            background: 'var(--color-card)',
            color: 'var(--color-foreground)',
            border: '1px solid var(--color-border)',
            borderRadius: '1rem',
            fontSize: '0.875rem',
            fontWeight: '600',
            boxShadow: '0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)',
          },
        }}
      />
      <Router>
        <Routes>
        {/* Public Login Route */}
        <Route path="/login" element={<Login />} />
        <Route path="/forgot-password" element={<ForgotPassword />} />
        <Route path="/reset-password" element={<ResetPassword />} />

        {/* Authenticated Dashboard / Layout Wrapper */}
        <Route path="/" element={<Layout />}>
          <Route index element={<RequirePermission path="/"><Dashboard /></RequirePermission>} />
          <Route path="pos" element={<RequirePermission path="/pos"><POS /></RequirePermission>} />
          <Route path="stock" element={<RequirePermission path="/stock"><Stock /></RequirePermission>} />
          <Route path="products" element={<RequirePermission path="/products"><Products /></RequirePermission>} />
          <Route path="categories" element={<RequirePermission path="/categories"><Categories /></RequirePermission>} />
          <Route path="reports" element={<RequirePermission path="/reports"><Reports /></RequirePermission>} />
          <Route path="settings" element={<RequirePermission path="/settings"><Settings /></RequirePermission>} />
          <Route path="users" element={<RequirePermission path="/users"><Users /></RequirePermission>} />
          <Route path="roles" element={<RequirePermission path="/roles"><Roles /></RequirePermission>} />
          <Route path="sync" element={<RequirePermission path="/sync"><Sync /></RequirePermission>} />
          <Route path="sales-history" element={<RequirePermission path="/sales-history"><SalesHistory /></RequirePermission>} />
        </Route>
      </Routes>
      </Router>
    </>
  );
}

export default App;

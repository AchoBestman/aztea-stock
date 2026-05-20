import { HashRouter as Router, Routes, Route } from 'react-router-dom';
import { Toaster } from 'react-hot-toast';
import Layout from './components/Layout';
import Dashboard from './pages/Dashboard';
import POS from './pages/POS';
import Stock from './pages/Stock';
import Products from './pages/Products';
import Categories from './pages/Categories';
import Reports from './pages/Reports';
import Settings from './pages/Settings';
import Login from './pages/Login';
import Users from './pages/Users';
import Roles from './pages/Roles';
import Sync from './pages/Sync';

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

        {/* Authenticated Dashboard / Layout Wrapper */}
        <Route path="/" element={<Layout />}>
          <Route index element={<Dashboard />} />
          <Route path="pos" element={<POS />} />
          <Route path="stock" element={<Stock />} />
          <Route path="products" element={<Products />} />
          <Route path="categories" element={<Categories />} />
          <Route path="reports" element={<Reports />} />
          <Route path="settings" element={<Settings />} />
          <Route path="users" element={<Users />} />
          <Route path="roles" element={<Roles />} />
          <Route path="sync" element={<Sync />} />
        </Route>
      </Routes>
      </Router>
    </>
  );
}

export default App;

import React, { useState, useEffect } from 'react';
import { Plus, Search, Edit2, Trash2, User } from 'lucide-react';
import { api, AdminUser, Role } from '../services/api';
import { toast } from 'react-hot-toast';
import { ConfirmModal } from '../components/ConfirmModal';

export default function Users() {
  const [users, setUsers] = useState<AdminUser[]>([]);
  const [roles, setRoles] = useState<Role[]>([]);
  const [loading, setLoading] = useState(true);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');

  // Form states
  const [newName, setNewName] = useState('');
  const [newEmail, setNewEmail] = useState('');
  const [newRoleId, setNewRoleId] = useState('');
  const [newUserTenantId, setNewUserTenantId] = useState('');

  // Delete modal
  const [deleteModalOpen, setDeleteModalOpen] = useState(false);
  const [userToDelete, setUserToDelete] = useState<AdminUser | null>(null);

  const [tenants, setTenants] = useState<any[]>([]);

  const fetchData = async () => {
    setLoading(true);
    try {
      const [usersRes, rolesRes] = await Promise.all([
        api.admin.users.list(),
        api.admin.roles.list()
      ]);
      setUsers(usersRes);
      setRoles(rolesRes);
      if (rolesRes.length > 0) {
        setNewRoleId(rolesRes[0].id);
      }
    } catch (error) {
      console.error("Failed to fetch users or roles", error);
    } finally {
      setLoading(false);
    }
  };

  const fetchTenantsIfSystem = async () => {
    try {
      const res = await api.admin.tenants.list();
      setTenants(res);
    } catch (error) {
      // Ignored: If it fails, it means the user is not a system user or doesn't have cross-tenant permissions
    }
  };

  useEffect(() => {
    fetchData();
    fetchTenantsIfSystem();
  }, []);

  const handleCreateUser = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await api.admin.users.create({
        name: newName,
        email: newEmail,
        role_id: newRoleId,
        tenant_id: newUserTenantId || undefined
      });
      setIsModalOpen(false);
      setNewName('');
      setNewEmail('');
      setNewUserTenantId('');
      fetchData();
      toast.success("Utilisateur créé avec succès.");
    } catch (error: any) {
      console.error("Error creating user", error);
      toast.error(error.message || "Erreur lors de la création.");
    }
  };

  const confirmDeleteUser = (u: AdminUser) => {
    setUserToDelete(u);
    setDeleteModalOpen(true);
  };

  const handleDeleteUser = async () => {
    if (!userToDelete) return;
    try {
      await api.admin.users.delete(userToDelete.id);
      setDeleteModalOpen(false);
      setUserToDelete(null);
      fetchData();
      toast.success("Utilisateur supprimé définitivement.");
    } catch (error: any) {
      console.error("Error deleting user", error);
      toast.error(error.message || "Impossible de supprimer l'utilisateur.");
    }
  };

  const filteredUsers = users.filter(u => 
    u.name.toLowerCase().includes(searchQuery.toLowerCase()) || 
    u.email.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="space-y-6 animate-slide-up select-none p-8 max-w-6xl mx-auto">
      
      {/* Top Header & Search */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <input
            type="text"
            placeholder="Rechercher par nom ou email..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-11 pr-4 py-2.5 bg-card border border-border rounded-xl focus:outline-none focus:ring-1 focus:ring-primary text-xs font-semibold text-foreground"
          />
        </div>

        <button
          onClick={() => setIsModalOpen(true)}
          className="flex items-center gap-1.5 px-4 py-2.5 rounded-xl font-bold text-xs shadow-md transition-all bg-primary text-primary-foreground hover:bg-opacity-95 cursor-pointer"
        >
          <Plus className="w-4 h-4" />
          <span>Nouvel Utilisateur</span>
        </button>
      </div>

      <div className="bg-card border border-border rounded-2xl shadow-sm overflow-hidden flex flex-col">
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs border-collapse">
            <thead>
              <tr className="border-b border-border text-muted-foreground font-bold bg-muted/10">
                <th className="py-3.5 px-6">Nom</th>
                <th className="py-3.5 px-4">Email</th>
                <th className="py-3.5 px-4">Rôles</th>
                <th className="py-3.5 px-6">Statut</th>
                <th className="py-3.5 px-6 text-right">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-border">
              {loading ? (
                <tr>
                  <td colSpan={4} className="py-8 text-center text-muted-foreground font-semibold">Chargement...</td>
                </tr>
              ) : filteredUsers.length === 0 ? (
                <tr>
                  <td colSpan={4} className="py-8 text-center text-muted-foreground font-semibold">Aucun utilisateur trouvé.</td>
                </tr>
              ) : (
                filteredUsers.map(user => (
                  <tr key={user.id} className="border-b border-border/50 hover:bg-accent/20 transition-colors font-medium">
                    <td className="py-4 px-6 font-bold text-foreground flex items-center gap-2">
                      <User className="w-4 h-4 text-primary shrink-0" />
                      <span>{user.name}</span>
                    </td>
                    <td className="py-4 px-4 text-muted-foreground font-semibold">{user.email}</td>
                    <td className="py-4 px-4 text-sm">
                      <div className="flex flex-wrap gap-1.5">
                        {user.roles?.map(role => (
                          <span key={role} className="bg-primary/10 text-primary px-2.5 py-1 rounded-md text-[10px] uppercase font-extrabold tracking-wider">
                            {role}
                          </span>
                        ))}
                      </div>
                    </td>
                    <td className="py-4 px-6">
                      {user.is_active ? (
                        <span className="inline-flex items-center px-2 py-0.5 rounded text-[10px] font-extrabold uppercase tracking-wider bg-emerald-100 text-emerald-800 dark:bg-emerald-900/30 dark:text-emerald-400">
                          Actif
                        </span>
                      ) : (
                        <span className="inline-flex items-center px-2 py-0.5 rounded text-[10px] font-extrabold uppercase tracking-wider bg-rose-100 text-rose-800 dark:bg-rose-900/30 dark:text-rose-400">
                          Inactif
                        </span>
                      )}
                    </td>
                    <td className="py-4 px-6 text-right">
                      <button
                        onClick={() => confirmDeleteUser(user)}
                        className="p-1.5 text-rose-500 hover:bg-rose-500/10 rounded-lg transition-colors cursor-pointer"
                        title="Supprimer l'utilisateur"
                      >
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Creation Modal */}
      {isModalOpen && (
        <div className="fixed inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-scale-in">
          <div className="bg-card border border-border w-full max-w-md rounded-3xl shadow-2xl p-6 relative">
            <h3 className="font-extrabold text-base text-foreground mb-4">Créer un Nouvel Utilisateur</h3>
            
            <form onSubmit={handleCreateUser} className="space-y-4">
              <div>
                <label className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Nom complet *</label>
                <input
                  type="text"
                  required
                  placeholder="Jean Dupont"
                  value={newName}
                  onChange={(e) => setNewName(e.target.value)}
                  className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-semibold focus:outline-none focus:ring-1 focus:ring-primary text-foreground"
                />
              </div>

              <div>
                <label className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Adresse email *</label>
                <input
                  type="email"
                  required
                  placeholder="jean@entreprise.com"
                  value={newEmail}
                  onChange={(e) => setNewEmail(e.target.value)}
                  className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-semibold focus:outline-none focus:ring-1 focus:ring-primary text-foreground"
                />
              </div>

              <div>
                <label className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Rôle *</label>
                <select
                  value={newRoleId}
                  onChange={(e) => setNewRoleId(e.target.value)}
                  className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-semibold focus:outline-none focus:ring-1 focus:ring-primary text-foreground"
                >
                  {roles.map(role => (
                    <option key={role.id} value={role.id}>{role.name}</option>
                  ))}
                </select>
              </div>

              {tenants.length > 0 && (
                <div>
                  <label className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Assigner à une Entreprise (Optionnel)</label>
                  <select
                    value={newUserTenantId}
                    onChange={(e) => setNewUserTenantId(e.target.value)}
                    className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-semibold focus:outline-none focus:ring-1 focus:ring-primary text-foreground"
                  >
                    <option value="">-- Mon Entreprise (Système) --</option>
                    {tenants.map(t => (
                      <option key={t.id} value={t.id}>{t.name}</option>
                    ))}
                  </select>
                </div>
              )}

              {/* Form buttons */}
              <div className="flex gap-3 pt-2">
                <button
                  type="button"
                  onClick={() => setIsModalOpen(false)}
                  className="flex-1 py-2.5 rounded-xl border border-border bg-card hover:bg-accent text-foreground text-xs font-bold cursor-pointer"
                >
                  Annuler
                </button>
                <button
                  type="submit"
                  className="flex-1 py-2.5 rounded-xl bg-primary text-primary-foreground text-xs font-bold shadow-sm hover:bg-opacity-95 cursor-pointer"
                >
                  Créer
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Delete Confirmation Modal */}
      <ConfirmModal
        isOpen={deleteModalOpen}
        title="Supprimer cet utilisateur ?"
        message={`Êtes-vous sûr de vouloir supprimer l'utilisateur "${userToDelete?.name}" ? Cette action est irréversible.`}
        confirmText="Oui, Supprimer"
        cancelText="Annuler"
        onConfirm={handleDeleteUser}
        onCancel={() => {
          setDeleteModalOpen(false);
          setUserToDelete(null);
        }}
      />
    </div>
  );
}

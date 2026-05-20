import React, { useState, useEffect } from 'react';
import { Plus, Shield, Check, Trash2, Edit2 } from 'lucide-react';
import { api, Role, GroupedPermission, Permission } from '../services/api';
import { toast } from 'react-hot-toast';
import { ConfirmModal } from '../components/ConfirmModal';

export default function Roles() {
  const [roles, setRoles] = useState<Role[]>([]);
  const [loading, setLoading] = useState(true);
  
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [newRoleName, setNewRoleName] = useState('');
  const [newRoleDesc, setNewRoleDesc] = useState('');
  const [newRoleTenantId, setNewRoleTenantId] = useState('');
  
  const [tenants, setTenants] = useState<any[]>([]);

  const [isPermModalOpen, setIsPermModalOpen] = useState(false);
  const [selectedRole, setSelectedRole] = useState<Role | null>(null);
  const [groupedPermissions, setGroupedPermissions] = useState<GroupedPermission[]>([]);
  const [rolePermissions, setRolePermissions] = useState<Set<string>>(new Set());

  // Delete modal state
  const [deleteModalOpen, setDeleteModalOpen] = useState(false);
  const [roleToDelete, setRoleToDelete] = useState<Role | null>(null);

  const fetchRoles = async () => {
    setLoading(true);
    try {
      const res = await api.admin.roles.list();
      setRoles(res);
    } catch (error) {
      console.error("Failed to fetch roles", error);
    } finally {
      setLoading(false);
    }
  };

  const fetchAllPermissions = async () => {
    try {
      const res = await api.admin.permissions.listGrouped();
      setGroupedPermissions(res);
    } catch (error) {
      console.error("Failed to fetch permissions", error);
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
    fetchRoles();
    fetchAllPermissions();
    fetchTenantsIfSystem();
  }, []);

  const handleCreateRole = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await api.admin.roles.create({
        name: newRoleName,
        description: newRoleDesc,
        tenant_id: newRoleTenantId || undefined
      });
      setIsCreateModalOpen(false);
      setNewRoleName('');
      setNewRoleDesc('');
      setNewRoleTenantId('');
      setNewRoleTenantId('');
      fetchRoles();
      toast.success("Le rôle a été créé avec succès.");
    } catch (error: any) {
      console.error("Error creating role", error);
      toast.error(error.message || "Impossible de créer le rôle.");
    }
  };

  const confirmDeleteRole = (role: Role) => {
    setRoleToDelete(role);
    setDeleteModalOpen(true);
  };

  const handleDeleteRole = async () => {
    if (!roleToDelete) return;
    try {
      await api.admin.roles.delete(roleToDelete.id);
      setDeleteModalOpen(false);
      setRoleToDelete(null);
      fetchRoles();
      toast.success("Le rôle a été supprimé définitivement.");
    } catch (error: any) {
      console.error("Error deleting role", error);
      toast.error(error.message || "Erreur lors de la suppression du rôle.");
    }
  };

  const openPermissionsModal = async (role: Role) => {
    setSelectedRole(role);
    setIsPermModalOpen(true);
    try {
      const perms = await api.admin.roles.listPermissions(role.id);
      setRolePermissions(new Set(perms.map(p => p.id)));
    } catch (error) {
      console.error("Failed to fetch role permissions", error);
    }
  };

  const togglePermission = (permId: string) => {
    setRolePermissions(prev => {
      const next = new Set(prev);
      if (next.has(permId)) {
        next.delete(permId);
      } else {
        next.add(permId);
      }
      return next;
    });
  };

  const handleSavePermissions = async () => {
    if (!selectedRole) return;
    try {
      await api.admin.roles.assignPermissions(selectedRole.id, Array.from(rolePermissions));
      setIsPermModalOpen(false);
      toast.success("Permissions sauvegardées.");
    } catch (error: any) {
      console.error("Error saving permissions", error);
      toast.error(error.message || "Erreur lors de la sauvegarde.");
    }
  };

  return (
    <div className="space-y-6 animate-slide-up select-none p-8 max-w-6xl mx-auto">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold text-foreground">Rôles & Permissions</h1>
          <p className="text-muted-foreground mt-1">Gérez les rôles et attribuez des permissions spécifiques.</p>
        </div>
        <button 
          onClick={() => setIsCreateModalOpen(true)}
          className="flex items-center gap-1.5 px-4 py-2.5 rounded-xl font-bold text-xs shadow-md transition-all bg-primary text-primary-foreground hover:bg-opacity-95 cursor-pointer"
        >
          <Plus className="w-4 h-4" />
          <span>Nouveau Rôle</span>
        </button>
      </div>

      <div className="bg-card border border-border rounded-2xl shadow-sm overflow-hidden flex flex-col">
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs border-collapse">
            <thead>
              <tr className="border-b border-border text-muted-foreground font-bold bg-muted/10">
                <th className="py-3.5 px-6">Nom du Rôle</th>
                <th className="py-3.5 px-4">Description</th>
                <th className="py-3.5 px-6 text-right">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-border">
              {loading ? (
                <tr>
                  <td colSpan={3} className="py-8 text-center text-muted-foreground font-semibold">Chargement...</td>
                </tr>
              ) : roles.length === 0 ? (
                <tr>
                  <td colSpan={3} className="py-8 text-center text-muted-foreground font-semibold">Aucun rôle trouvé.</td>
                </tr>
              ) : (
                roles.map(role => (
                  <tr key={role.id} className="border-b border-border/50 hover:bg-accent/20 transition-colors font-medium">
                    <td className="py-4 px-6 font-bold text-foreground flex items-center gap-2">
                      <Shield className="w-4 h-4 text-primary shrink-0" />
                      <span>{role.name}</span>
                      {role.is_system && (
                        <span className="ml-2 bg-muted text-muted-foreground px-2 py-0.5 rounded text-[10px] uppercase font-bold tracking-wider">
                          Système
                        </span>
                      )}
                    </td>
                    <td className="py-4 px-4 text-muted-foreground font-semibold">{role.description || '-'}</td>
                    <td className="py-4 px-6 text-right">
                      <div className="flex items-center justify-end gap-3">
                        <button
                          onClick={() => openPermissionsModal(role)}
                          className="text-xs font-bold text-primary hover:underline cursor-pointer"
                        >
                          Permissions
                        </button>
                        {role.name !== 'Super Admin' && (
                          <button
                            onClick={() => confirmDeleteRole(role)}
                            className="p-1.5 text-rose-500 hover:bg-rose-500/10 rounded-lg transition-colors cursor-pointer"
                            title="Supprimer le rôle"
                          >
                            <Trash2 className="w-4 h-4" />
                          </button>
                        )}
                      </div>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Create Role Modal */}
      {isCreateModalOpen && (
        <div className="fixed inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-scale-in">
          <div className="bg-card border border-border w-full max-w-md rounded-3xl shadow-2xl p-6 relative">
            <h3 className="font-extrabold text-base text-foreground mb-4">Créer un Nouveau Rôle</h3>
            
            <form onSubmit={handleCreateRole} className="space-y-4">
              <div>
                <label className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Nom du rôle *</label>
                <input
                  type="text"
                  required
                  placeholder="ex. Manager, Superviseur..."
                  value={newRoleName}
                  onChange={(e) => setNewRoleName(e.target.value)}
                  className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-semibold focus:outline-none focus:ring-1 focus:ring-primary text-foreground"
                />
              </div>

              <div>
                <label className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Description (optionnelle)</label>
                <textarea
                  value={newRoleDesc}
                  onChange={(e) => setNewRoleDesc(e.target.value)}
                  className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-semibold focus:outline-none focus:ring-1 focus:ring-primary text-foreground"
                  rows={3}
                />
              </div>

              {tenants.length > 0 && (
                <div>
                  <label className="text-[10px] font-extrabold text-muted-foreground uppercase block mb-1">Assigner à une Entreprise (Optionnel)</label>
                  <select
                    value={newRoleTenantId}
                    onChange={(e) => setNewRoleTenantId(e.target.value)}
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
                  onClick={() => setIsCreateModalOpen(false)}
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

      {/* Manage Permissions Modal */}
      {isPermModalOpen && selectedRole && (
        <div className="fixed inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-scale-in">
          <div className="bg-card border border-border w-full max-w-3xl max-h-[85vh] rounded-3xl shadow-2xl flex flex-col overflow-hidden relative">
            <div className="p-6 border-b border-border flex items-center justify-between">
              <h3 className="font-extrabold text-base text-foreground">
                Permissions : <span className="text-primary">{selectedRole.name}</span>
              </h3>
            </div>
            
            <div className="flex-1 overflow-y-auto p-6 space-y-8">
              {groupedPermissions.map(group => (
                <div key={group.group}>
                  <h3 className="text-sm font-bold text-muted-foreground uppercase tracking-wider mb-3 border-b border-border pb-2">
                    {group.group}
                  </h3>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                    {group.permissions.map(perm => {
                      const isSelected = rolePermissions.has(perm.id);
                      return (
                        <div 
                          key={perm.id} 
                          onClick={() => togglePermission(perm.id)}
                          className={`flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition-colors ${
                            isSelected ? 'bg-primary/5 border-primary shadow-sm' : 'bg-background border-border hover:bg-muted/50'
                          }`}
                        >
                          <div className={`mt-0.5 flex-shrink-0 w-5 h-5 rounded border flex items-center justify-center transition-colors ${
                            isSelected ? 'bg-primary border-primary text-primary-foreground' : 'border-input bg-background'
                          }`}>
                            {isSelected && <Check className="w-3.5 h-3.5" />}
                          </div>
                          <div>
                            <p className={`text-sm font-medium ${isSelected ? 'text-foreground' : 'text-foreground'}`}>
                              {perm.name}
                            </p>
                            {perm.description && (
                              <p className="text-xs text-muted-foreground mt-0.5 leading-snug">
                                {perm.description}
                              </p>
                            )}
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>
              ))}
            </div>

            <div className="p-6 border-t border-border flex items-center justify-end gap-3 bg-muted/10">
              <button
                onClick={() => setIsPermModalOpen(false)}
                className="px-6 py-2.5 rounded-xl border border-border bg-card hover:bg-accent text-foreground text-xs font-bold cursor-pointer transition-colors"
              >
                Annuler
              </button>
              <button
                onClick={handleSavePermissions}
                className="px-6 py-2.5 bg-primary text-primary-foreground rounded-xl text-xs font-bold shadow-sm hover:bg-opacity-95 cursor-pointer transition-opacity"
              >
                Sauvegarder
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Delete Confirmation Modal */}
      <ConfirmModal
        isOpen={deleteModalOpen}
        title="Supprimer ce rôle ?"
        message={`Êtes-vous sûr de vouloir supprimer le rôle "${roleToDelete?.name}" ? Cette action est irréversible et supprimera toutes les permissions associées.`}
        confirmText="Oui, Supprimer"
        cancelText="Annuler"
        onConfirm={handleDeleteRole}
        onCancel={() => {
          setDeleteModalOpen(false);
          setRoleToDelete(null);
        }}
      />
    </div>
  );
}

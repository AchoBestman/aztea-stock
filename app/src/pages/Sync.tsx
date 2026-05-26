import { useState, useEffect } from 'react';
import { RefreshCw, CheckCircle2, AlertCircle, Clock, Upload, Plus, Edit2, Trash2 } from 'lucide-react';
import { useSyncStore } from '../store/syncStore';
import { usePermissions } from '../hooks/usePermissions';
import { api, SyncLog } from '../services/api';

export default function Sync() {
  const { has } = usePermissions();
  const canManageSync = has('can_manage_sync_log');
  const { isOnline, isSyncing, pendingActions, sync, lastSyncAt } = useSyncStore();
  const pendingCount = pendingActions.length;
  const [logs, setLogs] = useState<SyncLog[]>([]);
  const [loadingLogs, setLoadingLogs] = useState(false);
  
  // Date filters
  const [startDate, setStartDate] = useState('');
  const [endDate, setEndDate] = useState('');

  const fetchLogs = async () => {
    setLoadingLogs(true);
    try {
      // Pass start/end date if available
      const response = await api.sync.logs(startDate || undefined, endDate || undefined, 1, 50);
      setLogs(response.data);
    } catch (error) {
      console.error("Failed to fetch sync logs:", error);
    } finally {
      setLoadingLogs(false);
    }
  };

  useEffect(() => {
    fetchLogs();
  }, [startDate, endDate]);

  const handleSync = async () => {
    await sync();
    // Refresh history after sync
    fetchLogs();
  };

  return (
    <div className="space-y-6 animate-slide-up select-none p-8 max-w-6xl mx-auto">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold text-foreground">Synchronisation</h1>
          <p className="text-muted-foreground mt-1">Gérez la synchronisation des données locales vers le cloud.</p>
        </div>
      </div>

      {/* Status Card */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="bg-card border border-border rounded-xl p-6 shadow-sm flex flex-col justify-between">
          <div>
            <h3 className="text-sm font-semibold text-muted-foreground">État de la connexion</h3>
            <div className="flex items-center gap-2 mt-2">
              <div className="relative flex h-3 w-3">
                <span className={`animate-ping absolute inline-flex h-full w-full rounded-full opacity-75 ${isOnline ? "bg-emerald-400" : "bg-rose-400"}`}></span>
                <span className={`relative inline-flex rounded-full h-3 w-3 ${isOnline ? "bg-emerald-500" : "bg-rose-500"}`}></span>
              </div>
              <span className="text-lg font-bold">{isOnline ? 'En ligne' : 'Hors ligne'}</span>
            </div>
          </div>
        </div>

        <div className="bg-card border border-border rounded-xl p-6 shadow-sm flex flex-col justify-between">
          <div>
            <h3 className="text-sm font-semibold text-muted-foreground">Éléments en attente</h3>
            <div className="flex items-center gap-2 mt-2">
              <span className="text-3xl font-bold text-foreground">{pendingCount}</span>
              <span className="text-sm text-muted-foreground">actions locales non synchronisées</span>
            </div>
          </div>
        </div>

        <div className="bg-card border border-border rounded-xl p-6 shadow-sm flex flex-col justify-between items-start">
          <div className="w-full">
            <h3 className="text-sm font-semibold text-muted-foreground mb-4">Action</h3>
            <button
              onClick={handleSync}
              disabled={!canManageSync || isSyncing || pendingCount === 0 || !isOnline}
              className="w-full flex items-center justify-center gap-2 py-3 px-4 bg-primary text-primary-foreground rounded-lg font-semibold shadow hover:opacity-90 transition-opacity disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <RefreshCw className={`w-5 h-5 ${isSyncing ? 'animate-spin' : ''}`} />
              {isSyncing ? 'Synchronisation...' : 'Lancer la synchronisation'}
            </button>
            {!canManageSync && (
              <p className="text-xs text-amber-600 dark:text-amber-400 mt-3 text-center font-medium">
                Permission « can_manage_sync_log » requise pour synchroniser.
              </p>
            )}
            {lastSyncAt && (
              <p className="text-xs text-muted-foreground mt-3 text-center">
                Dernière synchro : {lastSyncAt.toLocaleString()}
              </p>
            )}
          </div>
        </div>
      </div>

      {/* Pending Actions Section */}
      <div className="bg-card border border-border rounded-2xl shadow-sm overflow-hidden mt-8">
        <div className="p-6 border-b border-border flex items-center gap-3 bg-amber-500/5">
          <Upload className="w-5 h-5 text-amber-500" />
          <h2 className="text-lg font-bold text-foreground">Actions en attente ({pendingCount})</h2>
        </div>

        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs border-collapse">
            <thead>
              <tr className="border-b border-border text-muted-foreground font-bold bg-muted/10">
                <th className="py-3.5 px-6">Date & Heure</th>
                <th className="py-3.5 px-4">Type</th>
                <th className="py-3.5 px-4">Action</th>
                <th className="py-3.5 px-6">Description</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-border">
              {pendingActions.length === 0 ? (
                <tr>
                  <td colSpan={4} className="py-8 text-center text-muted-foreground font-semibold">Toutes les actions sont synchronisées.</td>
                </tr>
              ) : (
                pendingActions.map((action) => (
                  <tr key={action.id} className="border-b border-border/50 hover:bg-accent/20 transition-colors font-medium">
                    <td className="py-4 px-6 font-bold text-foreground">
                      {action.timestamp.toLocaleString()}
                    </td>
                    <td className="py-4 px-4 text-muted-foreground">
                      {action.entity_type}
                    </td>
                    <td className="py-4 px-4">
                      {action.action_type === 'CREATE' && (
                        <span className="inline-flex items-center gap-1 px-2.5 py-1 rounded bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 text-[10px] uppercase font-extrabold tracking-wider">
                          <Plus className="w-3 h-3" /> Création
                        </span>
                      )}
                      {action.action_type === 'UPDATE' && (
                        <span className="inline-flex items-center gap-1 px-2.5 py-1 rounded bg-blue-500/10 text-blue-600 dark:text-blue-400 text-[10px] uppercase font-extrabold tracking-wider">
                          <Edit2 className="w-3 h-3" /> Modif
                        </span>
                      )}
                      {action.action_type === 'DELETE' && (
                        <span className="inline-flex items-center gap-1 px-2.5 py-1 rounded bg-rose-500/10 text-rose-600 dark:text-rose-400 text-[10px] uppercase font-extrabold tracking-wider">
                          <Trash2 className="w-3 h-3" /> Suppr
                        </span>
                      )}
                    </td>
                    <td className="py-4 px-6 text-foreground font-semibold">
                      {action.description}
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* History Section */}
      <div className="bg-card border border-border rounded-2xl shadow-sm overflow-hidden mt-8">
        <div className="p-6 border-b border-border flex flex-col md:flex-row md:items-center justify-between gap-4">
          <h2 className="text-lg font-bold text-foreground">Historique des synchronisations</h2>
          
          <div className="flex items-center gap-3 text-xs font-semibold">
            <div className="flex items-center gap-2">
              <label className="text-muted-foreground">Du:</label>
              <input 
                type="date" 
                value={startDate}
                onChange={(e) => setStartDate(e.target.value)}
                className="bg-card border border-border rounded-lg px-2.5 py-1.5 focus:outline-none focus:ring-1 focus:ring-primary text-foreground"
              />
            </div>
            <div className="flex items-center gap-2">
              <label className="text-muted-foreground">Au:</label>
              <input 
                type="date" 
                value={endDate}
                onChange={(e) => setEndDate(e.target.value)}
                className="bg-card border border-border rounded-lg px-2.5 py-1.5 focus:outline-none focus:ring-1 focus:ring-primary text-foreground"
              />
            </div>
          </div>
        </div>

        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs border-collapse">
            <thead>
              <tr className="border-b border-border text-muted-foreground font-bold bg-muted/10">
                <th className="py-3.5 px-6">Date & Heure</th>
                <th className="py-3.5 px-4">Appareil</th>
                <th className="py-3.5 px-4 text-center">Éléments Sync</th>
                <th className="py-3.5 px-6">Statut</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-border">
              {loadingLogs ? (
                <tr>
                  <td colSpan={4} className="py-8 text-center text-muted-foreground font-semibold">Chargement de l'historique...</td>
                </tr>
              ) : logs.length === 0 ? (
                <tr>
                  <td colSpan={4} className="py-8 text-center text-muted-foreground font-semibold">Aucun historique trouvé pour cette période.</td>
                </tr>
              ) : (
                logs.map((log) => (
                  <tr key={log.id} className="border-b border-border/50 hover:bg-accent/20 transition-colors font-medium">
                    <td className="py-4 px-6 font-bold text-foreground">
                      {new Date(log.created_at).toLocaleString()}
                    </td>
                    <td className="py-4 px-4 text-muted-foreground font-mono">
                      {log.device_id || 'Inconnu'}
                    </td>
                    <td className="py-4 px-4 text-center font-extrabold text-foreground">
                      {log.records_synced} {log.records_synced > 1 ? 'éléments' : 'élément'}
                    </td>
                    <td className="py-4 px-6">
                      {log.status === 'success' ? (
                        <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[10px] font-extrabold tracking-wider uppercase bg-emerald-500/10 text-emerald-600 dark:text-emerald-400">
                          <CheckCircle2 className="w-3.5 h-3.5" /> Succès
                        </span>
                      ) : log.status === 'failed' ? (
                        <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[10px] font-extrabold tracking-wider uppercase bg-rose-500/10 text-rose-600 dark:text-rose-400">
                          <AlertCircle className="w-3.5 h-3.5" /> Échec
                        </span>
                      ) : (
                        <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[10px] font-extrabold tracking-wider uppercase bg-amber-500/10 text-amber-600 dark:text-amber-400">
                          <Clock className="w-3.5 h-3.5" /> Partiel
                        </span>
                      )}
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}

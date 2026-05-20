import { create } from 'zustand';

export interface PendingAction {
  id: string;
  entity_type: string;
  action_type: 'CREATE' | 'UPDATE' | 'DELETE';
  description: string;
  timestamp: Date;
}

interface SyncState {
  isOnline: boolean;
  lastSyncAt: Date | null;
  pendingActions: PendingAction[];
  isSyncing: boolean;
  setOnline: (online: boolean) => void;
  addPendingAction: (action: Omit<PendingAction, 'id' | 'timestamp'>) => void;
  sync: () => Promise<void>;
}

export const useSyncStore = create<SyncState>((set, get) => ({
  isOnline: true,
  lastSyncAt: new Date(Date.now() - 3600000), // 1 hour ago
  pendingActions: [
    { id: '1', entity_type: 'Product', action_type: 'CREATE', description: 'Nouveau produit: Paracétamol 500mg', timestamp: new Date(Date.now() - 300000) },
    { id: '2', entity_type: 'Category', action_type: 'UPDATE', description: 'Catégorie modifiée: Antidouleurs', timestamp: new Date(Date.now() - 150000) },
    { id: '3', entity_type: 'Sale', action_type: 'CREATE', description: 'Nouvelle vente #V-0042', timestamp: new Date(Date.now() - 60000) }
  ],
  isSyncing: false,

  setOnline: (online) => set({ isOnline: online }),
  
  addPendingAction: (action) => set((state) => ({ 
    pendingActions: [...state.pendingActions, {
      ...action,
      id: Math.random().toString(36).substring(7),
      timestamp: new Date()
    }] 
  })),

  sync: async () => {
    if (get().isSyncing) return;
    set({ isSyncing: true });
    
    // Simulate syncing delay
    await new Promise((resolve) => setTimeout(resolve, 2000));
    
    set({
      lastSyncAt: new Date(),
      pendingActions: [],
      isSyncing: false,
    });
  },
}));

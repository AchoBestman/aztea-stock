import { useState, useEffect } from 'react';
import { 
  Key, 
  Printer, 
  Barcode, 
  Database, 
  Save,
  AlertCircle,
  HelpCircle
} from 'lucide-react';
import { useAuthStore } from '../store/authStore';
import { api, getApiBaseUrl } from '../services/api';
import toast from 'react-hot-toast';

export default function Settings() {
  const { licenseKey, licenseStatus, activateLicense } = useAuthStore();
  
  const [newKey, setNewKey] = useState('');
  const [isActivating, setIsActivating] = useState(false);
  const [tenant, setTenant] = useState<any>(null);
  const [licenseDetails, setLicenseDetails] = useState<any>(null);

  // Hardware lists & selections
  const [printers, setPrinters] = useState<Array<{ name: string; connected: boolean; is_default: boolean }>>([]);
  const [scanners, setScanners] = useState<Array<{ name: string; connected: boolean; is_default: boolean }>>([]);
  
  const [selectedPrinter, setSelectedPrinter] = useState(() => 
    localStorage.getItem('aztea_default_printer') || ''
  );
  const [selectedScanner, setSelectedScanner] = useState(() => 
    localStorage.getItem('aztea_default_scanner') || ''
  );

  // Settings states
  const [printerWidth, setPrinterWidth] = useState(() => localStorage.getItem('aztea_printer_width') || '80');
  const [apiUrl, setApiUrl] = useState(() => getApiBaseUrl() || 'http://localhost:8000/api/v1');

  // Request url change simulation
  const [showRequestModal, setShowRequestModal] = useState(false);
  const [requestUrl, setRequestUrl] = useState('');

  const loadData = async () => {
    try {
      const [tRes, lRes] = await Promise.all([
        api.tenants.get(),
        api.licenses.getStatus()
      ]);
      setTenant(tRes);
      setLicenseDetails(lRes);
    } catch (e) {
      console.error("Failed to load settings data:", e);
    }
  };

  useEffect(() => {
    loadData();

    const detectDevices = async () => {
      try {
        const isTauri = typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__ !== undefined;
        
        if (isTauri) {
          const { invoke } = await import('@tauri-apps/api/core');
          const response = await invoke<any>('get_hardware_devices');
          
          const detectedPrinters = response.printers || [];
          const detectedScanners = response.scanners || [];
          
          setPrinters(detectedPrinters);
          setScanners(detectedScanners);

          if (!localStorage.getItem('aztea_default_printer')) {
            const defaultP = detectedPrinters.find((p: any) => p.is_default) || detectedPrinters[0];
            if (defaultP) {
              setSelectedPrinter(defaultP.name);
            }
          }
          if (!localStorage.getItem('aztea_default_scanner')) {
            const defaultS = detectedScanners.find((s: any) => s.is_default) || detectedScanners[0];
            if (defaultS) {
              setSelectedScanner(defaultS.name);
            }
          }
        } else {
          // Fallback in web browser: query media inputs & default PDF printer option
          const devs = await navigator.mediaDevices.enumerateDevices();
          const videoDevices = devs.filter(d => d.kind === 'videoinput');
          
          const fallbackPrinters = [
            { name: 'Enregistrer au format PDF (Simulé)', connected: true, is_default: true }
          ];
          const fallbackScanners = [
            { name: 'USB HID Barcode Scanner (Simulé)', connected: true, is_default: true },
            ...videoDevices.map(d => ({ name: `Caméra : ${d.label || 'Webcam intégrée'}`, connected: true, is_default: false }))
          ];

          setPrinters(fallbackPrinters);
          setScanners(fallbackScanners);

          if (!localStorage.getItem('aztea_default_printer')) {
            setSelectedPrinter(fallbackPrinters[0].name);
          }
          if (!localStorage.getItem('aztea_default_scanner')) {
            setSelectedScanner(fallbackScanners[0].name);
          }
        }
      } catch (err) {
        console.error("Hardware detection failed:", err);
        const fallbackPrinters = [
          { name: 'Enregistrer au format PDF (Simulé)', connected: true, is_default: true }
        ];
        const fallbackScanners = [
          { name: 'USB HID Barcode Scanner (Simulé)', connected: true, is_default: true }
        ];
        setPrinters(fallbackPrinters);
        setScanners(fallbackScanners);
        if (!localStorage.getItem('aztea_default_printer')) {
          setSelectedPrinter(fallbackPrinters[0].name);
        }
        if (!localStorage.getItem('aztea_default_scanner')) {
          setSelectedScanner(fallbackScanners[0].name);
        }
      }
    };

    detectDevices();
  }, []);

  const handleLicenseSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newKey.trim()) return;

    setIsActivating(true);
    const success = await activateLicense(newKey);
    setIsActivating(false);
    
    if (success) {
      toast.success("Licence activée avec succès !");
      setNewKey('');
      loadData();
    } else {
      toast.error("Clé de licence invalide. Veuillez réessayer.");
    }
  };

  const handleSaveSettings = async () => {
    // 1. Verify if API base URL has changed
    const currentBaseUrl = getApiBaseUrl();
    if (apiUrl !== currentBaseUrl) {
      try {
        // Temp change to verify
        setApiUrl(apiUrl);
        localStorage.setItem('aztea_api_base_url', apiUrl);
        
        const testTenant = await api.tenants.get();
        if (testTenant && testTenant.is_system) {
          // Allowed: System Tenant can change endpoints
          toast.success("Adresse API vérifiée et enregistrée avec succès !");
        } else {
          // Revert changes
          localStorage.setItem('aztea_api_base_url', currentBaseUrl);
          setApiUrl(currentBaseUrl);
          
          // Show request modal
          setRequestUrl(apiUrl);
          setShowRequestModal(true);
          return;
        }
      } catch (err: any) {
        // Revert on error
        localStorage.setItem('aztea_api_base_url', currentBaseUrl);
        setApiUrl(currentBaseUrl);
        toast.error("Erreur de connexion : Impossible de valider l'adresse API sur ce serveur.");
        return;
      }
    }

    // Save hardware choices
    localStorage.setItem('aztea_default_printer', selectedPrinter);
    localStorage.setItem('aztea_default_scanner', selectedScanner);
    localStorage.setItem('aztea_printer_width', printerWidth);
    
    toast.success("Paramètres enregistrés localement avec succès !");
  };

  const submitUrlChangeRequest = () => {
    setShowRequestModal(false);
    toast.success(`Demande de modification de l'adresse API vers "${requestUrl}" soumise avec succès au système. En attente de validation.`);
  };

  return (
    <div className="w-full space-y-8 animate-slide-up select-none">
      
      {/* Page Header */}
      <div>
        <h1 className="text-2xl font-bold text-foreground">Configuration du Système</h1>
        <p className="text-xs text-muted-foreground font-semibold mt-0.5">Configurez vos périphériques et gérez votre licence.</p>
      </div>

      {/* License Panel */}
      <div className="bg-card border border-border rounded-3xl p-6 shadow-sm space-y-4">
        <div className="flex items-center gap-3">
          <Key className="w-5 h-5 text-primary dark:text-amber-400" />
          <h3 className="font-bold text-sm text-foreground">Gestion de la Licence</h3>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6 pt-2">
          {/* Current License Details */}
          <div className="space-y-3 p-4 rounded-2xl bg-accent/30 border border-border/50 text-xs font-semibold">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Clé d'Activation :</span>
              <span className="font-mono text-foreground font-extrabold">{licenseDetails?.license_key || licenseKey || 'Aucune'}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Statut de la Licence :</span>
              <span className={`uppercase font-bold ${licenseDetails?.is_valid ? 'text-emerald-500' : 'text-primary dark:text-amber-400'}`}>
                {licenseDetails?.status || licenseStatus}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Expire le :</span>
              <span className="text-foreground">
                {licenseDetails?.expires_at ? new Date(licenseDetails.expires_at).toLocaleDateString('fr-FR') : 'Jamais'}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Terminal enregistré :</span>
              <span className="text-foreground">{tenant?.name || 'Caisse Locale'}</span>
            </div>
          </div>

          {/* Activation Form */}
          <form onSubmit={handleLicenseSubmit} className="space-y-3">
            <label className="text-[10px] font-extrabold text-muted-foreground uppercase block">ACTIVER UNE CLÉ DE LICENCE</label>
            <div className="flex gap-2">
              <input
                type="text"
                placeholder="Entrez votre clé AZTEA-XXXX..."
                value={newKey}
                onChange={(e) => setNewKey(e.target.value)}
                className="flex-1 px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-semibold focus:outline-none focus:ring-1 focus:ring-primary text-foreground"
              />
              <button
                type="submit"
                disabled={isActivating || !newKey.trim()}
                className="px-4 py-2 bg-primary dark:bg-amber-500 text-primary-foreground dark:text-amber-950 text-xs font-bold rounded-xl cursor-pointer hover:bg-opacity-95 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isActivating ? 'Activation...' : 'Activer'}
              </button>
            </div>
          </form>
        </div>
      </div>

      {/* Hardware Panel - Dynamic Devices detection */}
      <div className="bg-card border border-border rounded-3xl p-6 shadow-sm space-y-6">
        <div className="flex items-center gap-3">
          <Printer className="w-5 h-5 text-primary dark:text-amber-400" />
          <h3 className="font-bold text-sm text-foreground">Périphériques Matériels Connectés</h3>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {/* Printer Configuration */}
          <div className="space-y-4">
            <h4 className="font-bold text-xs text-foreground flex items-center gap-1.5">
              <Printer className="w-4 h-4 text-muted-foreground" />
              Imprimante de Ticket par Défaut
            </h4>
            
            <div className="space-y-3">
              <div>
                <label className="text-[10px] font-bold text-muted-foreground block mb-1">Sélectionner l'imprimante</label>
                <select
                  value={selectedPrinter}
                  onChange={(e) => setSelectedPrinter(e.target.value)}
                  className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-semibold focus:outline-none text-foreground"
                >
                  {printers.length === 0 && (
                    <option value="">Aucune imprimante détectée</option>
                  )}
                  {printers.map((p, i) => (
                    <option key={i} value={p.name}>
                      {p.name} ({p.connected ? 'Connecté' : 'Déconnecté'})
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="text-[10px] font-bold text-muted-foreground block mb-1">Largeur du Papier</label>
                <select
                  value={printerWidth}
                  onChange={(e) => setPrinterWidth(e.target.value)}
                  className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-semibold focus:outline-none text-foreground"
                >
                  <option value="80">Standard 80mm (Recommandé)</option>
                  <option value="58">Compact 58mm</option>
                </select>
              </div>
            </div>
          </div>

          {/* Scanner Configuration */}
          <div className="space-y-4">
            <h4 className="font-bold text-xs text-foreground flex items-center gap-1.5">
              <Barcode className="w-4 h-4 text-muted-foreground" />
              Scanner de Code-barres par Défaut
            </h4>

            <div className="space-y-3">
              <div>
                <label className="text-[10px] font-bold text-muted-foreground block mb-1">Sélectionner le Scanner actif</label>
                <select
                  value={selectedScanner}
                  onChange={(e) => setSelectedScanner(e.target.value)}
                  className="w-full px-3 py-2 bg-accent/30 border border-border rounded-xl text-xs font-semibold focus:outline-none text-foreground"
                >
                  {scanners.length === 0 && (
                    <option value="">Aucun lecteur de code-barres détecté</option>
                  )}
                  {scanners.map((s, i) => (
                    <option key={i} value={s.name}>
                      {s.name} ({s.connected ? 'Connecté' : 'Déconnecté'})
                    </option>
                  ))}
                </select>
              </div>

              <div className="p-3 bg-accent/30 rounded-xl border border-border/50 text-[10px] font-semibold text-muted-foreground flex items-start gap-2">
                <AlertCircle className="w-4 h-4 text-primary dark:text-amber-400 shrink-0" />
                <span>Le scanner sélectionné captera automatiquement les entrées en caisse pour l'ajout au panier.</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Sync endpoints */}
      <div className="bg-card border border-border rounded-3xl p-6 shadow-sm space-y-4">
        <div className="flex items-center gap-3">
          <Database className="w-5 h-5 text-primary dark:text-amber-400" />
          <h3 className="font-bold text-sm text-foreground">Serveur de Synchronisation</h3>
        </div>

        <div>
          <label className="text-[10px] font-bold text-muted-foreground block mb-1">Adresse API Cloud</label>
          <input
            type="url"
            value={apiUrl}
            onChange={(e) => setApiUrl(e.target.value)}
            className="w-full px-3 py-2.5 bg-accent/30 border border-border rounded-xl text-xs font-semibold focus:outline-none focus:ring-1 focus:ring-primary text-foreground"
          />
        </div>
      </div>

      {/* Save Button */}
      <div className="flex justify-end pt-4">
        <button
          onClick={handleSaveSettings}
          className="flex items-center gap-1.5 px-6 py-3 rounded-2xl bg-primary dark:bg-amber-500 text-primary-foreground dark:text-amber-950 font-bold text-xs shadow-md hover:bg-opacity-95 transition-all cursor-pointer"
        >
          <Save className="w-4 h-4" />
          <span>Enregistrer les paramètres</span>
        </button>
      </div>

      {/* Modif Request Modal - Centered and Styled */}
      {showRequestModal && (
        <div className="fixed inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-scale-in">
          <div className="bg-card border border-border w-full max-w-md rounded-3xl shadow-2xl p-6 relative text-center space-y-4">
            <div className="w-12 h-12 rounded-full bg-amber-500/10 text-amber-500 flex items-center justify-center mx-auto">
              <HelpCircle className="w-6 h-6" />
            </div>
            
            <h3 className="font-extrabold text-base text-foreground">Validation Requise</h3>
            
            <p className="text-xs text-muted-foreground leading-relaxed">
              Votre tenant actuel n'est pas configuré comme tenant système maître (<span className="font-bold">is_system == false</span>). 
              La modification directe de l'adresse API cloud vers <span className="font-bold text-foreground">{requestUrl}</span> requiert une validation administrative.
            </p>

            <div className="flex gap-3 pt-2">
              <button
                onClick={() => setShowRequestModal(false)}
                className="flex-1 py-2.5 rounded-xl border border-border bg-card hover:bg-accent text-foreground text-xs font-bold cursor-pointer"
              >
                Annuler
              </button>
              <button
                onClick={submitUrlChangeRequest}
                className="flex-1 py-2.5 rounded-xl bg-primary dark:bg-amber-500 text-primary-foreground dark:text-amber-950 text-xs font-bold shadow-sm hover:bg-opacity-95 cursor-pointer"
              >
                Soumettre Demande
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

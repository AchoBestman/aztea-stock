import { useCallback, useEffect, useLayoutEffect, useRef, useState, type RefObject } from 'react';
import { createPortal } from 'react-dom';
import {
  BarcodeFormat,
  BrowserMultiFormatReader,
  DecodeHintType,
  NotFoundException,
} from '@zxing/library';
import { Camera, ChevronDown, ChevronUp, GripVertical, Maximize2, Minimize2, X } from 'lucide-react';

const SCAN_COOLDOWN_MS = 1800;

const PANEL_WIDTH = {
  normal: 300,
  large: 440,
} as const;

type PanelSize = keyof typeof PANEL_WIDTH;

interface CameraBarcodeScannerProps {
  anchorRef?: RefObject<HTMLElement | null>;
  onScan: (code: string) => void;
  onClose: () => void;
}

function getDefaultPosition(anchor: HTMLElement | null, panelWidth: number) {
  if (anchor) {
    const rect = anchor.getBoundingClientRect();
    return {
      x: rect.left + 16,
      y: rect.top + 16,
    };
  }
  return {
    x: Math.max(16, window.innerWidth - panelWidth - 40),
    y: 88,
  };
}

function getEffectiveWidth(panelSize: PanelSize) {
  const base = PANEL_WIDTH[panelSize];
  const maxW = window.innerWidth - 24;
  if (panelSize === 'large') {
    return Math.min(maxW, Math.max(base, Math.floor(window.innerWidth * 0.38)));
  }
  return Math.min(base, maxW);
}

function stopMediaStream(stream: MediaStream | null | undefined) {
  if (!stream) return;
  stream.getTracks().forEach((track) => track.stop());
}

function createScanReader() {
  const hints = new Map();
  hints.set(DecodeHintType.TRY_HARDER, true);
  hints.set(DecodeHintType.ASSUME_GS1, false);
  hints.set(DecodeHintType.POSSIBLE_FORMATS, [
    BarcodeFormat.QR_CODE,
    BarcodeFormat.EAN_13,
    BarcodeFormat.EAN_8,
    BarcodeFormat.CODE_128,
    BarcodeFormat.CODE_39,
    BarcodeFormat.UPC_A,
    BarcodeFormat.UPC_E,
    BarcodeFormat.ITF,
    BarcodeFormat.DATA_MATRIX,
  ]);
  return new BrowserMultiFormatReader(hints, SCAN_COOLDOWN_MS / 2);
}

function CameraBarcodeScannerPanel({ anchorRef, onScan, onClose }: CameraBarcodeScannerProps) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const panelRef = useRef<HTMLDivElement>(null);
  const readerRef = useRef<BrowserMultiFormatReader | null>(null);
  const streamRef = useRef<MediaStream | null>(null);
  const cameraSessionRef = useRef(0);
  const onScanRef = useRef(onScan);
  const lastScanRef = useRef<{ code: string; at: number }>({ code: '', at: 0 });
  const dragOffsetRef = useRef({ x: 0, y: 0 });

  onScanRef.current = onScan;

  const stopCamera = useCallback(() => {
    cameraSessionRef.current += 1;

    const reader = readerRef.current;
    readerRef.current = null;
    if (reader) {
      try {
        reader.reset();
      } catch {
        /* ignore */
      }
    }

    stopMediaStream(streamRef.current);
    streamRef.current = null;

    const video = videoRef.current;
    if (video) {
      stopMediaStream(video.srcObject as MediaStream | null);
      video.srcObject = null;
      video.pause();
      video.removeAttribute('src');
      video.load();
    }
  }, []);

  const handleClose = useCallback(() => {
    stopCamera();
    onClose();
  }, [onClose, stopCamera]);

  const [collapsed, setCollapsed] = useState(false);
  const [panelSize, setPanelSize] = useState<PanelSize>('normal');
  const [panelWidth, setPanelWidth] = useState<number>(PANEL_WIDTH.normal);
  const [position, setPosition] = useState({ x: 0, y: 0 });
  const [positionReady, setPositionReady] = useState(false);
  const [dragging, setDragging] = useState(false);
  const [cameraError, setCameraError] = useState<string | null>(null);
  const [status, setStatus] = useState('Démarrage de la caméra…');
  const [lastDetected, setLastDetected] = useState<string | null>(null);

  const clampPosition = useCallback(
    (x: number, y: number, width = panelWidth) => {
      const panel = panelRef.current;
      const panelHeight = panel?.offsetHeight ?? 280;
      const maxX = Math.max(0, window.innerWidth - width);
      const maxY = Math.max(0, window.innerHeight - panelHeight);
      return {
        x: Math.min(Math.max(0, x), maxX),
        y: Math.min(Math.max(0, y), maxY),
      };
    },
    [panelWidth]
  );

  useLayoutEffect(() => {
    const width = getEffectiveWidth(panelSize);
    setPanelWidth(width);
    const defaultPos = getDefaultPosition(anchorRef?.current ?? null, width);
    setPosition(clampPosition(defaultPos.x, defaultPos.y, width));
    setPositionReady(true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    const updateLayout = () => {
      const width = getEffectiveWidth(panelSize);
      setPanelWidth(width);
      setPosition((prev) => clampPosition(prev.x, prev.y, width));
    };
    updateLayout();
    window.addEventListener('resize', updateLayout);
    return () => window.removeEventListener('resize', updateLayout);
  }, [panelSize, clampPosition]);

  const handleDetectedCode = useCallback((code: string) => {
    const trimmed = code.trim();
    console.log('[CameraScanner] handleDetectedCode:', trimmed || '(vide)');
    if (!trimmed) return;

    const now = Date.now();
    if (lastScanRef.current.code === trimmed && now - lastScanRef.current.at < SCAN_COOLDOWN_MS) {
      console.log('[CameraScanner] Code ignoré (cooldown):', trimmed);
      return;
    }

    lastScanRef.current = { code: trimmed, at: now };
    setLastDetected(trimmed);
    setStatus(`Code détecté : ${trimmed}`);
    console.log('[CameraScanner] Envoi au POS:', trimmed);
    onScanRef.current(trimmed);
  }, []);

  const startCamera = useCallback(async () => {
    stopCamera();
    const session = cameraSessionRef.current;

    const reader = createScanReader();
    readerRef.current = reader;
    setCameraError(null);
    setStatus('Démarrage de la caméra…');

    try {
      if (!navigator.mediaDevices?.getUserMedia) {
        throw new Error("La caméra n'est pas disponible sur cet appareil.");
      }

      // Demande d'accès caméra (sinon enumerateDevices renvoie des libellés vides sur macOS).
      try {
        const prime = await navigator.mediaDevices.getUserMedia({ video: true, audio: false });
        stopMediaStream(prime);
      } catch (primeErr) {
        console.log('[CameraScanner] Permission caméra (prime):', primeErr);
      }

      let videoInputs: MediaDeviceInfo[] = [];
      try {
        const devices = await navigator.mediaDevices.enumerateDevices();
        videoInputs = devices.filter((d) => d.kind === 'videoinput');
      } catch (enumErr) {
        console.log('[CameraScanner] enumerateDevices:', enumErr);
      }

      if (session !== cameraSessionRef.current) return;

      const preferred =
        videoInputs.find((d) => /back|rear|environment|arrière/i.test(d.label)) ?? videoInputs[0];

      const video = videoRef.current;
      if (!video || session !== cameraSessionRef.current) {
        throw new Error('Élément vidéo indisponible.');
      }

      video.setAttribute('playsinline', 'true');
      video.setAttribute('webkit-playsinline', 'true');
      video.muted = true;

      const deviceId = preferred?.deviceId;
      console.log('[CameraScanner] Caméras détectées:', videoInputs.map((d) => d.label || d.deviceId));
      console.log('[CameraScanner] Démarrage decodeFromVideoDevice, deviceId:', deviceId ?? '(défaut)');

      setStatus('Placez le code dans le cadre (écran ou étiquette)');

      await reader.decodeFromVideoDevice(deviceId ?? null, video, (result, err) => {
        if (session !== cameraSessionRef.current) return;
        if (result) {
          const text = result.getText();
          console.log('[CameraScanner] decodeFromVideoDevice OK:', text);
          handleDetectedCode(text);
        }
        if (err) {
          if (err instanceof NotFoundException) return;
          console.log('[CameraScanner] Erreur frame:', err);
        }
      });

      streamRef.current = (video.srcObject as MediaStream | null) ?? null;
      console.log('[CameraScanner] Flux vidéo actif, tracks:', streamRef.current?.getTracks().length ?? 0);
    } catch (e) {
      if (session !== cameraSessionRef.current) return;
      const message =
        e instanceof Error
          ? e.message
          : "Impossible d'accéder à la caméra. Autorisez l'accès dans les paramètres système.";
      setCameraError(message);
      setStatus('Caméra indisponible');
      stopMediaStream(streamRef.current);
      streamRef.current = null;
    }
  }, [handleDetectedCode, stopCamera]);

  useEffect(() => {
    startCamera();
    return () => {
      stopCamera();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const onHeaderPointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    if ((e.target as HTMLElement).closest('button')) return;
    e.preventDefault();
    setDragging(true);
    dragOffsetRef.current = { x: e.clientX - position.x, y: e.clientY - position.y };
    e.currentTarget.setPointerCapture(e.pointerId);
  };

  const onHeaderPointerMove = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!dragging) return;
    const next = clampPosition(e.clientX - dragOffsetRef.current.x, e.clientY - dragOffsetRef.current.y);
    setPosition(next);
  };

  const onHeaderPointerUp = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!dragging) return;
    setDragging(false);
    e.currentTarget.releasePointerCapture(e.pointerId);
  };

  const togglePanelSize = () => {
    setPanelSize((s) => (s === 'normal' ? 'large' : 'normal'));
  };

  if (!positionReady) return null;

  return (
    <div
      ref={panelRef}
      style={{ left: position.x, top: position.y, width: panelWidth }}
      className={`fixed z-[180] max-w-[calc(100vw-24px)] rounded-2xl border border-primary/30 bg-card/95 backdrop-blur-md shadow-2xl overflow-hidden transition-[width,box-shadow] ${
        dragging ? 'shadow-primary/25 ring-2 ring-primary/40 cursor-grabbing' : 'shadow-xl'
      }`}
    >
      <div
        className="flex items-center gap-1.5 px-2.5 py-2 bg-primary/10 border-b border-border cursor-grab active:cursor-grabbing touch-none"
        onPointerDown={onHeaderPointerDown}
        onPointerMove={onHeaderPointerMove}
        onPointerUp={onHeaderPointerUp}
        onPointerCancel={onHeaderPointerUp}
      >
        <GripVertical className="w-4 h-4 text-muted-foreground shrink-0" />
        <Camera className="w-4 h-4 text-primary dark:text-blue-500 shrink-0" />
        <span className="flex-1 text-[11px] font-extrabold text-foreground truncate">Scanner caméra</span>
        {!collapsed && (
          <button
            type="button"
            onClick={togglePanelSize}
            className="p-1.5 rounded-lg hover:bg-accent text-muted-foreground transition-colors cursor-pointer"
            title={panelSize === 'normal' ? 'Agrandir' : 'Réduire'}
          >
            {panelSize === 'normal' ? <Maximize2 className="w-4 h-4" /> : <Minimize2 className="w-4 h-4" />}
          </button>
        )}
        <button
          type="button"
          onClick={() => setCollapsed((v) => !v)}
          className="p-1.5 rounded-lg hover:bg-accent text-muted-foreground transition-colors cursor-pointer"
          title={collapsed ? 'Déplier' : 'Replier'}
        >
          {collapsed ? <ChevronDown className="w-4 h-4" /> : <ChevronUp className="w-4 h-4" />}
        </button>
        <button
          type="button"
          onClick={handleClose}
          className="p-1.5 rounded-lg hover:bg-rose-500/10 text-rose-500 transition-colors cursor-pointer"
          title="Fermer"
        >
          <X className="w-4 h-4" />
        </button>
      </div>

      <div
        className={
          collapsed
            ? 'absolute opacity-0 pointer-events-none overflow-hidden'
            : 'p-3 space-y-2'
        }
        style={collapsed ? { left: -9999, top: 0, width: 400, height: 320 } : undefined}
      >
        <div className="relative rounded-xl overflow-hidden bg-black" style={{ aspectRatio: '4 / 3', minHeight: 160 }}>
          <video ref={videoRef} className="w-full h-full object-cover" muted playsInline autoPlay />
          <div className="absolute inset-0 pointer-events-none flex items-center justify-center">
            <div className="w-[78%] h-[62%] border-2 border-primary/90 rounded-lg shadow-[0_0_0_9999px_rgba(0,0,0,0.4)]">
              <span className="absolute -top-0.5 -left-0.5 w-5 h-5 border-t-2 border-l-2 border-primary rounded-tl" />
              <span className="absolute -top-0.5 -right-0.5 w-5 h-5 border-t-2 border-r-2 border-primary rounded-tr" />
              <span className="absolute -bottom-0.5 -left-0.5 w-5 h-5 border-b-2 border-l-2 border-primary rounded-bl" />
              <span className="absolute -bottom-0.5 -right-0.5 w-5 h-5 border-b-2 border-r-2 border-primary rounded-br" />
            </div>
          </div>
        </div>

        {cameraError ? (
          <div className="rounded-xl bg-rose-500/10 border border-rose-500/20 px-3 py-2">
            <p className="text-[10px] font-semibold text-rose-600 leading-snug">{cameraError}</p>
            <button
              type="button"
              onClick={startCamera}
              className="mt-2 text-[10px] font-bold text-primary dark:text-blue-500 hover:underline cursor-pointer"
            >
              Réessayer
            </button>
          </div>
        ) : (
          <p className="text-[10px] font-semibold text-muted-foreground text-center leading-snug">{status}</p>
        )}

        {lastDetected && (
          <p className="text-[10px] font-mono text-center text-emerald-600 dark:text-emerald-400 truncate px-1">
            Dernier : {lastDetected}
          </p>
        )}
      </div>

      {collapsed && (
        <p className="px-3 py-2 text-[10px] font-semibold text-muted-foreground text-center">
          {cameraError ? 'Caméra indisponible' : 'Scan actif — déplier pour voir la caméra'}
        </p>
      )}
    </div>
  );
}

export function CameraBarcodeScanner(props: CameraBarcodeScannerProps) {
  return createPortal(<CameraBarcodeScannerPanel {...props} />, document.body);
}

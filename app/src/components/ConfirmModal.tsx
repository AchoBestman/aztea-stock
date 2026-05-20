import React from 'react';
import { AlertTriangle, X } from 'lucide-react';

interface ConfirmModalProps {
  isOpen: boolean;
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmModal({
  isOpen,
  title,
  message,
  confirmText = 'Confirmer',
  cancelText = 'Annuler',
  onConfirm,
  onCancel
}: ConfirmModalProps) {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center p-4 z-[100] animate-scale-in select-none">
      <div className="bg-card border border-border w-full max-w-sm rounded-3xl shadow-2xl p-8 relative flex flex-col items-center text-center">
        <button 
          onClick={onCancel}
          className="absolute top-4 right-4 p-2 rounded-full hover:bg-muted text-muted-foreground transition-colors cursor-pointer"
        >
          <X className="w-5 h-5" />
        </button>
        
        <div className="w-20 h-20 bg-rose-500/10 rounded-full flex items-center justify-center mb-5 text-rose-500 shadow-inner">
          <AlertTriangle className="w-10 h-10" />
        </div>
        
        <h3 className="text-xl font-extrabold text-foreground mb-2">{title}</h3>
        <p className="text-sm font-semibold text-muted-foreground mb-8">
          {message}
        </p>

        <div className="flex gap-4 w-full">
          <button
            onClick={onCancel}
            className="flex-1 py-3.5 rounded-2xl border border-border bg-card hover:bg-accent text-foreground text-xs font-bold transition-colors cursor-pointer shadow-sm"
          >
            {cancelText}
          </button>
          <button
            onClick={onConfirm}
            className="flex-1 py-3.5 rounded-2xl bg-rose-600 hover:bg-rose-700 text-white text-xs font-bold shadow-md transition-colors cursor-pointer"
          >
            {confirmText}
          </button>
        </div>
      </div>
    </div>
  );
}

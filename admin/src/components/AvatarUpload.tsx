import { useRef, useState } from "react";
import { ImagePlus, Loader2 } from "lucide-react";

interface Props {
  previewUrl?: string | null;
  onFileSelect: (file: File | null) => void;
  disabled?: boolean;
  progress?: number | null;
}

export default function AvatarUpload({ previewUrl, onFileSelect, disabled, progress }: Props) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [localPreview, setLocalPreview] = useState<string | null>(null);

  const shown = localPreview || previewUrl;
  const uploading = progress !== null && progress !== undefined;

  const onChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0] ?? null;
    if (file) {
      setLocalPreview(URL.createObjectURL(file));
      onFileSelect(file);
    } else {
      setLocalPreview(null);
      onFileSelect(null);
    }
  };

  return (
    <div className="flex items-center gap-4">
      <div className="flex flex-col gap-1.5">
        <button
          type="button"
          disabled={disabled || uploading}
          onClick={() => inputRef.current?.click()}
          className="relative w-20 h-20 rounded-2xl border-2 border-dashed border-border flex items-center justify-center overflow-hidden bg-muted/30 cursor-pointer hover:border-primary disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {shown ? (
            <img src={shown} alt="" className="w-full h-full object-cover" />
          ) : (
            <ImagePlus className="w-8 h-8 text-muted-foreground" />
          )}
          {uploading && (
            <div className="absolute inset-0 bg-black/50 flex flex-col items-center justify-center rounded-2xl gap-1">
              <Loader2 className="w-5 h-5 text-white animate-spin" />
              <span className="text-white text-xs font-bold">{progress}%</span>
            </div>
          )}
        </button>
        {uploading && (
          <div className="w-20 bg-muted rounded-full h-1.5 overflow-hidden">
            <div
              className="bg-primary h-1.5 rounded-full transition-all duration-150 ease-out"
              style={{ width: `${progress}%` }}
            />
          </div>
        )}
      </div>
      <div className="text-sm text-muted-foreground">
        <p className="font-medium text-foreground">
          {uploading ? `Upload en cours…` : "Avatar (optionnel)"}
        </p>
        <p>{uploading ? `${progress}% envoyé` : "PNG, JPG ou WebP — stocké sur R2"}</p>
      </div>
      <input
        ref={inputRef}
        type="file"
        accept="image/png,image/jpeg,image/webp"
        className="hidden"
        onChange={onChange}
      />
    </div>
  );
}

type Props = {
  value: string;
  onChange: (url: string | undefined) => void;
  hint?: string;
};

export default function LogoUrlField({ value, onChange, hint }: Props) {
  return (
    <div className="space-y-1">
      <label className="text-sm font-medium text-muted-foreground">Logo URL (optionnel)</label>
      <input
        type="url"
        placeholder="https://…"
        className="form-input"
        value={value}
        onChange={(e) => onChange(e.target.value || undefined)}
      />
      <p className="text-xs text-muted-foreground">
        {hint ??
          "Upload R2 non configuré — collez une URL d'image (hébergement externe ou bucket public)."}
      </p>
    </div>
  );
}

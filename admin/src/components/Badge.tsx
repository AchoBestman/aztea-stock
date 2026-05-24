import clsx from "clsx";

const styles: Record<string, string> = {
  active: "bg-emerald-500/15 text-emerald-700",
  trial: "bg-sky-500/15 text-sky-700",
  suspended: "bg-amber-500/15 text-amber-700",
  cancelled: "bg-zinc-500/15 text-zinc-600",
  revoked: "bg-rose-500/15 text-rose-700",
  success: "bg-emerald-500/15 text-emerald-700",
  failed: "bg-rose-500/15 text-rose-700",
  partial: "bg-amber-500/15 text-amber-700",
  default: "bg-muted text-muted-foreground",
};

export default function Badge({
  label,
  tone = "default",
}: {
  label: string;
  tone?: keyof typeof styles;
}) {
  return (
    <span
      className={clsx(
        "inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold capitalize",
        styles[tone] || styles.default
      )}
    >
      {label}
    </span>
  );
}

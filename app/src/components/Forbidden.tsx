import { Link } from "react-router-dom";
import { ShieldAlert } from "lucide-react";
import { usePermissions } from "../hooks/usePermissions";

type Props = {
  message?: string;
};

export default function Forbidden({
  message = "Vous n'avez pas la permission d'accéder à cette page.",
}: Props) {
  const { firstAllowedPath } = usePermissions();
  const fallback = firstAllowedPath();

  return (
    <div className="flex flex-col items-center justify-center min-h-[50vh] text-center p-8">
      <div className="w-14 h-14 rounded-full bg-rose-500/10 flex items-center justify-center mb-4 text-rose-500">
        <ShieldAlert className="w-7 h-7" />
      </div>
      <h2 className="text-xl font-bold text-foreground mb-2">Accès refusé</h2>
      <p className="text-muted-foreground max-w-md text-sm font-medium">{message}</p>
      {fallback && (
        <Link
          to={fallback}
          className="mt-6 px-5 py-2.5 rounded-xl bg-primary text-primary-foreground font-semibold text-sm"
        >
          Retour à l&apos;accueil autorisé
        </Link>
      )}
    </div>
  );
}

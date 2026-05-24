import { LogOut, User } from "lucide-react";
import { useAuthStore } from "../store/authStore";

export default function Header({ title }: { title?: string }) {
  const { user, logout } = useAuthStore();

  return (
    <header className="h-16 shrink-0 border-b border-border bg-card/80 backdrop-blur px-8 flex items-center justify-between">
      <h1 className="text-lg font-bold text-foreground">{title || "AzteaStock Admin"}</h1>
      <div className="flex items-center gap-4">
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <User className="w-4 h-4" />
          <span className="font-medium text-foreground">{user?.name}</span>
        </div>
        <button
          type="button"
          onClick={logout}
          className="flex items-center gap-2 px-3 py-2 rounded-xl border border-border text-sm font-semibold hover:bg-accent cursor-pointer"
        >
          <LogOut className="w-4 h-4" />
          Déconnexion
        </button>
      </div>
    </header>
  );
}

interface ProgressBarProps {
  percentage: number;
  label?: string;
  sublabel?: string;
  variant?: "default" | "success" | "warning" | "error";
}

export function ProgressBar({
  percentage,
  label,
  sublabel,
  variant = "default",
}: ProgressBarProps) {
  const getBarColor = () => {
    switch (variant) {
      case "success":
        return "bg-emerald-500";
      case "warning":
        return "bg-amber-500";
      case "error":
        return "bg-red-500";
      default:
        return "bg-violet-500";
    }
  };

  const getGlowColor = () => {
    switch (variant) {
      case "success":
        return "shadow-emerald-500/30";
      case "warning":
        return "shadow-amber-500/30";
      case "error":
        return "shadow-red-500/30";
      default:
        return "shadow-violet-500/30";
    }
  };

  return (
    <div className="space-y-1.5 sm:space-y-2">
      {(label || sublabel) && (
        <div className="flex items-center justify-between text-xs sm:text-sm">
          <span className="text-slate-300 font-medium">{label}</span>
          <span className="text-slate-500">{sublabel}</span>
        </div>
      )}
      <div className="h-1.5 sm:h-2 bg-slate-700/50 rounded-full overflow-hidden">
        <div
          className={`h-full ${getBarColor()} rounded-full transition-all duration-300 shadow-lg ${getGlowColor()} progress-animated`}
          style={{ width: `${Math.min(100, Math.max(0, percentage))}%` }}
        />
      </div>
    </div>
  );
}

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

  return (
    <div className="space-y-1">
      {(label || sublabel) && (
        <div className="flex items-center justify-between text-[10px]">
          <span className="text-slate-300 font-medium">{label}</span>
          <span className="text-slate-500">{sublabel}</span>
        </div>
      )}
      <div className="h-1.5 bg-slate-700/50 rounded-full overflow-hidden">
        <div
          className={`h-full ${getBarColor()} rounded-full transition-all duration-200`}
          style={{ width: `${Math.min(100, Math.max(0, percentage))}%` }}
        />
      </div>
    </div>
  );
}

interface ProgressBarProps {
  percentage: number;
  label?: string;
  sublabel?: string;
  variant?: "default" | "success" | "warning" | "error" | "cyan" | "amber" | "fuchsia";
}

export function ProgressBar({
  percentage,
  label,
  sublabel,
  variant = "default",
}: ProgressBarProps) {
  const getBarStyle = () => {
    switch (variant) {
      case "success":
        return {
          bg: "bg-emerald-500",
          glow: "shadow-[0_0_10px_rgba(16,185,129,0.5)]",
          text: "text-emerald-400",
        };
      case "warning":
        return {
          bg: "bg-amber-500",
          glow: "shadow-[0_0_10px_rgba(245,158,11,0.5)]",
          text: "text-amber-400",
        };
      case "error":
        return {
          bg: "bg-red-500",
          glow: "shadow-[0_0_10px_rgba(239,68,68,0.5)]",
          text: "text-red-400",
        };
      case "cyan":
        return {
          bg: "bg-cyan-500",
          glow: "shadow-[0_0_10px_rgba(6,182,212,0.5)]",
          text: "text-cyan-400",
        };
      case "amber":
        return {
          bg: "bg-amber-500",
          glow: "shadow-[0_0_10px_rgba(245,158,11,0.5)]",
          text: "text-amber-400",
        };
      case "fuchsia":
        return {
          bg: "bg-fuchsia-500",
          glow: "shadow-[0_0_10px_rgba(217,70,239,0.5)]",
          text: "text-fuchsia-400",
        };
      default:
        return {
          bg: "bg-violet-500",
          glow: "shadow-[0_0_10px_rgba(139,92,246,0.5)]",
          text: "text-violet-400",
        };
    }
  };

  const style = getBarStyle();

  return (
    <div className="space-y-1">
      {(label || sublabel) && (
        <div className="flex items-center justify-between text-[10px]">
          <span className={`font-medium ${style.text}`}>{label}</span>
          <span className="text-slate-400">{sublabel}</span>
        </div>
      )}
      <div className="h-1.5 bg-slate-700/50 rounded-full overflow-hidden">
        <div
          className={`h-full ${style.bg} ${style.glow} rounded-full transition-all duration-200`}
          style={{ width: `${Math.min(100, Math.max(0, percentage))}%` }}
        />
      </div>
    </div>
  );
}

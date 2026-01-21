import { ButtonHTMLAttributes, forwardRef } from "react";
import { Loader2 } from "lucide-react";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "secondary" | "success" | "danger" | "ghost" | "cyan" | "amber" | "fuchsia";
  size?: "sm" | "md" | "lg";
  isLoading?: boolean;
  leftIcon?: React.ReactNode;
  rightIcon?: React.ReactNode;
}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      variant = "primary",
      size = "md",
      isLoading,
      leftIcon,
      rightIcon,
      children,
      disabled,
      className = "",
      ...props
    },
    ref
  ) => {
    const baseStyles =
      "inline-flex items-center justify-center font-medium rounded-md transition-all duration-150 focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed border";

    const variants = {
      primary:
        "bg-violet-500/15 text-violet-300 border-violet-500/50 hover:bg-violet-500/25 hover:border-violet-400/60 shadow-[0_0_10px_rgba(139,92,246,0.2)] hover:shadow-[0_0_15px_rgba(139,92,246,0.35)]",
      secondary:
        "bg-slate-500/15 text-slate-300 border-slate-500/50 hover:bg-slate-500/25 hover:border-slate-400/60",
      success:
        "bg-emerald-500/15 text-emerald-300 border-emerald-500/50 hover:bg-emerald-500/25 hover:border-emerald-400/60 shadow-[0_0_10px_rgba(16,185,129,0.2)] hover:shadow-[0_0_15px_rgba(16,185,129,0.35)]",
      danger:
        "bg-red-500/15 text-red-300 border-red-500/50 hover:bg-red-500/25 hover:border-red-400/60 shadow-[0_0_10px_rgba(239,68,68,0.2)] hover:shadow-[0_0_15px_rgba(239,68,68,0.35)]",
      ghost:
        "bg-transparent text-slate-300 border-transparent hover:bg-slate-700/50 hover:border-slate-600/50",
      cyan:
        "bg-cyan-500/15 text-cyan-300 border-cyan-500/50 hover:bg-cyan-500/25 hover:border-cyan-400/60 shadow-[0_0_10px_rgba(6,182,212,0.2)] hover:shadow-[0_0_15px_rgba(6,182,212,0.35)]",
      amber:
        "bg-amber-500/15 text-amber-300 border-amber-500/50 hover:bg-amber-500/25 hover:border-amber-400/60 shadow-[0_0_10px_rgba(245,158,11,0.2)] hover:shadow-[0_0_15px_rgba(245,158,11,0.35)]",
      fuchsia:
        "bg-fuchsia-500/15 text-fuchsia-300 border-fuchsia-500/50 hover:bg-fuchsia-500/25 hover:border-fuchsia-400/60 shadow-[0_0_10px_rgba(217,70,239,0.2)] hover:shadow-[0_0_15px_rgba(217,70,239,0.35)]",
    };

    const sizes = {
      sm: "px-2 py-1 text-xs gap-1",
      md: "px-3 py-1.5 text-xs gap-1.5",
      lg: "px-4 py-2 text-sm gap-1.5",
    };

    return (
      <button
        ref={ref}
        disabled={disabled || isLoading}
        className={`${baseStyles} ${variants[variant]} ${sizes[size]} ${className}`}
        {...props}
      >
        {isLoading ? (
          <Loader2 className="animate-spin" size={12} />
        ) : (
          leftIcon && <span className="drop-shadow-[0_0_4px_currentColor]">{leftIcon}</span>
        )}
        {children}
        {rightIcon && <span className="drop-shadow-[0_0_4px_currentColor]">{rightIcon}</span>}
      </button>
    );
  }
);

Button.displayName = "Button";

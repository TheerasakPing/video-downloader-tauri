import { InputHTMLAttributes, forwardRef } from "react";

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  leftIcon?: React.ReactNode;
  rightElement?: React.ReactNode;
  iconColor?: "violet" | "cyan" | "amber" | "emerald" | "blue" | "fuchsia";
}

const iconColors = {
  violet: "text-violet-400",
  cyan: "text-cyan-400",
  amber: "text-amber-400",
  emerald: "text-emerald-400",
  blue: "text-blue-400",
  fuchsia: "text-fuchsia-400",
};

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ label, error, leftIcon, rightElement, iconColor = "violet", className = "", ...props }, ref) => {
    return (
      <div className="space-y-1">
        {label && (
          <label className="block text-xs font-medium text-slate-400">
            {label}
          </label>
        )}
        <div className="relative">
          {leftIcon && (
            <div className={`absolute left-2.5 top-1/2 -translate-y-1/2 ${iconColors[iconColor]} drop-shadow-[0_0_4px_currentColor]`}>
              {leftIcon}
            </div>
          )}
          <input
            ref={ref}
            className={`
              w-full bg-slate-800/50 border border-slate-700 rounded-lg
              px-3 py-1.5 text-sm text-white placeholder-slate-500
              focus:outline-none focus:ring-1 focus:ring-violet-500/50 focus:border-violet-500
              transition-all duration-150
              ${leftIcon ? "pl-8" : ""}
              ${rightElement ? "pr-20" : ""}
              ${error ? "border-red-500 focus:ring-red-500/50 focus:border-red-500" : ""}
              ${className}
            `}
            {...props}
          />
          {rightElement && (
            <div className="absolute right-1 top-1/2 -translate-y-1/2">
              {rightElement}
            </div>
          )}
        </div>
        {error && <p className="text-xs text-red-400">{error}</p>}
      </div>
    );
  }
);

Input.displayName = "Input";

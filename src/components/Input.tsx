import { InputHTMLAttributes, forwardRef } from "react";

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  leftIcon?: React.ReactNode;
  rightElement?: React.ReactNode;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ label, error, leftIcon, rightElement, className = "", ...props }, ref) => {
    return (
      <div className="space-y-1">
        {label && (
          <label className="block text-xs font-medium text-slate-400">
            {label}
          </label>
        )}
        <div className="relative">
          {leftIcon && (
            <div className="absolute left-2.5 top-1/2 -translate-y-1/2 text-slate-500">
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

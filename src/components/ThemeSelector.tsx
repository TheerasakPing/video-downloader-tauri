import React from "react";
import { Palette, Check } from "lucide-react";
import { CustomTheme } from "../hooks/useCustomTheme";

interface ThemeSelectorProps {
  themes: CustomTheme[];
  activeThemeId: string;
  onSelect: (themeId: string) => void;
}

export const ThemeSelector: React.FC<ThemeSelectorProps> = ({
  themes,
  activeThemeId,
  onSelect,
}) => {
  return (
    <div className="space-y-3">
      <div className="flex items-center gap-2 text-sm font-medium text-slate-300">
        <span className="icon-glow icon-glow-sm icon-glow-fuchsia">
          <Palette size={14} />
        </span>
        Color Theme
      </div>

      <div className="grid grid-cols-5 gap-2">
        {themes.map((theme) => (
          <button
            key={theme.id}
            onClick={() => onSelect(theme.id)}
            className={`relative group flex flex-col items-center gap-2 p-2 rounded-xl border transition-all ${
              activeThemeId === theme.id
                ? "border-white/30 bg-slate-700/50"
                : "border-slate-700/50 bg-slate-800/30 hover:bg-slate-700/30"
            }`}
            title={theme.name}
          >
            {/* Color preview */}
            <div className="relative w-10 h-10 rounded-lg overflow-hidden">
              {/* Primary color */}
              <div
                className="absolute inset-0"
                style={{ backgroundColor: theme.colors.primary }}
              />
              {/* Accent overlay */}
              <div
                className="absolute bottom-0 right-0 w-1/2 h-1/2 rounded-tl-lg"
                style={{ backgroundColor: theme.colors.accent }}
              />
              {/* Background corner */}
              <div
                className="absolute top-0 left-0 w-3 h-3 rounded-br-lg"
                style={{ backgroundColor: theme.colors.background }}
              />

              {/* Active indicator */}
              {activeThemeId === theme.id && (
                <div className="absolute inset-0 flex items-center justify-center bg-black/30">
                  <Check size={16} className="text-white" />
                </div>
              )}
            </div>

            {/* Theme name */}
            <span className="text-[10px] text-slate-400 truncate w-full text-center">
              {theme.name.split(" ")[0]}
            </span>
          </button>
        ))}
      </div>
    </div>
  );
};

export default ThemeSelector;

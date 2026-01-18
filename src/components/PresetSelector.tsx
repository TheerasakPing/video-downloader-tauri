import React from "react";
import { Zap, Check } from "lucide-react";
import { DownloadPreset } from "../hooks/useDownloadPresets";

interface PresetSelectorProps {
  presets: DownloadPreset[];
  activePresetId: string | null;
  onSelect: (presetId: string) => void;
}

export const PresetSelector: React.FC<PresetSelectorProps> = ({
  presets,
  activePresetId,
  onSelect,
}) => {
  return (
    <div className="space-y-3">
      <div className="flex items-center gap-2 text-sm font-medium text-slate-300">
        <span className="icon-glow icon-glow-sm icon-glow-amber">
          <Zap size={14} />
        </span>
        Quick Presets
      </div>

      <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
        {presets.map((preset) => (
          <button
            key={preset.id}
            onClick={() => onSelect(preset.id)}
            className={`relative flex flex-col items-center gap-1.5 p-3 rounded-xl border transition-all ${
              activePresetId === preset.id
                ? "bg-violet-500/20 border-violet-500/50 text-white"
                : "bg-slate-800/50 border-slate-700/50 text-slate-400 hover:bg-slate-700/50 hover:text-white"
            }`}
          >
            {activePresetId === preset.id && (
              <div className="absolute top-1.5 right-1.5">
                <Check size={12} className="text-violet-400" />
              </div>
            )}
            <span className="text-xl">{preset.icon}</span>
            <span className="text-xs font-medium truncate w-full text-center">
              {preset.name}
            </span>
          </button>
        ))}
      </div>
    </div>
  );
};

export default PresetSelector;

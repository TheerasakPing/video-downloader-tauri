import { useState, useCallback } from "react";
import { Settings } from "./useSettings";

export interface DownloadPreset {
  id: string;
  name: string;
  icon: string;
  description: string;
  settings: Partial<Settings>;
}

export const PRESETS: DownloadPreset[] = [
  {
    id: "hq",
    name: "High Quality",
    icon: "💎",
    description: "Max quality, single download",
    settings: {
      concurrentDownloads: 1,
      speedLimit: 0,
      autoMerge: true,
    },
  },
  {
    id: "turbo",
    name: "Turbo Mode",
    icon: "🚀",
    description: "Max speed, 5 concurrent",
    settings: {
      concurrentDownloads: 5,
      speedLimit: 0,
    },
  },
  {
    id: "saver",
    name: "Data Saver",
    icon: "🍃",
    description: "Limited speed (2MB/s)",
    settings: {
      speedLimit: 2000 * 1024, // 2MB/s
    },
  },
  {
    id: "night",
    name: "Night Mode",
    icon: "🌙",
    description: "Dark theme, silent",
    settings: {
      theme: "dark",
      soundEnabled: false,
    },
  },
];

export const useDownloadPresets = (
  onUpdateSettings: (newSettings: Partial<Settings>) => void,
) => {
  const [activePresetId, setActivePresetId] = useState<string | null>(null);

  const applyPreset = useCallback(
    (presetId: string) => {
      const preset = PRESETS.find((p) => p.id === presetId);
      if (preset) {
        onUpdateSettings(preset.settings);
        setActivePresetId(presetId);
      }
    },
    [onUpdateSettings],
  );

  return {
    presets: PRESETS,
    activePresetId,
    applyPreset,
  };
};

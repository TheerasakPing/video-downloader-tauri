import { useState, useCallback, useEffect } from "react";
import { Settings } from "./useSettings";

export interface DownloadPreset {
  id: string;
  name: string;
  icon: string;
  settings: Partial<Settings>;
}

const DEFAULT_PRESETS: DownloadPreset[] = [
  {
    id: "high-quality",
    name: "High Quality",
    icon: "🎬",
    settings: {
      concurrentDownloads: 2,
      speedLimit: 0,
      autoMerge: true,
      deleteAfterMerge: true,
    },
  },
  {
    id: "save-bandwidth",
    name: "Save Bandwidth",
    icon: "📶",
    settings: {
      concurrentDownloads: 1,
      speedLimit: 500,
      autoMerge: false,
      deleteAfterMerge: false,
    },
  },
  {
    id: "fast-download",
    name: "Fast Download",
    icon: "⚡",
    settings: {
      concurrentDownloads: 5,
      speedLimit: 0,
      autoMerge: false,
      deleteAfterMerge: false,
    },
  },
  {
    id: "night-mode",
    name: "Night Mode",
    icon: "🌙",
    settings: {
      concurrentDownloads: 3,
      speedLimit: 0,
      autoMerge: true,
      deleteAfterMerge: true,
      soundEnabled: false,
      notificationsEnabled: false,
    },
  },
];

const STORAGE_KEY = "rongyok-download-presets";

export function useDownloadPresets() {
  const [presets, setPresets] = useState<DownloadPreset[]>(() => {
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      return saved ? JSON.parse(saved) : DEFAULT_PRESETS;
    } catch {
      return DEFAULT_PRESETS;
    }
  });

  const [activePresetId, setActivePresetId] = useState<string | null>(null);

  // Save to localStorage
  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(presets));
  }, [presets]);

  const addPreset = useCallback((preset: Omit<DownloadPreset, "id">) => {
    const newPreset: DownloadPreset = {
      ...preset,
      id: `preset-${Date.now()}`,
    };
    setPresets((prev) => [...prev, newPreset]);
    return newPreset;
  }, []);

  const updatePreset = useCallback((id: string, updates: Partial<DownloadPreset>) => {
    setPresets((prev) =>
      prev.map((p) => (p.id === id ? { ...p, ...updates } : p))
    );
  }, []);

  const deletePreset = useCallback((id: string) => {
    setPresets((prev) => prev.filter((p) => p.id !== id));
    if (activePresetId === id) {
      setActivePresetId(null);
    }
  }, [activePresetId]);

  const applyPreset = useCallback((
    presetId: string,
    updateSetting: <K extends keyof Settings>(key: K, value: Settings[K]) => void
  ) => {
    const preset = presets.find((p) => p.id === presetId);
    if (preset) {
      Object.entries(preset.settings).forEach(([key, value]) => {
        updateSetting(key as keyof Settings, value as Settings[keyof Settings]);
      });
      setActivePresetId(presetId);
      return true;
    }
    return false;
  }, [presets]);

  const resetPresets = useCallback(() => {
    setPresets(DEFAULT_PRESETS);
    setActivePresetId(null);
  }, []);

  const getActivePreset = useCallback(() => {
    return presets.find((p) => p.id === activePresetId) || null;
  }, [presets, activePresetId]);

  return {
    presets,
    activePresetId,
    addPreset,
    updatePreset,
    deletePreset,
    applyPreset,
    resetPresets,
    getActivePreset,
    setActivePresetId,
  };
}

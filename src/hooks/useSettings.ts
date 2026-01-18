import { useState, useEffect } from "react";

export interface Settings {
  concurrentDownloads: number;
  speedLimit: number; // KB/s, 0 = unlimited
  autoMerge: boolean;
  deleteAfterMerge: boolean;
  notificationsEnabled: boolean;
  soundEnabled: boolean;
  theme: "dark" | "light" | "system";
  fileNaming: "ep_001" | "episode_1" | "title_ep1";
  outputDir: string;
}

const DEFAULT_SETTINGS: Settings = {
  concurrentDownloads: 3,
  speedLimit: 0,
  autoMerge: true,
  deleteAfterMerge: true,
  notificationsEnabled: true,
  soundEnabled: true,
  theme: "dark",
  fileNaming: "ep_001",
  outputDir: "~/Downloads/rongyok",
};

const STORAGE_KEY = "rongyok-settings-v2";

export function useSettings() {
  const [settings, setSettings] = useState<Settings>(DEFAULT_SETTINGS);
  const [isLoaded, setIsLoaded] = useState(false);

  // Load settings on mount
  useEffect(() => {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      try {
        const parsed = JSON.parse(stored);
        setSettings({ ...DEFAULT_SETTINGS, ...parsed });
      } catch (e) {
        console.error("Failed to parse settings:", e);
      }
    }
    setIsLoaded(true);
  }, []);

  // Save settings when changed
  useEffect(() => {
    if (isLoaded) {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
    }
  }, [settings, isLoaded]);

  const updateSetting = <K extends keyof Settings>(key: K, value: Settings[K]) => {
    setSettings((prev) => ({ ...prev, [key]: value }));
  };

  const resetSettings = () => {
    setSettings(DEFAULT_SETTINGS);
  };

  return { settings, updateSetting, resetSettings, isLoaded };
}

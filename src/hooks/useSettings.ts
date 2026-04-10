import { useState, useEffect } from "react";
import { ScheduleConfig, ProxyConfig, RetryConfig } from "../types";

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
  groupBySource: boolean; // สร้าง subfolder ตามชื่อเว็บ (rongyok/, titan/, baanjeen/, hsck/, njavtv/)
  defaultQuality: 'best' | 'ask';
  // Phase 2 settings
  scheduleConfig: ScheduleConfig;
  proxyConfig: ProxyConfig;
  retryConfig: RetryConfig;
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
  groupBySource: true, // เปิดใช้งานโดย default
  defaultQuality: 'best',
  // Phase 2 settings
  scheduleConfig: {
    enabled: false,
    activeStart: undefined,
    activeEnd: undefined,
    speedDuringActive: 0,
    speedOutsideActive: 0,
    autoPause: false,
    autoResume: false,
  },
  proxyConfig: {
    proxyType: 'Direct' as const,
    host: '',
    port: 0,
    username: undefined,
    password: undefined,
  },
  retryConfig: {
    maxRetries: 3,
    retryDelayMs: 2000,
    fallbackUrls: [],
    autoRetry: true,
    skipFailedSegments: false,
  },
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

import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { DomainSettings } from "../types";

const DEFAULT_DOMAINS: DomainSettings = {
  titanDomain: "51cg1.com",
  baanjeenDomain: "xn--82c7abb4jua0l.com",
  rongyokDomain: "rongyok.com",
  groupByDomain: false,
};

export function useDomainSettings() {
  const [domainSettings, setDomainSettings] =
    useState<DomainSettings>(DEFAULT_DOMAINS);
  const [isLoaded, setIsLoaded] = useState(false);
  const [isSaving, setIsSaving] = useState(false);

  const loadSettings = useCallback(async () => {
    try {
      const settings = await invoke<DomainSettings>("get_domain_settings");
      setDomainSettings(settings);
    } catch (e) {
      console.error("Failed to load domain settings:", e);
    } finally {
      setIsLoaded(true);
    }
  }, []);

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  const updateDomainSetting = async <K extends keyof DomainSettings>(
    key: K,
    value: DomainSettings[K],
  ) => {
    const newSettings = { ...domainSettings, [key]: value };
    setDomainSettings(newSettings);

    setIsSaving(true);
    try {
      await invoke("save_domain_settings", { settings: newSettings });
    } catch (e) {
      console.error("Failed to save domain settings:", e);
    } finally {
      setIsSaving(false);
    }
  };

  const resetDomainSettings = async () => {
    setDomainSettings(DEFAULT_DOMAINS);
    setIsSaving(true);
    try {
      await invoke("save_domain_settings", { settings: DEFAULT_DOMAINS });
    } catch (e) {
      console.error("Failed to reset domain settings:", e);
    } finally {
      setIsSaving(false);
    }
  };

  return {
    domainSettings,
    updateDomainSetting,
    resetDomainSettings,
    isLoaded,
    isSaving,
  };
}

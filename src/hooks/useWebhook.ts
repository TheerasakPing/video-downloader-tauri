import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface WebhookConfig {
  enabled: boolean;
  url: string;
  webhookType: string;
  secret?: string;
  events: string[];
}

export interface UseWebhookReturn {
  config: WebhookConfig | null;
  loading: boolean;
  error: string | null;
  loadConfig: () => Promise<void>;
  saveConfig: (config: WebhookConfig) => Promise<void>;
  testWebhook: () => Promise<string>;
  checkNewEpisodes: () => Promise<string[]>;
}

export function useWebhook(): UseWebhookReturn {
  const [config, setConfig] = useState<WebhookConfig | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadConfig = async () => {
    setLoading(true);
    setError(null);
    try {
      const cfg = await invoke<WebhookConfig>('cmd_get_webhook_config');
      setConfig(cfg);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const saveConfig = async (newConfig: WebhookConfig) => {
    setLoading(true);
    setError(null);
    try {
      await invoke('cmd_save_webhook_config', { config: newConfig });
      setConfig(newConfig);
    } catch (e) {
      setError(String(e));
      throw e;
    } finally {
      setLoading(false);
    }
  };

  const testWebhook = async () => {
    setError(null);
    try {
      const result = await invoke<string>('cmd_test_webhook');
      return result;
    } catch (e) {
      setError(String(e));
      throw e;
    }
  };

  const checkNewEpisodes = async () => {
    setError(null);
    try {
      const results = await invoke<string[]>('cmd_check_new_episodes');
      return results;
    } catch (e) {
      setError(String(e));
      throw e;
    }
  };

  useEffect(() => {
    loadConfig();
  }, []);

  return {
    config,
    loading,
    error,
    loadConfig,
    saveConfig,
    testWebhook,
    checkNewEpisodes,
  };
}

import React, { useState } from 'react';
import { Bell, TestTube, Check, X, Loader2 } from 'lucide-react';
import { useWebhook, WebhookConfig } from '../hooks/useWebhook';
import { useI18n } from '../hooks/useI18n';

interface WebhookSettingsProps {
  onClose?: () => void;
}

export function WebhookSettings({ onClose }: WebhookSettingsProps) {
  const { config, loading, error, saveConfig, testWebhook, checkNewEpisodes } = useWebhook();
  const [localConfig, setLocalConfig] = useState<WebhookConfig | null>(null);
  const [testResult, setTestResult] = useState<{ success: boolean; message: string } | null>(null);
  const [testing, setTesting] = useState(false);
  const [checking, setChecking] = useState(false);
  const { t } = useI18n();

  React.useEffect(() => {
    if (config && !localConfig) {
      setLocalConfig(config);
    }
  }, [config, localConfig]);

  const handleSave = async () => {
    if (localConfig) {
      try {
        await saveConfig(localConfig);
        setTestResult({ success: true, message: t('webhook.saved') });
      } catch (e) {
        setTestResult({ success: false, message: String(e) });
      }
    }
  };

  const handleTest = async () => {
    setTesting(true);
    setTestResult(null);
    try {
      const result = await testWebhook();
      setTestResult({ success: true, message: result });
    } catch (e) {
      setTestResult({ success: false, message: String(e) });
    } finally {
      setTesting(false);
    }
  };

  const handleCheckEpisodes = async () => {
    setChecking(true);
    setTestResult(null);
    try {
      const results = await checkNewEpisodes();
      if (results.length > 0) {
        setTestResult({
          success: true,
          message: t('webhook.foundNewEpisodes', { count: results.length, series: results.join('\n') }),
        });
      } else {
        setTestResult({
          success: true,
          message: t('webhook.noNewEpisodes'),
        });
      }
    } catch (e) {
      setTestResult({ success: false, message: String(e) });
    } finally {
      setChecking(false);
    }
  };

  const webhookTypes = [
    { value: 'discord', labelKey: 'webhook.discord', icon: '💬' },
    { value: 'line', labelKey: 'webhook.lineNotify', icon: '💚' },
    { value: 'custom', labelKey: 'webhook.customWebhook', icon: '🔗' },
  ];

  const eventTypes = [
    { value: 'download_complete', labelKey: 'webhook.eventDownloadComplete' },
    { value: 'download_failed', labelKey: 'webhook.eventDownloadFailed' },
    { value: 'new_episode', labelKey: 'webhook.eventNewEpisode' },
  ];

  if (!localConfig) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loader2 className="w-8 h-8 animate-spin text-blue-500" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-blue-500/10 rounded-lg">
            <Bell className="w-5 h-5 text-blue-500" />
          </div>
          <div>
            <h2 className="text-xl font-semibold">{t("webhook.title")}</h2>
            <p className="text-sm text-gray-500">{t("webhook.subtitle")}</p>
          </div>
        </div>
        {onClose && (
          <button
            onClick={onClose}
            className="p-2 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        )}
      </div>

      {/* Error Display */}
      {error && (
        <div className="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
          <p className="text-sm text-red-600 dark:text-red-400">{error}</p>
        </div>
      )}

      {/* Test Result */}
      {testResult && (
        <div
          className={`p-4 rounded-lg border ${
            testResult.success
              ? 'bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800'
              : 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800'
          }`}
        >
          <div className="flex items-start gap-3">
            {testResult.success ? (
              <Check className="w-5 h-5 text-green-600 dark:text-green-400 flex-shrink-0 mt-0.5" />
            ) : (
              <X className="w-5 h-5 text-red-600 dark:text-red-400 flex-shrink-0 mt-0.5" />
            )}
            <p className="text-sm whitespace-pre-wrap">{testResult.message}</p>
          </div>
        </div>
      )}

      {/* Settings Form */}
      <div className="space-y-4">
        {/* Enable/Disable */}
        <div className="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
          <div>
            <label className="font-medium">{t("webhook.enable")}</label>
            <p className="text-sm text-gray-500">{t("webhook.enableDesc")}</p>
          </div>
          <button
            onClick={() => setLocalConfig({ ...localConfig, enabled: !localConfig.enabled })}
            className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
              localConfig.enabled ? 'bg-blue-500' : 'bg-gray-300 dark:bg-gray-600'
            }`}
          >
            <span
              className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                localConfig.enabled ? 'translate-x-6' : 'translate-x-1'
              }`}
            />
          </button>
        </div>

        {/* Webhook Type */}
        <div>
          <label className="block text-sm font-medium mb-2">{t("webhook.type")}</label>
          <div className="grid grid-cols-3 gap-3">
            {webhookTypes.map((type) => (
              <button
                key={type.value}
                onClick={() => setLocalConfig({ ...localConfig, webhookType: type.value })}
                className={`p-4 rounded-lg border-2 text-center transition-all ${
                  localConfig.webhookType === type.value
                    ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                    : 'border-gray-200 dark:border-gray-700 hover:border-gray-300'
                }`}
              >
                <span className="text-2xl mb-1 block">{type.icon}</span>
                <span className="text-sm font-medium">{t(type.labelKey)}</span>
              </button>
            ))}
          </div>
        </div>

        {/* Webhook URL */}
        <div>
          <label className="block text-sm font-medium mb-2">{t("webhook.url")}</label>
          <input
            type="text"
            value={localConfig.url}
            onChange={(e) => setLocalConfig({ ...localConfig, url: e.target.value })}
            placeholder={
              localConfig.webhookType === 'discord'
                ? t('webhook.discordUrl')
                : localConfig.webhookType === 'line'
                ? t('webhook.lineNoUrl')
                : t('webhook.urlPlaceholder')
            }
            className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent dark:bg-gray-700"
          />
          {localConfig.webhookType === 'line' && (
            <p className="text-xs text-gray-500 mt-1">
              {t("webhook.lineNoUrlHint")}
            </p>
          )}
        </div>

        {/* Secret (for LINE or custom auth) */}
        {(localConfig.webhookType === 'line' || localConfig.webhookType === 'custom') && (
          <div>
            <label className="block text-sm font-medium mb-2">
              {localConfig.webhookType === 'line' ? t("webhook.lineToken") : t("webhook.secret")}
            </label>
            <input
              type="password"
              value={localConfig.secret || ''}
              onChange={(e) => setLocalConfig({ ...localConfig, secret: e.target.value })}
              placeholder={
                localConfig.webhookType === 'line'
                  ? t('webhook.lineTokenPlaceholder')
                  : t('webhook.bearerPlaceholder')
              }
              className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent dark:bg-gray-700"
            />
          </div>
        )}

        {/* Event Types */}
        <div>
          <label className="block text-sm font-medium mb-2">{t("webhook.events")}</label>
          <div className="space-y-2">
            {eventTypes.map((event) => (
              <label
                key={event.value}
                className="flex items-center gap-3 p-3 bg-gray-50 dark:bg-gray-800 rounded-lg cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-750 transition-colors"
              >
                <input
                  type="checkbox"
                  checked={localConfig.events.includes(event.value)}
                  onChange={(e) => {
                    if (e.target.checked) {
                      setLocalConfig({ ...localConfig, events: [...localConfig.events, event.value] });
                    } else {
                      setLocalConfig({
                        ...localConfig,
                        events: localConfig.events.filter((ev) => ev !== event.value),
                      });
                    }
                  }}
                  className="w-4 h-4 text-blue-500 rounded focus:ring-2 focus:ring-blue-500"
                />
                <span className="flex-1">{t(event.labelKey)}</span>
              </label>
            ))}
          </div>
        </div>
      </div>

      {/* Action Buttons */}
      <div className="flex gap-3 pt-4 border-t border-gray-200 dark:border-gray-700">
        <button
          onClick={handleSave}
          disabled={loading}
          className="flex-1 px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
        >
          {loading ? <Loader2 className="w-4 h-4 animate-spin" /> : <Check className="w-4 h-4" />}
          {t("webhook.save")}
        </button>
        <button
          onClick={handleTest}
          disabled={testing || !localConfig.enabled}
          className="px-4 py-2 bg-gray-100 hover:bg-gray-200 dark:bg-gray-700 dark:hover:bg-gray-600 rounded-lg font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
        >
          {testing ? <Loader2 className="w-4 h-4 animate-spin" /> : <TestTube className="w-4 h-4" />}
          {t("webhook.test")}
        </button>
        <button
          onClick={handleCheckEpisodes}
          disabled={checking || !localConfig.enabled}
          className="px-4 py-2 bg-gray-100 hover:bg-gray-200 dark:bg-gray-700 dark:hover:bg-gray-600 rounded-lg font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
        >
          {checking ? <Loader2 className="w-4 h-4 animate-spin" /> : <Bell className="w-4 h-4" />}
          {t("webhook.checkEpisodes")}
        </button>
      </div>

      {/* Help Text */}
      <div className="p-4 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
        <h3 className="font-medium text-blue-900 dark:text-blue-100 mb-2">{t("webhook.setupInstructions")}</h3>
        <ul className="text-sm text-blue-800 dark:text-blue-200 space-y-1">
          {localConfig.webhookType === 'discord' && (
            <>
              <li>{t("webhook.discordSetup1")}</li>
              <li>{t("webhook.discordSetup2")}</li>
              <li>{t("webhook.discordSetup3")}</li>
            </>
          )}
          {localConfig.webhookType === 'line' && (
            <>
              <li>{t("webhook.lineSetup1")}</li>
              <li>{t("webhook.lineSetup2")}</li>
              <li>{t("webhook.lineSetup3")}</li>
            </>
          )}
          {localConfig.webhookType === 'custom' && (
            <>
              <li>{t("webhook.customSetup1")}</li>
              <li>{t("webhook.customSetup2")}</li>
              <li>{t("webhook.customSetup3")}</li>
            </>
          )}
        </ul>
      </div>
    </div>
  );
}

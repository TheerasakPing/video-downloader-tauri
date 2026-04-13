import {
  Moon,
  Sun,
  Monitor,
  Download,
  Bell,
  Volume2,
  Trash2,
  FolderOpen,
  Gauge,
  RotateCcw,
  RefreshCw,
  Sparkles,
  Languages,
  Globe,
} from "lucide-react";
import { Settings as SettingsType } from "../hooks/useSettings";
import { DomainSettings } from "../types";
import { CustomTheme } from "../hooks/useCustomTheme";
import { Language } from "../hooks/useI18n";
import { Button } from "./Button";
import { ThemeSelector } from "./ThemeSelector";
import { WebhookSettings } from "./WebhookSettings";
import { useState } from "react";

interface SettingsPanelProps {
  settings: SettingsType;
  onUpdate: <K extends keyof SettingsType>(
    key: K,
    value: SettingsType[K],
  ) => void;
  onReset: () => void;
  onOpenFolder: () => void;
  onCheckUpdates?: () => void;
  isCheckingUpdates?: boolean;
  // i18n
  language: Language;
  onLanguageChange: (lang: Language) => void;
  t: (key: string) => string;
  // Custom themes
  themes: CustomTheme[];
  activeThemeId: string;
  onThemeSelect: (themeId: string) => void;
  // Domains
  domainSettings: DomainSettings;
  onUpdateDomain: <K extends keyof DomainSettings>(
    key: K,
    value: DomainSettings[K],
  ) => void;
  onResetDomains: () => void;
}

export function SettingsPanel({
  settings,
  onUpdate,
  onReset,
  onOpenFolder,
  onCheckUpdates,
  isCheckingUpdates,
  language,
  onLanguageChange,
  t,
  themes,
  activeThemeId,
  onThemeSelect,
  domainSettings,
  onUpdateDomain,
  onResetDomains,
}: SettingsPanelProps) {
  const [showWebhookSettings, setShowWebhookSettings] = useState(false);

  return (
    <div className="space-y-6">
      {/* Download Settings */}
      <section className="bg-slate-800/50 rounded-xl p-4 border border-slate-700">
        <h3 className="text-sm font-medium text-slate-300 mb-4 flex items-center gap-2">
          <span className="icon-glow icon-glow-sm icon-glow-violet">
            <Download size={16} />
          </span>
          {t("settings.downloadSettings")}
        </h3>

        <div className="space-y-4">
          {/* Concurrent Downloads */}
          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm text-white">{t("settings.concurrentDownloads")}</label>
              <p className="text-xs text-slate-500">
                {t("settings.concurrentDesc")}
              </p>
            </div>
            <select
              value={settings.concurrentDownloads}
              onChange={(e) =>
                onUpdate("concurrentDownloads", parseInt(e.target.value))
              }
              className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white"
            >
              {[1, 2, 3, 4, 5].map((n) => (
                <option key={n} value={n}>
                  {n}
                </option>
              ))}
            </select>
          </div>

          {/* Speed Limit */}
          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm text-white flex items-center gap-2">
                <span className="icon-glow icon-glow-sm icon-glow-cyan">
                  <Gauge size={14} />
                </span>
                {t("settings.speedLimit")}
              </label>
              <p className="text-xs text-slate-500">{t("settings.speedLimitDesc")}</p>
            </div>
            <div className="flex items-center gap-2">
              <input
                type="number"
                min="0"
                step="100"
                value={settings.speedLimit}
                onChange={(e) =>
                  onUpdate("speedLimit", parseInt(e.target.value) || 0)
                }
                className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white w-24"
              />
              <span className="text-xs text-slate-500">{t("settings.kbs")}</span>
            </div>
          </div>

          {/* File Naming */}
          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm text-white">{t("settings.fileNaming")}</label>
              <p className="text-xs text-slate-500">{t("settings.fileNamingDesc")}</p>
            </div>
            <select
              value={settings.fileNaming}
              onChange={(e) =>
                onUpdate(
                  "fileNaming",
                  e.target.value as SettingsType["fileNaming"],
                )
              }
              className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white"
            >
              <option value="ep_001">ep_001.mp4</option>
              <option value="episode_1">episode_1.mp4</option>
              <option value="title_ep1">Title_EP1.mp4</option>
            </select>
          </div>

          {/* Auto Merge */}
          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm text-white">{t("settings.autoMerge")}</label>
              <p className="text-xs text-slate-500">
                {t("settings.autoMergeDesc")}
              </p>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={settings.autoMerge}
                onChange={(e) => onUpdate("autoMerge", e.target.checked)}
                className="sr-only peer"
              />
              <div className="w-11 h-6 bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-violet-600"></div>
            </label>
          </div>

          {/* Group by Source */}
          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm text-white flex items-center gap-2">
                <span className="icon-glow icon-glow-sm icon-glow-emerald">
                  <FolderOpen size={14} />
                </span>
                {t("settings.groupBySource")}
              </label>
              <p className="text-xs text-slate-500">
                {t("settings.groupBySourceDesc")}
              </p>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={settings.groupBySource}
                onChange={(e) => onUpdate("groupBySource", e.target.checked)}
                className="sr-only peer"
              />
              <div className="w-11 h-6 bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-emerald-600"></div>
            </label>
          </div>

          {/* Delete After Merge */}
          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm text-white flex items-center gap-2">
                <span className="icon-glow icon-glow-sm icon-glow-red">
                  <Trash2 size={14} />
                </span>
                {t("settings.deleteAfterMerge")}
              </label>
              <p className="text-xs text-slate-500">
                {t("settings.deleteAfterMergeDesc")}
              </p>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={settings.deleteAfterMerge}
                onChange={(e) => onUpdate("deleteAfterMerge", e.target.checked)}
                className="sr-only peer"
              />
              <div className="w-11 h-6 bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-violet-600"></div>
            </label>
          </div>
        </div>
      </section>

      {/* Domain Server Configuration */}
      <section className="bg-slate-800/50 rounded-xl p-4 border border-slate-700">
        <h3 className="text-sm font-medium text-slate-300 mb-4 flex items-center gap-2">
          <span className="icon-glow icon-glow-sm icon-glow-blue">
            <Globe size={16} />
          </span>
          {t("settings.serverDomains")}
        </h3>

        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm text-white">{t("settings.titanServer")}</label>
              <p className="text-xs text-slate-500">
                {t("settings.titanServerDesc")}
              </p>
            </div>
            <input
              type="text"
              value={domainSettings.titanDomain}
              onChange={(e) => onUpdateDomain("titanDomain", e.target.value)}
              className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white w-64 text-right"
              placeholder="51cg1.com, 51cm.com"
            />
          </div>

          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm text-white">{t("settings.baanjeenServer")}</label>
              <p className="text-xs text-slate-500">
                {t("settings.baanjeenServerDesc")}
              </p>
            </div>
            <input
              type="text"
              value={domainSettings.baanjeenDomain}
              onChange={(e) => onUpdateDomain("baanjeenDomain", e.target.value)}
              className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white w-48 text-right"
              placeholder="xn--82c7abb4jua0l.com"
            />
          </div>

          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm text-white">{t("settings.rongyokServer")}</label>
              <p className="text-xs text-slate-500">
                {t("settings.rongyokServerDesc")}
              </p>
            </div>
            <input
              type="text"
              value={domainSettings.rongyokDomain}
              onChange={(e) => onUpdateDomain("rongyokDomain", e.target.value)}
              className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white w-48 text-right"
              placeholder="rongyok.com"
            />
          </div>

          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm text-white">{t("settings.hsckServer")}</label>
              <p className="text-xs text-slate-500">
                {t("settings.hsckServerDesc")}
              </p>
            </div>
            <input
              type="text"
              value={domainSettings.hsckDomain ?? "hsck123.com"}
              onChange={(e) => onUpdateDomain("hsckDomain", e.target.value)}
              className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white w-48 text-right"
              placeholder="hsck123.com"
            />
          </div>

          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm text-white">{t("settings.njavtvServer")}</label>
              <p className="text-xs text-slate-500">
                {t("settings.njavtvServerDesc")}
              </p>
            </div>
            <input
              type="text"
              value={domainSettings.njavtvDomain ?? "njavtv.com"}
              onChange={(e) => onUpdateDomain("njavtvDomain", e.target.value)}
              className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white w-48 text-right"
              placeholder="njavtv.com"
            />
          </div>

          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm text-white">{t("settings.avkuyServer")}</label>
              <p className="text-xs text-slate-500">
                {t("settings.avkuyServerDesc")}
              </p>
            </div>
            <input
              type="text"
              value={domainSettings.avkuyDomain ?? "www2.avkuy.com"}
              onChange={(e) => onUpdateDomain("avkuyDomain", e.target.value)}
              className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white w-48 text-right"
              placeholder="www2.avkuy.com"
            />
          </div>
        </div>
      </section>

      {/* Notifications */}
      <section className="bg-slate-800/50 rounded-xl p-4 border border-slate-700">
        <h3 className="text-sm font-medium text-slate-300 mb-4 flex items-center gap-2">
          <span className="icon-glow icon-glow-sm icon-glow-amber">
            <Bell size={16} />
          </span>
          {t("settings.notifications")}
        </h3>

        <div className="space-y-4">
          {/* System Notifications */}
          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm text-white">{t("settings.systemNotifications")}</label>
              <p className="text-xs text-slate-500">
                {t("settings.systemNotificationsDesc")}
              </p>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={settings.notificationsEnabled}
                onChange={(e) =>
                  onUpdate("notificationsEnabled", e.target.checked)
                }
                className="sr-only peer"
              />
              <div className="w-11 h-6 bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-violet-600"></div>
            </label>
          </div>

          {/* Sound */}
          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm text-white flex items-center gap-2">
                <span className="icon-glow icon-glow-sm icon-glow-blue">
                  <Volume2 size={14} />
                </span>
                {t("settings.soundAlert")}
              </label>
              <p className="text-xs text-slate-500">{t("settings.soundDesc")}</p>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={settings.soundEnabled}
                onChange={(e) => onUpdate("soundEnabled", e.target.checked)}
                className="sr-only peer"
              />
              <div className="w-11 h-6 bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-violet-600"></div>
            </label>
          </div>
        </div>
      </section>

      {/* Appearance */}
      <section className="bg-slate-800/50 rounded-xl p-4 border border-slate-700">
        <h3 className="text-sm font-medium text-slate-300 mb-4 flex items-center gap-2">
          <span className="icon-glow icon-glow-sm icon-glow-amber">
            <Sun size={16} />
          </span>
          {t("settings.appearance")}
        </h3>

        <div className="flex gap-2">
          <button
            onClick={() => onUpdate("theme", "light")}
            className={`flex-1 flex items-center justify-center gap-2 px-4 py-3 rounded-lg transition-all ${
              settings.theme === "light"
                ? "bg-violet-600 text-white tab-glow-active"
                : "bg-slate-700 text-slate-400 hover:bg-slate-600"
            }`}
            aria-label={t("settings.lightThemeAria")}
          >
            <span
              className={`icon-glow icon-glow-sm ${settings.theme === "light" ? "icon-glow-amber icon-glow-animated" : ""}`}
            >
              <Sun size={18} />
            </span>
            {t("settings.light")}
          </button>
          <button
            onClick={() => onUpdate("theme", "dark")}
            className={`flex-1 flex items-center justify-center gap-2 px-4 py-3 rounded-lg transition-all ${
              settings.theme === "dark"
                ? "bg-violet-600 text-white tab-glow-active"
                : "bg-slate-700 text-slate-400 hover:bg-slate-600"
            }`}
            aria-label={t("settings.darkThemeAria")}
          >
            <span
              className={`icon-glow icon-glow-sm ${settings.theme === "dark" ? "icon-glow-violet icon-glow-animated" : ""}`}
            >
              <Moon size={18} />
            </span>
            {t("settings.dark")}
          </button>
          <button
            onClick={() => onUpdate("theme", "system")}
            className={`flex-1 flex items-center justify-center gap-2 px-4 py-3 rounded-lg transition-all ${
              settings.theme === "system"
                ? "bg-violet-600 text-white tab-glow-active"
                : "bg-slate-700 text-slate-400 hover:bg-slate-600"
            }`}
            aria-label={t("settings.systemThemeAria")}
          >
            <span
              className={`icon-glow icon-glow-sm ${settings.theme === "system" ? "icon-glow-cyan icon-glow-animated" : ""}`}
            >
              <Monitor size={18} />
            </span>
            {t("settings.system")}
          </button>
        </div>
      </section>

      {/* Language */}
      <section className="bg-slate-800/50 rounded-xl p-4 border border-slate-700">
        <h3 className="text-sm font-medium text-slate-300 mb-4 flex items-center gap-2">
          <span className="icon-glow icon-glow-sm icon-glow-emerald">
            <Languages size={16} />
          </span>
          {t("settings.language")}
        </h3>

        <div className="flex gap-2">
          <button
            onClick={() => onLanguageChange("en")}
            className={`flex-1 flex items-center justify-center gap-2 px-4 py-3 rounded-lg transition-all ${
              language === "en"
                ? "bg-violet-600 text-white tab-glow-active"
                : "bg-slate-700 text-slate-400 hover:bg-slate-600"
            }`}
            aria-label={t("settings.englishLangAria")}
          >
            <span className="text-lg">🇬🇧</span>
            English
          </button>
          <button
            onClick={() => onLanguageChange("th")}
            className={`flex-1 flex items-center justify-center gap-2 px-4 py-3 rounded-lg transition-all ${
              language === "th"
                ? "bg-violet-600 text-white tab-glow-active"
                : "bg-slate-700 text-slate-400 hover:bg-slate-600"
            }`}
            aria-label={t("settings.thaiLangAria")}
          >
            <span className="text-lg">🇹🇭</span>
            ไทย
          </button>
        </div>
      </section>

      {/* Color Themes */}
      <section className="bg-slate-800/50 rounded-xl p-4 border border-slate-700">
        <ThemeSelector
          themes={themes}
          activeThemeId={activeThemeId}
          onSelect={onThemeSelect}
        />
      </section>

      {/* Download Schedule */}
      <section className="bg-slate-800/50 rounded-xl p-4 border border-slate-700">
        <h3 className="text-sm font-medium text-slate-300 mb-4 flex items-center gap-2">
          <span className="icon-glow icon-glow-sm icon-glow-cyan">
            <Gauge size={16} />
          </span>
          {t("settings.downloadSchedule")}
        </h3>

        <div className="space-y-4">
          {/* Enable Scheduling */}
          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm text-white">{t("settings.enableScheduling")}</label>
              <p className="text-xs text-slate-500">
                {t("settings.enableScheduleDesc")}
              </p>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={settings.scheduleConfig.enabled}
                onChange={(e) =>
                  onUpdate("scheduleConfig", {
                    ...settings.scheduleConfig,
                    enabled: e.target.checked,
                  })
                }
                className="sr-only peer"
              />
              <div className="w-11 h-6 bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-cyan-600"></div>
            </label>
          </div>

          {settings.scheduleConfig.enabled && (
            <div className="grid grid-cols-2 gap-3 pl-2">
              <div>
                <label className="text-xs text-slate-500">{t("settings.activeStart")}</label>
                <input
                  type="time"
                  value={settings.scheduleConfig.activeStart || ""}
                  onChange={(e) =>
                    onUpdate("scheduleConfig", {
                      ...settings.scheduleConfig,
                      activeStart: e.target.value || undefined,
                    })
                  }
                  className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white"
                />
              </div>
              <div>
                <label className="text-xs text-slate-500">{t("settings.activeEnd")}</label>
                <input
                  type="time"
                  value={settings.scheduleConfig.activeEnd || ""}
                  onChange={(e) =>
                    onUpdate("scheduleConfig", {
                      ...settings.scheduleConfig,
                      activeEnd: e.target.value || undefined,
                    })
                  }
                  className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white"
                />
              </div>
              <div>
                <label className="text-xs text-slate-500">
                  {t("settings.speedDuringActive")}
                </label>
                <input
                  type="number"
                  min="0"
                  step="100"
                  value={settings.scheduleConfig.speedDuringActive}
                  onChange={(e) =>
                    onUpdate("scheduleConfig", {
                      ...settings.scheduleConfig,
                      speedDuringActive: parseInt(e.target.value) || 0,
                    })
                  }
                  className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white"
                />
              </div>
              <div>
                <label className="text-xs text-slate-500">
                  {t("settings.speedOutsideActive")}
                </label>
                <input
                  type="number"
                  min="0"
                  step="100"
                  value={settings.scheduleConfig.speedOutsideActive}
                  onChange={(e) =>
                    onUpdate("scheduleConfig", {
                      ...settings.scheduleConfig,
                      speedOutsideActive: parseInt(e.target.value) || 0,
                    })
                  }
                  className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white"
                />
              </div>
            </div>
          )}

          {settings.scheduleConfig.enabled && (
            <div className="flex gap-6 pl-2">
              {/* Auto Pause */}
              <div className="flex items-center justify-between gap-2">
                <div>
                  <label className="text-sm text-white">{t("settings.autoPause")}</label>
                  <p className="text-xs text-slate-500">{t("settings.autoPauseDesc")}</p>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input
                    type="checkbox"
                    checked={settings.scheduleConfig.autoPause}
                    onChange={(e) =>
                      onUpdate("scheduleConfig", {
                        ...settings.scheduleConfig,
                        autoPause: e.target.checked,
                      })
                    }
                    className="sr-only peer"
                  />
                  <div className="w-11 h-6 bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-violet-600"></div>
                </label>
              </div>

              {/* Auto Resume */}
              <div className="flex items-center justify-between gap-2">
                <div>
                  <label className="text-sm text-white">{t("settings.autoResume")}</label>
                  <p className="text-xs text-slate-500">{t("settings.autoResumeDesc")}</p>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input
                    type="checkbox"
                    checked={settings.scheduleConfig.autoResume}
                    onChange={(e) =>
                      onUpdate("scheduleConfig", {
                        ...settings.scheduleConfig,
                        autoResume: e.target.checked,
                      })
                    }
                    className="sr-only peer"
                  />
                  <div className="w-11 h-6 bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-emerald-600"></div>
                </label>
              </div>
            </div>
          )}
        </div>
      </section>

      {/* Proxy Configuration */}
      <section className="bg-slate-800/50 rounded-xl p-4 border border-slate-700">
        <h3 className="text-sm font-medium text-slate-300 mb-4 flex items-center gap-2">
          <span className="icon-glow icon-glow-sm icon-glow-blue">
            <Globe size={16} />
          </span>
          {t("settings.proxyConfiguration")}
        </h3>

        <div className="space-y-4">
          {/* Proxy Type */}
          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm text-white">{t("settings.proxyType")}</label>
              <p className="text-xs text-slate-500">{t("settings.proxyTypeDesc")}</p>
            </div>
            <select
              value={settings.proxyConfig.proxyType}
              onChange={(e) =>
                onUpdate("proxyConfig", {
                  ...settings.proxyConfig,
                  proxyType: e.target.value as "Direct" | "Http" | "Socks5",
                })
              }
              className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white"
            >
              <option value="Direct">{t("settings.noProxy")}</option>
              <option value="Http">{t("settings.httpProxy")}</option>
              <option value="Socks5">{t("settings.socks5Proxy")}</option>
            </select>
          </div>

          {settings.proxyConfig.proxyType !== "Direct" && (
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="text-xs text-slate-500">{t("settings.proxyHost")}</label>
                <input
                  type="text"
                  value={settings.proxyConfig.host}
                  onChange={(e) =>
                    onUpdate("proxyConfig", {
                      ...settings.proxyConfig,
                      host: e.target.value,
                    })
                  }
                  placeholder="127.0.0.1"
                  className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white"
                />
              </div>
              <div>
                <label className="text-xs text-slate-500">{t("settings.proxyPort")}</label>
                <input
                  type="number"
                  min="0"
                  value={settings.proxyConfig.port}
                  onChange={(e) =>
                    onUpdate("proxyConfig", {
                      ...settings.proxyConfig,
                      port: parseInt(e.target.value) || 0,
                    })
                  }
                  placeholder="8080"
                  className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white"
                />
              </div>
            </div>
          )}
        </div>
      </section>

      {/* Retry & Fallback */}
      <section className="bg-slate-800/50 rounded-xl p-4 border border-slate-700">
        <h3 className="text-sm font-medium text-slate-300 mb-4 flex items-center gap-2">
          <span className="icon-glow icon-glow-sm icon-glow-amber">
            <RotateCcw size={16} />
          </span>
          {t("settings.retryFallback")}
        </h3>

        <div className="space-y-4">
          {/* Max Retries */}
          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm text-white">{t("settings.maxRetries")}</label>
              <p className="text-xs text-slate-500">
                {t("settings.maxRetriesDesc")}
              </p>
            </div>
            <input
              type="number"
              min="0"
              max="10"
              value={settings.retryConfig.maxRetries}
              onChange={(e) =>
                onUpdate("retryConfig", {
                  ...settings.retryConfig,
                  maxRetries: parseInt(e.target.value) || 0,
                })
              }
              className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white w-24"
            />
          </div>

          {/* Retry Delay */}
          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm text-white">{t("settings.retryDelay")}</label>
              <p className="text-xs text-slate-500">
                {t("settings.retryDelayDesc")}
              </p>
            </div>
            <input
              type="number"
              min="100"
              step="500"
              value={settings.retryConfig.retryDelayMs}
              onChange={(e) =>
                onUpdate("retryConfig", {
                  ...settings.retryConfig,
                  retryDelayMs: parseInt(e.target.value) || 1000,
                })
              }
              className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white w-24"
            />
          </div>

          {/* Auto Retry */}
          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm text-white">{t("settings.autoRetry")}</label>
              <p className="text-xs text-slate-500">
                {t("settings.autoRetryDesc")}
              </p>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={settings.retryConfig.autoRetry}
                onChange={(e) =>
                  onUpdate("retryConfig", {
                    ...settings.retryConfig,
                    autoRetry: e.target.checked,
                  })
                }
                className="sr-only peer"
              />
              <div className="w-11 h-6 bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-violet-600"></div>
            </label>
          </div>

          {/* Skip Failed Segments */}
          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm text-white">{t("settings.skipFailed")}</label>
              <p className="text-xs text-slate-500">
                {t("settings.skipFailedDesc")}
              </p>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={settings.retryConfig.skipFailedSegments}
                onChange={(e) =>
                  onUpdate("retryConfig", {
                    ...settings.retryConfig,
                    skipFailedSegments: e.target.checked,
                  })
                }
                className="sr-only peer"
              />
              <div className="w-11 h-6 bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-red-600"></div>
            </label>
          </div>
        </div>
      </section>

      {/* Updates */}
      <section className="bg-slate-800/50 rounded-xl p-4 border border-slate-700">
        <h3 className="text-sm font-medium text-slate-300 mb-4 flex items-center gap-2">
          <span className="icon-glow icon-glow-sm icon-glow-fuchsia icon-glow-animated">
            <Sparkles size={16} />
          </span>
          {t("settings.updates")}
        </h3>

        <div className="flex items-center justify-between">
          <div>
            <label className="text-sm text-white">{t("settings.checkForUpdates")}</label>
            <p className="text-xs text-slate-500">{t("settings.currentVersion")}</p>
          </div>
          {onCheckUpdates && (
            <Button
              variant="secondary"
              size="sm"
              leftIcon={
                <RefreshCw
                  size={14}
                  className={isCheckingUpdates ? "animate-spin" : ""}
                />
              }
              onClick={onCheckUpdates}
              disabled={isCheckingUpdates}
            >
              {isCheckingUpdates ? t("settings.checking") : t("settings.checkNow")}
            </Button>
          )}
        </div>
      </section>

      {/* Webhook Settings */}
      <section className="bg-slate-800/50 rounded-xl p-4 border border-slate-700">
        <h3 className="text-sm font-medium text-slate-300 mb-4 flex items-center gap-2">
          <span className="icon-glow icon-glow-sm icon-glow-violet">
            <Bell size={16} />
          </span>
          {t("settings.webhookNotifications")}
        </h3>

        <div className="space-y-4">
          <p className="text-sm text-slate-400">
            {t("settings.webhooksDesc")}
          </p>

          <Button
            variant="secondary"
            size="sm"
            leftIcon={
              <span className="icon-glow icon-glow-sm icon-glow-blue">
                <Bell size={14} />
              </span>
            }
            onClick={() => setShowWebhookSettings(true)}
            className="btn-glow-blue"
          >
            {t("settings.configureWebhooks")}
          </Button>
        </div>
      </section>

      {/* Webhook Settings Modal */}
      {showWebhookSettings && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-slate-800 rounded-xl shadow-xl max-w-2xl w-full max-h-[90vh] overflow-y-auto border border-slate-700">
            <WebhookSettings onClose={() => setShowWebhookSettings(false)} />
          </div>
        </div>
      )}

      {/* Actions */}
      <div className="flex gap-3">
        <Button
          variant="secondary"
          leftIcon={
            <span className="icon-glow icon-glow-sm icon-glow-blue">
              <FolderOpen size={16} />
            </span>
          }
          onClick={onOpenFolder}
          className="flex-1"
        >
          {t("settings.openOutputFolder")}
        </Button>
        <Button
          variant="danger"
          leftIcon={
            <span className="icon-glow icon-glow-sm icon-glow-red">
              <RotateCcw size={16} />
            </span>
          }
          onClick={() => {
            onReset();
            onResetDomains();
          }}
          className="btn-glow-red"
        >
          {t("settings.resetSettings")}
        </Button>
      </div>
    </div>
  );
}

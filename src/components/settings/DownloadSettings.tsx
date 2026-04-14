import React from "react";
import { Download } from "lucide-react";
import { Settings } from "../../hooks/useSettings";
import { SettingsSection } from "./SettingsSection";
import { SettingsRow } from "./SettingsRow";
import { Toggle } from "./Toggle";
import { useI18n } from "../../hooks/useI18n";

interface Props {
  settings: Settings;
  onUpdate: <K extends keyof Settings>(key: K, value: Settings[K]) => void;
}

export const DownloadSettings: React.FC<Props> = ({ settings, onUpdate }) => {
  const { t } = useI18n();
  return (
    <SettingsSection icon={Download} iconColor="violet" title={t("settings.downloadSettings")}>
      <SettingsRow label={t("settings.concurrentDownloads")} description={t("settings.concurrentDesc")}>
        <select
          value={settings.concurrentDownloads}
          onChange={(e) => onUpdate("concurrentDownloads", parseInt(e.target.value))}
          className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white"
        >
          {[1, 2, 3, 4, 5].map((n) => (
            <option key={n} value={n}>{n}</option>
          ))}
        </select>
      </SettingsRow>

      <SettingsRow label={t("settings.speedLimit")} description={t("settings.speedLimitDesc")}>
        <div className="flex items-center gap-2">
          <input
            type="number" min="0" step="100"
            value={settings.speedLimit}
            onChange={(e) => onUpdate("speedLimit", parseInt(e.target.value) || 0)}
            className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white w-24"
          />
          <span className="text-xs text-slate-500">{t("settings.kbs")}</span>
        </div>
      </SettingsRow>

      <SettingsRow label={t("settings.fileNaming")} description={t("settings.fileNamingDesc")}>
        <select
          value={settings.fileNaming}
          onChange={(e) => onUpdate("fileNaming", e.target.value as Settings["fileNaming"])}
          className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white"
        >
          <option value="ep_001">ep_001.mp4</option>
          <option value="episode_1">episode_1.mp4</option>
          <option value="title_ep1">Title_EP1.mp4</option>
        </select>
      </SettingsRow>

      <SettingsRow label={t("settings.autoMerge")} description={t("settings.autoMergeDesc")}>
        <Toggle checked={settings.autoMerge} onChange={(v) => onUpdate("autoMerge", v)} color="violet" />
      </SettingsRow>

      <SettingsRow label={t("settings.groupBySource")} description={t("settings.groupBySourceDesc")}>
        <Toggle checked={settings.groupBySource} onChange={(v) => onUpdate("groupBySource", v)} color="emerald" />
      </SettingsRow>

      <SettingsRow label={t("settings.deleteAfterMerge")} description={t("settings.deleteAfterMergeDesc")}>
        <Toggle checked={settings.deleteAfterMerge} onChange={(v) => onUpdate("deleteAfterMerge", v)} />
      </SettingsRow>
    </SettingsSection>
  );
};

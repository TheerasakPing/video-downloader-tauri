import React from "react";
import { Globe } from "lucide-react";
import { DomainSettings as DomainSettingsType } from "../../types";
import { SettingsSection } from "./SettingsSection";
import { SettingsRow } from "./SettingsRow";
import { useI18n } from "../../hooks/useI18n";

interface Props {
  domainSettings: DomainSettingsType;
  onUpdateDomain: <K extends keyof DomainSettingsType>(key: K, value: DomainSettingsType[K]) => void;
}

export const DomainSettings: React.FC<Props> = ({ domainSettings, onUpdateDomain }) => {
  const { t } = useI18n();
  return (
    <SettingsSection icon={Globe} iconColor="blue" title={t("settings.serverDomains")}>
      <SettingsRow label={t("settings.titanServer")} description={t("settings.titanServerDesc")}>
        <input
          type="text"
          value={domainSettings.titanDomain}
          onChange={(e) => onUpdateDomain("titanDomain", e.target.value)}
          className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white w-64 text-right"
          placeholder="51cg1.com, 51cm.com"
        />
      </SettingsRow>
      <SettingsRow label={t("settings.baanjeenServer")} description={t("settings.baanjeenServerDesc")}>
        <input
          type="text"
          value={domainSettings.baanjeenDomain}
          onChange={(e) => onUpdateDomain("baanjeenDomain", e.target.value)}
          className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white w-48 text-right"
          placeholder="xn--82c7abb4jua0l.com"
        />
      </SettingsRow>
      <SettingsRow label={t("settings.rongyokServer")} description={t("settings.rongyokServerDesc")}>
        <input
          type="text"
          value={domainSettings.rongyokDomain}
          onChange={(e) => onUpdateDomain("rongyokDomain", e.target.value)}
          className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white w-48 text-right"
          placeholder="rongyok.com"
        />
      </SettingsRow>
      <SettingsRow label={t("settings.hsckServer")} description={t("settings.hsckServerDesc")}>
        <input
          type="text"
          value={domainSettings.hsckDomain ?? "hsck123.com"}
          onChange={(e) => onUpdateDomain("hsckDomain", e.target.value)}
          className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white w-48 text-right"
          placeholder="hsck123.com"
        />
      </SettingsRow>
      <SettingsRow label={t("settings.njavtvServer")} description={t("settings.njavtvServerDesc")}>
        <input
          type="text"
          value={domainSettings.njavtvDomain ?? "njavtv.com"}
          onChange={(e) => onUpdateDomain("njavtvDomain", e.target.value)}
          className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white w-48 text-right"
          placeholder="njavtv.com"
        />
      </SettingsRow>
      <SettingsRow label={t("settings.avkuyServer")} description={t("settings.avkuyServerDesc")}>
        <input
          type="text"
          value={domainSettings.avkuyDomain ?? "www2.avkuy.com"}
          onChange={(e) => onUpdateDomain("avkuyDomain", e.target.value)}
          className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white w-48 text-right"
          placeholder="www2.avkuy.com"
        />
      </SettingsRow>
    </SettingsSection>
  );
};

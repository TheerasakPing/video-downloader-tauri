import React, { ReactNode } from "react";

interface SettingsRowProps {
  label: string;
  description?: string;
  children: ReactNode;
}

export const SettingsRow: React.FC<SettingsRowProps> = ({
  label,
  description,
  children,
}) => (
  <div className="flex items-center justify-between">
    <div className="min-w-0 mr-4">
      <label className="text-sm text-white">{label}</label>
      {description && <p className="text-xs text-slate-500">{description}</p>}
    </div>
    <div className="flex-shrink-0">{children}</div>
  </div>
);

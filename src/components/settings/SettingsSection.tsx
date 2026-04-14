import React, { ReactNode } from "react";
import { LucideIcon } from "lucide-react";

interface SettingsSectionProps {
  icon: LucideIcon;
  iconColor?: string;
  title: string;
  children: ReactNode;
  className?: string;
}

export const SettingsSection: React.FC<SettingsSectionProps> = ({
  icon: Icon,
  iconColor = "violet",
  title,
  children,
  className = "",
}) => (
  <section
    className={`rounded-xl p-4 border ${className}`}
    style={{
      backgroundColor: "var(--card, rgba(30, 41, 59, 0.5))",
      borderColor: "var(--border, #334155)",
    }}
  >
    <h3 className="text-sm font-medium text-slate-300 mb-4 flex items-center gap-2">
      <span className={`icon-glow icon-glow-sm icon-glow-${iconColor}`}>
        <Icon size={16} />
      </span>
      {title}
    </h3>
    <div className="space-y-4">{children}</div>
  </section>
);

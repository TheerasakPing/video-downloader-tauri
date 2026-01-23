import { useState, useEffect } from "react";

export interface CustomTheme {
  id: string;
  name: string;
  colors: {
    primary: string;
    accent: string;
    background: string;
  };
  glowColor: string;
}

export const THEMES: CustomTheme[] = [
  {
    id: "violet",
    name: "Violet",
    colors: {
      primary: "#8b5cf6",
      accent: "#a78bfa",
      background: "#0f172a",
    },
    glowColor: "rgba(139, 92, 246, 0.3)",
  },
  {
    id: "blue",
    name: "Ocean Blue",
    colors: {
      primary: "#3b82f6",
      accent: "#60a5fa",
      background: "#0f172a",
    },
    glowColor: "rgba(59, 130, 246, 0.3)",
  },
  {
    id: "emerald",
    name: "Emerald",
    colors: {
      primary: "#10b981",
      accent: "#34d399",
      background: "#064e3b",
    },
    glowColor: "rgba(16, 185, 129, 0.3)",
  },
  {
    id: "amber",
    name: "Amber",
    colors: {
      primary: "#f59e0b",
      accent: "#fbbf24",
      background: "#451a03",
    },
    glowColor: "rgba(245, 158, 11, 0.3)",
  },
  {
    id: "rose",
    name: "Rose",
    colors: {
      primary: "#f43f5e",
      accent: "#fb7185",
      background: "#881337",
    },
    glowColor: "rgba(244, 63, 94, 0.3)",
  },
];

export const useCustomTheme = () => {
  const [activeThemeId, setActiveThemeId] = useState<string>(() => {
    return localStorage.getItem("app-accent-theme") || "violet";
  });

  const setActiveTheme = (id: string) => {
    setActiveThemeId(id);
    localStorage.setItem("app-accent-theme", id);
  };

  useEffect(() => {
    const theme = THEMES.find((t) => t.id === activeThemeId) || THEMES[0];
    const root = document.documentElement;

    // Update CSS variables for accent color
    root.style.setProperty("--accent", theme.colors.primary);
    root.style.setProperty("--accent-glow", theme.glowColor);

    // Update global glow classes logic if needed (handled by index.css usually via classes)
  }, [activeThemeId]);

  return {
    themes: THEMES,
    activeThemeId,
    setActiveTheme,
  };
};

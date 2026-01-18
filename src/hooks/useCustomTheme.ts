import { useState, useCallback, useEffect } from "react";

export interface CustomTheme {
  id: string;
  name: string;
  colors: {
    primary: string;
    secondary: string;
    accent: string;
    background: string;
    surface: string;
    text: string;
    textMuted: string;
    border: string;
    success: string;
    warning: string;
    error: string;
  };
}

const DEFAULT_THEMES: CustomTheme[] = [
  {
    id: "violet",
    name: "Violet (Default)",
    colors: {
      primary: "#8B5CF6",
      secondary: "#A855F7",
      accent: "#D946EF",
      background: "#0f172a",
      surface: "#1e293b",
      text: "#ffffff",
      textMuted: "#94a3b8",
      border: "#334155",
      success: "#10B981",
      warning: "#F59E0B",
      error: "#EF4444",
    },
  },
  {
    id: "ocean",
    name: "Ocean Blue",
    colors: {
      primary: "#0EA5E9",
      secondary: "#06B6D4",
      accent: "#22D3EE",
      background: "#0c1929",
      surface: "#152238",
      text: "#ffffff",
      textMuted: "#7dd3fc",
      border: "#1e3a5f",
      success: "#10B981",
      warning: "#F59E0B",
      error: "#EF4444",
    },
  },
  {
    id: "emerald",
    name: "Emerald",
    colors: {
      primary: "#10B981",
      secondary: "#059669",
      accent: "#34D399",
      background: "#0f1f1a",
      surface: "#1a2f28",
      text: "#ffffff",
      textMuted: "#6ee7b7",
      border: "#2d4a40",
      success: "#10B981",
      warning: "#F59E0B",
      error: "#EF4444",
    },
  },
  {
    id: "rose",
    name: "Rose",
    colors: {
      primary: "#F43F5E",
      secondary: "#E11D48",
      accent: "#FB7185",
      background: "#1f0f14",
      surface: "#2f1a22",
      text: "#ffffff",
      textMuted: "#fda4af",
      border: "#4a2d38",
      success: "#10B981",
      warning: "#F59E0B",
      error: "#EF4444",
    },
  },
  {
    id: "amber",
    name: "Amber",
    colors: {
      primary: "#F59E0B",
      secondary: "#D97706",
      accent: "#FBBF24",
      background: "#1a1508",
      surface: "#2a2410",
      text: "#ffffff",
      textMuted: "#fcd34d",
      border: "#4a3f1a",
      success: "#10B981",
      warning: "#F59E0B",
      error: "#EF4444",
    },
  },
];

const STORAGE_KEY = "rongyok-custom-themes";
const ACTIVE_THEME_KEY = "rongyok-active-theme";

export function useCustomTheme() {
  const [themes, setThemes] = useState<CustomTheme[]>(() => {
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      return saved ? [...DEFAULT_THEMES, ...JSON.parse(saved)] : DEFAULT_THEMES;
    } catch {
      return DEFAULT_THEMES;
    }
  });

  const [activeThemeId, setActiveThemeId] = useState<string>(() => {
    try {
      return localStorage.getItem(ACTIVE_THEME_KEY) || "violet";
    } catch {
      return "violet";
    }
  });

  // Save custom themes to localStorage
  useEffect(() => {
    const customThemes = themes.filter(
      (t) => !DEFAULT_THEMES.find((d) => d.id === t.id)
    );
    localStorage.setItem(STORAGE_KEY, JSON.stringify(customThemes));
  }, [themes]);

  // Apply theme to CSS variables
  useEffect(() => {
    const theme = themes.find((t) => t.id === activeThemeId);
    if (theme) {
      const root = document.documentElement;
      root.style.setProperty("--color-primary", theme.colors.primary);
      root.style.setProperty("--color-secondary", theme.colors.secondary);
      root.style.setProperty("--color-accent", theme.colors.accent);
      root.style.setProperty("--color-background", theme.colors.background);
      root.style.setProperty("--color-surface", theme.colors.surface);
      root.style.setProperty("--color-text", theme.colors.text);
      root.style.setProperty("--color-text-muted", theme.colors.textMuted);
      root.style.setProperty("--color-border", theme.colors.border);
      root.style.setProperty("--color-success", theme.colors.success);
      root.style.setProperty("--color-warning", theme.colors.warning);
      root.style.setProperty("--color-error", theme.colors.error);

      // Also update accent glow
      root.style.setProperty("--accent", theme.colors.primary);
      root.style.setProperty(
        "--accent-glow",
        `${theme.colors.primary}4D` // 30% opacity
      );

      // Update background gradient
      root.style.setProperty("--bg-primary", theme.colors.background);
      root.style.setProperty("--bg-secondary", theme.colors.surface);

      // Apply background color to body
      document.body.style.background = `linear-gradient(to bottom right, ${theme.colors.background}, ${theme.colors.surface}, ${theme.colors.background})`;

      localStorage.setItem(ACTIVE_THEME_KEY, activeThemeId);
    }
  }, [activeThemeId, themes]);

  const setActiveTheme = useCallback((themeId: string) => {
    setActiveThemeId(themeId);
  }, []);

  const addTheme = useCallback((theme: Omit<CustomTheme, "id">) => {
    const newTheme: CustomTheme = {
      ...theme,
      id: `custom-${Date.now()}`,
    };
    setThemes((prev) => [...prev, newTheme]);
    return newTheme;
  }, []);

  const updateTheme = useCallback((id: string, updates: Partial<CustomTheme>) => {
    setThemes((prev) =>
      prev.map((t) => (t.id === id ? { ...t, ...updates } : t))
    );
  }, []);

  const deleteTheme = useCallback((id: string) => {
    // Don't allow deleting default themes
    if (DEFAULT_THEMES.find((t) => t.id === id)) return;

    setThemes((prev) => prev.filter((t) => t.id !== id));
    if (activeThemeId === id) {
      setActiveThemeId("violet");
    }
  }, [activeThemeId]);

  const getActiveTheme = useCallback(() => {
    return themes.find((t) => t.id === activeThemeId) || themes[0];
  }, [themes, activeThemeId]);

  const resetThemes = useCallback(() => {
    setThemes(DEFAULT_THEMES);
    setActiveThemeId("violet");
  }, []);

  return {
    themes,
    activeThemeId,
    setActiveTheme,
    addTheme,
    updateTheme,
    deleteTheme,
    getActiveTheme,
    resetThemes,
    defaultThemes: DEFAULT_THEMES,
  };
}

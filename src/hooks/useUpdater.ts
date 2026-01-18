import { useState, useEffect, useCallback } from "react";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export interface UpdateInfo {
  version: string;
  currentVersion: string;
  body: string;
  date: string;
}

export interface UpdateState {
  checking: boolean;
  available: boolean;
  downloading: boolean;
  progress: number;
  error: string | null;
  updateInfo: UpdateInfo | null;
}

export function useUpdater() {
  const [state, setState] = useState<UpdateState>({
    checking: false,
    available: false,
    downloading: false,
    progress: 0,
    error: null,
    updateInfo: null,
  });

  const checkForUpdates = useCallback(async () => {
    setState((prev) => ({ ...prev, checking: true, error: null }));

    try {
      const update = await check();

      if (update) {
        setState((prev) => ({
          ...prev,
          checking: false,
          available: true,
          updateInfo: {
            version: update.version,
            currentVersion: update.currentVersion,
            body: update.body || "No release notes available",
            date: update.date || "",
          },
        }));
        return update;
      } else {
        setState((prev) => ({
          ...prev,
          checking: false,
          available: false,
        }));
        return null;
      }
    } catch (error) {
      setState((prev) => ({
        ...prev,
        checking: false,
        error: error instanceof Error ? error.message : "Failed to check for updates",
      }));
      return null;
    }
  }, []);

  const downloadAndInstall = useCallback(async () => {
    setState((prev) => ({ ...prev, downloading: true, progress: 0, error: null }));

    try {
      const update = await check();

      if (!update) {
        setState((prev) => ({ ...prev, downloading: false, error: "No update available" }));
        return;
      }

      let downloaded = 0;
      let contentLength = 0;

      await update.downloadAndInstall((event) => {
        switch (event.event) {
          case "Started":
            contentLength = event.data.contentLength || 0;
            break;
          case "Progress":
            downloaded += event.data.chunkLength;
            const progress = contentLength > 0 ? (downloaded / contentLength) * 100 : 0;
            setState((prev) => ({ ...prev, progress }));
            break;
          case "Finished":
            setState((prev) => ({ ...prev, progress: 100 }));
            break;
        }
      });

      // Relaunch the app
      await relaunch();
    } catch (error) {
      setState((prev) => ({
        ...prev,
        downloading: false,
        error: error instanceof Error ? error.message : "Failed to download update",
      }));
    }
  }, []);

  const dismissUpdate = useCallback(() => {
    setState((prev) => ({ ...prev, available: false, updateInfo: null }));
  }, []);

  // Check for updates on mount (with delay to not block startup)
  useEffect(() => {
    const timer = setTimeout(() => {
      checkForUpdates();
    }, 3000);
    return () => clearTimeout(timer);
  }, [checkForUpdates]);

  return {
    ...state,
    checkForUpdates,
    downloadAndInstall,
    dismissUpdate,
  };
}

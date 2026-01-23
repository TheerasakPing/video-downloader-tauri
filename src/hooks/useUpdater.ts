import { useState, useEffect, useCallback } from "react";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

interface UpdateInfo {
  version: string;
  date?: string;
  body?: string;
}

export const useUpdater = () => {
  const [checking, setChecking] = useState(false);
  const [available, setAvailable] = useState(false);
  const [downloading, setDownloading] = useState(false);
  const [progress, setProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [updateObject, setUpdateObject] = useState<any>(null);

  const checkForUpdates = useCallback(async (silent = false) => {
    setChecking(true);
    setError(null);
    try {
      const update = await check();
      if (update && update.available) {
        setAvailable(true);
        setUpdateInfo({
          version: update.version,
          date: update.date,
          body: update.body,
        });
        setUpdateObject(update);
        return true;
      } else if (!silent) {
        setAvailable(false);
      }
      return false;
    } catch (e) {
      if (!silent) {
        setError(
          e instanceof Error ? e.message : "Failed to check for updates",
        );
      }
      return false;
    } finally {
      setChecking(false);
    }
  }, []);

  const downloadAndInstall = useCallback(async () => {
    if (!updateObject) return;

    setDownloading(true);
    setProgress(0);
    setError(null);

    try {
      let downloaded = 0;
      let total = 0;

      await updateObject.downloadAndInstall((event: any) => {
        switch (event.event) {
          case "Started":
            total = event.data.contentLength || 0;
            break;
          case "Progress":
            downloaded += event.data.chunkLength;
            if (total > 0) {
              setProgress(Math.round((downloaded / total) * 100));
            }
            break;
          case "Finished":
            setProgress(100);
            break;
        }
      });

      // Relaunch application
      await relaunch();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to install update");
      setDownloading(false);
    }
  }, [updateObject]);

  const dismissUpdate = useCallback(() => {
    setAvailable(false);
    setUpdateObject(null);
  }, []);

  // Auto-check on mount (dev mode safe check)
  useEffect(() => {
    // Optional: Auto-check
    // checkForUpdates(true);
  }, []);

  return {
    checking,
    available,
    downloading,
    progress,
    error,
    updateInfo,
    checkForUpdates,
    downloadAndInstall,
    dismissUpdate,
  };
};

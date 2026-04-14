import { useState, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { SeriesInfo, DownloadState, DownloadProgress } from "../types";
import { Settings } from "./useSettings";
import { QueueItem } from "../components/DownloadQueue";

export interface DownloadResult {
  episode: number;
  success: boolean;
  filePath?: string;
  error?: string;
}

interface UseDownloadManagerDeps {
  settings: Settings;
  ffmpegAvailable: boolean;
  selectedQuality: string | null;
  addRecord: (record: Omit<import("../hooks/useHistory").DownloadRecord, "id">) => string;
  updateRecord: (id: string, updates: Partial<import("../hooks/useHistory").DownloadRecord>) => void;
  log: (msg: string) => void;
  success: (msg: string) => void;
  warning: (msg: string) => void;
  error: (msg: string) => void;
  resetSpeedGraph: () => void;
  addDataPoint: (speed: number) => void;
  onFilesRefresh: () => void;
  isBatchMode: boolean;
}

export function useDownloadManager(deps: UseDownloadManagerDeps) {
  const {
    settings, ffmpegAvailable, selectedQuality,
    addRecord, updateRecord,
    log, success, warning, error,
    resetSpeedGraph, addDataPoint, onFilesRefresh,
    isBatchMode,
  } = deps;

  const [downloadState, setDownloadState] = useState<DownloadState>({
    isDownloading: false,
    isPaused: false,
    currentEpisode: 0,
    completedEpisodes: [],
    failedEpisodes: [],
    totalSelected: 0,
  });

  const [progress, setProgress] = useState<DownloadProgress>({
    episode: 0,
    downloaded: 0,
    total: 0,
    speed: 0,
    percentage: 0,
  });

  const [queue, setQueue] = useState<QueueItem[]>([]);

  const currentEpisodeRef = useRef(downloadState.currentEpisode);
  currentEpisodeRef.current = downloadState.currentEpisode;

  const runDownload = useCallback(
    async (targetSeries: SeriesInfo, targetEpisodes: Set<number>): Promise<boolean> => {
      const episodes = Array.from(targetEpisodes).sort((a, b) => a - b);
      log(`Starting download of ${episodes.length} episodes for ${targetSeries.title}`);

      const recordId = addRecord({
        seriesId: targetSeries.seriesId,
        seriesTitle: targetSeries.title,
        episodes,
        completedEpisodes: [],
        failedEpisodes: [],
        startTime: new Date().toISOString(),
        totalSize: 0,
        status: "partial",
      });

      setQueue(
        episodes.map((ep, i) => ({
          id: `${targetSeries.seriesId}-${ep}`,
          seriesId: targetSeries.seriesId,
          seriesTitle: targetSeries.title,
          episode: ep,
          status: i === 0 ? "downloading" : "pending",
          progress: 0,
          priority: i,
        })),
      );

      setDownloadState({
        isDownloading: true,
        isPaused: false,
        currentEpisode: 0,
        completedEpisodes: [],
        failedEpisodes: [],
        totalSelected: episodes.length,
      });

      resetSpeedGraph();

      try {
        await invoke("update_series_state", { series: targetSeries });

        const results = await invoke<DownloadResult[]>("start_download", {
          request: {
            seriesId: targetSeries.seriesId,
            episodes,
            outputDir: settings.outputDir,
            autoMerge: settings.autoMerge && ffmpegAvailable,
            concurrentDownloads: settings.concurrentDownloads,
            speedLimit: settings.speedLimit,
            fileNaming: settings.fileNaming,
            seriesTitle: targetSeries.title,
            groupBySource: settings.groupBySource,
            preferredQuality: selectedQuality,
          },
        });

        const successCount = results.filter((r) => r.success).length;
        const failCount = results.filter((r) => !r.success).length;
        // TODO: Backend doesn't return file sizes yet — using 100MB/episode estimate.
        // Replace with actual sizes once DownloadResult includes fileSize field.
        const totalSize = 100 * 1024 * 1024 * successCount;

        updateRecord(recordId, {
          completedEpisodes: results.filter((r) => r.success).map((r) => r.episode),
          failedEpisodes: results.filter((r) => !r.success).map((r) => r.episode),
          endTime: new Date().toISOString(),
          totalSize,
          status: failCount === 0 ? "completed" : failCount === episodes.length ? "failed" : "partial",
        });

        if (failCount === 0) {
          success(`All ${successCount} episodes of ${targetSeries.title} downloaded!`);
          onFilesRefresh();
          return true;
        } else {
          warning(`Downloaded ${successCount}/${episodes.length} episodes (${failCount} failed)`);
          onFilesRefresh();
          return false;
        }
      } catch (e) {
        error(`Download failed: ${e}`);
        return false;
      } finally {
        setDownloadState((prev) => ({ ...prev, isDownloading: false }));
        if (!isBatchMode) {
          setQueue([]);
        }
      }
    },
    [settings, ffmpegAvailable, addRecord, updateRecord, log, success, warning, error, resetSpeedGraph, isBatchMode, selectedQuality, onFilesRefresh],
  );

  const handlePause = useCallback(async () => {
    if (!downloadState.isDownloading) return;
    setDownloadState((prev) => ({ ...prev, isPaused: true }));
    try {
      const ep = currentEpisodeRef.current;
      if (ep > 0) await invoke("pause_download", { episode: ep });
      log("Paused download");
    } catch {
      log("Pause completed (download may have finished)");
    }
  }, [downloadState.isDownloading, log]);

  const handleResume = useCallback(async () => {
    if (!downloadState.isDownloading) return;
    setDownloadState((prev) => ({ ...prev, isPaused: false }));
    try {
      const ep = currentEpisodeRef.current;
      if (ep > 0) await invoke("resume_download", { episode: ep });
      log("Resumed download");
    } catch {
      log("Resume completed (download may have finished)");
    }
  }, [downloadState.isDownloading, log]);

  const handleCancel = useCallback(async () => {
    if (!downloadState.isDownloading) return;
    setDownloadState((prev) => ({ ...prev, isDownloading: false, isPaused: false }));
    try {
      const ep = currentEpisodeRef.current;
      if (ep > 0) await invoke("cancel_download", { episode: ep });
      warning("Cancelled download");
      setQueue([]);
    } catch (e) {
      error(`Failed to cancel: ${e}`);
    }
  }, [downloadState.isDownloading, warning, error]);

  const handlePauseResume = useCallback(() => {
    if (downloadState.isPaused) handleResume();
    else handlePause();
  }, [downloadState.isPaused, handlePause, handleResume]);

  return {
    downloadState,
    setDownloadState,
    progress,
    setProgress,
    queue,
    setQueue,
    runDownload,
    handlePause,
    handleResume,
    handleCancel,
    handlePauseResume,
    addDataPoint,
  };
}

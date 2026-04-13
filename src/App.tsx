import React, { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { readText } from "@tauri-apps/plugin-clipboard-manager";
import {
  Download,
  Pause,
  Play,
  X,
  FolderOpen,
  Link,
  Clipboard,
  Search,
  Merge,
  Settings,
  HardDrive,
  Minimize2,
  Keyboard,
  Clock,
  AlertCircle,
  ListOrdered,
  Loader2,
  CheckCircle,
  Image as ImageIcon,
  BookOpen,
  Compass,
} from "lucide-react";
import {
  Button,
  Input,
  ProgressBar,
  EpisodeSelector,
  SeriesCard,
  LogPanel,
  SettingsPanel,
  HistoryPanel,
  SpeedGraph,
  FileBrowser,
  DownloadQueue,
  Logo,
  UpdateDialog,
  MiniMode,
  ShortcutsHelp,
  BrowsePanel,
  ToastContainer,
  ErrorBoundary,
} from "./components";
import LibraryPanel from "./components/LibraryPanel";
import QualitySelector from "./components/QualitySelector";
import NotificationCenter from "./components/NotificationCenter";
import { SchedulerPanel } from "./components/SchedulerPanel";
import { BackupPanel } from "./components/BackupPanel";
import { useLogger } from "./hooks/useLogger";
import { useSettings } from "./hooks/useSettings";
import { useHistory } from "./hooks/useHistory";
import { useSpeedGraph } from "./hooks/useSpeedGraph";
import { useUpdater } from "./hooks/useUpdater";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
import { useDomainSettings } from "./hooks/useDomainSettings";

import { useI18n } from "./hooks/useI18n";
import { useCustomTheme } from "./hooks/useCustomTheme";
import { useToast } from "./hooks/useToast";
import { SeriesInfo, DownloadState, DownloadProgress } from "./types";
import { QueueItem } from "./components/DownloadQueue";

interface DownloadResult {
  episode: number;
  success: boolean;
  filePath?: string;
  error?: string;
}

interface FileInfo {
  name: string;
  path: string;
  size: number;
  isEpisode: boolean;
  isMerged: boolean;
}

interface BatchItem {
  url: string;
  status:
    | "pending"
    | "fetching"
    | "ready"
    | "error"
    | "downloading"
    | "completed"
    | "failed";
  info?: SeriesInfo;
  error?: string;
}

type TabType = "download" | "library" | "browse" | "files" | "history" | "settings" | "logs";

function App() {
  const {
    domainSettings,
    updateDomainSetting,
    resetDomainSettings,
    isLoaded: domainsLoaded,
  } = useDomainSettings();

  // State
  const [url, setUrl] = useState("");
  const [series, setSeries] = useState<SeriesInfo | null>(null);
  const [selectedEpisodes, setSelectedEpisodes] = useState<Set<number>>(
    new Set(),
  );
  const [isFetching, setIsFetching] = useState(false);
  const [activeTab, setActiveTab] = useState<TabType>("download");
  const [ffmpegAvailable, setFfmpegAvailable] = useState(false);
  const [files, setFiles] = useState<FileInfo[]>([]);
  const [queue, setQueue] = useState<QueueItem[]>([]);
  const [selectedQuality, setSelectedQuality] = useState<string | null>(null);

  // Batch Mode State
  const [isBatchMode, setIsBatchMode] = useState(false);
  const [batchQueue, setBatchQueue] = useState<BatchItem[]>([]);
  const [isBatchProcessing, setIsBatchProcessing] = useState(false);
  const [isAutoCapture, setIsAutoCapture] = useState(false);
  const [isBatchItemRunning, setIsBatchItemRunning] = useState(false);
  const isBatchItemRunningRef = useRef(false);

  const [mergeState, setMergeState] = useState<{
    isMerging: boolean;
    mergedFile: string | null;
    mergeError: string | null;
    progress: number;
    currentTime: number;
    totalDuration: number;
  }>({
    isMerging: false,
    mergedFile: null,
    mergeError: null,
    progress: 0,
    currentTime: 0,
    totalDuration: 0,
  });

  // New UI states
  const [isDragging, setIsDragging] = useState(false);
  const [showMiniMode, setShowMiniMode] = useState(false);
  const [showShortcuts, setShowShortcuts] = useState(false);

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

  const [detectionState, setDetectionState] = useState<{
    isDetecting: boolean;
    message: string;
    progress: number;
  }>({
    isDetecting: false,
    message: "",
    progress: 0,
  });

  // Helper to check if URL is valid for this app
  // NOTE: URL must start with http:// or https:// to be valid.
  // This prevents reqwest "builder error" from malformed/relative URLs.
  const isValidSeriesUrl = useCallback(
    (text: string): boolean => {
      if (!text) return false;

      // Must be an absolute URL to avoid reqwest "builder error"
      if (!text.startsWith("http://") && !text.startsWith("https://")) {
        return false;
      }

      const checkDomains = (settingDomain: string) => {
        return settingDomain.split(",").some((d) => {
          const domain = d.trim();
          return domain.length > 0 && text.includes(domain);
        });
      };

      return (
        checkDomains(domainSettings.rongyokDomain) ||
        checkDomains(domainSettings.titanDomain) ||
        checkDomains(domainSettings.baanjeenDomain) ||
        checkDomains(domainSettings.njavtvDomain || "njavtv.com") ||
        checkDomains(domainSettings.javwowDomain || "javwow.com") ||
        checkDomains(domainSettings.avkuyDomain || "www2.avkuy.com") ||
        text.includes("rongyok.com") ||
        text.includes("thongyok.com") ||
        text.includes("51cg") ||
        text.includes("357ms") || // Titan wildcard fallback
        text.includes("xn--82c7abb4jua0l.com") ||
        text.includes("njavtv.com") ||
        text.includes("javwow.com") ||
        text.includes("avkuy.com") ||
        text.includes("av-kuy.com")
      );
    },
    [domainSettings],
  );

  const extractUrls = useCallback(
    (text: string) => {
      return text
        .split(/[\n\s]+/)
        .map((l) => {
          const trimmed = l.trim();
          // Normalize protocol-relative URLs (//example.com) -> https://example.com
          if (trimmed.startsWith("//")) return `https:${trimmed}`;
          return trimmed;
        })
        .filter((l) => l.length > 0 && isValidSeriesUrl(l));
    },
    [isValidSeriesUrl],
  );

  // Hooks
  const { logs, log, success, warning, error, clearLogs } = useLogger();
  const { settings, updateSetting, resetSettings } = useSettings();
  const {
    history,
    addRecord,
    updateRecord,
    deleteRecord,
    clearHistory,
    getStats,
  } = useHistory();
  const {
    speedData,
    currentSpeed,
    avgSpeed,
    peakSpeed,
    addDataPoint,
    reset: resetSpeedGraph,
  } = useSpeedGraph();
  const {
    checking: isCheckingUpdates,
    available: updateAvailable,
    downloading: updateDownloading,
    progress: updateProgress,
    error: updateError,
    updateInfo,
    checkForUpdates,
    downloadAndInstall,
    dismissUpdate,
  } = useUpdater();
  const { language, setLanguage, t } = useI18n();
  const { themes, activeThemeId, setActiveTheme } = useCustomTheme();
  const { toasts, removeToast } = useToast();

  // Tab navigation
  const tabs: TabType[] = ["download", "library", "browse", "files", "history", "settings", "logs"];
  const handleNextTab = useCallback(() => {
    const currentIndex = tabs.indexOf(activeTab);
    const nextIndex = (currentIndex + 1) % tabs.length;
    setActiveTab(tabs[nextIndex]);
  }, [activeTab]);

  const handlePrevTab = useCallback(() => {
    const currentIndex = tabs.indexOf(activeTab);
    const prevIndex = (currentIndex - 1 + tabs.length) % tabs.length;
    setActiveTab(tabs[prevIndex]);
  }, [activeTab]);

  // Handlers
  const handlePaste = useCallback(async () => {
    try {
      const text = await readText();
      if (!text) {
        error("Clipboard is empty");
        return;
      }

      // Check for multiple URLs
      const lines = text
        .split(/[\n\s]+/)
        .map((l) => l.trim())
        .filter((l) => l.length > 0);
      const validUrls = lines.filter((l) => isValidSeriesUrl(l));

      if (validUrls.length > 1) {
        // Batch Mode
        log(`Detected ${validUrls.length} URLs - Switching to Batch Mode`);
        setIsBatchMode(true);
        const newBatch: BatchItem[] = validUrls.map((u) => ({
          url: u,
          status: "pending",
        }));
        setBatchQueue(newBatch);

        // Process batch
        for (let i = 0; i < newBatch.length; i++) {
          const url = newBatch[i].url;
          setBatchQueue((prev) =>
            prev.map((item, idx) =>
              idx === i ? { ...item, status: "fetching" } : item,
            ),
          );
          try {
            const result = await invoke<SeriesInfo>("fetch_series", { url });
            setBatchQueue((prev) =>
              prev.map((item, idx) =>
                idx === i
                  ? {
                      ...item,
                      status: "ready",
                      info: { ...result, url },
                    }
                  : item,
              ),
            );
          } catch (e) {
            setBatchQueue((prev) =>
              prev.map((item, idx) =>
                idx === i
                  ? { ...item, status: "error", error: String(e) }
                  : item,
              ),
            );
          }
        }
        success(`Ready to download ${validUrls.length} series`);
        setIsBatchProcessing(true); // Auto-start processing
        return;
      }

      setUrl(text);
      log(`Pasted URL: ${text}`);

      // Auto-fetch if valid URL (Single)
      if (isValidSeriesUrl(text)) {
        setIsBatchMode(false); // Reset batch mode if single
        setIsFetching(true);
        try {
          const result = await invoke<SeriesInfo>("fetch_series", {
            url: text,
          });
          setSeries(result);
          const allEpisodes = new Set(
            Array.from({ length: result.totalEpisodes }, (_, i) => i + 1),
          );
          setSelectedEpisodes(allEpisodes);
          success(`Loaded: ${result.title} (${result.totalEpisodes} episodes)`);
          log(`Cached ${Object.keys(result.episodeUrls).length} video URLs`);
        } catch (e) {
          error(`Failed to fetch: ${e}`);
        } finally {
          setIsFetching(false);
        }
      }
    } catch (e) {
      error("Failed to read clipboard");
    }
  }, [log, error, success, isValidSeriesUrl]);

  const runDownload = useCallback(
    async (
      targetSeries: SeriesInfo,
      targetEpisodes: Set<number>,
    ): Promise<boolean> => {
      const episodes = Array.from(targetEpisodes).sort((a, b) => a - b);
      log(
        `Starting download of ${episodes.length} episodes for ${targetSeries.title}`,
      );

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
        // Ensure backend has the correct series loaded (sync frontend state to backend)
        await invoke("update_series_state", {
          series: targetSeries,
        });

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
        const totalSize = 100 * 1024 * 1024 * successCount;

        updateRecord(recordId, {
          completedEpisodes: results
            .filter((r) => r.success)
            .map((r) => r.episode),
          failedEpisodes: results
            .filter((r) => !r.success)
            .map((r) => r.episode),
          endTime: new Date().toISOString(),
          totalSize,
          status:
            failCount === 0
              ? "completed"
              : failCount === episodes.length
                ? "failed"
                : "partial",
        });

        if (failCount === 0) {
          success(
            `All ${successCount} episodes of ${targetSeries.title} downloaded!`,
          );
          refreshFiles();
          return true;
        } else {
          warning(
            `Downloaded ${successCount}/${episodes.length} episodes (${failCount} failed)`,
          );
          refreshFiles();
          return false;
        }
      } catch (e) {
        error(`Download failed: ${e}`);
        return false;
      } finally {
        setDownloadState((prev) => ({ ...prev, isDownloading: false }));
        // Only clear queue if NOT in batch mode (Batch mode manages its own queue)
        if (!isBatchMode) {
          setQueue([]);
        }
      }
    },
    [
      settings,
      ffmpegAvailable,
      addRecord,
      updateRecord,
      log,
      success,
      warning,
      error,
      resetSpeedGraph,
      isBatchMode,
    ],
  );

  const handleStartDownload = useCallback(() => {
    if (!series || selectedEpisodes.size === 0) {
      error("Please select at least one episode");
      return;
    }
          isBatchItemRunningRef.current = false;
    setIsBatchItemRunning(false);
    setIsBatchProcessing(false);
    setIsBatchMode(false);
    runDownload(series, selectedEpisodes);
  }, [series, selectedEpisodes, runDownload, error]);

  // Clipboard Monitor
  const lastClipboard = useRef<string>("");

  useEffect(() => {
    let interval: number;
    if (isAutoCapture) {
      interval = window.setInterval(async () => {
        try {
          const text = await readText();
          if (text && text !== lastClipboard.current) {
            lastClipboard.current = text;
            const urls = extractUrls(text);
            if (urls.length > 0) {
              log(`Auto-captured ${urls.length} links`);

              // Switch to batch mode if needed
              setIsBatchMode(true);

              // Add unique URLs to queue
              setBatchQueue((prev) => {
                const existing = new Set(prev.map((i) => i.url));
                const newItems = urls
                  .filter((u) => !existing.has(u))
                  .map((u) => ({ url: u, status: "pending" }) as BatchItem);

                if (newItems.length > 0) {
                  success(`Added ${newItems.length} links to queue`);
                  setIsBatchProcessing(true); // Auto-start processing
                  return [...prev, ...newItems];
                }
                return prev;
              });
            }
          }
        } catch (e) {
          // Ignore clipboard errors
        }
      }, 1000);
    }
    return () => clearInterval(interval);
  }, [isAutoCapture, extractUrls, log, success]);

  // Taskbar Progress
  // Call Rust to set progress
  useEffect(() => {
    if (downloadState.isDownloading) {
      invoke("set_taskbar_progress", {
        progress: Math.round(progress.percentage),
      }).catch(() => {});
    } else {
      invoke("set_taskbar_progress", { progress: -1 }).catch(() => {}); // -1 to clear
    }
  }, [downloadState.isDownloading, progress.percentage]);

  // Batch Fetcher (Pending -> Ready)
  useEffect(() => {
    // Only one fetch at a time
    const pendingIdx = batchQueue.findIndex((i) => i.status === "pending");
    if (pendingIdx !== -1 && !isFetching) {
      const item = batchQueue[pendingIdx];
      // log(`Batch Fetcher: Fetching info for ${item.url}`);

      const fetchItem = async () => {
        // Mark as fetching
        setBatchQueue((prev) =>
          prev.map((it, idx) =>
            idx === pendingIdx ? { ...it, status: "fetching" } : it,
          ),
        );
        setIsFetching(true);

        try {
          // Use invoke directly to fetch series info
          const result = await invoke<SeriesInfo>("fetch_series", {
            url: item.url,
          });

          setBatchQueue((prev) =>
            prev.map((it, idx) =>
              idx === pendingIdx
                ? { ...it, status: "ready", info: result }
                : it,
            ),
          );
          log(`Queue: Ready - ${result.title}`);
        } catch (e) {
          error(`Queue Fetch failed for ${item.url}: ${e}`);
          setBatchQueue((prev) =>
            prev.map((it, idx) =>
              idx === pendingIdx
                ? { ...it, status: "failed", error: String(e) }
                : it,
            ),
          );
        } finally {
          setIsFetching(false);
        }
      };

      fetchItem();
    }
  }, [batchQueue, isFetching, log, error]);

  // Continuous Batch Processing
  useEffect(() => {
    // Console log is safe (doesn't trigger re-render)
    // console.log(`Queue Check: Processing=${isBatchProcessing}, Downloading=${downloadState.isDownloading}`);

    if (!isBatchProcessing) return;
    if (downloadState.isDownloading) return;
    if (isBatchItemRunning || isBatchItemRunningRef.current) return;

    // Disabled check to prevent stale status deadlock
    // const isAnyDownloading = batchQueue.some((i) => i.status === "downloading");
    // if (isAnyDownloading) return;

    const nextIdx = batchQueue.findIndex((i) => i.status === "ready" && i.info);
    if (nextIdx !== -1) {
      console.log(
        `Queue Manager: Found next item at index ${nextIdx} (${batchQueue[nextIdx].url})`,
      );
      const item = batchQueue[nextIdx];

      const processItem = async () => {
        isBatchItemRunningRef.current = true;
        // Use console.log to avoid infinite render loops
        console.log(`Processing batch item: ${item.info?.title}`);
        setIsBatchItemRunning(true);

        // Mark as downloading
        setBatchQueue((prev) =>
          prev.map((it, idx) =>
            idx === nextIdx ? { ...it, status: "downloading" } : it,
          ),
        );

        try {
          // Check if item.info exists before using it
          if (!item.info) {
            throw new Error("Series info missing for batch item");
          }

          const allEpisodes = new Set(
            Array.from({ length: item.info.totalEpisodes }, (_, i) => i + 1),
          );

          // runDownload now fetches series info internally to ensure consistency
          const success = await runDownload(item.info, allEpisodes);

          console.log(`Batch item finished found success=${success}`);

          setBatchQueue((prev) =>
            prev.map((it, idx) =>
              idx === nextIdx
                ? { ...it, status: success ? "completed" : "failed" }
                : it,
            ),
          );
        } catch (e) {
          console.error(`Batch item failed: ${e}`);
          // Error state update is safe because it breaks the "ready" condition
          setBatchQueue((prev) =>
            prev.map((it, idx) =>
              idx === nextIdx
                ? { ...it, status: "failed", error: String(e) }
                : it,
            ),
          );
        } finally {
          isBatchItemRunningRef.current = false;
          setIsBatchItemRunning(false);
        }
      };

      processItem();
    }
  }, [batchQueue, isBatchProcessing, downloadState.isDownloading, isBatchItemRunning, runDownload]);

  const handlePause = useCallback(async () => {
    if (downloadState.currentEpisode === 0) {
      warning("No episode currently downloading");
      return;
    }
    setDownloadState((prev) => ({ ...prev, isPaused: true }));
    try {
      await invoke("pause_download", { episode: downloadState.currentEpisode });
      log("Paused download");
    } catch (e) {
      // Download may have completed, don't show error
      log("Pause completed (download may have finished)");
    }
  }, [downloadState.currentEpisode, log, warning]);

  const handleResume = useCallback(async () => {
    if (downloadState.currentEpisode === 0) {
      warning("No episode currently downloading");
      return;
    }
    setDownloadState((prev) => ({ ...prev, isPaused: false }));
    try {
      await invoke("resume_download", {
        episode: downloadState.currentEpisode,
      });
      log("Resumed download");
    } catch (e) {
      // Download may have completed, don't show error
      log("Resume completed (download may have finished)");
    }
  }, [downloadState.currentEpisode, log, warning]);

  const handleCancel = useCallback(async () => {
    if (downloadState.currentEpisode === 0) return;
    setDownloadState((prev) => ({
      ...prev,
      isDownloading: false,
      isPaused: false,
    }));
    try {
      await invoke("cancel_download", {
        episode: downloadState.currentEpisode,
      });
      warning("Cancelled download");
    } catch (e) {
      error(`Failed to cancel: ${e}`);
    }
  }, [downloadState.currentEpisode, warning, error]);

  const handlePauseResume = useCallback(() => {
    if (downloadState.isPaused) {
      handleResume();
    } else {
      handlePause();
    }
  }, [downloadState.isPaused, handlePause, handleResume]);

  // Keyboard shortcuts
  useKeyboardShortcuts({
    onPaste: handlePaste,
    onDownload: handleStartDownload,
    onPauseResume: handlePauseResume,
    onCancel: handleCancel,
    onToggleMiniMode: () => setShowMiniMode((prev) => !prev),
    onNextTab: handleNextTab,
    onPrevTab: handlePrevTab,
    isDownloading: downloadState.isDownloading,
    isPaused: downloadState.isPaused,
  });

  // Global ESC key handler for modals
  useEffect(() => {
    const handleEsc = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        // Close modals/dropdowns when ESC is pressed
        if (showShortcuts) {
          setShowShortcuts(false);
        }
        if (showMiniMode) {
          setShowMiniMode(false);
        }
        // Note: NotificationCenter dropdown is handled internally by its component
      }
    };

    window.addEventListener('keydown', handleEsc);
    return () => window.removeEventListener('keydown', handleEsc);
  }, [showShortcuts, showMiniMode]);

  // Auto-paste and auto-fetch from clipboard
  const autoFetchFromClipboard = useCallback(async () => {
    try {
      const text = await readText();
      if (text && isValidSeriesUrl(text) && text !== url) {
        setUrl(text);
        log(`Auto-pasted URL: ${text}`);

        // Auto-fetch the series info
        setIsFetching(true);
        try {
          const result = await invoke<SeriesInfo>("fetch_series", {
            url: text,
          });
          setSeries(result);
          const allEpisodes = new Set(
            Array.from({ length: result.totalEpisodes }, (_, i) => i + 1),
          );
          setSelectedEpisodes(allEpisodes);
          success(
            `Auto-loaded: ${result.title} (${result.totalEpisodes} episodes)`,
          );
          log(`Cached ${Object.keys(result.episodeUrls).length} video URLs`);
        } catch (e) {
          error(`Failed to fetch: ${e}`);
        } finally {
          setIsFetching(false);
        }
      }
    } catch (e) {
      // Clipboard access denied or empty - silently ignore
    }
  }, [url, isValidSeriesUrl, log, success, error]);

  // Initialize
  const initialized = React.useRef(false);
  useEffect(() => {
    if (!domainsLoaded) return;
    if (initialized.current) return;
    initialized.current = true;

    log("Application started");
    checkFfmpeg();
    
    let cleanup: (() => void) | undefined;
    setupEventListeners().then(fn => {
      cleanup = fn;
    }).catch(err => {
      error(`Failed to setup event listeners: ${err}`);
    });

    // Auto-paste from clipboard on startup
    autoFetchFromClipboard();
    
    // Return cleanup function
    return () => {
      cleanup?.();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [domainsLoaded, autoFetchFromClipboard]);

  // Auto-fetch when window gains focus
  useEffect(() => {
    const handleFocus = () => {
      // Only auto-fetch if not currently downloading or fetching
      if (!downloadState.isDownloading && !isFetching) {
        autoFetchFromClipboard();
      }
    };

    window.addEventListener("focus", handleFocus);
    return () => window.removeEventListener("focus", handleFocus);
  }, [downloadState.isDownloading, isFetching, autoFetchFromClipboard]);

  // Apply theme
  useEffect(() => {
    const root = document.documentElement;
    if (settings.theme === "dark") {
      root.classList.add("dark");
      root.classList.remove("light");
    } else if (settings.theme === "light") {
      root.classList.remove("dark");
      root.classList.add("light");
    } else {
      const prefersDark = window.matchMedia(
        "(prefers-color-scheme: dark)",
      ).matches;
      if (prefersDark) {
        root.classList.add("dark");
        root.classList.remove("light");
      } else {
        root.classList.remove("dark");
        root.classList.add("light");
      }
    }
  }, [settings.theme]);

  // Refresh files when switching to Files tab
  useEffect(() => {
    if (activeTab === "files") {
      refreshFiles();
    }
  }, [activeTab]);

  const setupEventListeners = async () => {
    const unsubscribers: (() => void)[] = [];

    unsubscribers.push(await listen<{ message: string; progress: number }>(
      "detection-progress",
      (event) => {
        setDetectionState({
          isDetecting: true,
          message: event.payload.message,
          progress: event.payload.progress,
        });

        if (event.payload.progress >= 100) {
          setTimeout(() => {
            setDetectionState((prev) => ({ ...prev, isDetecting: false }));
          }, 2000);
        }
      },
    ));

    unsubscribers.push(await listen<DownloadProgress>("download-progress", (event) => {
      setProgress(event.payload);
      addDataPoint(event.payload.speed);
      setDownloadState((prev) => ({
        ...prev,
        currentEpisode: event.payload.episode,
      }));
    }));

    unsubscribers.push(await listen<DownloadResult>("download-result", (event) => {
      const result = event.payload;
      if (result.success) {
        setDownloadState((prev) => ({
          ...prev,
          completedEpisodes: [...prev.completedEpisodes, result.episode],
        }));
        success(`Episode ${result.episode} downloaded`);
      } else {
        setDownloadState((prev) => ({
          ...prev,
          failedEpisodes: [...prev.failedEpisodes, result.episode],
        }));
        error(`Episode ${result.episode} failed: ${result.error}`);
      }

      setQueue((prev) =>
        prev.map((q) =>
          q.episode === result.episode
            ? {
                ...q,
                status: result.success ? "completed" : "failed",
                progress: 100,
              }
            : q,
        ),
      );
    }));

    unsubscribers.push(await listen("merge-started", () => {
      log("Merging videos...");
      setMergeState({
        isMerging: true,
        mergedFile: null,
        mergeError: null,
        progress: 0,
        currentTime: 0,
        totalDuration: 0,
      });
    }));

    unsubscribers.push(await listen<{
      percentage: number;
      currentTime: number;
      totalDuration: number;
    }>("merge-progress", (event) => {
      setMergeState((prev) => ({
        ...prev,
        progress: event.payload.percentage,
        currentTime: event.payload.currentTime,
        totalDuration: event.payload.totalDuration,
      }));
    }));

    unsubscribers.push(await listen<string>("merge-complete", (event) => {
      success(`Merged to: ${event.payload}`);
      setMergeState({
        isMerging: false,
        mergedFile: event.payload,
        mergeError: null,
        progress: 100,
        currentTime: 0,
        totalDuration: 0,
      });
      playNotificationSound();
      showNotification("Merge Complete", "Videos merged successfully!");
      refreshFiles();
    }));

    unsubscribers.push(await listen<string>("merge-error", (event) => {
      error(`Merge failed: ${event.payload}`);
      setMergeState({
        isMerging: false,
        mergedFile: null,
        mergeError: event.payload,
        progress: 0,
        currentTime: 0,
        totalDuration: 0,
      });
    }));

    unsubscribers.push(await listen<string>("log-info", (event) => {
      log(event.payload);
    }));

    // Return cleanup function to prevent memory leak
    return () => {
      unsubscribers.forEach(unsub => unsub());
    };
  };

  const checkFfmpeg = async () => {
    try {
      const available = await invoke<boolean>("check_ffmpeg_available");
      setFfmpegAvailable(available);
      if (available) {
        success("FFmpeg is available");
      } else {
        warning("FFmpeg not found - video merging will be disabled");
      }
    } catch (e) {
      warning("Could not check FFmpeg status");
    }
  };

  const toggleBatchProcessing = useCallback(() => {
    if (isBatchProcessing) {
      setIsBatchProcessing(false);
      log("Batch processing paused");
    } else {
      setIsBatchProcessing(true);
      log("Batch processing started");
    }
  }, [isBatchProcessing, log]);

  const playNotificationSound = () => {
    if (settings.soundEnabled) {
      const audio = new Audio("/notification.mp3");
      audio.play().catch(() => {});
    }
  };

  const showNotification = (title: string, body: string) => {
    if (settings.notificationsEnabled && "Notification" in window) {
      if (Notification.permission === "granted") {
        new Notification(title, { body });
      } else if (Notification.permission !== "denied") {
        Notification.requestPermission().then((permission) => {
          if (permission === "granted") {
            new Notification(title, { body });
          }
        });
      }
    }
  };

  const handleFetch = async () => {
    if (!url.trim()) {
      error("Please enter a URL");
      return;
    }

    setIsFetching(true);
    setDetectionState({ isDetecting: false, message: "", progress: 0 });
    log(`Fetching: ${url}`);

    try {
      const result = await invoke<SeriesInfo>("fetch_series", { url });
      setSeries(result);

      const allEpisodes = new Set(
        Array.from({ length: result.totalEpisodes }, (_, i) => i + 1),
      );
      setSelectedEpisodes(allEpisodes);

      success(`Loaded: ${result.title} (${result.totalEpisodes} episodes)`);
      log(`Cached ${Object.keys(result.episodeUrls).length} video URLs`);
    } catch (e) {
      error(`Failed to fetch: ${e}`);
    } finally {
      setIsFetching(false);
    }
  };

  const toggleEpisode = (ep: number) => {
    setSelectedEpisodes((prev) => {
      const next = new Set(prev);
      if (next.has(ep)) {
        next.delete(ep);
      } else {
        next.add(ep);
      }
      return next;
    });
  };

  const selectAllEpisodes = () => {
    if (series) {
      setSelectedEpisodes(
        new Set(Array.from({ length: series.totalEpisodes }, (_, i) => i + 1)),
      );
    }
  };

  const deselectAllEpisodes = () => {
    setSelectedEpisodes(new Set());
  };

  const refreshFiles = async () => {
    try {
      const fileList = await invoke<FileInfo[]>("list_files", {
        dir: settings.outputDir,
      });
      setFiles(fileList);
      log(`Found ${fileList.length} files`);
    } catch (e) {
      warning("Could not list files");
    }
  };

  const handleDeleteFiles = async (paths: string[]) => {
    try {
      const deleted = await invoke<number>("delete_files", { paths });
      setFiles((prev) => prev.filter((f) => !paths.includes(f.path)));
      success(`Deleted ${deleted} file${deleted !== 1 ? "s" : ""}`);
    } catch (e) {
      error("Failed to delete files");
    }
  };

  const handlePlayFile = async (path: string) => {
    try {
      await invoke("play_file", { path });
      log(`Playing: ${path}`);
    } catch (e) {
      warning("Could not open file");
    }
  };

  const handleSelectOutputFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: t("app.selectOutputFolder"),
      });
      if (selected && typeof selected === "string") {
        updateSetting("outputDir", selected);
        success(`Output folder set to: ${selected}`);
      }
    } catch (e) {
      warning("Could not select folder");
    }
  };

  const handleOpenOutputFolder = async () => {
    try {
      await invoke("open_folder", { path: settings.outputDir });
    } catch (e) {
      warning("Could not open folder");
    }
  };

  // Drag and drop handlers
  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);

    const text = e.dataTransfer.getData("text/plain");
    if (text && isValidSeriesUrl(text)) {
      setUrl(text);
      log(`Dropped URL: ${text}`);
      success("URL added from drop");

      // Auto-fetch
      setIsFetching(true);
      try {
        const result = await invoke<SeriesInfo>("fetch_series", { url: text });
        setSeries(result);
        const allEpisodes = new Set(
          Array.from({ length: result.totalEpisodes }, (_, i) => i + 1),
        );
        setSelectedEpisodes(allEpisodes);
        success(`Loaded: ${result.title} (${result.totalEpisodes} episodes)`);
        log(`Cached ${Object.keys(result.episodeUrls).length} video URLs`);
      } catch (err) {
        error(`Failed to fetch: ${err}`);
      } finally {
        setIsFetching(false);
      }
    } else if (text) {
      setUrl(text);
      log(`Dropped text: ${text}`);
    }
  };

  const overallProgress =
    downloadState.totalSelected > 0
      ? ((downloadState.completedEpisodes.length + progress.percentage / 100) /
          downloadState.totalSelected) *
        100
      : 0;

  const tabsConfig: {
    id: TabType;
    label: string;
    icon: React.ReactNode;
    glowColor: string;
    activeClass: string;
  }[] = [
    {
      id: "download",
      label: t("tabs.download"),
      icon: (
        <Download
          size={16}
          className="drop-shadow-[0_0_4px_rgba(139,92,246,0.6)]"
        />
      ),
      glowColor: "violet",
      activeClass:
        "bg-violet-500/20 text-violet-300 border border-violet-500/40",
    },
    {
      id: "library",
      label: t("tabs.library"),
      icon: (
        <BookOpen
          size={16}
          className="drop-shadow-[0_0_4px_rgba(168,85,247,0.6)]"
        />
      ),
      glowColor: "purple",
      activeClass:
        "bg-purple-500/20 text-purple-300 border border-purple-500/40",
    },
    {
      id: "browse",
      label: t("tabs.browse"),
      icon: (
        <Compass
          size={16}
          className="drop-shadow-[0_0_4px_rgba(52,211,153,0.6)]"
        />
      ),
      glowColor: "emerald",
      activeClass: "bg-emerald-500/20 text-emerald-300 border border-emerald-500/40",
    },
    {
      id: "files",
      label: t("tabs.files"),
      icon: (
        <HardDrive
          size={16}
          className="drop-shadow-[0_0_4px_rgba(59,130,246,0.6)]"
        />
      ),
      glowColor: "blue",
      activeClass: "bg-blue-500/20 text-blue-300 border border-blue-500/40",
    },
    {
      id: "history",
      label: t("tabs.history"),
      icon: (
        <Clock
          size={16}
          className="drop-shadow-[0_0_4px_rgba(251,191,36,0.6)]"
        />
      ),
      glowColor: "amber",
      activeClass: "bg-amber-500/20 text-amber-300 border border-amber-500/40",
    },
    {
      id: "settings",
      label: t("tabs.settings"),
      icon: (
        <Settings
          size={16}
          className="drop-shadow-[0_0_4px_rgba(148,163,184,0.6)]"
        />
      ),
      glowColor: "slate",
      activeClass: "bg-slate-500/20 text-slate-200 border border-slate-500/40",
    },
    {
      id: "logs",
      label: t("app.logsCount", { count: logs.length }),
      icon: (
        <AlertCircle
          size={16}
          className="drop-shadow-[0_0_4px_rgba(34,211,238,0.6)]"
        />
      ),
      glowColor: "cyan",
      activeClass: "bg-cyan-500/20 text-cyan-300 border border-cyan-500/40",
    },
  ];

  return (
    <div
      className="min-h-screen text-white"
      style={{
        background:
          "var(--bg-primary, linear-gradient(to bottom right, #0f172a, #1e293b, #0f172a))",
      }}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      {/* Drag overlay */}
      {isDragging && (
        <div className="fixed inset-0 z-[100] bg-violet-500/20 backdrop-blur-sm flex items-center justify-center pointer-events-none">
          <div className="bg-slate-800/90 rounded-2xl p-8 border-2 border-dashed border-violet-400 animate-pulse-glow">
            <div className="flex flex-col items-center gap-3">
              <span className="icon-glow icon-glow-lg icon-glow-violet icon-glow-animated">
                <Download size={48} />
              </span>
              <span className="text-xl font-medium text-violet-300">
                {t("app.dropUrl")}
              </span>
            </div>
          </div>
        </div>
      )}

      {/* Header - Compact */}
      <header className="sticky top-0 z-50 glass border-b border-slate-800/50">
        <div className="container-responsive py-1.5 sm:py-2">
          <div className="flex items-center justify-between gap-2">
            <div className="flex items-center gap-2">
              <Logo size={28} className="sm:w-8 sm:h-8" />
              <div>
                <h1 className="text-sm font-bold flex items-center gap-1.5">
                  <span className="bg-gradient-to-r from-violet-400 to-fuchsia-400 bg-clip-text text-transparent">
                    Rongyok
                  </span>
                  {ffmpegAvailable && (
                    <span className="text-violet-400 text-[10px]">FFmpeg</span>
                  )}
                  {downloadState.isDownloading && currentSpeed > 0 && (
                    <span className="text-violet-400 text-[10px]">
                      {(currentSpeed / 1024 / 1024).toFixed(1)} MB/s
                    </span>
                  )}
                </h1>
              </div>
            </div>

            {/* Header Actions */}
            <div className="flex items-center gap-1">
              {downloadState.isDownloading && (
                <button
                  onClick={() => setShowMiniMode(true)}
                  className="p-1.5 hover:bg-slate-700/50 rounded-md"
                  title={t("app.openMiniMode")}
                  aria-label={t("app.openMiniMode")}
                >
                  <Minimize2
                    size={14}
                    className="text-violet-400 drop-shadow-[0_0_4px_currentColor]"
                  />
                </button>
              )}
              <NotificationCenter />
              <button
                onClick={() => setShowShortcuts(true)}
                className="p-1.5 hover:bg-slate-700/50 rounded-md"
                title={t("app.openShortcuts")}
                aria-label={t("app.openShortcuts")}
              >
                <Keyboard
                  size={14}
                  className="text-amber-400 drop-shadow-[0_0_4px_currentColor]"
                />
              </button>

              {/* Tabs */}
              <div className="flex bg-slate-800/70 rounded-lg p-0.5 ml-1" role="tablist">
                {tabsConfig.map((tab) => (
                  <button
                    key={tab.id}
                    onClick={() => setActiveTab(tab.id)}
                    role="tab"
                    aria-selected={activeTab === tab.id}
                    aria-label={`${tab.label} tab`}
                    tabIndex={activeTab === tab.id ? 0 : -1}
                    className={`px-2 py-1 text-xs font-medium rounded-md transition-all flex items-center gap-1 ${
                      activeTab === tab.id
                        ? tab.activeClass
                        : "text-slate-400 hover:text-white border border-transparent"
                    }`}
                  >
                    {tab.icon}
                    <span className="hidden sm:inline">{tab.label}</span>
                  </button>
                ))}
              </div>
            </div>
          </div>
        </div>
      </header>

      {/* Main Content - Compact */}
      <main className="flex-1 overflow-hidden relative">
        {activeTab === "download" && (
          <div className="h-full flex flex-col md:flex-row overflow-hidden">
            <div className="flex-1 flex flex-col min-w-0 border-r border-slate-700/30 bg-slate-900/50 relative">
              <div className="flex-1 overflow-y-auto custom-scrollbar p-3 space-y-4">
                {/* Input Area */}
                <div className="sticky top-0 z-10 bg-slate-900/90 backdrop-blur pb-4 -mt-2 pt-2">
                  <Input
                    placeholder="https://rongyok.com/watch/?series_id=XXX"
                    value={url}
                    onChange={(e) => setUrl(e.target.value)}
                    leftIcon={<Link size={14} />}
                    iconColor="cyan"
                    rightElement={
                      <div className="flex gap-0.5 items-center">
                        <Button
                          size="sm"
                          variant="ghost"
                          onClick={() => {
                            setUrl("");
                            setSeries(null);
                            setSelectedEpisodes(new Set());
                          }}
                          disabled={!url}
                          className="px-1.5"
                        >
                          <X size={14} />
                        </Button>
                        <Button
                          size="sm"
                          variant="ghost"
                          onClick={handlePaste}
                          className="px-1.5"
                          title={t("app.paste")}
                        >
                          <Clipboard size={14} />
                        </Button>
                        <Button
                          size="sm"
                          onClick={handleFetch}
                          isLoading={isFetching}
                          className="px-2"
                        >
                          <Search size={14} />
                        </Button>
                      </div>
                    }
                  />

                  <Input
                    placeholder="~/Downloads/rongyok"
                    value={settings.outputDir}
                    onChange={(e) => updateSetting("outputDir", e.target.value)}
                    leftIcon={<FolderOpen size={14} />}
                    iconColor="amber"
                    rightElement={
                      <div className="flex gap-0.5">
                        <Button
                          size="sm"
                          variant="ghost"
                          onClick={handleSelectOutputFolder}
                          className="px-1.5"
                        >
                          📂
                        </Button>
                        <Button
                          size="sm"
                          variant="ghost"
                          onClick={handleOpenOutputFolder}
                          className="px-1.5"
                        >
                          ↗
                        </Button>
                      </div>
                    }
                  />
                </div>

                {/* Smart Queue Bar - Always Visible */}
                <div
                  className={`bg-slate-800/90 backdrop-blur rounded-lg border border-slate-700 p-2 flex items-center justify-between mb-2 shadow-lg shadow-black/20 sticky top-14 z-40 transform transition-all duration-300 ${batchQueue.length === 0 && !isAutoCapture ? "opacity-80 hover:opacity-100" : ""}`}
                >
                  <div className="flex items-center gap-3 pl-1">
                    <div className="flex items-center gap-2">
                      <ListOrdered
                        size={16}
                        className={`text-emerald-400 ${batchQueue.length > 0 ? "drop-shadow-[0_0_8px_rgba(52,211,153,0.5)]" : "opacity-50"}`}
                      />
                      <span className="text-xs font-bold text-slate-200 tracking-wide flex items-center gap-2">
                        {t("app.smartQueueLabel")}
                        {batchQueue.length > 0 && (
                          <span className="bg-slate-700 text-slate-300 px-1.5 rounded-full text-[10px]">
                            {batchQueue.length}
                          </span>
                        )}
                      </span>
                    </div>
                    {isBatchProcessing && (
                      <span className="text-[10px] bg-emerald-500/10 text-emerald-400 px-2 py-0.5 rounded-full flex items-center gap-1.5 animate-pulse border border-emerald-500/20 font-medium">
                        <Loader2 size={10} className="animate-spin" /> {t("app.runningLabel")}
                      </span>
                    )}
                  </div>

                  <div className="flex items-center gap-1.5">
                    <Button
                      size="sm"
                      variant={isAutoCapture ? "cyan" : "ghost"}
                      onClick={() => setIsAutoCapture((p) => !p)}
                      title="Clipboard Monitor (Auto Capture)"
                      className={`h-7 text-xs px-2 ${isAutoCapture ? "ring-1 ring-cyan-500/50 shadow-[0_0_10px_rgba(34,211,238,0.3)]" : "text-slate-400 hover:text-cyan-300"}`}
                    >
                      <Clipboard
                        size={14}
                        className={isAutoCapture ? "animate-pulse" : ""}
                      />
                      <span className="ml-1 hidden sm:inline text-[10px] font-bold">
                        {isAutoCapture ? t("app.on") : t("app.auto")}
                      </span>
                    </Button>

                    <div className="w-px h-4 bg-slate-700 mx-1"></div>

                    <Button
                      size="sm"
                      variant={isBatchProcessing ? "amber" : "success"}
                      onClick={toggleBatchProcessing}
                      title={
                        isBatchProcessing
                          ? t("queue.pauseDownload")
                          : t("app.startQueue")
                      }
                      className="h-7 w-7 p-0 shadow-sm"
                    >
                      {isBatchProcessing ? (
                        <Pause size={14} />
                      ) : (
                        <Play size={14} />
                      )}
                    </Button>

                    <Button
                      size="sm"
                      variant={isBatchMode ? "primary" : "ghost"}
                      onClick={() => setIsBatchMode((p) => !p)}
                      title={
                        isBatchMode ? t("app.hideQueueList") : t("app.showQueueList")
                      }
                      className={`h-7 w-7 p-0 ${isBatchMode ? "shadow-md shadow-violet-500/20" : "text-slate-400 hover:text-white"}`}
                    >
                      {isBatchMode ? (
                        <Minimize2 size={14} />
                      ) : (
                        <ListOrdered size={14} />
                      )}
                    </Button>

                    <Button
                      size="sm"
                      variant="ghost"
                      className="h-7 w-7 p-0 text-slate-500 hover:text-red-400 hover:bg-red-500/10 transition-colors"
                      onClick={() => {
                        if (
                          batchQueue.length === 0 ||
                          confirm("Clear entire queue?")
                        ) {
                          setBatchQueue([]);
                          setIsBatchMode(false);
                          setIsBatchProcessing(false);
                        }
                      }}
                      title={t("app.removeFromBatch")}
                      aria-label={t("app.removeFromBatch")}
                    >
                      <X size={14} />
                    </Button>
                  </div>
                </div>

                {/* Detection Progress */}
                {detectionState.isDetecting && (
                  <div className="bg-slate-800/50 rounded-lg p-3 border border-slate-700 animate-pulse">
                    <div className="flex justify-between text-xs text-slate-400 mb-1">
                      <span className="flex items-center gap-2">
                        <Search size={12} className="animate-spin" />
                        {detectionState.message}
                      </span>
                      <span>{detectionState.progress}%</span>
                    </div>
                    <div className="h-1.5 bg-slate-700 rounded-full overflow-hidden">
                      <div
                        className="h-full bg-violet-500 transition-all duration-300"
                        style={{ width: `${detectionState.progress}%` }}
                      />
                    </div>
                  </div>
                )}

                {/* Series Info - Compact */}
                <SeriesCard series={series} isLoading={isFetching} />

                {/* Episode Selector */}
                {series && (
                  <EpisodeSelector
                    totalEpisodes={series.totalEpisodes}
                    selectedEpisodes={selectedEpisodes}
                    onToggle={toggleEpisode}
                    onSelectAll={selectAllEpisodes}
                    onDeselectAll={deselectAllEpisodes}
                    disabled={downloadState.isDownloading}
                  />
                )}

                {/* Quality Selector */}
                {series && (
                  <QualitySelector
                    episodeUrl={Object.values(series.episodeUrls)[0]}
                    onSelect={(q) => setSelectedQuality(q)}
                    defaultQuality={settings.defaultQuality || 'best'}
                  />
                )}

                {/* Speed Graph & Progress - Compact */}
                {(downloadState.isDownloading || speedData.length > 0) && (
                  <div className="grid grid-cols-1 lg:grid-cols-2 gap-2">
                    <SpeedGraph
                      data={speedData}
                      currentSpeed={currentSpeed}
                      avgSpeed={avgSpeed}
                      peakSpeed={peakSpeed}
                    />
                    <div className="glass rounded-lg p-2 border border-slate-700/50 space-y-2">
                      <ProgressBar
                        percentage={progress.percentage}
                        label={`EP ${progress.episode}`}
                        sublabel={`${(progress.speed / 1024 / 1024).toFixed(1)} MB/s`}
                        variant="cyan"
                      />
                      <ProgressBar
                        percentage={overallProgress}
                        label={t("app.overall")}
                        sublabel={`${downloadState.completedEpisodes.length}/${downloadState.totalSelected}`}
                        variant="success"
                      />
                      {mergeState.isMerging && (
                        <div className="p-2 bg-fuchsia-500/20 rounded-md border border-fuchsia-500/30">
                          <div className="flex items-center justify-between text-xs">
                            <span className="text-fuchsia-300 flex items-center gap-1">
                              <Merge
                                size={12}
                                className="animate-pulse drop-shadow-[0_0_4px_currentColor]"
                              />{" "}
                              {t("app.mergingVideos")}
                            </span>
                            <span className="text-fuchsia-400 font-mono">
                              {mergeState.progress.toFixed(0)}%
                            </span>
                          </div>
                          <div className="h-1.5 bg-slate-700 rounded-full overflow-hidden mt-1">
                            <div
                              className="h-full bg-gradient-to-r from-fuchsia-500 to-violet-500 rounded-full"
                              style={{ width: `${mergeState.progress}%` }}
                            />
                          </div>
                        </div>
                      )}
                      {mergeState.mergedFile && !mergeState.isMerging && (
                        <div className="p-2 bg-emerald-500/20 rounded-md border border-emerald-500/30 text-xs text-emerald-300 flex items-center gap-1">
                          <span className="text-emerald-400">✅</span> {t("app.mergedLabel")}{" "}
                          {mergeState.mergedFile.split("/").pop()}
                        </div>
                      )}
                    </div>
                  </div>
                )}

                {/* Queue - Compact */}
                {queue.length > 0 && (
                  <ErrorBoundary>
                    <DownloadQueue
                      queue={queue}
                      onMoveUp={() => {}}
                      onMoveDown={() => {}}
                      onRemove={(id) =>
                        setQueue((prev) => prev.filter((q) => q.id !== id))
                      }
                      onPause={() => {}}
                    />
                  </ErrorBoundary>
                )}

                {/* Options & Actions - Compact inline */}
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <label className="flex items-center gap-1.5 cursor-pointer text-xs text-slate-300">
                    <input
                      type="checkbox"
                      checked={settings.autoMerge}
                      onChange={(e) =>
                        updateSetting("autoMerge", e.target.checked)
                      }
                      disabled={!ffmpegAvailable}
                      className="w-3.5 h-3.5 rounded bg-slate-700 border-slate-600 text-violet-600"
                    />
                    <Merge
                      size={12}
                      className="text-fuchsia-400 drop-shadow-[0_0_4px_currentColor]"
                    />{" "}
                    {t("app.autoMerge")}
                  </label>

                  <div className="flex gap-2">
                    {!downloadState.isDownloading ? (
                      <Button
                        onClick={handleStartDownload}
                        disabled={!series || selectedEpisodes.size === 0}
                        leftIcon={<Download size={14} />}
                        variant="success"
                      >
                        Download ({selectedEpisodes.size})
                      </Button>
                    ) : (
                      <>
                        {!downloadState.isPaused ? (
                          <Button
                            variant="amber"
                            onClick={handlePause}
                            leftIcon={<Pause size={14} />}
                          >
                            {t("app.pause")}
                          </Button>
                        ) : (
                          <Button
                            variant="success"
                            onClick={handleResume}
                            leftIcon={<Play size={14} />}
                          >
                            {t("app.resume")}
                          </Button>
                        )}
                        <Button
                          variant="danger"
                          onClick={handleCancel}
                          leftIcon={<X size={14} />}
                        >
                          {t("app.cancel")}
                        </Button>
                      </>
                    )}
                  </div>
                </div>
              </div>
            </div>

            {/* Right Panel: Smart Queue */}
            <div className="hidden md:flex w-80 flex-col bg-slate-900/80 border-l border-slate-700/30">
              <div className="bg-slate-800/90 backdrop-blur p-3 flex items-center justify-between border-b border-slate-700 sticky top-0 z-10">
                <div className="flex items-center gap-2">
                  <ListOrdered size={16} className="text-emerald-400" />
                  <span className="text-xs font-bold text-slate-200 tracking-wide">
                    {t("app.smartQueueLabel")}
                  </span>
                  <span className="bg-slate-700 text-slate-300 px-1.5 rounded-full text-[10px]">
                    {batchQueue.length}
                  </span>
                </div>
                <div className="flex items-center gap-1">
                  <Button
                    size="sm"
                    variant={isBatchProcessing ? "amber" : "success"}
                    className="h-6 w-6 p-0"
                    onClick={toggleBatchProcessing}
                    title={isBatchProcessing ? t("queue.pauseDownload") : t("app.startQueue")}
                  >
                    {isBatchProcessing ? (
                      <Pause size={12} />
                    ) : (
                      <Play size={12} />
                    )}
                  </Button>
                  <Button
                    size="sm"
                    variant="ghost"
                    className="h-6 w-6 p-0"
                    onClick={() => {
                      setBatchQueue([]);
                      setIsBatchProcessing(false);
                    }}
                    title={t("app.clear")}
                  >
                    <X size={14} />
                  </Button>
                </div>
              </div>

              <div className="flex-1 overflow-y-auto custom-scrollbar p-2 space-y-2">
                {batchQueue.length === 0 ? (
                  <div className="h-full flex flex-col items-center justify-center text-slate-500 opacity-40">
                    <ListOrdered size={32} className="mb-2" />
                    <p className="text-xs">{t("queue.empty")}</p>
                  </div>
                ) : (
                  batchQueue.map((item, idx) => (
                    <div
                      key={idx}
                      className="p-2 rounded border border-slate-700/50 bg-slate-800/30 flex gap-2 text-xs relative group"
                    >
                      <div className="w-12 h-16 bg-slate-900 rounded overflow-hidden flex-shrink-0 relative">
                        {item.info?.posterUrl ? (
                          <img
                            src={item.info.posterUrl}
                            className="w-full h-full object-cover"
                          />
                        ) : (
                          <div className="w-full h-full flex items-center justify-center">
                            <ImageIcon size={14} className="text-slate-600" />
                          </div>
                        )}
                        {item.status === "downloading" && (
                          <div className="absolute inset-0 bg-black/40 flex items-center justify-center">
                            <Loader2
                              size={12}
                              className="animate-spin text-white"
                            />
                          </div>
                        )}
                        {item.status === "completed" && (
                          <div className="absolute inset-0 bg-black/40 flex items-center justify-center">
                            <CheckCircle
                              size={12}
                              className="text-emerald-400"
                            />
                          </div>
                        )}
                      </div>
                      <div className="flex-1 min-w-0 flex flex-col gap-1">
                        <div className="font-medium truncate text-slate-200">
                          {item.info?.title || item.url}
                        </div>
                        <div className="text-[10px] text-slate-400 flex items-center gap-1">
                          {item.status === "pending" && t("queue.pending")}
                          {item.status === "fetching" && t("app.fetchingInfo")}
                          {item.status === "ready" &&
                            `${t("app.ready")} (${item.info?.totalEpisodes})`}
                          {item.status === "downloading" && (
                            <span className="text-cyan-400 flex items-center gap-1">
                              {t("app.downloadingStatus")} {progress.percentage.toFixed(0)}%
                            </span>
                          )}
                          {item.status === "completed" && (
                            <span className="text-emerald-400">{t("app.completed")}</span>
                          )}
                          {item.status === "error" && (
                            <span className="text-red-400">{t("app.error")}</span>
                          )}
                        </div>
                        {item.status === "downloading" && (
                          <div className="h-1 bg-slate-700/50 rounded-full overflow-hidden mt-1 w-full">
                            <div
                              className="h-full bg-cyan-400/80 transition-all duration-300 shadow-[0_0_4px_currentColor]"
                              style={{ width: `${progress.percentage}%` }}
                            />
                          </div>
                        )}
                      </div>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          setBatchQueue((prev) =>
                            prev.filter((_, i) => i !== idx),
                          );
                        }}
                        className="absolute top-1 right-1 opacity-0 group-hover:opacity-100 hover:text-red-400 transition-opacity"
                        aria-label={t("app.removeFromBatch")}
                      >
                        <X size={12} />
                      </button>
                    </div>
                  ))
                )}
              </div>
            </div>
          </div>
        )}

        {activeTab === "library" && (
          <div className="page-transition animate-fade-in h-full overflow-y-auto custom-scrollbar">
            <ErrorBoundary>
              <LibraryPanel />
            </ErrorBoundary>
          </div>
        )}
        {activeTab === "browse" && (
          <div className="page-transition animate-fade-in h-full overflow-hidden">
            <ErrorBoundary>
              <BrowsePanel settings={settings} ffmpegAvailable={ffmpegAvailable} />
            </ErrorBoundary>
          </div>
        )}

        {activeTab === "files" && (
          <div className="page-transition animate-fade-in">
            <ErrorBoundary>
              <FileBrowser
                outputDir={settings.outputDir}
                files={files}
                onRefresh={refreshFiles}
                onOpenFolder={handleOpenOutputFolder}
                onDelete={handleDeleteFiles}
                onPlay={handlePlayFile}
              />
            </ErrorBoundary>
          </div>
        )}

        {activeTab === "history" && (
          <div className="page-transition animate-slide-in">
            <ErrorBoundary>
              <HistoryPanel
                history={history}
                stats={getStats()}
                onDelete={deleteRecord}
                onClear={clearHistory}
              />
            </ErrorBoundary>
          </div>
        )}

        {activeTab === "settings" && (
          <div className="page-transition animate-fade-in space-y-6 container mx-auto max-w-4xl">
            <ErrorBoundary>
              <SettingsPanel
                settings={settings}
                onUpdate={updateSetting}
                onReset={resetSettings}
                onOpenFolder={handleOpenOutputFolder}
                onCheckUpdates={checkForUpdates}
                isCheckingUpdates={isCheckingUpdates}
                language={language}
                onLanguageChange={setLanguage}
                t={t}
                themes={themes}
                activeThemeId={activeThemeId}
                onThemeSelect={setActiveTheme}
                domainSettings={domainSettings}
                onUpdateDomain={updateDomainSetting}
                onResetDomains={resetDomainSettings}
              />
              <SchedulerPanel />
              <BackupPanel />
            </ErrorBoundary>
          </div>
        )}


        {activeTab === "logs" && (
          <div className="h-full overflow-hidden page-transition animate-fade-in flex flex-col">
            <ErrorBoundary>
              <LogPanel logs={logs} onClear={clearLogs} />
            </ErrorBoundary>
          </div>
        )}
      </main>

      {/* Mini Mode Overlay */}
      <MiniMode
        isOpen={showMiniMode}
        onClose={() => setShowMiniMode(false)}
        onExpand={() => setShowMiniMode(false)}
        progress={progress}
        overallProgress={overallProgress}
        isDownloading={downloadState.isDownloading}
        currentEpisode={downloadState.currentEpisode}
        totalEpisodes={downloadState.totalSelected}
        isPaused={downloadState.isPaused}
        onPause={handlePause}
        onResume={handleResume}
        seriesTitle={series?.title}
      />

      {/* Shortcuts Help Modal */}
      <ShortcutsHelp
        isOpen={showShortcuts}
        onClose={() => setShowShortcuts(false)}
      />

      {/* Update Dialog Modal */}
      <UpdateDialog
        isOpen={updateAvailable}
        updateInfo={updateInfo}
        downloading={updateDownloading}
        progress={updateProgress}
        error={updateError}
        onDownload={downloadAndInstall}
        onDismiss={dismissUpdate}
      />

      {/* Toast Notifications */}
      <ToastContainer toasts={toasts} onRemove={removeToast} />
    </div>
  );
}

export default App;

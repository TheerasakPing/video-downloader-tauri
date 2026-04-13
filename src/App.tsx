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

  // Ref to always have the latest currentEpisode for pause/cancel handlers
  const currentEpisodeRef = useRef(downloadState.currentEpisode);
  currentEpisodeRef.current = downloadState.currentEpisode;

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
    if (!downloadState.isDownloading) return;
    setDownloadState((prev) => ({ ...prev, isPaused: true }));
    try {
      const ep = currentEpisodeRef.current;
      if (ep > 0) {
        await invoke("pause_download", { episode: ep });
      }
      log("Paused download");
    } catch (e) {
      log("Pause completed (download may have finished)");
    }
  }, [downloadState.isDownloading, log]);

  const handleResume = useCallback(async () => {
    if (!downloadState.isDownloading) return;
    setDownloadState((prev) => ({ ...prev, isPaused: false }));
    try {
      const ep = currentEpisodeRef.current;
      if (ep > 0) {
        await invoke("resume_download", { episode: ep });
      }
      log("Resumed download");
    } catch (e) {
      log("Resume completed (download may have finished)");
    }
  }, [downloadState.isDownloading, log]);

  const handleCancel = useCallback(async () => {
    if (!downloadState.isDownloading) return;
    setDownloadState((prev) => ({
      ...prev,
      isDownloading: false,
      isPaused: false,
    }));
    try {
      const ep = currentEpisodeRef.current;
      if (ep > 0) {
        await invoke("cancel_download", { episode: ep });
      }
      warning("Cancelled download");
      setQueue([]);
    } catch (e) {
      error(`Failed to cancel: ${e}`);
    }
  }, [downloadState.isDownloading, warning, error]);

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
          <div className="h-full flex flex-col overflow-hidden">
            {/* Main scrollable content */}
            <div className="flex-1 overflow-y-auto custom-scrollbar relative">
              <div className="max-w-4xl mx-auto p-4 space-y-4">

                {/* ===== HERO URL BAR ===== */}
                <div className="relative">
                  <div className="absolute inset-0 bg-gradient-to-b from-cyan-500/5 via-violet-500/5 to-transparent rounded-2xl pointer-events-none" />
                  <div className="relative bg-slate-800/60 backdrop-blur-sm rounded-xl border border-slate-700/60 p-3 space-y-2.5">
                    <div className="flex items-center gap-2 mb-1">
                      <div className="w-1.5 h-1.5 rounded-full bg-cyan-400 shadow-[0_0_6px_rgba(34,211,238,0.6)]" />
                      <span className="text-[10px] font-bold tracking-widest text-slate-400 uppercase">Video URL</span>
                    </div>
                    <div className="relative">
                      <Input
                        placeholder="Paste video URL here..."
                        value={url}
                        onChange={(e) => setUrl(e.target.value)}
                        leftIcon={<Link size={15} />}
                        iconColor="cyan"
                        className="text-sm py-2.5 bg-slate-900/60"
                        rightElement={
                          <div className="flex gap-1 items-center pr-1">
                            <Button
                              size="sm"
                              variant="ghost"
                              onClick={() => {
                                setUrl("");
                                setSeries(null);
                                setSelectedEpisodes(new Set());
                              }}
                              disabled={!url}
                              className="h-7 w-7 p-0 text-slate-500 hover:text-slate-300"
                            >
                              <X size={14} />
                            </Button>
                            <Button
                              size="sm"
                              variant="ghost"
                              onClick={handlePaste}
                              className="h-7 px-2 text-cyan-400 hover:text-cyan-300 hover:bg-cyan-500/10"
                              title={t("app.paste")}
                            >
                              <Clipboard size={14} />
                            </Button>
                            <Button
                              size="sm"
                              onClick={handleFetch}
                              isLoading={isFetching}
                              className="h-7 px-3 bg-cyan-500/20 text-cyan-300 border-cyan-500/40 hover:bg-cyan-500/30 hover:border-cyan-400/50 shadow-[0_0_12px_rgba(34,211,238,0.15)]"
                            >
                              <Search size={14} />
                              <span className="text-[10px] font-bold">Fetch</span>
                            </Button>
                          </div>
                        }
                      />
                    </div>
                    <div className="flex items-center gap-2">
                      <FolderOpen size={12} className="text-amber-400/70" />
                      <input
                        type="text"
                        placeholder="Save to..."
                        value={settings.outputDir}
                        onChange={(e) => updateSetting("outputDir", e.target.value)}
                        className="flex-1 bg-transparent text-[11px] text-slate-400 placeholder-slate-600 border-none outline-none"
                      />
                      <button
                        onClick={handleSelectOutputFolder}
                        className="text-[10px] text-slate-500 hover:text-amber-400 transition-colors px-1"
                      >
                        Change
                      </button>
                      <button
                        onClick={handleOpenOutputFolder}
                        className="text-[10px] text-slate-500 hover:text-amber-400 transition-colors px-1"
                      >
                        Open
                      </button>
                    </div>
                  </div>
                </div>

                {/* ===== SMART QUEUE BAR ===== */}
                <div className={`flex items-center justify-between px-3 py-2 rounded-lg border transition-all duration-200 ${
                  batchQueue.length > 0
                    ? "bg-emerald-500/5 border-emerald-500/30 shadow-[0_0_15px_rgba(16,185,129,0.08)]"
                    : "bg-slate-800/40 border-slate-700/40 opacity-70 hover:opacity-100"
                }`}>
                  <div className="flex items-center gap-2.5">
                    <ListOrdered
                      size={14}
                      className={`transition-colors ${batchQueue.length > 0 ? "text-emerald-400 drop-shadow-[0_0_6px_rgba(52,211,153,0.5)]" : "text-slate-500"}`}
                    />
                    <span className="text-[11px] font-bold text-slate-300 tracking-wide">
                      Smart Queue
                    </span>
                    {batchQueue.length > 0 && (
                      <span className="bg-emerald-500/15 text-emerald-400 px-1.5 py-0.5 rounded-full text-[10px] font-bold border border-emerald-500/25">
                        {batchQueue.length}
                      </span>
                    )}
                    {isBatchProcessing && (
                      <span className="text-[10px] bg-emerald-500/10 text-emerald-400 px-2 py-0.5 rounded-full flex items-center gap-1 animate-pulse border border-emerald-500/20 font-medium">
                        <Loader2 size={10} className="animate-spin" /> Running
                      </span>
                    )}
                  </div>
                  <div className="flex items-center gap-1">
                    <Button
                      size="sm"
                      variant={isAutoCapture ? "cyan" : "ghost"}
                      onClick={() => setIsAutoCapture((p) => !p)}
                      title="Auto Capture Clipboard"
                      className={`h-6 text-[10px] px-1.5 ${isAutoCapture ? "ring-1 ring-cyan-500/40" : "text-slate-500 hover:text-cyan-400"}`}
                    >
                      <Clipboard size={12} className={isAutoCapture ? "animate-pulse" : ""} />
                    </Button>
                    <div className="w-px h-3.5 bg-slate-700/50 mx-0.5" />
                    <Button
                      size="sm"
                      variant={isBatchProcessing ? "amber" : "success"}
                      onClick={toggleBatchProcessing}
                      title={isBatchProcessing ? t("queue.pauseDownload") : t("app.startQueue")}
                      className="h-6 w-6 p-0"
                    >
                      {isBatchProcessing ? <Pause size={11} /> : <Play size={11} />}
                    </Button>
                    <Button
                      size="sm"
                      variant={isBatchMode ? "primary" : "ghost"}
                      onClick={() => setIsBatchMode((p) => !p)}
                      title={isBatchMode ? "Hide queue" : "Show queue list"}
                      className={`h-6 w-6 p-0 ${isBatchMode ? "shadow-md shadow-violet-500/15" : "text-slate-500 hover:text-white"}`}
                    >
                      {isBatchMode ? <Minimize2 size={11} /> : <ListOrdered size={11} />}
                    </Button>
                    <Button
                      size="sm"
                      variant="ghost"
                      className="h-6 w-6 p-0 text-slate-500 hover:text-red-400 hover:bg-red-500/10"
                      onClick={() => {
                        if (batchQueue.length === 0 || confirm("Clear entire queue?")) {
                          setBatchQueue([]);
                          setIsBatchMode(false);
                          setIsBatchProcessing(false);
                        }
                      }}
                    >
                      <X size={11} />
                    </Button>
                  </div>
                </div>

                {/* ===== DETECTION PROGRESS ===== */}
                {detectionState.isDetecting && (
                  <div className="bg-violet-500/5 rounded-lg p-3 border border-violet-500/20 animate-pulse">
                    <div className="flex justify-between text-xs text-slate-400 mb-1.5">
                      <span className="flex items-center gap-2">
                        <Search size={12} className="animate-spin text-violet-400" />
                        {detectionState.message}
                      </span>
                      <span className="font-mono text-violet-300">{detectionState.progress}%</span>
                    </div>
                    <div className="h-1 bg-slate-700/50 rounded-full overflow-hidden">
                      <div
                        className="h-full bg-gradient-to-r from-violet-500 to-fuchsia-500 rounded-full transition-all duration-300"
                        style={{ width: `${detectionState.progress}%` }}
                      />
                    </div>
                  </div>
                )}

                {/* ===== SERIES INFO ===== */}
                <SeriesCard series={series} isLoading={isFetching} />

                {/* ===== EPISODE SELECTOR ===== */}
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

                {/* ===== QUALITY SELECTOR ===== */}
                {series && (
                  <QualitySelector
                    episodeUrl={Object.values(series.episodeUrls)[0]}
                    onSelect={(q) => setSelectedQuality(q)}
                    defaultQuality={settings.defaultQuality || 'best'}
                  />
                )}

                {/* ===== PROGRESS DASHBOARD ===== */}
                {(downloadState.isDownloading || speedData.length > 0) && (
                  <div className="bg-slate-800/40 backdrop-blur-sm rounded-xl border border-cyan-500/20 overflow-hidden">
                    <div className="px-3 py-2 border-b border-slate-700/50 flex items-center gap-2">
                      <div className="w-1.5 h-1.5 rounded-full bg-cyan-400 shadow-[0_0_6px_rgba(34,211,238,0.6)] animate-pulse" />
                      <span className="text-[10px] font-bold tracking-widest text-cyan-400 uppercase">Download Progress</span>
                      <span className="ml-auto text-[10px] font-mono text-slate-500">
                        {downloadState.completedEpisodes.length}/{downloadState.totalSelected} eps
                      </span>
                    </div>
                    <div className="p-3 space-y-3">
                      <div className="grid grid-cols-1 lg:grid-cols-2 gap-3">
                        <SpeedGraph
                          data={speedData}
                          currentSpeed={currentSpeed}
                          avgSpeed={avgSpeed}
                          peakSpeed={peakSpeed}
                        />
                        <div className="space-y-2.5">
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
                            <div className="p-2 bg-fuchsia-500/10 rounded-lg border border-fuchsia-500/20">
                              <div className="flex items-center justify-between text-xs mb-1">
                                <span className="text-fuchsia-300 flex items-center gap-1.5">
                                  <Merge size={12} className="animate-pulse drop-shadow-[0_0_4px_currentColor]" />
                                  {t("app.mergingVideos")}
                                </span>
                                <span className="text-fuchsia-400 font-mono">{mergeState.progress.toFixed(0)}%</span>
                              </div>
                              <div className="h-1.5 bg-slate-700/50 rounded-full overflow-hidden">
                                <div
                                  className="h-full bg-gradient-to-r from-fuchsia-500 to-violet-500 rounded-full transition-all"
                                  style={{ width: `${mergeState.progress}%` }}
                                />
                              </div>
                            </div>
                          )}
                          {mergeState.mergedFile && !mergeState.isMerging && (
                            <div className="p-2 bg-emerald-500/10 rounded-lg border border-emerald-500/20 text-xs text-emerald-300 flex items-center gap-1.5">
                              <CheckCircle size={12} className="text-emerald-400" />
                              {t("app.mergedLabel")} {mergeState.mergedFile.split("/").pop()}
                            </div>
                          )}
                        </div>
                      </div>
                    </div>
                  </div>
                )}

                {/* ===== DOWNLOAD QUEUE ===== */}
                {queue.length > 0 && (
                  <ErrorBoundary>
                    <DownloadQueue
                      queue={queue}
                      onMoveUp={() => {}}
                      onMoveDown={() => {}}
                      onRemove={(id) => setQueue((prev) => prev.filter((q) => q.id !== id))}
                      onPause={() => {}}
                    />
                  </ErrorBoundary>
                )}

                {/* Spacer for sticky action bar */}
                <div className="h-16" />
              </div>
            </div>

            {/* ===== STICKY ACTION BAR ===== */}
            <div className="flex-shrink-0 border-t border-slate-700/50 bg-slate-900/95 backdrop-blur-md px-4 py-2.5">
              <div className="max-w-4xl mx-auto flex items-center justify-between gap-3">
                <label className="flex items-center gap-2 cursor-pointer text-xs text-slate-400 hover:text-slate-300 transition-colors select-none">
                  <input
                    type="checkbox"
                    checked={settings.autoMerge}
                    onChange={(e) => updateSetting("autoMerge", e.target.checked)}
                    disabled={!ffmpegAvailable}
                    className="w-3.5 h-3.5 rounded bg-slate-700 border-slate-600 text-fuchsia-600 focus:ring-fuchsia-500/30"
                  />
                  <Merge size={13} className="text-fuchsia-400/70" />
                  <span>{t("app.autoMerge")}</span>
                </label>

                <div className="flex items-center gap-2">
                  {!downloadState.isDownloading ? (
                    <Button
                      onClick={handleStartDownload}
                      disabled={!series || selectedEpisodes.size === 0}
                      leftIcon={<Download size={15} />}
                      variant="success"
                      size="md"
                      className="px-5 py-2 shadow-[0_0_20px_rgba(16,185,129,0.2)] hover:shadow-[0_0_30px_rgba(16,185,129,0.35)]"
                    >
                      Download{selectedEpisodes.size > 0 ? ` (${selectedEpisodes.size})` : ""}
                    </Button>
                  ) : (
                    <>
                      {!downloadState.isPaused ? (
                        <Button
                          variant="amber"
                          onClick={handlePause}
                          leftIcon={<Pause size={14} />}
                          size="md"
                          className="px-4"
                        >
                          {t("app.pause")}
                        </Button>
                      ) : (
                        <Button
                          variant="success"
                          onClick={handleResume}
                          leftIcon={<Play size={14} />}
                          size="md"
                          className="px-4 shadow-[0_0_15px_rgba(16,185,129,0.2)]"
                        >
                          {t("app.resume")}
                        </Button>
                      )}
                      <Button
                        variant="danger"
                        onClick={handleCancel}
                        leftIcon={<X size={14} />}
                        size="md"
                        className="px-4"
                      >
                        {t("app.cancel")}
                      </Button>
                    </>
                  )}
                </div>
              </div>
            </div>

            {/* ===== BATCH QUEUE DRAWER ===== */}
            {isBatchMode && (
              <div className="fixed inset-y-0 right-0 w-80 bg-slate-900/98 backdrop-blur-xl border-l border-slate-700/50 shadow-2xl shadow-black/50 z-50 flex flex-col animate-slide-in-right">
                <div className="px-3 py-2.5 border-b border-slate-700/50 flex items-center justify-between bg-slate-800/60">
                  <div className="flex items-center gap-2">
                    <ListOrdered size={14} className="text-emerald-400" />
                    <span className="text-xs font-bold text-slate-200 tracking-wide">Queue</span>
                    <span className="bg-slate-700/80 text-slate-300 px-1.5 py-0.5 rounded-full text-[10px]">
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
                      {isBatchProcessing ? <Pause size={11} /> : <Play size={11} />}
                    </Button>
                    <Button
                      size="sm"
                      variant="ghost"
                      className="h-6 w-6 p-0 text-slate-500 hover:text-red-400"
                      onClick={() => {
                        setBatchQueue([]);
                        setIsBatchProcessing(false);
                      }}
                    >
                      <X size={12} />
                    </Button>
                    <Button
                      size="sm"
                      variant="ghost"
                      className="h-6 w-6 p-0 text-slate-400 hover:text-white"
                      onClick={() => setIsBatchMode(false)}
                    >
                      <X size={14} />
                    </Button>
                  </div>
                </div>

                <div className="flex-1 overflow-y-auto custom-scrollbar p-2 space-y-1.5">
                  {batchQueue.length === 0 ? (
                    <div className="h-full flex flex-col items-center justify-center text-slate-500 opacity-40">
                      <ListOrdered size={28} className="mb-2" />
                      <p className="text-[11px]">{t("queue.empty")}</p>
                    </div>
                  ) : (
                    batchQueue.map((item, idx) => (
                      <div
                        key={idx}
                        className={`p-2 rounded-lg border flex gap-2.5 text-xs relative group transition-all ${
                          item.status === "downloading"
                            ? "bg-cyan-500/5 border-cyan-500/30 shadow-[0_0_12px_rgba(34,211,238,0.08)]"
                            : item.status === "completed"
                            ? "bg-emerald-500/5 border-emerald-500/20"
                            : item.status === "error" || item.status === "failed"
                            ? "bg-red-500/5 border-red-500/20"
                            : "bg-slate-800/30 border-slate-700/40"
                        }`}
                      >
                        <div className="w-11 h-14 bg-slate-900/80 rounded overflow-hidden flex-shrink-0 relative">
                          {item.info?.posterUrl ? (
                            <img src={item.info.posterUrl} className="w-full h-full object-cover" />
                          ) : (
                            <div className="w-full h-full flex items-center justify-center">
                              <ImageIcon size={14} className="text-slate-600" />
                            </div>
                          )}
                          {item.status === "downloading" && (
                            <div className="absolute inset-0 bg-black/40 flex items-center justify-center">
                              <Loader2 size={12} className="animate-spin text-cyan-400" />
                            </div>
                          )}
                          {item.status === "completed" && (
                            <div className="absolute inset-0 bg-black/30 flex items-center justify-center">
                              <CheckCircle size={12} className="text-emerald-400" />
                            </div>
                          )}
                        </div>
                        <div className="flex-1 min-w-0 flex flex-col gap-0.5">
                          <div className="font-medium truncate text-slate-200 text-[11px]">
                            {item.info?.title || item.url}
                          </div>
                          <div className="text-[10px] text-slate-400">
                            {item.status === "pending" && t("queue.pending")}
                            {item.status === "fetching" && t("app.fetchingInfo")}
                            {item.status === "ready" && `${t("app.ready")} (${item.info?.totalEpisodes})`}
                            {item.status === "downloading" && (
                              <span className="text-cyan-400">{t("app.downloadingStatus")} {progress.percentage.toFixed(0)}%</span>
                            )}
                            {item.status === "completed" && (
                              <span className="text-emerald-400">{t("app.completed")}</span>
                            )}
                            {item.status === "error" && (
                              <span className="text-red-400">{t("app.error")}</span>
                            )}
                          </div>
                          {item.status === "downloading" && (
                            <div className="h-1 bg-slate-700/50 rounded-full overflow-hidden mt-0.5">
                              <div
                                className="h-full bg-cyan-400/80 transition-all duration-300 shadow-[0_0_4px_currentColor] rounded-full"
                                style={{ width: `${progress.percentage}%` }}
                              />
                            </div>
                          )}
                        </div>
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            setBatchQueue((prev) => prev.filter((_, i) => i !== idx));
                          }}
                          className="absolute top-1 right-1 opacity-0 group-hover:opacity-100 hover:text-red-400 transition-opacity text-slate-600"
                          aria-label={t("app.removeFromBatch")}
                        >
                          <X size={11} />
                        </button>
                      </div>
                    ))
                  )}
                </div>
              </div>
            )}
            {isBatchMode && (
              <div
                className="fixed inset-0 bg-black/30 z-40"
                onClick={() => setIsBatchMode(false)}
              />
            )}
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

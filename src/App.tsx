import React, { useState, useEffect, useCallback } from "react";
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
} from "./components";
import { useLogger } from "./hooks/useLogger";
import { useSettings } from "./hooks/useSettings";
import { useHistory } from "./hooks/useHistory";
import { useSpeedGraph } from "./hooks/useSpeedGraph";
import { useUpdater } from "./hooks/useUpdater";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
import { useI18n } from "./hooks/useI18n";
import { useCustomTheme } from "./hooks/useCustomTheme";
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

type TabType = "download" | "files" | "history" | "settings" | "logs";

function App() {
  // State
  const [url, setUrl] = useState("");
  const [series, setSeries] = useState<SeriesInfo | null>(null);
  const [selectedEpisodes, setSelectedEpisodes] = useState<Set<number>>(new Set());
  const [isFetching, setIsFetching] = useState(false);
  const [activeTab, setActiveTab] = useState<TabType>("download");
  const [ffmpegAvailable, setFfmpegAvailable] = useState(false);
  const [files, setFiles] = useState<FileInfo[]>([]);
  const [queue, setQueue] = useState<QueueItem[]>([]);
  const [mergeState, setMergeState] = useState<{
    isMerging: boolean;
    mergedFile: string | null;
    mergeError: string | null;
    progress: number;
    currentTime: number;
    totalDuration: number;
  }>({ isMerging: false, mergedFile: null, mergeError: null, progress: 0, currentTime: 0, totalDuration: 0 });

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

  // Hooks
  const { logs, log, success, warning, error, clearLogs } = useLogger();
  const { settings, updateSetting, resetSettings } = useSettings();
  const { history, addRecord, updateRecord, deleteRecord, clearHistory, getStats } = useHistory();
  const { speedData, currentSpeed, avgSpeed, peakSpeed, addDataPoint, reset: resetSpeedGraph } = useSpeedGraph();
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

  // Tab navigation
  const tabs: TabType[] = ["download", "files", "history", "settings", "logs"];
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
      setUrl(text);
      log(`Pasted URL: ${text}`);

      // Auto-fetch if valid URL
      if (text.includes("rongyok.com") || text.includes("thongyok.com")) {
        setIsFetching(true);
        try {
          const result = await invoke<SeriesInfo>("fetch_series", { url: text });
          setSeries(result);
          const allEpisodes = new Set(
            Array.from({ length: result.totalEpisodes }, (_, i) => i + 1)
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
  }, [log, error, success]);

  const handleStartDownload = useCallback(async () => {
    if (!series || selectedEpisodes.size === 0) {
      error("Please select at least one episode");
      return;
    }

    const episodes = Array.from(selectedEpisodes).sort((a, b) => a - b);
    log(`Starting download of ${episodes.length} episodes`);

    const recordId = addRecord({
      seriesId: series.seriesId,
      seriesTitle: series.title,
      episodes,
      completedEpisodes: [],
      failedEpisodes: [],
      startTime: new Date().toISOString(),
      totalSize: 0,
      status: "partial",
    });

    setQueue(
      episodes.map((ep, i) => ({
        id: `${series.seriesId}-${ep}`,
        seriesId: series.seriesId,
        seriesTitle: series.title,
        episode: ep,
        status: i === 0 ? "downloading" : "pending",
        progress: 0,
        priority: i,
      }))
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
      const results = await invoke<DownloadResult[]>("start_download", {
        request: {
          seriesId: series.seriesId,
          episodes,
          outputDir: settings.outputDir,
          autoMerge: settings.autoMerge && ffmpegAvailable,
          concurrentDownloads: settings.concurrentDownloads,
          speedLimit: settings.speedLimit,
          fileNaming: settings.fileNaming,
          seriesTitle: series.title,
        },
      });

      const successCount = results.filter((r) => r.success).length;
      const failCount = results.filter((r) => !r.success).length;
      const totalSize = 100 * 1024 * 1024 * successCount;

      updateRecord(recordId, {
        completedEpisodes: results.filter((r) => r.success).map((r) => r.episode),
        failedEpisodes: results.filter((r) => !r.success).map((r) => r.episode),
        endTime: new Date().toISOString(),
        totalSize,
        status: failCount === 0 ? "completed" : failCount === episodes.length ? "failed" : "partial",
      });

      if (failCount === 0) {
        success(`All ${successCount} episodes downloaded successfully!`);
        showNotification("Download Complete", `${successCount} episodes downloaded`);
        playNotificationSound();
      } else {
        warning(`Downloaded ${successCount}/${episodes.length} episodes (${failCount} failed)`);
      }

      refreshFiles();
    } catch (e) {
      error(`Download failed: ${e}`);
    } finally {
      setDownloadState((prev) => ({ ...prev, isDownloading: false }));
      setQueue([]);
    }
  }, [series, selectedEpisodes, settings, ffmpegAvailable, addRecord, updateRecord, log, success, warning, error, resetSpeedGraph]);

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
      await invoke("resume_download", { episode: downloadState.currentEpisode });
      log("Resumed download");
    } catch (e) {
      // Download may have completed, don't show error
      log("Resume completed (download may have finished)");
    }
  }, [downloadState.currentEpisode, log, warning]);

  const handleCancel = useCallback(async () => {
    if (downloadState.currentEpisode === 0) return;
    setDownloadState((prev) => ({ ...prev, isDownloading: false, isPaused: false }));
    try {
      await invoke("cancel_download", { episode: downloadState.currentEpisode });
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

  // Helper to check if URL is valid for this app
  const isValidSeriesUrl = useCallback((text: string): boolean => {
    if (!text) return false;
    return text.includes("rongyok.com") || text.includes("thongyok.com");
  }, []);

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
          const result = await invoke<SeriesInfo>("fetch_series", { url: text });
          setSeries(result);
          const allEpisodes = new Set(
            Array.from({ length: result.totalEpisodes }, (_, i) => i + 1)
          );
          setSelectedEpisodes(allEpisodes);
          success(`Auto-loaded: ${result.title} (${result.totalEpisodes} episodes)`);
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
    if (initialized.current) return;
    initialized.current = true;

    log("Application started");
    checkFFmpeg();
    setupEventListeners();

    // Auto-paste from clipboard on startup
    autoFetchFromClipboard();
  }, []);

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
      const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
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
    await listen<DownloadProgress>("download-progress", (event) => {
      setProgress(event.payload);
      addDataPoint(event.payload.speed);
      setDownloadState((prev) => ({
        ...prev,
        currentEpisode: event.payload.episode,
      }));
    });

    await listen<DownloadResult>("download-result", (event) => {
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
            ? { ...q, status: result.success ? "completed" : "failed", progress: 100 }
            : q
        )
      );
    });

    await listen("merge-started", () => {
      log("Merging videos...");
      setMergeState({ isMerging: true, mergedFile: null, mergeError: null, progress: 0, currentTime: 0, totalDuration: 0 });
    });

    await listen<{ percentage: number; currentTime: number; totalDuration: number }>("merge-progress", (event) => {
      setMergeState((prev) => ({
        ...prev,
        progress: event.payload.percentage,
        currentTime: event.payload.currentTime,
        totalDuration: event.payload.totalDuration,
      }));
    });

    await listen<string>("merge-complete", (event) => {
      success(`Merged to: ${event.payload}`);
      setMergeState({ isMerging: false, mergedFile: event.payload, mergeError: null, progress: 100, currentTime: 0, totalDuration: 0 });
      playNotificationSound();
      showNotification("Merge Complete", "Videos merged successfully!");
      refreshFiles();
    });

    await listen<string>("merge-error", (event) => {
      error(`Merge failed: ${event.payload}`);
      setMergeState({ isMerging: false, mergedFile: null, mergeError: event.payload, progress: 0, currentTime: 0, totalDuration: 0 });
    });

    await listen<string>("log-info", (event) => {
      log(event.payload);
    });
  };

  const checkFFmpeg = async () => {
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
    log(`Fetching: ${url}`);

    try {
      const result = await invoke<SeriesInfo>("fetch_series", { url });
      setSeries(result);

      const allEpisodes = new Set(
        Array.from({ length: result.totalEpisodes }, (_, i) => i + 1)
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
        new Set(Array.from({ length: series.totalEpisodes }, (_, i) => i + 1))
      );
    }
  };

  const deselectAllEpisodes = () => {
    setSelectedEpisodes(new Set());
  };

  const refreshFiles = async () => {
    try {
      const fileList = await invoke<FileInfo[]>("list_files", { dir: settings.outputDir });
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
        title: "Select Output Folder",
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
    if (text && (text.includes("rongyok.com") || text.includes("thongyok.com"))) {
      setUrl(text);
      log(`Dropped URL: ${text}`);
      success("URL added from drop");

      // Auto-fetch
      setIsFetching(true);
      try {
        const result = await invoke<SeriesInfo>("fetch_series", { url: text });
        setSeries(result);
        const allEpisodes = new Set(
          Array.from({ length: result.totalEpisodes }, (_, i) => i + 1)
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
      ? (downloadState.completedEpisodes.length / downloadState.totalSelected) * 100
      : 0;

  const tabsConfig: { id: TabType; label: string; icon: React.ReactNode; glowColor: string }[] = [
    { id: "download", label: "Download", icon: <Download size={16} />, glowColor: "violet" },
    { id: "files", label: "Files", icon: <HardDrive size={16} />, glowColor: "blue" },
    { id: "history", label: "History", icon: <Clock size={16} />, glowColor: "amber" },
    { id: "settings", label: "Settings", icon: <Settings size={16} />, glowColor: "slate" },
    { id: "logs", label: `Logs (${logs.length})`, icon: <AlertCircle size={16} />, glowColor: "cyan" },
  ];

  return (
    <div
      className="min-h-screen text-white"
      style={{ background: 'var(--bg-primary, linear-gradient(to bottom right, #0f172a, #1e293b, #0f172a))' }}
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
              <span className="text-xl font-medium text-violet-300">Drop URL here</span>
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
                    <span className="text-emerald-400 text-[10px]">FFmpeg</span>
                  )}
                  {downloadState.isDownloading && currentSpeed > 0 && (
                    <span className="text-violet-400 text-[10px]">{(currentSpeed / 1024 / 1024).toFixed(1)} MB/s</span>
                  )}
                </h1>
              </div>
            </div>

            {/* Header Actions */}
            <div className="flex items-center gap-1">
              {downloadState.isDownloading && (
                <button onClick={() => setShowMiniMode(true)} className="p-1.5 hover:bg-slate-700/50 rounded-md" title="Mini Mode">
                  <Minimize2 size={14} className="text-slate-400" />
                </button>
              )}
              <button onClick={() => setShowShortcuts(true)} className="p-1.5 hover:bg-slate-700/50 rounded-md" title="Shortcuts">
                <Keyboard size={14} className="text-slate-400" />
              </button>

              {/* Tabs */}
              <div className="flex bg-slate-800/70 rounded-lg p-0.5 ml-1">
                {tabsConfig.map((tab) => (
                  <button
                    key={tab.id}
                    onClick={() => setActiveTab(tab.id)}
                    className={`px-2 py-1 text-xs font-medium rounded-md transition-all flex items-center gap-1 ${
                      activeTab === tab.id
                        ? "bg-slate-700 text-white"
                        : "text-slate-400 hover:text-white"
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
      <main className="container-responsive py-2 sm:py-3 space-y-2 sm:space-y-3">
        {activeTab === "download" && (
          <div className="space-y-2 sm:space-y-3">
            {/* URL Input - Compact */}
            <div className="space-y-2">
              <Input
                placeholder="https://rongyok.com/watch/?series_id=XXX"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                leftIcon={<Link size={14} />}
                rightElement={
                  <div className="flex gap-0.5 items-center">
                    <Button size="sm" variant="ghost" onClick={() => { setUrl(""); setSeries(null); setSelectedEpisodes(new Set()); }} disabled={!url} className="px-1.5">
                      <X size={14} />
                    </Button>
                    <Button size="sm" variant="ghost" onClick={handlePaste} className="px-1.5" title="Paste">
                      <Clipboard size={14} />
                    </Button>
                    <Button size="sm" onClick={handleFetch} isLoading={isFetching} className="px-2">
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
                rightElement={
                  <div className="flex gap-0.5">
                    <Button size="sm" variant="ghost" onClick={handleSelectOutputFolder} className="px-1.5">📂</Button>
                    <Button size="sm" variant="ghost" onClick={handleOpenOutputFolder} className="px-1.5">↗</Button>
                  </div>
                }
              />
            </div>

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
                  />
                  <ProgressBar
                    percentage={overallProgress}
                    label="Overall"
                    sublabel={`${downloadState.completedEpisodes.length}/${downloadState.totalSelected}`}
                    variant="success"
                  />
                  {mergeState.isMerging && (
                    <div className="p-2 bg-violet-500/20 rounded-md border border-violet-500/30">
                      <div className="flex items-center justify-between text-xs">
                        <span className="text-violet-300 flex items-center gap-1">
                          <Merge size={12} className="animate-pulse" /> Merging...
                        </span>
                        <span className="text-violet-400 font-mono">{mergeState.progress.toFixed(0)}%</span>
                      </div>
                      <div className="h-1.5 bg-slate-700 rounded-full overflow-hidden mt-1">
                        <div className="h-full bg-gradient-to-r from-violet-500 to-fuchsia-500 rounded-full" style={{ width: `${mergeState.progress}%` }} />
                      </div>
                    </div>
                  )}
                  {mergeState.mergedFile && !mergeState.isMerging && (
                    <div className="p-2 bg-emerald-500/20 rounded-md border border-emerald-500/30 text-xs text-emerald-300">
                      ✅ Merged: {mergeState.mergedFile.split('/').pop()}
                    </div>
                  )}
                </div>
              </div>
            )}

            {/* Queue - Compact */}
            {queue.length > 0 && (
              <DownloadQueue
                queue={queue}
                onMoveUp={() => {}}
                onMoveDown={() => {}}
                onRemove={(id) => setQueue((prev) => prev.filter((q) => q.id !== id))}
                onPause={() => {}}
              />
            )}

            {/* Options & Actions - Compact inline */}
            <div className="flex flex-wrap items-center justify-between gap-2">
              <label className="flex items-center gap-1.5 cursor-pointer text-xs text-slate-300">
                <input
                  type="checkbox"
                  checked={settings.autoMerge}
                  onChange={(e) => updateSetting("autoMerge", e.target.checked)}
                  disabled={!ffmpegAvailable}
                  className="w-3.5 h-3.5 rounded bg-slate-700 border-slate-600 text-violet-600"
                />
                <Merge size={12} /> Auto merge
              </label>

              <div className="flex gap-2">
                {!downloadState.isDownloading ? (
                  <Button
                    onClick={handleStartDownload}
                    disabled={!series || selectedEpisodes.size === 0}
                    leftIcon={<Download size={14} />}
                    className="btn-glow-violet"
                  >
                    Download ({selectedEpisodes.size})
                  </Button>
                ) : (
                  <>
                    {!downloadState.isPaused ? (
                      <Button variant="secondary" onClick={handlePause} leftIcon={<Pause size={14} />}>Pause</Button>
                    ) : (
                      <Button variant="success" onClick={handleResume} leftIcon={<Play size={14} />}>Resume</Button>
                    )}
                    <Button variant="danger" onClick={handleCancel} leftIcon={<X size={14} />}>Cancel</Button>
                  </>
                )}
              </div>
            </div>
          </div>
        )}

        {activeTab === "files" && (
          <div className="page-transition animate-fade-in">
            <FileBrowser
              outputDir={settings.outputDir}
              files={files}
              onRefresh={refreshFiles}
              onOpenFolder={handleOpenOutputFolder}
              onDelete={handleDeleteFiles}
              onPlay={handlePlayFile}
            />
          </div>
        )}

        {activeTab === "history" && (
          <div className="page-transition animate-slide-in">
            <HistoryPanel
              history={history}
              stats={getStats()}
              onDelete={deleteRecord}
              onClear={clearHistory}
            />
          </div>
        )}

        {activeTab === "settings" && (
          <div className="page-transition animate-fade-in">
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
            />
          </div>
        )}

        {activeTab === "logs" && (
          <div className="h-[calc(100vh-140px)] sm:h-[calc(100vh-180px)] page-transition animate-fade-in">
            <LogPanel logs={logs} onClear={clearLogs} />
          </div>
        )}
      </main>

      {/* Mini Mode */}
      <MiniMode
        isOpen={showMiniMode && downloadState.isDownloading}
        onClose={() => setShowMiniMode(false)}
        onExpand={() => setShowMiniMode(false)}
        progress={progress}
        overallProgress={overallProgress}
        completedEpisodes={downloadState.completedEpisodes.length}
        totalEpisodes={downloadState.totalSelected}
        isPaused={downloadState.isPaused}
        onPause={handlePause}
        onResume={handleResume}
        seriesTitle={series?.title}
      />

      {/* Shortcuts Help */}
      <ShortcutsHelp
        isOpen={showShortcuts}
        onClose={() => setShowShortcuts(false)}
      />

      {/* Update Dialog */}
      <UpdateDialog
        isOpen={updateAvailable}
        updateInfo={updateInfo}
        downloading={updateDownloading}
        progress={updateProgress}
        error={updateError}
        onDownload={downloadAndInstall}
        onDismiss={dismissUpdate}
      />
    </div>
  );
}

export default App;

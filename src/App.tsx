import React, { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
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
  RefreshCw,
  Settings,
  Film,
  Zap,
  CheckCircle2,
  AlertCircle,
  Clock,
  HardDrive,
  Minimize2,
  Keyboard,
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
  PresetSelector,
} from "./components";
import { useLogger } from "./hooks/useLogger";
import { useSettings } from "./hooks/useSettings";
import { useHistory } from "./hooks/useHistory";
import { useSpeedGraph } from "./hooks/useSpeedGraph";
import { useUpdater } from "./hooks/useUpdater";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
import { useDownloadPresets } from "./hooks/useDownloadPresets";
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
  }>({ isMerging: false, mergedFile: null, mergeError: null });

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
  const { presets, activePresetId, applyPreset } = useDownloadPresets();
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
      const text = await navigator.clipboard.readText();
      setUrl(text);
      log(`Pasted URL: ${text}`);
    } catch (e) {
      error("Failed to read clipboard");
    }
  }, [log, error]);

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

  const handlePause = useCallback(() => {
    setDownloadState((prev) => ({ ...prev, isPaused: true }));
    warning("Pause not yet implemented");
  }, [warning]);

  const handleResume = useCallback(() => {
    setDownloadState((prev) => ({ ...prev, isPaused: false }));
    log("Resume not yet implemented");
  }, [log]);

  const handleCancel = useCallback(() => {
    setDownloadState((prev) => ({ ...prev, isDownloading: false }));
    warning("Cancel requested");
  }, [warning]);

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

  // Initialize
  const initialized = React.useRef(false);
  useEffect(() => {
    if (initialized.current) return;
    initialized.current = true;

    log("Application started");
    checkFFmpeg();
    setupEventListeners();
  }, []);

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
      setMergeState({ isMerging: true, mergedFile: null, mergeError: null });
    });

    await listen<string>("merge-complete", (event) => {
      success(`Merged to: ${event.payload}`);
      setMergeState({ isMerging: false, mergedFile: event.payload, mergeError: null });
      playNotificationSound();
      showNotification("Merge Complete", "Videos merged successfully!");
      refreshFiles();
    });

    await listen<string>("merge-error", (event) => {
      error(`Merge failed: ${event.payload}`);
      setMergeState({ isMerging: false, mergedFile: null, mergeError: event.payload });
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

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);

    const text = e.dataTransfer.getData("text/plain");
    if (text && text.includes("rongyok.com")) {
      setUrl(text);
      log(`Dropped URL: ${text}`);
      success("URL added from drop");
    } else if (text) {
      setUrl(text);
      log(`Dropped text: ${text}`);
    }
  };

  const handlePresetSelect = (presetId: string) => {
    applyPreset(presetId, updateSetting);
    success(`Preset "${presets.find(p => p.id === presetId)?.name}" applied`);
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

      {/* Header */}
      <header className="sticky top-0 z-50 glass border-b border-slate-800/50 animate-fade-in">
        <div className="container-responsive py-3 sm:py-4">
          <div className="flex items-center justify-between gap-2 sm:gap-4">
            <div className="flex items-center gap-2 sm:gap-3">
              <div className="hover-scale">
                <Logo size={40} className="sm:w-11 sm:h-11" />
              </div>
              <div>
                <h1 className="text-base sm:text-lg font-bold flex items-center gap-2">
                  <span className="bg-gradient-to-r from-violet-400 to-fuchsia-400 bg-clip-text text-transparent">
                    Rongyok
                  </span>
                  <span className="icon-glow icon-glow-sm icon-glow-fuchsia"><Film size={16} /></span>
                </h1>
                <p className="text-[10px] sm:text-xs text-slate-500 flex flex-wrap items-center gap-1">
                  <span className="hidden xs:inline">Video Downloader</span>
                  {ffmpegAvailable && (
                    <span className="animate-fade-in flex items-center gap-0.5">
                      <span className="icon-glow icon-glow-sm icon-glow-emerald"><CheckCircle2 size={10} /></span>
                      <span className="text-emerald-400">FFmpeg</span>
                    </span>
                  )}
                  {downloadState.isDownloading && currentSpeed > 0 && (
                    <span className="animate-bounce-subtle flex items-center gap-0.5">
                      <span className="icon-glow icon-glow-sm icon-glow-violet icon-glow-animated"><Zap size={10} /></span>
                      <span className="text-violet-400">{(currentSpeed / 1024 / 1024).toFixed(1)} MB/s</span>
                    </span>
                  )}
                </p>
              </div>
            </div>

            {/* Header Actions */}
            <div className="flex items-center gap-2">
              {/* Mini Mode Toggle */}
              {downloadState.isDownloading && (
                <button
                  onClick={() => setShowMiniMode(true)}
                  className="p-2 hover:bg-slate-700/50 rounded-lg transition-colors"
                  title="Mini Mode (Ctrl+M)"
                >
                  <Minimize2 size={16} className="text-slate-400" />
                </button>
              )}

              {/* Shortcuts Help */}
              <button
                onClick={() => setShowShortcuts(true)}
                className="p-2 hover:bg-slate-700/50 rounded-lg transition-colors"
                title="Keyboard Shortcuts"
              >
                <Keyboard size={16} className="text-slate-400" />
              </button>

              {/* Tabs */}
              <div className="flex bg-slate-800/70 rounded-xl p-0.5 sm:p-1 overflow-x-auto scrollbar-hide">
                {tabsConfig.map((tab, index) => (
                  <button
                    key={tab.id}
                    onClick={() => setActiveTab(tab.id)}
                    className={`px-2 sm:px-3 py-1.5 sm:py-2 text-xs sm:text-sm font-medium rounded-lg transition-all flex items-center gap-1 sm:gap-1.5 whitespace-nowrap btn-ripple ${
                      activeTab === tab.id
                        ? "tab-glow-active text-white shadow-lg animate-scale-in"
                        : "text-slate-400 hover:text-white hover:bg-slate-700/50"
                    } stagger-${index + 1}`}
                  >
                    <span className={`icon-glow icon-glow-sm ${activeTab === tab.id ? `icon-glow-${tab.glowColor} icon-glow-animated` : ""}`}>
                      {tab.icon}
                    </span>
                    <span className="hidden sm:inline">{tab.label}</span>
                  </button>
                ))}
              </div>
            </div>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="container-responsive py-4 sm:py-6 space-y-4 sm:space-y-6">
        {activeTab === "download" && (
          <div className="page-transition space-y-4 sm:space-y-6">
            {/* Presets */}
            <div className="animate-fade-in">
              <PresetSelector
                presets={presets}
                activePresetId={activePresetId}
                onSelect={handlePresetSelect}
              />
            </div>

            {/* URL Input */}
            <div className="space-y-3 sm:space-y-4 animate-fade-in">
              <Input
                label="Series URL"
                placeholder="https://rongyok.com/watch/?series_id=XXX (or drag & drop)"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                leftIcon={<Link size={16} className="sm:w-[18px] sm:h-[18px]" />}
                rightElement={
                  <div className="flex gap-1 items-center">
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => {
                        setUrl("");
                        setSeries(null);
                        setSelectedEpisodes(new Set());
                        log("URL cleared");
                      }}
                      disabled={!url}
                      className={`hover-scale ${url ? "text-red-400 hover:text-red-300 hover:bg-red-500/20" : "text-slate-600"}`}
                      title="Clear URL"
                    >
                      <X size={16} />
                    </Button>
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={handlePaste}
                      leftIcon={<Clipboard size={14} />}
                      className="hover-scale"
                      title="Paste (Ctrl+V)"
                    >
                      <span className="hidden sm:inline">Paste</span>
                      <span className="sm:hidden">📋</span>
                    </Button>
                    <Button
                      size="sm"
                      onClick={handleFetch}
                      isLoading={isFetching}
                      leftIcon={<Search size={14} />}
                      className="hover-glow"
                    >
                      <span className="hidden sm:inline">Fetch</span>
                      <span className="sm:hidden">🔍</span>
                    </Button>
                  </div>
                }
              />

              <Input
                label="Output Directory"
                placeholder="~/Downloads/rongyok"
                value={settings.outputDir}
                onChange={(e) => updateSetting("outputDir", e.target.value)}
                leftIcon={<FolderOpen size={16} className="sm:w-[18px] sm:h-[18px]" />}
                rightElement={
                  <div className="flex gap-1">
                    <Button size="sm" variant="ghost" onClick={handleSelectOutputFolder} className="hover-scale">
                      <span className="hidden sm:inline">Browse</span>
                      <span className="sm:hidden">📂</span>
                    </Button>
                    <Button size="sm" variant="ghost" onClick={handleOpenOutputFolder} className="hover-scale">
                      <span className="hidden sm:inline">Open</span>
                      <span className="sm:hidden">↗</span>
                    </Button>
                  </div>
                }
              />
            </div>

            {/* Series Info */}
            <div className="animate-fade-in stagger-1">
              <SeriesCard series={series} isLoading={isFetching} />
            </div>

            {/* Episode Selector */}
            {series && (
              <div className="animate-slide-in stagger-2">
                <EpisodeSelector
                  totalEpisodes={series.totalEpisodes}
                  selectedEpisodes={selectedEpisodes}
                  onToggle={toggleEpisode}
                  onSelectAll={selectAllEpisodes}
                  onDeselectAll={deselectAllEpisodes}
                  disabled={downloadState.isDownloading}
                />
              </div>
            )}

            {/* Speed Graph & Progress */}
            {(downloadState.isDownloading || speedData.length > 0) && (
              <div className="grid grid-cols-1 lg:grid-cols-2 gap-3 sm:gap-4 animate-fade-in">
                <div className="hover-lift">
                  <SpeedGraph
                    data={speedData}
                    currentSpeed={currentSpeed}
                    avgSpeed={avgSpeed}
                    peakSpeed={peakSpeed}
                  />
                </div>
                <div className="glass rounded-xl p-3 sm:p-4 border border-slate-700/50 space-y-3 sm:space-y-4 hover-lift">
                  <ProgressBar
                    percentage={progress.percentage}
                    label={`Episode ${progress.episode}`}
                    sublabel={`${(progress.speed / 1024 / 1024).toFixed(1)} MB/s`}
                  />
                  <ProgressBar
                    percentage={overallProgress}
                    label="Overall Progress"
                    sublabel={`${downloadState.completedEpisodes.length}/${downloadState.totalSelected} episodes`}
                    variant="success"
                  />
                  {mergeState.isMerging && (
                    <div className="flex items-center gap-3 p-3 bg-violet-500/20 rounded-lg border border-violet-500/30 animate-pulse-glow">
                      <div className="w-5 h-5 border-2 border-violet-400 border-t-transparent rounded-full animate-spin" />
                      <span className="text-violet-300 text-sm font-medium">Merging videos...</span>
                    </div>
                  )}
                  {mergeState.mergedFile && !mergeState.isMerging && (
                    <div className="p-3 bg-emerald-500/20 rounded-lg border border-emerald-500/30 animate-scale-in">
                      <div className="text-emerald-300 text-sm font-medium flex items-center gap-2">
                        <span className="animate-bounce-subtle">✅</span> Merge Complete!
                      </div>
                      <div className="text-emerald-400/70 text-xs mt-1 truncate">{mergeState.mergedFile}</div>
                    </div>
                  )}
                  {mergeState.mergeError && (
                    <div className="p-3 bg-red-500/20 rounded-lg border border-red-500/30 animate-scale-in">
                      <div className="text-red-300 text-sm font-medium">Merge Failed</div>
                      <div className="text-red-400/70 text-xs mt-1">{mergeState.mergeError}</div>
                    </div>
                  )}
                </div>
              </div>
            )}

            {/* Queue */}
            {queue.length > 0 && (
              <div className="animate-slide-in">
                <DownloadQueue
                  queue={queue}
                  onMoveUp={() => {}}
                  onMoveDown={() => {}}
                  onRemove={(id) => setQueue((prev) => prev.filter((q) => q.id !== id))}
                  onPause={() => {}}
                />
              </div>
            )}

            {/* Options */}
            <div className="flex flex-wrap items-center gap-3 sm:gap-4 animate-fade-in stagger-3">
              <label className="flex items-center gap-2 cursor-pointer hover-scale p-2 -m-2 rounded-lg">
                <input
                  type="checkbox"
                  checked={settings.autoMerge}
                  onChange={(e) => updateSetting("autoMerge", e.target.checked)}
                  disabled={!ffmpegAvailable}
                  className="w-4 h-4 rounded bg-slate-700 border-slate-600 text-violet-600 focus:ring-violet-500 focus-ring disabled:opacity-50"
                />
                <span className={`text-xs sm:text-sm flex items-center gap-1 ${ffmpegAvailable ? "text-slate-300" : "text-slate-500"}`}>
                  <Merge size={14} />
                  <span className="hidden sm:inline">Merge videos after download</span>
                  <span className="sm:hidden">Auto merge</span>
                </span>
              </label>
            </div>

            {/* Action Buttons */}
            <div className="flex flex-wrap gap-2 sm:gap-3 animate-fade-in stagger-4">
              {!downloadState.isDownloading ? (
                <>
                  <Button
                    size="lg"
                    onClick={handleStartDownload}
                    disabled={!series || selectedEpisodes.size === 0}
                    leftIcon={<span className="icon-glow icon-glow-sm icon-glow-violet icon-glow-animated"><Download size={18} /></span>}
                    className="flex-1 sm:flex-none btn-glow-violet btn-ripple"
                    title="Download (Ctrl+D)"
                  >
                    Download ({selectedEpisodes.size})
                  </Button>
                  <Button size="lg" variant="secondary" leftIcon={<span className="icon-glow icon-glow-sm icon-glow-slate"><RefreshCw size={18} /></span>} disabled className="flex-1 sm:flex-none hover-lift">
                    <span className="hidden sm:inline">Resume Previous</span>
                    <span className="sm:hidden">Resume</span>
                  </Button>
                </>
              ) : (
                <>
                  {!downloadState.isPaused ? (
                    <Button size="lg" variant="secondary" onClick={handlePause} leftIcon={<span className="icon-glow icon-glow-sm icon-glow-amber"><Pause size={18} /></span>} className="flex-1 sm:flex-none hover-lift btn-ripple" title="Pause (Space)">
                      Pause
                    </Button>
                  ) : (
                    <Button size="lg" variant="success" onClick={handleResume} leftIcon={<span className="icon-glow icon-glow-sm icon-glow-emerald icon-glow-animated"><Play size={18} /></span>} className="flex-1 sm:flex-none btn-glow-emerald btn-ripple" title="Resume (Space)">
                      Resume
                    </Button>
                  )}
                  <Button size="lg" variant="danger" onClick={handleCancel} leftIcon={<span className="icon-glow icon-glow-sm icon-glow-red"><X size={18} /></span>} className="flex-1 sm:flex-none btn-glow-red btn-ripple" title="Cancel (Escape)">
                    Cancel
                  </Button>
                </>
              )}
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

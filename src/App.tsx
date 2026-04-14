import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { readText } from "@tauri-apps/plugin-clipboard-manager";
import {
  Download, Pause, Play, X, Link, Clipboard, Search, Merge,
  Settings, HardDrive, Minimize2, Keyboard, Clock, AlertCircle,
  ListOrdered, Loader2, CheckCircle, BookOpen, Compass,
} from "lucide-react";
import {
  Button, ProgressBar, SeriesCard, LogPanel, SettingsPanel,
  HistoryPanel, SpeedGraph, FileBrowser, Logo, UpdateDialog, MiniMode,
  ShortcutsHelp, BrowsePanel, ToastContainer, ErrorBoundary,
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
import { useEventListeners } from "./hooks/useEventListeners";
import { useDownloadManager } from "./hooks/useDownloadManager";
import { useBatchProcessor, BatchItem } from "./hooks/useBatchProcessor";
import { useSeriesFetch } from "./hooks/useSeriesFetch";
import { SeriesInfo, DownloadProgress } from "./types";
import { useMergeAndDetection } from "./hooks/useMergeAndDetection";
import { BatchQueueDrawer } from "./components/BatchQueueDrawer";

// --- Local types ---
interface FileInfo {
  name: string;
  path: string;
  size: number;
  isEpisode: boolean;
  isMerged: boolean;
}

type TabType = "download" | "library" | "browse" | "files" | "history" | "settings" | "logs";

function App() {
  // --- Domain settings ---
  const { domainSettings, updateDomainSetting, resetDomainSettings, isLoaded: _domainsLoaded } = useDomainSettings();

  // --- Core hooks ---
  const { logs, log, success, warning, error, clearLogs } = useLogger();
  const { settings, updateSetting, resetSettings } = useSettings();
  const { history, addRecord, updateRecord, deleteRecord, clearHistory, getStats } = useHistory();
  const { speedData, currentSpeed, avgSpeed, peakSpeed, addDataPoint, reset: resetSpeedGraph } = useSpeedGraph();
  const { checking: isCheckingUpdates, available: updateAvailable, downloading: updateDownloading,
    progress: updateProgress, error: updateError, updateInfo, checkForUpdates,
    downloadAndInstall, dismissUpdate } = useUpdater();
  const { language, setLanguage, t } = useI18n();
  const { themes, activeThemeId, setActiveTheme } = useCustomTheme();
  const { toasts, removeToast } = useToast();

  // --- Episode selection ---
  const [selectedEpisodes, setSelectedEpisodes] = useState<Set<number>>(new Set());
  const [selectedQuality, setSelectedQuality] = useState<string | null>(null);

  // --- FFmpeg check ---
  const [ffmpegAvailable, setFfmpegAvailable] = useState(false);
  useEffect(() => {
    invoke<boolean>("check_ffmpeg").then(setFfmpegAvailable).catch(() => setFfmpegAvailable(false));
  }, []);

  // --- File browser ---
  const [files, setFiles] = useState<FileInfo[]>([]);
  const refreshFiles = useCallback(async () => {
    try {
      const result = await invoke<FileInfo[]>("list_files", { dir: settings.outputDir });
      setFiles(result);
    } catch { /* ignore */ }
  }, [settings.outputDir]);

  // --- Merge & detection state ---
  const {
    mergeState, detectionState,
    onMergeStarted, onMergeProgress, onMergeComplete, onMergeError, onDetectionProgress,
  } = useMergeAndDetection({ success, error, refreshFiles });

  // --- Series fetch hook ---
  const onSingleFetch = useCallback((result: SeriesInfo) => {
    const allEpisodes = new Set(Array.from({ length: result.totalEpisodes }, (_, i) => i + 1));
    setSelectedEpisodes(allEpisodes);
  }, []);

  // Note: setBatchQueue/setIsBatchMode come from useBatchProcessor below,
  // but onBatchStart is passed to useSeriesFetch which runs first.
  // We use a ref to break the circular dependency.
  const batchQueueSetterRef = useRef<(items: BatchItem[]) => void>(() => {});
  const batchModeSetterRef = useRef<(v: boolean) => void>(() => {});

  const onBatchStart = useCallback((items: BatchItem[]) => {
    batchQueueSetterRef.current(items);
    batchModeSetterRef.current(true);
  }, []);

  const {
    url, setUrl, series, setSeries, isFetching, setIsFetching,
    isValidSeriesUrl, extractUrls, handlePaste, fetchSeries, autoFetchFromClipboard,
  } = useSeriesFetch({ domainSettings, log, success, error, onBatchStart, onSingleFetch });

  // --- UI state ---
  const [activeTab, setActiveTab] = useState<TabType>("download");
  const [isDragging, setIsDragging] = useState(false);
  const [showMiniMode, setShowMiniMode] = useState(false);
  const [showShortcuts, setShowShortcuts] = useState(false);

  // --- Batch processor ---
  const {
    isBatchMode, setIsBatchMode, batchQueue, setBatchQueue,
    isBatchProcessing, setIsBatchProcessing, isAutoCapture, setIsAutoCapture,
    toggleBatchProcessing, resetBatchFlags, setRunDownload, setIsDownloading: setBatchIsDownloading,
  } = useBatchProcessor({ isFetching, setIsFetching, log, error });

  // Wire refs so onBatchStart can queue items before hooks fully initialize
  batchQueueSetterRef.current = useCallback((items: BatchItem[]) => {
    setBatchQueue(items);
    setIsBatchProcessing(true);
  }, [setBatchQueue, setIsBatchProcessing]);
  batchModeSetterRef.current = setIsBatchMode;

  // --- Download manager ---
  const {
    downloadState, setDownloadState, progress, setProgress, setQueue,
    runDownload, handlePause, handleResume,
    handleCancel, handlePauseResume,
  } = useDownloadManager({
    settings, ffmpegAvailable, selectedQuality, addRecord, updateRecord,
    log, success, warning, error, resetSpeedGraph, addDataPoint,
    onFilesRefresh: refreshFiles,
    isBatchMode,
  });

  // Wire download manager -> batch processor (ref-based to avoid dep cycle)
  useEffect(() => {
    setRunDownload(runDownload);
  }, [runDownload, setRunDownload]);
  useEffect(() => {
    setBatchIsDownloading(downloadState.isDownloading);
  }, [downloadState.isDownloading, setBatchIsDownloading]);

  // --- Clipboard monitor ---
  const lastClipboard = useRef<string>("");
  useEffect(() => {
    if (!isAutoCapture) return;
    const interval = window.setInterval(async () => {
      try {
        const text = await readText();
        if (text && text !== lastClipboard.current) {
          lastClipboard.current = text;
          const urls = extractUrls(text);
          if (urls.length > 0) {
            log(`Auto-captured ${urls.length} links`);
            setIsBatchMode(true);
            const existing = new Set(batchQueue.map((i) => i.url));
            const newItems = urls.filter((u) => !existing.has(u)).map((u) => ({ url: u, status: "pending" as const }));
            if (newItems.length > 0) {
              setBatchQueue((prev) => [...prev, ...newItems]);
              success(`Added ${newItems.length} links to queue`);
              setIsBatchProcessing(true);
            }
          }
        }
      } catch { /* ignore */ }
    }, 1000);
    return () => clearInterval(interval);
  }, [isAutoCapture, extractUrls, log, success, setIsBatchMode, setBatchQueue, setIsBatchProcessing, batchQueue]);

  // --- Taskbar progress ---
  useEffect(() => {
    invoke("set_taskbar_progress", {
      progress: downloadState.isDownloading ? Math.round(progress.percentage) : -1,
    }).catch(() => {});
  }, [downloadState.isDownloading, progress.percentage]);

  // --- Event listeners ---
  useEventListeners({
    onDownloadProgress: useCallback((p: DownloadProgress) => {
      setProgress(p);
      setDownloadState((prev) => ({ ...prev, currentEpisode: p.episode }));
      addDataPoint(p.speed);
    }, [setProgress, setDownloadState, addDataPoint]),
    onDownloadResult: useCallback((result: { episode: number; success: boolean; error?: string }) => {
      if (result.success) {
        setDownloadState((prev) => ({ ...prev, completedEpisodes: [...prev.completedEpisodes, result.episode] }));
      } else {
        setDownloadState((prev) => ({ ...prev, failedEpisodes: [...prev.failedEpisodes, result.episode] }));
      }
      setQueue((prev) => prev.map((q) =>
        q.episode === result.episode ? { ...q, status: result.success ? 'completed' as const : 'failed' as const, progress: 100 } : q
      ));
    }, [setDownloadState, setQueue]),
    onMergeStarted,
    onMergeProgress,
    onMergeComplete,
    onMergeError,
    onLogInfo: useCallback((message: string) => { log(message); }, [log]),
    onDetectionProgress,
  });

  // --- Tab navigation ---
  const tabs: TabType[] = ["download", "library", "browse", "files", "history", "settings", "logs"];
  const handleNextTab = useCallback(() => {
    setActiveTab(tabs[(tabs.indexOf(activeTab) + 1) % tabs.length]);
  }, [activeTab]);
  const handlePrevTab = useCallback(() => {
    setActiveTab(tabs[(tabs.indexOf(activeTab) - 1 + tabs.length) % tabs.length]);
  }, [activeTab]);

  // --- Episode handlers ---
  const toggleEpisode = useCallback((ep: number) => {
    setSelectedEpisodes((prev) => {
      const next = new Set(prev);
      if (next.has(ep)) next.delete(ep); else next.add(ep);
      return next;
    });
  }, []);
  const selectAllEpisodes = useCallback(() => {
    if (series) setSelectedEpisodes(new Set(Array.from({ length: series.totalEpisodes }, (_, i) => i + 1)));
  }, [series]);
  const deselectAllEpisodes = useCallback(() => setSelectedEpisodes(new Set()), []);

  // --- Start download ---
  const handleStartDownload = useCallback(() => {
    if (!series || selectedEpisodes.size === 0) {
      error("Please select at least one episode");
      return;
    }
    resetBatchFlags();
    runDownload(series, selectedEpisodes);
  }, [series, selectedEpisodes, runDownload, error, resetBatchFlags]);

  // --- File operations ---
  const handleOpenOutputFolder = useCallback(async () => {
    try { await invoke("open_folder", { path: settings.outputDir }); } catch { /* ignore */ }
  }, [settings.outputDir]);

  const handleDeleteFiles = useCallback(async (paths: string[]) => {
    try { await invoke("delete_files", { paths }); refreshFiles(); } catch { /* ignore */ }
  }, [refreshFiles]);

  const handlePlayFile = useCallback(async (path: string) => {
    try { await invoke("play_file", { path }); log(`Playing: ${path}`); } catch { /* ignore */ }
  }, [log]);

  // --- Keyboard shortcuts ---
  useKeyboardShortcuts({
    onPaste: handlePaste, onDownload: handleStartDownload,
    onPauseResume: handlePauseResume, onCancel: handleCancel,
    onToggleMiniMode: () => setShowMiniMode((p) => !p),
    onNextTab: handleNextTab, onPrevTab: handlePrevTab,
    isDownloading: downloadState.isDownloading, isPaused: downloadState.isPaused,
  });

  // --- Global ESC ---
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (showShortcuts) setShowShortcuts(false);
        if (showMiniMode) setShowMiniMode(false);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [showShortcuts, showMiniMode]);

  // --- Auto-init ---
  useEffect(() => {
    log("Application started");
    autoFetchFromClipboard();
    refreshFiles();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // --- Derived ---
  const overallProgress = downloadState.totalSelected > 0
    ? Math.round((downloadState.completedEpisodes.length / downloadState.totalSelected) * 100)
    : 0;

  // --- Tab icon helper ---
  const tabIcon = (tab: TabType, size = 16) => {
    switch (tab) {
      case "download": return <Download size={size} />;
      case "library": return <BookOpen size={size} />;
      case "browse": return <Compass size={size} />;
      case "files": return <HardDrive size={size} />;
      case "history": return <Clock size={size} />;
      case "settings": return <Settings size={size} />;
      case "logs": return <AlertCircle size={size} />;
    }
  };

  const tabLabel = (tab: TabType) => t(`tab.${tab}`);

  // ========== RENDER ==========
  return (
    <div className="h-screen flex flex-col bg-[var(--bg-primary)] text-slate-100 overflow-hidden select-none">
      {/* Header */}
      <header className="flex items-center justify-between px-3 py-1.5 border-b border-slate-700/50 bg-slate-900/80 backdrop-blur-md" data-tauri-drag-region>
        <div className="flex items-center gap-2" data-tauri-drag-region>
          <Logo />
          <span className="text-sm font-bold bg-gradient-to-r from-violet-400 to-fuchsia-400 bg-clip-text text-transparent">
            {t("app.title")}
          </span>
          <span className="text-[10px] text-slate-500 hidden sm:inline">v1.12.0</span>
        </div>
        <div className="flex items-center gap-1">
          <Button size="sm" variant="ghost" onClick={() => setShowMiniMode(true)} title={t("app.openMiniMode")} className="h-7 w-7 p-0 text-slate-500 hover:text-slate-300">
            <Minimize2 size={13} />
          </Button>
          <Button size="sm" variant="ghost" onClick={() => setShowShortcuts(true)} title={t("app.openShortcuts")} className="h-7 w-7 p-0 text-slate-500 hover:text-slate-300">
            <Keyboard size={13} />
          </Button>
          <NotificationCenter />
        </div>
      </header>

      {/* Tab Bar */}
      <nav className="flex items-center border-b border-slate-700/40 bg-slate-900/50 px-2">
        {tabs.map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`flex items-center gap-1.5 px-3 py-2 text-xs font-medium transition-colors border-b-2 ${
              activeTab === tab
                ? "border-violet-400 text-violet-300"
                : "border-transparent text-slate-400 hover:text-slate-200"
            }`}
          >
            {tabIcon(tab, 13)}
            <span className="hidden sm:inline">{tabLabel(tab)}</span>
          </button>
        ))}
      </nav>

      {/* Main Content */}
      <main className="flex-1 overflow-hidden">
        {activeTab === "download" && (
          <div className="h-full flex flex-col relative">
            <div className="flex-1 overflow-y-auto custom-scrollbar">
              <div className="max-w-4xl mx-auto px-4 py-3 space-y-3">
                {/* URL Input */}
                <div className="flex gap-2">
                  <div
                    className={`flex-1 flex items-center gap-2 px-3 py-2 rounded-lg border transition-all duration-200 ${
                      isDragging ? "border-violet-400 bg-violet-500/10 shadow-[0_0_20px_rgba(139,92,246,0.2)]" : "border-slate-700/50 bg-slate-800/50"
                    }`}
                    onDragOver={(e) => { e.preventDefault(); setIsDragging(true); }}
                    onDragLeave={() => setIsDragging(false)}
                    onDrop={(e) => {
                      e.preventDefault();
                      setIsDragging(false);
                      const text = e.dataTransfer.getData("text/plain");
                      if (text && isValidSeriesUrl(text)) {
                        setUrl(text);
                        fetchSeries(text);
                        log("URL added from drop");
                      }
                    }}
                  >
                    <Link size={14} className="text-slate-500 flex-shrink-0" />
                    <input
                      value={url}
                      onChange={(e) => setUrl(e.target.value)}
                      onKeyDown={(e) => { if (e.key === "Enter" && url) { fetchSeries(url); } }}
                      placeholder={t("download.urlPlaceholder")}
                      className="flex-1 bg-transparent text-sm text-slate-200 placeholder-slate-500 outline-none"
                    />
                    {url && (
                      <button onClick={() => { setUrl(""); setSeries(null); setSelectedEpisodes(new Set()); }} className="text-slate-500 hover:text-red-400">
                        <X size={13} />
                      </button>
                    )}
                  </div>
                  <Button onClick={handlePaste} leftIcon={<Clipboard size={14} />} variant="primary" size="sm" className="px-3">
                    {t("download.paste")}
                  </Button>
                  <Button
                    onClick={() => { if (url) fetchSeries(url); }}
                    leftIcon={<Search size={14} />}
                    variant="secondary"
                    size="sm"
                    disabled={isFetching || !url}
                    className="px-3"
                  >
                    {isFetching ? <Loader2 size={14} className="animate-spin" /> : t("download.fetch")}
                  </Button>
                </div>

                {/* Merge status */}
                {mergeState.isMerging && (
                  <div className="flex items-center gap-2 text-[11px] text-fuchsia-400 px-1">
                    <Merge size={10} className="animate-spin" />
                    Merging {mergeState.progress.toFixed(0)}%
                  </div>
                )}

                {/* Smart Queue Bar */}
                <div className={`flex items-center justify-between px-3 py-2 rounded-lg border transition-all duration-200 ${
                  batchQueue.length > 0
                    ? "bg-emerald-500/5 border-emerald-500/30 shadow-[0_0_15px_rgba(16,185,129,0.08)]"
                    : "bg-slate-800/40 border-slate-700/40 opacity-70 hover:opacity-100"
                }`}>
                  <div className="flex items-center gap-2.5">
                    <ListOrdered size={14} className={`transition-colors ${batchQueue.length > 0 ? "text-emerald-400 drop-shadow-[0_0_6px_rgba(52,211,153,0.5)]" : "text-slate-500"}`} />
                    <span className="text-[11px] font-bold text-slate-300 tracking-wide">Smart Queue</span>
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
                    <Button size="sm" variant={isAutoCapture ? "cyan" : "ghost"} onClick={() => setIsAutoCapture((p) => !p)} title="Auto Capture Clipboard"
                      className={`h-6 text-[10px] px-1.5 ${isAutoCapture ? "ring-1 ring-cyan-500/40" : "text-slate-500 hover:text-cyan-400"}`}>
                      <Clipboard size={12} className={isAutoCapture ? "animate-pulse" : ""} />
                    </Button>
                    <div className="w-px h-3.5 bg-slate-700/50 mx-0.5" />
                    <Button size="sm" variant={isBatchProcessing ? "amber" : "success"} onClick={toggleBatchProcessing}
                      title={isBatchProcessing ? t("queue.pauseDownload") : t("app.startQueue")} className="h-6 w-6 p-0">
                      {isBatchProcessing ? <Pause size={11} /> : <Play size={11} />}
                    </Button>
                    <Button size="sm" variant={isBatchMode ? "primary" : "ghost"} onClick={() => setIsBatchMode((p) => !p)}
                      title={isBatchMode ? "Hide queue" : "Show queue list"}
                      className={`h-6 w-6 p-0 ${isBatchMode ? "shadow-md shadow-violet-500/15" : "text-slate-500 hover:text-white"}`}>
                      {isBatchMode ? <Minimize2 size={11} /> : <ListOrdered size={11} />}
                    </Button>
                    <Button size="sm" variant="ghost" className="h-6 w-6 p-0 text-slate-500 hover:text-red-400 hover:bg-red-500/10"
                      onClick={() => {
                        if (batchQueue.length === 0 || confirm("Clear entire queue?")) {
                          setBatchQueue([]); setIsBatchMode(false); setIsBatchProcessing(false);
                        }
                      }}>
                      <X size={11} />
                    </Button>
                  </div>
                </div>

                {/* Detection progress */}
                {detectionState.isDetecting && (
                  <div className="bg-violet-500/5 rounded-lg p-3 border border-violet-500/20 animate-pulse">
                    <div className="flex justify-between text-xs text-slate-400 mb-1.5">
                      <span className="flex items-center gap-2"><Search size={12} className="animate-spin text-violet-400" />{detectionState.message}</span>
                      <span className="font-mono text-violet-300">{detectionState.progress}%</span>
                    </div>
                    <div className="h-1 bg-slate-700/50 rounded-full overflow-hidden">
                      <div className="h-full bg-gradient-to-r from-violet-500 to-fuchsia-500 rounded-full transition-all duration-300" style={{ width: `${detectionState.progress}%` }} />
                    </div>
                  </div>
                )}

                {/* Series + Episodes */}
                <SeriesCard
                  series={series} isLoading={isFetching} selectedEpisodes={selectedEpisodes}
                  onToggleEpisode={toggleEpisode} onSelectAll={selectAllEpisodes} onDeselectAll={deselectAllEpisodes}
                  disabled={downloadState.isDownloading} downloadingEpisode={progress.episode}
                  completedEpisodes={downloadState.completedEpisodes} failedEpisodes={downloadState.failedEpisodes}
                />

                {/* Quality Selector */}
                {series && (
                  <QualitySelector
                    episodeUrl={Object.values(series.episodeUrls)[0]}
                    onSelect={(q) => setSelectedQuality(q)}
                    defaultQuality={settings.defaultQuality || "best"}
                  />
                )}

                {/* Progress Dashboard */}
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
                        <SpeedGraph data={speedData} currentSpeed={currentSpeed} avgSpeed={avgSpeed} peakSpeed={peakSpeed} />
                        <div className="space-y-2.5">
                          <ProgressBar percentage={progress.percentage} label={`EP ${progress.episode}`}
                            sublabel={`${(progress.speed / 1024 / 1024).toFixed(1)} MB/s`} variant="cyan" />
                          <ProgressBar percentage={overallProgress} label={t("app.overall")}
                            sublabel={`${downloadState.completedEpisodes.length}/${downloadState.totalSelected}`} variant="success" />
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
                                <div className="h-full bg-gradient-to-r from-fuchsia-500 to-violet-500 rounded-full transition-all"
                                  style={{ width: `${mergeState.progress}%` }} />
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

                <div className="h-16" />
              </div>
            </div>

            {/* Sticky Action Bar */}
            <div className="flex-shrink-0 border-t border-slate-700/50 bg-slate-900/95 backdrop-blur-md px-4 py-2.5">
              <div className="max-w-4xl mx-auto flex items-center justify-between gap-3">
                <label className="flex items-center gap-2 cursor-pointer text-xs text-slate-400 hover:text-slate-300 transition-colors select-none">
                  <input type="checkbox" checked={settings.autoMerge} onChange={(e) => updateSetting("autoMerge", e.target.checked)}
                    disabled={!ffmpegAvailable}
                    className="w-3.5 h-3.5 rounded bg-slate-700 border-slate-600 text-fuchsia-600 focus:ring-fuchsia-500/30" />
                  <Merge size={13} className="text-fuchsia-400/70" />
                  <span>{t("app.autoMerge")}</span>
                </label>
                <div className="flex items-center gap-2">
                  {!downloadState.isDownloading ? (
                    <Button onClick={handleStartDownload} disabled={!series || selectedEpisodes.size === 0}
                      leftIcon={<Download size={15} />} variant="success" size="md"
                      className="px-5 py-2 shadow-[0_0_20px_rgba(16,185,129,0.2)] hover:shadow-[0_0_30px_rgba(16,185,129,0.35)]">
                      Download{selectedEpisodes.size > 0 ? ` (${selectedEpisodes.size})` : ""}
                    </Button>
                  ) : (
                    <>
                      <Button variant={downloadState.isPaused ? "success" : "amber"}
                        onClick={downloadState.isPaused ? handleResume : handlePause}
                        leftIcon={downloadState.isPaused ? <Play size={14} /> : <Pause size={14} />}
                        size="md" className="px-4">
                        {downloadState.isPaused ? t("app.resume") : t("app.pause")}
                      </Button>
                      <Button variant="danger" onClick={handleCancel} leftIcon={<X size={14} />} size="md" className="px-4">
                        {t("app.cancel")}
                      </Button>
                    </>
                  )}
                </div>
              </div>
            </div>

            {/* Batch Queue Drawer */}
            <BatchQueueDrawer
              isOpen={isBatchMode}
              batchQueue={batchQueue}
              setBatchQueue={setBatchQueue}
              isBatchProcessing={isBatchProcessing}
              toggleBatchProcessing={toggleBatchProcessing}
              setIsBatchProcessing={setIsBatchProcessing}
              setIsBatchMode={setIsBatchMode}
              progress={progress}
            />
          </div>
        )}
        {activeTab === "library" && (
          <div className="page-transition animate-fade-in h-full overflow-hidden">
            <ErrorBoundary><LibraryPanel /></ErrorBoundary>
          </div>
        )}
        {activeTab === "browse" && (
          <div className="page-transition animate-fade-in h-full overflow-hidden">
            <ErrorBoundary><BrowsePanel settings={settings} ffmpegAvailable={ffmpegAvailable} /></ErrorBoundary>
          </div>
        )}
        {activeTab === "files" && (
          <div className="page-transition animate-fade-in">
            <ErrorBoundary>
              <FileBrowser outputDir={settings.outputDir} files={files} onRefresh={refreshFiles}
                onOpenFolder={handleOpenOutputFolder} onDelete={handleDeleteFiles} onPlay={handlePlayFile} />
            </ErrorBoundary>
          </div>
        )}
        {activeTab === "history" && (
          <div className="page-transition animate-slide-in">
            <ErrorBoundary><HistoryPanel history={history} stats={getStats()} onDelete={deleteRecord} onClear={clearHistory} /></ErrorBoundary>
          </div>
        )}
        {activeTab === "settings" && (
          <div className="page-transition animate-fade-in space-y-6 container mx-auto max-w-4xl">
            <ErrorBoundary>
              <SettingsPanel settings={settings} onUpdate={updateSetting} onReset={resetSettings}
                onOpenFolder={handleOpenOutputFolder} onCheckUpdates={checkForUpdates}
                isCheckingUpdates={isCheckingUpdates} language={language} onLanguageChange={setLanguage} t={t}
                themes={themes} activeThemeId={activeThemeId} onThemeSelect={setActiveTheme}
                domainSettings={domainSettings} onUpdateDomain={updateDomainSetting} onResetDomains={resetDomainSettings} />
              <SchedulerPanel />
              <BackupPanel />
            </ErrorBoundary>
          </div>
        )}
        {activeTab === "logs" && (
          <div className="h-full overflow-hidden page-transition animate-fade-in flex flex-col">
            <ErrorBoundary><LogPanel logs={logs} onClear={clearLogs} /></ErrorBoundary>
          </div>
        )}
      </main>

      {/* Overlays */}
      <MiniMode isOpen={showMiniMode} onClose={() => setShowMiniMode(false)} onExpand={() => setShowMiniMode(false)}
        progress={progress} overallProgress={overallProgress}
        completedEpisodes={downloadState.completedEpisodes.length} totalEpisodes={downloadState.totalSelected}
        isPaused={downloadState.isPaused} onPause={handlePause} onResume={handleResume} seriesTitle={series?.title} />
      <ShortcutsHelp isOpen={showShortcuts} onClose={() => setShowShortcuts(false)} />
      <UpdateDialog isOpen={updateAvailable} updateInfo={updateInfo} downloading={updateDownloading}
        progress={updateProgress} error={updateError} onDownload={downloadAndInstall} onDismiss={dismissUpdate} />
      <ToastContainer toasts={toasts} onRemove={removeToast} />
    </div>
  );
}

export default App;

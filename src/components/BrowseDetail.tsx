import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ArrowLeft, Download, Loader2, AlertCircle } from "lucide-react";
import { Button, EpisodeSelector } from "./";
import QualitySelector from "./QualitySelector";
import type { SearchResult, SeriesInfo } from "../types";

interface BrowseDetailProps {
  result: SearchResult;
  onBack: () => void;
  settings: any;
  ffmpegAvailable: boolean;
}

export default function BrowseDetail({ result, onBack, settings, ffmpegAvailable }: BrowseDetailProps) {
  const [series, setSeries] = useState<SeriesInfo | null>(null);
  const [selectedEpisodes, setSelectedEpisodes] = useState<Set<number>>(new Set());
  const [selectedQuality, setSelectedQuality] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadSeries = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const info = await invoke<SeriesInfo>("fetch_series", { url: result.url });
      setSeries(info);
      setSelectedEpisodes(new Set(Array.from({ length: info.totalEpisodes }, (_, i) => i + 1)));
    } catch (e) {
      setError(String(e));
    } finally {
      setIsLoading(false);
    }
  };

  const handleDownload = async () => {
    if (!series || selectedEpisodes.size === 0) return;
    try {
      await invoke("update_series_state", { series });
      const episodes = Array.from(selectedEpisodes).sort((a, b) => a - b);
      await invoke("start_download", {
        request: {
          seriesId: series.seriesId,
          episodes,
          outputDir: settings.outputDir,
          autoMerge: settings.autoMerge && ffmpegAvailable,
          concurrentDownloads: settings.concurrentDownloads,
          speedLimit: settings.speedLimit,
          fileNaming: settings.fileNaming,
          seriesTitle: series.title,
          groupBySource: settings.groupBySource,
          preferredQuality: selectedQuality,
        },
      });
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div className="p-3 space-y-4">
      <button onClick={onBack} className="flex items-center gap-1.5 text-xs text-slate-400 hover:text-white">
        <ArrowLeft size={14} /> Back to browse
      </button>

      <div className="flex gap-4">
        {result.posterUrl && (
          <img src={result.posterUrl} alt={result.title} className="w-32 h-48 object-cover rounded-lg" />
        )}
        <div className="flex-1 space-y-2">
          <h2 className="text-sm font-bold text-white">{result.title}</h2>
          <p className="text-xs text-slate-400">Source: {result.source}</p>
          {result.totalEpisodes && <p className="text-xs text-slate-400">Episodes: {result.totalEpisodes}</p>}

          {!series && !isLoading && !error && (
            <Button onClick={loadSeries} leftIcon={<Download size={14} />} size="sm">
              Load Series
            </Button>
          )}
        </div>
      </div>

      {isLoading && (
        <div className="flex items-center justify-center py-8 gap-2 text-slate-400">
          <Loader2 size={20} className="animate-spin" /> Loading series info...
        </div>
      )}

      {error && (
        <div className="flex items-center gap-2 p-3 bg-red-500/10 border border-red-500/30 rounded-lg text-xs text-red-300">
          <AlertCircle size={14} />
          <span>Could not load series: {error}</span>
          <Button onClick={loadSeries} size="sm" variant="ghost" className="ml-auto">Retry</Button>
        </div>
      )}

      {series && (
        <div className="space-y-3">
          <EpisodeSelector
            totalEpisodes={series.totalEpisodes}
            selectedEpisodes={selectedEpisodes}
            onToggle={(ep) => {
              setSelectedEpisodes((prev) => {
                const next = new Set(prev);
                next.has(ep) ? next.delete(ep) : next.add(ep);
                return next;
              });
            }}
            onSelectAll={() => setSelectedEpisodes(new Set(Array.from({ length: series.totalEpisodes }, (_, i) => i + 1)))}
            onDeselectAll={() => setSelectedEpisodes(new Set())}
          />
          <QualitySelector
            episodeUrl={Object.values(series.episodeUrls)[0]}
            onSelect={(q) => setSelectedQuality(q)}
            defaultQuality={settings.defaultQuality || "best"}
          />
          <Button onClick={handleDownload} leftIcon={<Download size={14} />} variant="success" disabled={selectedEpisodes.size === 0}>
            Download ({selectedEpisodes.size})
          </Button>
        </div>
      )}
    </div>
  );
}

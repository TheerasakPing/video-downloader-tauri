import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ArrowLeft, Download, Loader2, AlertCircle, Star, Clock } from "lucide-react";
import { Button, EpisodeSelector } from "./";
import QualitySelector from "./QualitySelector";
import type { SearchResult, SeriesInfo } from "../types";
import { useI18n } from "../hooks/useI18n";

interface BrowseDetailProps {
  result: SearchResult;
  onBack: () => void;
  settings: any;
  ffmpegAvailable: boolean;
}

export default function BrowseDetail({ result, onBack, settings, ffmpegAvailable }: BrowseDetailProps) {
  const { t } = useI18n();
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
      <button onClick={onBack} className="flex items-center gap-1.5 text-xs text-slate-400 hover:text-white" aria-label={t("browse.goBack")}>
        <ArrowLeft size={14} /> {t("browse.backToBrowse")}
      </button>

      <div className="flex gap-4">
        {result.posterUrl && (
          <img src={result.posterUrl} alt={result.title} className="w-32 h-48 object-cover rounded-lg" />
        )}
        <div className="flex-1 space-y-2">
          <h2 className="text-sm font-bold text-white">{result.title}</h2>
          <p className="text-xs text-slate-400">{t("browse.source")}: {result.source}</p>
          {result.totalEpisodes && <p className="text-xs text-slate-400">{t("browse.episodes")}: {result.totalEpisodes}</p>}

          {/* Metadata badges */}
          {(result.rating != null || result.year != null || result.duration || result.genre) && (
            <div className="flex flex-wrap items-center gap-1.5">
              {result.rating != null && (
                <span className="flex items-center gap-0.5 px-1.5 py-0.5 rounded text-[10px] bg-amber-500/20 text-amber-300 border border-amber-500/30">
                  <Star size={10} className="fill-amber-300" /> {result.rating}
                </span>
              )}
              {result.year != null && (
                <span className="px-1.5 py-0.5 rounded text-[10px] bg-slate-700/60 text-slate-300">
                  {result.year}
                </span>
              )}
              {result.duration && (
                <span className="flex items-center gap-0.5 px-1.5 py-0.5 rounded text-[10px] bg-slate-700/60 text-slate-300">
                  <Clock size={10} /> {result.duration}
                </span>
              )}
              {result.genre && result.genre.split(",").map(g => g.trim()).filter(Boolean).map(g => (
                <span key={g} className="px-1.5 py-0.5 rounded text-[10px] bg-slate-700/40 text-slate-400">
                  {g}
                </span>
              ))}
            </div>
          )}

          {/* Description */}
          {result.description && (
            <p className="text-[11px] text-slate-400 leading-relaxed line-clamp-3">
              {result.description}
            </p>
          )}

          {!series && !isLoading && !error && (
            <Button onClick={loadSeries} leftIcon={<Download size={14} />} size="sm">
              {t("browse.loadSeries")}
            </Button>
          )}
        </div>
      </div>

      {isLoading && (
        <div className="flex items-center justify-center py-8 gap-2 text-slate-400">
          <Loader2 size={20} className="animate-spin" /> {t("browse.loadingInfo")}
        </div>
      )}

      {error && (
        <div className="flex items-center gap-2 p-3 bg-red-500/10 border border-red-500/30 rounded-lg text-xs text-red-300">
          <AlertCircle size={14} />
          <span>{t("browse.couldNotLoad")}: {error}</span>
          <Button onClick={loadSeries} size="sm" variant="ghost" className="ml-auto">{t("browse.retry")}</Button>
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

import { SeriesInfo } from "../types";
import { Film, Tv } from "lucide-react";

interface SeriesCardProps {
  series: SeriesInfo | null;
  isLoading?: boolean;
}

export function SeriesCard({ series, isLoading }: SeriesCardProps) {
  if (isLoading) {
    return (
      <div className="glass rounded-lg p-2 border border-slate-700/50">
        <div className="flex gap-2">
          <div className="w-12 h-16 skeleton rounded" />
          <div className="flex-1 space-y-1.5">
            <div className="h-4 skeleton rounded w-3/4" />
            <div className="h-3 skeleton rounded w-1/2" />
          </div>
        </div>
      </div>
    );
  }

  if (!series) {
    return (
      <div className="glass rounded-lg p-3 border border-dashed border-slate-600/50 text-center">
        <Tv
          size={20}
          className="mx-auto mb-1 text-violet-400 drop-shadow-[0_0_4px_currentColor]"
        />
        <p className="text-slate-500 text-xs">
          Enter URL and fetch to load series
        </p>
      </div>
    );
  }

  return (
    <div className="glass rounded-lg p-2 border border-slate-700/50 flex items-center gap-3">
      {series.posterUrl ? (
        <img
          src={series.posterUrl}
          alt={series.title}
          className="w-10 h-14 object-cover rounded shadow ring-1 ring-slate-700/50"
        />
      ) : (
        <div className="w-10 h-14 bg-slate-700/50 rounded flex items-center justify-center ring-1 ring-slate-700/50">
          <Film
            size={16}
            className="text-violet-400 drop-shadow-[0_0_4px_currentColor]"
          />
        </div>
      )}

      <div className="flex-1 min-w-0">
        <div className="flex items-center justify-between mb-0.5">
          <h2
            className="text-sm font-bold text-white truncate pr-2"
            title={series.title}
          >
            {series.title}
          </h2>
          <span className="px-1.5 py-0.5 bg-violet-500/10 text-violet-300 rounded text-[10px] border border-violet-500/20 whitespace-nowrap">
            ID: {series.seriesId}
          </span>
        </div>

        <div className="flex items-center gap-3 text-xs text-slate-400">
          <div className="flex items-center gap-1.5" title="Total Episodes">
            <Tv size={12} className="text-cyan-400" />
            <span>{series.totalEpisodes} eps</span>
          </div>
          <div className="w-px h-3 bg-slate-700" />
          <div className="flex items-center gap-1.5" title="Cached URLs">
            <Film size={12} className="text-emerald-400" />
            <span>{Object.keys(series.episodeUrls).length} cached</span>
          </div>
        </div>
      </div>
    </div>
  );
}

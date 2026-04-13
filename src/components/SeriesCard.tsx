import { SeriesInfo } from "../types";
import { Film, Tv, Layers } from "lucide-react";

interface SeriesCardProps {
  series: SeriesInfo | null;
  isLoading?: boolean;
}

export function SeriesCard({ series, isLoading }: SeriesCardProps) {
  if (isLoading) {
    return (
      <div className="bg-slate-800/40 backdrop-blur-sm rounded-xl border border-slate-700/40 p-4">
        <div className="flex gap-3">
          <div className="w-20 h-28 skeleton rounded-lg flex-shrink-0" />
          <div className="flex-1 space-y-2 pt-1">
            <div className="h-5 skeleton rounded w-4/5" />
            <div className="h-3 skeleton rounded w-2/5" />
            <div className="flex gap-3 mt-3">
              <div className="h-6 skeleton rounded w-16" />
              <div className="h-6 skeleton rounded w-16" />
            </div>
          </div>
        </div>
      </div>
    );
  }

  if (!series) {
    return (
      <div className="bg-slate-800/30 rounded-xl border border-dashed border-slate-700/40 p-6 text-center">
        <div className="w-12 h-12 mx-auto mb-3 rounded-xl bg-slate-800/60 flex items-center justify-center">
          <Tv size={22} className="text-slate-500" />
        </div>
        <p className="text-slate-500 text-xs">Paste a URL and click Fetch to load series info</p>
        <p className="text-slate-600 text-[10px] mt-1">Supports multiple domains</p>
      </div>
    );
  }

  const cachedCount = Object.keys(series.episodeUrls).length;

  return (
    <div className="bg-slate-800/40 backdrop-blur-sm rounded-xl border border-slate-700/40 overflow-hidden">
      <div className="flex gap-3.5 p-3">
        {/* Poster */}
        <div className="flex-shrink-0">
          {series.posterUrl ? (
            <img
              src={series.posterUrl}
              alt={series.title}
              className="w-20 h-28 object-cover rounded-lg shadow-lg ring-1 ring-white/5"
            />
          ) : (
            <div className="w-20 h-28 bg-slate-700/50 rounded-lg flex items-center justify-center">
              <Film size={24} className="text-slate-500" />
            </div>
          )}
        </div>

        {/* Info */}
        <div className="flex-1 min-w-0 flex flex-col justify-between py-0.5">
          <div>
            <h2
              className="text-sm font-bold text-white truncate mb-1.5 leading-tight"
              title={series.title}
            >
              {series.title}
            </h2>
            <div className="flex items-center gap-1.5 mb-2">
              <span className="px-1.5 py-0.5 bg-violet-500/10 text-violet-300 rounded text-[10px] border border-violet-500/15 font-mono">
                {series.seriesId}
              </span>
            </div>
          </div>

          <div className="flex items-center gap-3">
            <div className="flex items-center gap-1.5 text-[11px]" title="Total Episodes">
              <div className="w-5 h-5 rounded-md bg-cyan-500/10 flex items-center justify-center">
                <Tv size={10} className="text-cyan-400" />
              </div>
              <span className="text-slate-300 font-medium">{series.totalEpisodes}</span>
              <span className="text-slate-500">eps</span>
            </div>

            <div className="w-px h-3.5 bg-slate-700/60" />

            <div className="flex items-center gap-1.5 text-[11px]" title="Cached URLs">
              <div className="w-5 h-5 rounded-md bg-emerald-500/10 flex items-center justify-center">
                <Layers size={10} className="text-emerald-400" />
              </div>
              <span className="text-slate-300 font-medium">{cachedCount}</span>
              <span className="text-slate-500">cached</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

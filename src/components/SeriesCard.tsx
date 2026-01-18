import { SeriesInfo } from "../types";
import { Film, Tv, Sparkles } from "lucide-react";

interface SeriesCardProps {
  series: SeriesInfo | null;
  isLoading?: boolean;
}

export function SeriesCard({ series, isLoading }: SeriesCardProps) {
  if (isLoading) {
    return (
      <div className="card-glow card-glow-violet card-glow-animated p-3 sm:p-4">
        <div className="flex gap-3 sm:gap-4">
          <div className="w-16 h-20 sm:w-24 sm:h-32 skeleton rounded-lg" />
          <div className="flex-1 space-y-2 sm:space-y-3">
            <div className="h-4 sm:h-5 skeleton rounded w-3/4" />
            <div className="h-3 sm:h-4 skeleton rounded w-1/2" />
            <div className="h-3 skeleton rounded w-1/3" />
          </div>
        </div>
      </div>
    );
  }

  if (!series) {
    return (
      <div className="card-glow p-6 sm:p-8 border-dashed text-center">
        <div className="icon-glow icon-glow-lg icon-glow-violet mx-auto mb-2 sm:mb-3">
          <Tv size={32} />
        </div>
        <p className="text-slate-500 text-xs sm:text-sm">
          Enter a URL and click Fetch to load series information
        </p>
      </div>
    );
  }

  return (
    <div className="card-glow card-glow-violet p-3 sm:p-4">
      <div className="flex gap-3 sm:gap-4">
        {series.posterUrl ? (
          <div className="relative">
            <img
              src={series.posterUrl}
              alt={series.title}
              className="w-16 h-20 sm:w-24 sm:h-32 object-cover rounded-lg shadow-lg shadow-violet-500/20 hover:scale-105 transition-transform"
            />
            <div className="absolute -top-1 -right-1">
              <span className="icon-glow icon-glow-sm icon-glow-fuchsia icon-glow-animated">
                <Sparkles size={10} />
              </span>
            </div>
          </div>
        ) : (
          <div className="w-16 h-20 sm:w-24 sm:h-32 bg-slate-700/50 rounded-lg flex items-center justify-center border border-slate-600/50">
            <span className="icon-glow icon-glow-violet">
              <Film size={24} />
            </span>
          </div>
        )}
        <div className="flex-1 min-w-0">
          <h2 className="text-sm sm:text-lg font-semibold text-white truncate mb-1 animate-fade-in">
            {series.title}
          </h2>
          <div className="flex flex-wrap items-center gap-1.5 sm:gap-2 text-xs sm:text-sm text-slate-400">
            <span className="px-1.5 sm:px-2 py-0.5 bg-violet-500/20 text-violet-300 rounded-md text-[10px] sm:text-xs border border-violet-500/30">
              ID: {series.seriesId}
            </span>
            <span className="hidden sm:inline">•</span>
            <span className="flex items-center gap-1">
              <Film size={12} className="text-violet-400" />
              {series.totalEpisodes} Episodes
            </span>
          </div>
          <p className="mt-1.5 sm:mt-2 text-[10px] sm:text-xs text-emerald-400 animate-fade-in flex items-center gap-1">
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
            {Object.keys(series.episodeUrls).length} URLs cached
          </p>
        </div>
      </div>
    </div>
  );
}

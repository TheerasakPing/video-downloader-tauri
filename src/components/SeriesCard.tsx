import { SeriesInfo } from "../types";
import { Film, Tv, Layers } from "lucide-react";
import { useI18n } from "../hooks/useI18n";

interface SeriesCardProps {
  series: SeriesInfo | null;
  isLoading?: boolean;
  selectedEpisodes?: Set<number>;
  onToggleEpisode?: (episode: number) => void;
  onSelectAll?: () => void;
  onDeselectAll?: () => void;
  disabled?: boolean;
}

export function SeriesCard({
  series,
  isLoading,
  selectedEpisodes,
  onToggleEpisode,
  onSelectAll,
  onDeselectAll,
  disabled,
}: SeriesCardProps) {
  const { t } = useI18n();

  if (isLoading) {
    return (
      <div className="bg-slate-800/40 backdrop-blur-sm rounded-xl border border-slate-700/40 p-4 animate-pulse">
        <div className="flex gap-3">
          <div className="w-24 h-32 skeleton rounded-lg flex-shrink-0" />
          <div className="flex-1 space-y-2 pt-1">
            <div className="h-5 skeleton rounded w-4/5" />
            <div className="h-3 skeleton rounded w-2/5" />
            <div className="grid grid-cols-[repeat(auto-fill,minmax(2rem,1fr))] gap-1 mt-3">
              {Array.from({ length: 12 }).map((_, i) => (
                <div key={i} className="h-7 skeleton rounded-md" />
              ))}
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
  const episodes = Array.from({ length: series.totalEpisodes }, (_, i) => i + 1);
  const hasEpisodes = selectedEpisodes !== undefined;

  return (
    <div className="bg-slate-800/40 backdrop-blur-sm rounded-xl border border-slate-700/40 overflow-hidden">
      <div className="flex">
        {/* Left: Detail + Poster (30%) */}
        <div className="w-[30%] flex-shrink-0 p-3 flex flex-col">
          {/* Poster */}
          <div className="flex-shrink-0 mb-2.5">
            {series.posterUrl ? (
              <img
                src={series.posterUrl}
                alt={series.title}
                className="w-full aspect-[3/4] object-cover rounded-lg shadow-lg ring-1 ring-white/5"
              />
            ) : (
              <div className="w-full aspect-[3/4] bg-slate-700/50 rounded-lg flex items-center justify-center">
                <Film size={24} className="text-slate-500" />
              </div>
            )}
          </div>

          {/* Title */}
          <h2
            className="text-xs font-bold text-white leading-tight mb-1.5 line-clamp-2"
            title={series.title}
          >
            {series.title}
          </h2>

          {/* Meta */}
          <span className="inline-block self-start px-1.5 py-0.5 bg-violet-500/10 text-violet-300 rounded text-[9px] border border-violet-500/15 font-mono mb-2">
            {series.seriesId}
          </span>

          <div className="flex items-center gap-2 text-[10px] mt-auto">
            <div className="flex items-center gap-1">
              <Tv size={9} className="text-cyan-400" />
              <span className="text-slate-300 font-medium">{series.totalEpisodes}</span>
            </div>
            <div className="w-px h-2.5 bg-slate-700/60" />
            <div className="flex items-center gap-1">
              <Layers size={9} className="text-emerald-400" />
              <span className="text-slate-300 font-medium">{cachedCount}</span>
            </div>
          </div>
        </div>

        {/* Right: Episodes (70%) */}
        {hasEpisodes && (
          <div className="w-[70%] border-l border-slate-700/30 flex flex-col">
            {/* Episode header */}
            <div className="px-3 py-2 flex items-center justify-between gap-2 border-b border-slate-700/25">
              <div className="flex items-center gap-2">
                <div className="w-1.5 h-1.5 rounded-full bg-emerald-400 shadow-[0_0_6px_rgba(52,211,153,0.5)]" />
                <span className="text-[10px] font-bold tracking-widest text-slate-400 uppercase">Episodes</span>
                <span className="bg-emerald-500/10 text-emerald-400 px-1.5 py-0.5 rounded-full text-[10px] font-bold border border-emerald-500/20">
                  {selectedEpisodes!.size}/{series.totalEpisodes}
                </span>
              </div>
              <div className="flex gap-1">
                <button
                  onClick={onSelectAll}
                  disabled={disabled}
                  className="px-2 py-0.5 text-[9px] font-bold text-emerald-400 bg-emerald-500/10 hover:bg-emerald-500/20 rounded transition-colors disabled:opacity-40 border border-emerald-500/25"
                  aria-label={t("episodes.selectAllAria")}
                >
                  {t("episodes.all")}
                </button>
                <button
                  onClick={onDeselectAll}
                  disabled={disabled}
                  className="px-2 py-0.5 text-[9px] font-bold text-slate-400 bg-slate-700/30 hover:bg-slate-700/50 rounded transition-colors disabled:opacity-40 border border-slate-600/30"
                  aria-label={t("episodes.deselectAllAria")}
                >
                  {t("episodes.none")}
                </button>
              </div>
            </div>

            {/* Episode grid */}
            <div className="flex-1 p-2.5 overflow-y-auto custom-scrollbar">
              <div className="grid grid-cols-[repeat(auto-fill,minmax(2rem,1fr))] gap-1">
                {episodes.map((ep) => {
                  const isSelected = selectedEpisodes!.has(ep);
                  return (
                    <button
                      key={ep}
                      onClick={() => onToggleEpisode?.(ep)}
                      disabled={disabled}
                      className={`
                        h-6.5 text-[10px] font-semibold rounded-md transition-all border flex items-center justify-center
                        ${
                          isSelected
                            ? "bg-emerald-500/20 text-emerald-300 border-emerald-500/40 shadow-[0_0_6px_rgba(16,185,129,0.15)]"
                            : "bg-slate-800/40 text-slate-500 border-slate-700/40 hover:bg-slate-700/50 hover:text-slate-300 hover:border-slate-600/50"
                        }
                        disabled:opacity-30 disabled:cursor-not-allowed
                        active:scale-95
                      `}
                      title={t("episodes.episodeTitle", { number: ep })}
                      aria-label={t("episodes.episodeTitle", { number: ep })}
                    >
                      {ep}
                    </button>
                  );
                })}
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

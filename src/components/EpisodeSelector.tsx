import { useI18n } from "../hooks/useI18n";

interface EpisodeSelectorProps {
  totalEpisodes: number;
  selectedEpisodes: Set<number>;
  onToggle: (episode: number) => void;
  onSelectAll: () => void;
  onDeselectAll: () => void;
  disabled?: boolean;
}

export function EpisodeSelector({
  totalEpisodes,
  selectedEpisodes,
  onToggle,
  onSelectAll,
  onDeselectAll,
  disabled,
}: EpisodeSelectorProps) {
  const { t } = useI18n();
  const episodes = Array.from({ length: totalEpisodes }, (_, i) => i + 1);

  return (
    <div className="bg-slate-800/40 backdrop-blur-sm rounded-xl border border-slate-700/40 overflow-hidden">
      {/* Header */}
      <div className="px-3 py-2 flex items-center justify-between gap-2 border-b border-slate-700/30">
        <div className="flex items-center gap-2">
          <div className="w-1.5 h-1.5 rounded-full bg-emerald-400 shadow-[0_0_6px_rgba(52,211,153,0.5)]" />
          <span className="text-[10px] font-bold tracking-widest text-slate-400 uppercase">Episodes</span>
          <span className="bg-emerald-500/10 text-emerald-400 px-1.5 py-0.5 rounded-full text-[10px] font-bold border border-emerald-500/20">
            {selectedEpisodes.size}/{totalEpisodes}
          </span>
        </div>
        <div className="flex gap-1">
          <button
            onClick={onSelectAll}
            disabled={disabled}
            className="px-2.5 py-1 text-[10px] font-bold text-emerald-400 bg-emerald-500/10 hover:bg-emerald-500/20 rounded-md transition-colors disabled:opacity-40 border border-emerald-500/25"
            aria-label={t("episodes.selectAllAria")}
          >
            {t("episodes.all")}
          </button>
          <button
            onClick={onDeselectAll}
            disabled={disabled}
            className="px-2.5 py-1 text-[10px] font-bold text-slate-400 bg-slate-700/30 hover:bg-slate-700/50 rounded-md transition-colors disabled:opacity-40 border border-slate-600/30"
            aria-label={t("episodes.deselectAllAria")}
          >
            {t("episodes.none")}
          </button>
        </div>
      </div>

      {/* Grid */}
      <div className="p-3">
        <div className="max-h-28 overflow-y-auto custom-scrollbar pr-1">
          <div className="grid grid-cols-[repeat(auto-fill,minmax(2.25rem,1fr))] gap-1">
            {episodes.map((ep) => {
              const isSelected = selectedEpisodes.has(ep);
              return (
                <button
                  key={ep}
                  onClick={() => onToggle(ep)}
                  disabled={disabled}
                  className={`
                    h-7 text-[10px] font-semibold rounded-md transition-all border flex items-center justify-center
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
    </div>
  );
}

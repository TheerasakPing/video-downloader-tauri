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
  const episodes = Array.from({ length: totalEpisodes }, (_, i) => i + 1);

  return (
    <div className="card-glow card-glow-fuchsia p-3 sm:p-4">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 mb-3">
        <h3 className="text-xs sm:text-sm font-medium text-slate-300 flex items-center gap-2">
          <span className="px-2 py-0.5 bg-fuchsia-500/20 text-fuchsia-300 rounded-md text-xs border border-fuchsia-500/30">
            {selectedEpisodes.size}/{totalEpisodes}
          </span>
          Episodes
        </h3>
        <div className="flex gap-2">
          <button
            onClick={onSelectAll}
            disabled={disabled}
            className="px-3 py-1.5 text-xs font-medium text-emerald-400 hover:bg-emerald-500/10 rounded-lg transition-colors disabled:opacity-50 border border-emerald-500/30 hover:border-emerald-500/50"
          >
            Select All
          </button>
          <button
            onClick={onDeselectAll}
            disabled={disabled}
            className="px-3 py-1.5 text-xs font-medium text-slate-400 hover:bg-slate-700 rounded-lg transition-colors disabled:opacity-50 border border-slate-600/50 hover:border-slate-500/50"
          >
            Deselect All
          </button>
        </div>
      </div>

      <div className="max-h-32 sm:max-h-40 overflow-y-auto pr-1 sm:pr-2 scrollbar-hide">
        <div className="episode-grid">
          {episodes.map((ep, index) => (
            <button
              key={ep}
              onClick={() => onToggle(ep)}
              disabled={disabled}
              className={`
                px-1.5 sm:px-2 py-1 sm:py-1.5 text-[10px] sm:text-xs font-medium rounded-lg transition-all
                ${
                  selectedEpisodes.has(ep)
                    ? "bg-gradient-to-br from-violet-500 to-fuchsia-500 text-white shadow-lg shadow-violet-500/30 border border-violet-400/50"
                    : "bg-slate-700/50 text-slate-400 hover:bg-slate-700 hover:text-white border border-slate-600/30 hover:border-slate-500/50"
                }
                disabled:opacity-50 disabled:cursor-not-allowed
              `}
              style={{ animationDelay: `${Math.min(index * 10, 200)}ms` }}
            >
              {ep}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}

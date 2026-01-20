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
    <div className="glass rounded-lg p-2 border border-slate-700/50">
      <div className="flex items-center justify-between gap-2 mb-2">
        <span className="text-xs text-slate-400">
          <span className="px-1.5 py-0.5 bg-fuchsia-500/20 text-fuchsia-300 rounded text-[10px] mr-1">
            {selectedEpisodes.size}/{totalEpisodes}
          </span>
          Episodes
        </span>
        <div className="flex gap-1">
          <button
            onClick={onSelectAll}
            disabled={disabled}
            className="px-2 py-1 text-[10px] font-medium text-emerald-400 hover:bg-emerald-500/10 rounded transition-colors disabled:opacity-50 border border-emerald-500/30"
          >
            All
          </button>
          <button
            onClick={onDeselectAll}
            disabled={disabled}
            className="px-2 py-1 text-[10px] font-medium text-slate-400 hover:bg-slate-700 rounded transition-colors disabled:opacity-50 border border-slate-600/50"
          >
            None
          </button>
        </div>
      </div>

      <div className="max-h-24 overflow-y-auto scrollbar-hide">
        <div className="flex flex-wrap gap-1">
          {episodes.map((ep) => (
            <button
              key={ep}
              onClick={() => onToggle(ep)}
              disabled={disabled}
              className={`
                w-7 h-6 text-[10px] font-medium rounded transition-all
                ${
                  selectedEpisodes.has(ep)
                    ? "bg-gradient-to-br from-violet-500 to-fuchsia-500 text-white shadow shadow-violet-500/30"
                    : "bg-slate-700/50 text-slate-400 hover:bg-slate-700 hover:text-white"
                }
                disabled:opacity-50 disabled:cursor-not-allowed
              `}
            >
              {ep}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}

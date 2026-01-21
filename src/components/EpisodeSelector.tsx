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
          <span className="px-1.5 py-0.5 bg-cyan-500/10 text-cyan-400 rounded border border-cyan-500/30 text-[10px] mr-1">
            {selectedEpisodes.size}/{totalEpisodes}
          </span>
          Episodes
        </span>
        <div className="flex gap-1">
          <button
            onClick={onSelectAll}
            disabled={disabled}
            className="px-2 py-1 text-[10px] font-medium text-emerald-400 bg-emerald-500/10 hover:bg-emerald-500/20 rounded transition-colors disabled:opacity-50 border border-emerald-500/40 shadow-[0_0_6px_rgba(16,185,129,0.2)]"
          >
            All
          </button>
          <button
            onClick={onDeselectAll}
            disabled={disabled}
            className="px-2 py-1 text-[10px] font-medium text-amber-400 bg-amber-500/10 hover:bg-amber-500/20 rounded transition-colors disabled:opacity-50 border border-amber-500/40"
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
                w-7 h-6 text-[10px] font-medium rounded transition-all border
                ${
                  selectedEpisodes.has(ep)
                    ? "bg-emerald-500/20 text-emerald-300 border-emerald-500/50 shadow-[0_0_8px_rgba(16,185,129,0.3)]"
                    : "bg-slate-700/30 text-slate-400 border-slate-600/50 hover:bg-slate-700/50 hover:text-white hover:border-slate-500/50"
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

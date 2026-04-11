import { LibraryEntry } from '../types';
import { Play, Trash2, Star } from 'lucide-react';

interface LibraryCardProps {
  entry: LibraryEntry;
  onClick: (id: number) => void;
  onRemove: (id: number) => void;
  onToggleFavorite: (id: number) => void;
}

export function LibraryCard({ entry, onClick, onRemove, onToggleFavorite }: LibraryCardProps) {
  const progress = entry.totalEpisodes > 0
    ? Math.round((entry.completedCount / entry.totalEpisodes) * 100)
    : 0;

  const statusColor = progress === 100
    ? 'bg-green-500' : progress > 0
    ? 'bg-yellow-500' : 'bg-gray-500';

  const posterSrc = entry.posterPath
    ? `tauri://localhost/${encodeURIComponent(entry.posterPath)}`
    : null;

  return (
    <div
      className="bg-[var(--card)] border border-[var(--border)] rounded-xl overflow-hidden cursor-pointer hover:border-[var(--accent)] transition-all hover:shadow-lg group"
      onClick={() => onClick(entry.id)}
    >
      {/* Poster */}
      <div className="relative aspect-[2/3] bg-[var(--bg)]">
        {posterSrc ? (
          <img
            src={posterSrc}
            alt={entry.title}
            className="w-full h-full object-cover"
          />
        ) : (
          <div className="w-full h-full flex items-center justify-center text-[var(--text)] opacity-30">
            <Play size={48} />
          </div>
        )}
        {/* Source badge */}
        <span className="absolute top-2 left-2 px-2 py-0.5 rounded text-xs font-medium bg-[var(--accent)] text-white">
          {entry.source}
        </span>
        {/* Status badge */}
        <span className={`absolute top-2 right-2 px-2 py-0.5 rounded text-xs font-medium text-white ${statusColor}`}>
          {progress === 100 ? 'Complete' : progress > 0 ? `${progress}%` : 'New'}
        </span>
        {/* Favorite star */}
        <button
          onClick={(e) => { e.stopPropagation(); onToggleFavorite(entry.id); }}
          className="absolute bottom-2 right-2 p-1 rounded-full bg-black/40 hover:bg-black/60 transition-all"
        >
          <Star
            size={16}
            className={entry.favorite ? 'text-yellow-400 fill-yellow-400' : 'text-white/60'}
          />
        </button>
      </div>

      {/* Info */}
      <div className="p-3">
        <h3 className="text-sm font-medium text-[var(--text)] truncate" title={entry.title}>
          {entry.title}
        </h3>
        {entry.tags.length > 0 && (
          <div className="flex flex-wrap gap-1 mt-1">
            {entry.tags.slice(0, 2).map(tag => (
              <span key={tag.id} className="text-[10px] px-1.5 py-0.5 rounded bg-[var(--accent)]/10 text-[var(--accent)] truncate max-w-[80px]">
                {tag.name}
              </span>
            ))}
            {entry.tags.length > 2 && (
              <span className="text-[10px] text-[var(--text)] opacity-40">+{entry.tags.length - 2}</span>
            )}
          </div>
        )}
        <div className="flex items-center justify-between mt-1">
          <span className="text-xs text-[var(--text)] opacity-60">
            {entry.completedCount}/{entry.totalEpisodes} episodes
          </span>
          <button
            onClick={(e) => { e.stopPropagation(); onRemove(entry.id); }}
            className="p-1 rounded opacity-0 group-hover:opacity-60 hover:!opacity-100 hover:text-red-500 transition-all"
          >
            <Trash2 size={14} />
          </button>
        </div>
        {/* Progress bar */}
        <div className="mt-2 h-1 bg-[var(--bg)] rounded-full overflow-hidden">
          <div
            className="h-full bg-[var(--accent)] rounded-full transition-all"
            style={{ width: `${progress}%` }}
          />
        </div>
      </div>
    </div>
  );
}

export default LibraryCard;

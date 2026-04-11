import { SeriesDetail as SeriesDetailType, LibraryTag } from '../types';
import { ArrowLeft, RefreshCw, Trash2, Star, Play } from 'lucide-react';

interface SeriesDetailProps {
  detail: SeriesDetailType;
  onBack: () => void;
  onRemove: (id: number) => void;
  onRefetch: (id: number) => void;
  onToggleFavorite: (id: number) => void;
  tags: LibraryTag[];
  onAssignTag: (libraryId: number, tagId: number) => void;
  onUnassignTag: (libraryId: number, tagId: number) => void;
  onOpenEpisode: (libraryId: number, episodeNumber: number) => void;
}

export function SeriesDetail({ detail, onBack, onRemove, onRefetch, onToggleFavorite, tags, onAssignTag, onUnassignTag, onOpenEpisode }: SeriesDetailProps) {
  const { entry, episodes, canRefetch } = detail;

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'completed': return 'bg-green-500/20 text-green-400 border-green-500/30';
      case 'failed': return 'bg-red-500/20 text-red-400 border-red-500/30';
      case 'downloading': return 'bg-blue-500/20 text-blue-400 border-blue-500/30';
      default: return 'bg-gray-500/20 text-gray-400 border-gray-500/30';
    }
  };

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center gap-3">
        <button onClick={onBack} className="p-2 rounded-lg hover:bg-[var(--card)] transition-colors">
          <ArrowLeft size={20} className="text-[var(--text)]" />
        </button>
        <h2 className="text-lg font-semibold text-[var(--text)] flex-1 truncate">{entry.title}</h2>
        {/* Favorite star */}
        <button
          onClick={() => onToggleFavorite(entry.id)}
          className="p-1 rounded hover:bg-[var(--card)] transition-colors"
          title={entry.favorite ? 'Remove from favorites' : 'Add to favorites'}
        >
          <Star size={18} className={entry.favorite ? 'text-yellow-400 fill-yellow-400' : 'text-[var(--text)] opacity-40'} />
        </button>
        <div className="flex gap-2">
          {canRefetch && (
            <button
              onClick={() => onRefetch(entry.id)}
              className="p-2 rounded-lg hover:bg-[var(--card)] transition-colors text-[var(--text)] opacity-60 hover:opacity-100"
              title="Re-fetch series info"
            >
              <RefreshCw size={18} />
            </button>
          )}
          <button
            onClick={() => { onRemove(entry.id); onBack(); }}
            className="p-2 rounded-lg hover:bg-red-500/10 transition-colors text-[var(--text)] opacity-60 hover:opacity-100 hover:text-red-500"
            title="Remove from library"
          >
            <Trash2 size={18} />
          </button>
        </div>
      </div>

      {/* Meta info */}
      <div className="flex items-center gap-4 text-sm text-[var(--text)] opacity-60">
        <span className="px-2 py-0.5 rounded bg-[var(--accent)]/20 text-[var(--accent)]">{entry.source}</span>
        <span>{entry.totalEpisodes} episodes</span>
        <span>Added {new Date(entry.dateAdded).toLocaleDateString()}</span>
        {entry.lastDownloaded && (
          <span>Last download: {new Date(entry.lastDownloaded).toLocaleDateString()}</span>
        )}
      </div>

      {/* Tags */}
      <div className="flex items-center gap-2 flex-wrap mt-2">
        {entry.tags.map(tag => (
          <span key={tag.id} className="flex items-center gap-1 px-2 py-0.5 rounded-full text-xs bg-[var(--accent)]/10 text-[var(--accent)]">
            {tag.name}
            <button onClick={() => onUnassignTag(entry.id, tag.id)} className="hover:text-red-400">×</button>
          </span>
        ))}
        <select
          onChange={(e) => {
            if (e.target.value) { onAssignTag(entry.id, Number(e.target.value)); e.target.value = ''; }
          }}
          className="text-xs bg-transparent text-[var(--text)] opacity-60"
          defaultValue=""
        >
          <option value="" disabled>+ Add tag</option>
          {tags.filter(t => !entry.tags.some(et => et.id === t.id)).map(tag => (
            <option key={tag.id} value={tag.id}>{tag.name}</option>
          ))}
        </select>
      </div>

      {/* Episode grid */}
      <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-2">
        {episodes.map((ep) => {
          const isClickable = ep.status === 'completed' || ep.status === 'failed' || ep.status === 'pending';
          const handleClick = () => {
            if (ep.status === 'completed' && ep.filePath) {
              onOpenEpisode(entry.id, ep.episodeNumber);
            }
          };

          return (
            <div
              key={ep.id}
              onClick={isClickable ? handleClick : undefined}
              className={`p-3 rounded-lg border text-center text-sm transition-all ${getStatusColor(ep.status)} ${isClickable ? 'cursor-pointer hover:scale-105 hover:shadow-md' : ''}`}
            >
              <div className="font-medium">Ep {ep.episodeNumber}</div>
              <div className="text-xs mt-1 capitalize">{ep.status}</div>
              {ep.status === 'completed' && (
                <Play size={12} className="mx-auto mt-1 opacity-40" />
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

export default SeriesDetail;

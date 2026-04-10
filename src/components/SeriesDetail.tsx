import { SeriesDetail as SeriesDetailType } from '../types';
import { ArrowLeft, RefreshCw, Trash2 } from 'lucide-react';

interface SeriesDetailProps {
  detail: SeriesDetailType;
  onBack: () => void;
  onRemove: (id: number) => void;
  onRefetch: (id: number) => void;
}

export function SeriesDetail({ detail, onBack, onRemove, onRefetch }: SeriesDetailProps) {
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
      </div>

      {/* Episode grid */}
      <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-2">
        {episodes.map((ep) => (
          <div
            key={ep.id}
            className={`p-3 rounded-lg border text-center text-sm ${getStatusColor(ep.status)}`}
          >
            <div className="font-medium">Ep {ep.episodeNumber}</div>
            <div className="text-xs mt-1 capitalize">{ep.status}</div>
          </div>
        ))}
      </div>
    </div>
  );
}

export default SeriesDetail;

import { SeriesDetail as SeriesDetailType, LibraryTag } from '../types';
import { ArrowLeft, RefreshCw, Trash2, Star, Play, Eye, EyeOff, Clock } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { useState } from 'react';
import { useI18n } from '../hooks/useI18n';

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

type WatchFilter = 'all' | 'watched' | 'unwatched';

export function SeriesDetail({ detail, onBack, onRemove, onRefetch, onToggleFavorite, tags, onAssignTag, onUnassignTag, onOpenEpisode }: SeriesDetailProps) {
  const { t, language } = useI18n();
  const locale = language === "th" ? "th-TH" : undefined;
  const { entry, episodes, canRefetch } = detail;
  const [watchFilter, setWatchFilter] = useState<WatchFilter>('all');

  const handleToggleWatched = async (episodeNumber: number, e: React.MouseEvent) => {
    e.stopPropagation();
    const isWatched = episodes.find(ep => ep.episodeNumber === episodeNumber)?.watched;
    try {
      if (isWatched) {
        await invoke('cmd_mark_episode_unwatched', { libraryId: entry.id, episodeNumber });
      } else {
        await invoke('cmd_mark_episode_watched', { libraryId: entry.id, episodeNumber });
      }
      onRefetch(entry.id);
    } catch (error) {
      console.error('Failed to toggle watched:', error);
    }
  };

  const filteredEpisodes = episodes.filter(ep => {
    if (watchFilter === 'watched') return ep.watched;
    if (watchFilter === 'unwatched') return !ep.watched;
    return true;
  });

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
        <button onClick={onBack} className="p-2 rounded-lg hover:bg-[var(--card)] transition-colors" aria-label={t("library.detail.goBack")}>
          <ArrowLeft size={20} className="text-[var(--text)]" />
        </button>
        <h2 className="text-lg font-semibold text-[var(--text)] flex-1 truncate">{entry.title}</h2>
        {/* Favorite star */}
        <button
          onClick={() => onToggleFavorite(entry.id)}
          className="p-1 rounded hover:bg-[var(--card)] transition-colors"
          title={entry.favorite ? t("library.removeFromFavorites") : t("library.addToFavorites")}
          aria-label={t("library.toggleFavorite")}
        >
          <Star size={18} className={entry.favorite ? 'text-yellow-400 fill-yellow-400' : 'text-[var(--text)] opacity-40'} />
        </button>
        <div className="flex gap-2">
          {canRefetch && (
            <button
              onClick={() => onRefetch(entry.id)}
              className="p-2 rounded-lg hover:bg-[var(--card)] transition-colors text-[var(--text)] opacity-60 hover:opacity-100"
              title={t("library.detail.refetch")}
              aria-label={t("library.detail.refetch")}
            >
              <RefreshCw size={18} />
            </button>
          )}
          <button
            onClick={() => { onRemove(entry.id); onBack(); }}
            className="p-2 rounded-lg hover:bg-red-500/10 transition-colors text-[var(--text)] opacity-60 hover:opacity-100 hover:text-red-500"
            title={t("library.detail.remove")}
            aria-label={t("library.detail.remove")}
          >
            <Trash2 size={18} />
          </button>
        </div>
      </div>

      {/* Meta info */}
      <div className="flex items-center gap-4 text-sm text-[var(--text)] opacity-60">
        <span className="px-2 py-0.5 rounded bg-[var(--accent)]/20 text-[var(--accent)]">{entry.source}</span>
        <span>{entry.totalEpisodes} {t("library.detail.episodes")}</span>
        <span>{t("library.detail.added")} {new Date(entry.dateAdded).toLocaleDateString(locale)}</span>
        {entry.lastDownloaded && (
          <span>{t("library.detail.lastDownload")}: {new Date(entry.lastDownloaded).toLocaleDateString(locale)}</span>
        )}
      </div>

      {/* Metadata badges */}
      {(entry.rating != null || entry.year != null || entry.duration || entry.genre) && (
        <div className="flex flex-wrap items-center gap-2">
          {entry.rating != null && (
            <span className="flex items-center gap-1 px-2 py-0.5 rounded text-xs bg-amber-500/20 text-amber-400 border border-amber-500/30">
              <Star size={12} className="fill-amber-400" /> {entry.rating}
            </span>
          )}
          {entry.year != null && (
            <span className="px-2 py-0.5 rounded text-xs bg-[var(--card)] text-[var(--text)] opacity-70 border border-[var(--border)]">
              {entry.year}
            </span>
          )}
          {entry.duration && (
            <span className="flex items-center gap-1 px-2 py-0.5 rounded text-xs bg-[var(--card)] text-[var(--text)] opacity-70 border border-[var(--border)]">
              <Clock size={12} /> {entry.duration}
            </span>
          )}
          {entry.genre && entry.genre.split(",").map(g => g.trim()).filter(Boolean).map(g => (
            <span key={g} className="px-2 py-0.5 rounded text-xs bg-[var(--accent)]/10 text-[var(--accent)]">
              {g}
            </span>
          ))}
        </div>
      )}

      {/* Description */}
      {entry.description && (
        <p className="text-xs text-[var(--text)] opacity-60 leading-relaxed line-clamp-3">
          {entry.description}
        </p>
      )}

      {/* Tags */}
      <div className="flex items-center gap-2 flex-wrap mt-2">
        {entry.tags.map(tag => (
          <span key={tag.id} className="flex items-center gap-1 px-2 py-0.5 rounded-full text-xs bg-[var(--accent)]/10 text-[var(--accent)]">
            {tag.name}
            <button onClick={() => onUnassignTag(entry.id, tag.id)} className="hover:text-red-400" aria-label={`Remove tag ${tag.name}`}>×</button>
          </span>
        ))}
        <select
          onChange={(e) => {
            if (e.target.value) { onAssignTag(entry.id, Number(e.target.value)); e.target.value = ''; }
          }}
          className="text-xs bg-transparent text-[var(--text)] opacity-60"
          defaultValue=""
        >
          <option value="" disabled>{t("library.detail.addTag")}</option>
          {tags.filter(t => !entry.tags.some(et => et.id === t.id)).map(tag => (
            <option key={tag.id} value={tag.id}>{tag.name}</option>
          ))}
        </select>
      </div>

      {/* Watch filter */}
      <div className="flex items-center gap-2">
        <span className="text-xs text-[var(--text)] opacity-60">{t("library.detail.filter")}:</span>
        {(['all', 'watched', 'unwatched'] as WatchFilter[]).map(filter => (
          <button
            key={filter}
            onClick={() => setWatchFilter(filter)}
            className={`px-3 py-1 rounded-full text-xs capitalize transition-all ${
              watchFilter === filter
                ? 'bg-[var(--accent)] text-white'
                : 'bg-[var(--card)] text-[var(--text)] opacity-60 hover:opacity-100'
            }`}
            aria-label={`Filter by ${filter}`}
          >
            {t(`library.detail.filter${filter.charAt(0).toUpperCase()}${filter.slice(1)}`)}
          </button>
        ))}
      </div>

      {/* Episode grid */}
      <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-2">
        {filteredEpisodes.map((ep) => {
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
              className={`p-3 rounded-lg border text-center text-sm transition-all relative ${getStatusColor(ep.status)} ${isClickable ? 'cursor-pointer hover:scale-105 hover:shadow-md' : ''} ${ep.watched ? 'opacity-70' : ''}`}
            >
              <div className="font-medium">{t("library.detail.epPrefix")} {ep.episodeNumber}</div>
              <div className="text-xs mt-1 capitalize">{ep.status}</div>
              {ep.status === 'completed' && (
                <Play size={12} className="mx-auto mt-1 opacity-40" />
              )}
              <button
                onClick={(e) => handleToggleWatched(ep.episodeNumber, e)}
                className="absolute top-1 right-1 p-1 rounded hover:bg-black/20 transition-all"
                title={ep.watched ? t("library.detail.markUnwatched") : t("library.detail.markWatched")}
              >
                {ep.watched ? <EyeOff size={14} /> : <Eye size={14} />}
              </button>
            </div>
          );
        })}
      </div>
    </div>
  );
}

export default SeriesDetail;

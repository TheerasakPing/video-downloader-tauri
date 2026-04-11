import { useLibrary } from '../hooks/useLibrary';
import { LibraryCard } from './LibraryCard';
import { SeriesDetail } from './SeriesDetail';
import { Search, Plus, Loader2 } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

export function LibraryPanel() {
  const { entries, tags, loading, detail, query, search, updateQuery, loadDetail, remove, closeDetail, toggleFavorite, createTag, assignTag, unassignTag, openEpisode } = useLibrary();

  const handleRefetch = async (id: number) => {
    try {
      await invoke('cmd_refetch_series', { libraryId: id });
      loadDetail(id); // Refresh detail view
    } catch (e) {
      console.error('Refetch failed:', e);
    }
  };

  const sources = [...new Set(entries.map(e => e.source))];

  if (detail) {
    return (
      <div className="p-4">
        <SeriesDetail
          detail={detail}
          onBack={closeDetail}
          onRemove={remove}
          onRefetch={handleRefetch}
          onToggleFavorite={toggleFavorite}
          tags={tags}
          onAssignTag={assignTag}
          onUnassignTag={unassignTag}
          onOpenEpisode={openEpisode}
        />
      </div>
    );
  }

  return (
    <div className="p-4 space-y-4">
      {/* Search and filter bar */}
      <div className="flex items-center gap-3">
        <div className="flex-1 relative">
          <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--text)] opacity-40" />
          <input
            type="text"
            value={query.search || ''}
            onChange={(e) => search(e.target.value)}
            placeholder="Search library..."
            className="w-full pl-9 pr-4 py-2 rounded-lg bg-[var(--card)] border border-[var(--border)] text-[var(--text)] text-sm focus:outline-none focus:border-[var(--accent)]"
          />
        </div>
        <select
          value={query.source || 'all'}
          onChange={(e) => updateQuery({ source: e.target.value === 'all' ? undefined : e.target.value })}
          className="px-3 py-2 rounded-lg bg-[var(--card)] border border-[var(--border)] text-[var(--text)] text-sm"
        >
          <option value="all">All sources</option>
          {sources.map(s => (
            <option key={s} value={s}>{s}</option>
          ))}
        </select>
      </div>

      {/* Toolbar */}
      <div className="flex items-center gap-2 flex-wrap">
        <select
          value={query.sort || 'date_added'}
          onChange={(e) => updateQuery({ sort: e.target.value })}
          className="px-2 py-1 rounded bg-[var(--card)] border border-[var(--border)] text-[var(--text)] text-xs"
        >
          <option value="date_added">Date Added</option>
          <option value="title">Title</option>
          <option value="progress">Progress</option>
          <option value="source">Source</option>
          <option value="last_downloaded">Last Downloaded</option>
        </select>

        <button
          onClick={() => updateQuery({ order: query.order === 'asc' ? 'desc' : 'asc' })}
          className="px-2 py-1 rounded bg-[var(--card)] border border-[var(--border)] text-[var(--text)] text-xs"
          title={query.order === 'asc' ? 'Ascending' : 'Descending'}
        >
          {query.order === 'asc' ? '↑' : '↓'}
        </button>

        <button
          onClick={() => updateQuery({ favoriteOnly: query.favoriteOnly ? undefined : true })}
          className={`px-2 py-1 rounded text-xs border ${query.favoriteOnly ? 'bg-yellow-500/20 border-yellow-500/40 text-yellow-400' : 'bg-[var(--card)] border-[var(--border)] text-[var(--text)]'}`}
        >
          ★ Favorites
        </button>

        <select
          value={query.status || 'all'}
          onChange={(e) => updateQuery({ status: e.target.value === 'all' ? undefined : e.target.value })}
          className="px-2 py-1 rounded bg-[var(--card)] border border-[var(--border)] text-[var(--text)] text-xs"
        >
          <option value="all">All status</option>
          <option value="complete">Complete</option>
          <option value="in_progress">In Progress</option>
          <option value="not_started">Not Started</option>
        </select>
      </div>

      {/* Tag filter chips */}
      {tags.length > 0 && (
        <div className="flex gap-2 overflow-x-auto pb-1">
          <button
            onClick={() => updateQuery({ tagId: undefined })}
            className={`px-3 py-1 rounded-full text-xs whitespace-nowrap ${!query.tagId ? 'bg-[var(--accent)] text-white' : 'bg-[var(--card)] text-[var(--text)] border border-[var(--border)]'}`}
          >
            All
          </button>
          {tags.map(tag => (
            <button
              key={tag.id}
              onClick={() => updateQuery({ tagId: query.tagId === tag.id ? undefined : tag.id })}
              className={`px-3 py-1 rounded-full text-xs whitespace-nowrap ${query.tagId === tag.id ? 'bg-[var(--accent)] text-white' : 'bg-[var(--card)] text-[var(--text)] border border-[var(--border)]'}`}
            >
              {tag.name} ({tag.usageCount})
            </button>
          ))}
          <button
            onClick={() => {
              const name = prompt('Tag name:');
              if (name?.trim()) createTag(name.trim());
            }}
            className="px-3 py-1 rounded-full text-xs whitespace-nowrap bg-[var(--card)] text-[var(--accent)] border border-dashed border-[var(--accent)]/40"
          >
            + New Tag
          </button>
        </div>
      )}

      {/* Library grid */}
      {loading ? (
        <div className="flex justify-center py-12">
          <Loader2 size={32} className="animate-spin text-[var(--accent)]" />
        </div>
      ) : entries.length === 0 ? (
        <div className="text-center py-12 text-[var(--text)] opacity-40">
          <Plus size={48} className="mx-auto mb-3 opacity-30" />
          <p>No series in library yet</p>
          <p className="text-sm mt-1">Fetch a series to auto-add it here</p>
        </div>
      ) : (
        <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4">
          {entries.map(entry => (
            <LibraryCard
              key={entry.id}
              entry={entry}
              onClick={loadDetail}
              onRemove={remove}
              onToggleFavorite={toggleFavorite}
            />
          ))}
        </div>
      )}
    </div>
  );
}

export default LibraryPanel;

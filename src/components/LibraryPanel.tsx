import { useState } from 'react';
import { useLibrary } from '../hooks/useLibrary';
import { useVirtualScroll } from '../hooks/useVirtualScroll';
import { LibraryCard } from './LibraryCard';
import { SeriesDetail } from './SeriesDetail';
import { LibraryStats } from './LibraryStats';
import ImportExport from './ImportExport';
import { useToast } from '../hooks/useToast';
import { Search, Plus, Loader2, BarChart3 } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { useI18n } from '../hooks/useI18n';

interface VirtualizedLibraryGridProps {
  entries: any[];
  onLoadDetail: (id: number) => void;
  onRemove: (id: number) => void;
  onToggleFavorite: (id: number) => void;
}

function VirtualizedLibraryGrid({ entries, onLoadDetail, onRemove, onToggleFavorite }: VirtualizedLibraryGridProps) {
  // Estimate card height: 320px (poster ~280px + padding/text)
  const ITEM_HEIGHT = 320;
  const { containerRef, visibleItems, totalHeight, handleScroll } = useVirtualScroll({
    itemCount: entries.length,
    itemHeight: ITEM_HEIGHT,
    overscan: 5,
  });

  // Calculate grid columns based on screen width
  const getCols = () => {
    if (typeof window === 'undefined') return 2;
    const width = window.innerWidth;
    if (width >= 1024) return 5;
    if (width >= 768) return 4;
    if (width >= 640) return 3;
    return 2;
  };

  const cols = getCols();
  const itemsPerRow = cols;

  return (
    <div
      ref={containerRef}
      onScroll={handleScroll}
      className="overflow-y-auto max-h-[calc(100vh-300px)]"
      style={{ position: 'relative' }}
    >
      <div style={{ height: totalHeight }}>
        {visibleItems.map((itemIndex) => {
          const entry = entries[itemIndex];
          if (!entry) return null;

          const row = Math.floor(itemIndex / itemsPerRow);
          const col = itemIndex % itemsPerRow;
          const rowOffset = row * ITEM_HEIGHT;
          const colWidth = 100 / itemsPerRow;

          return (
            <div
              key={entry.id}
              style={{
                position: 'absolute',
                top: rowOffset,
                left: `${col * colWidth}%`,
                width: `${colWidth}%`,
                height: ITEM_HEIGHT,
                padding: '8px',
                boxSizing: 'border-box',
              }}
            >
              <LibraryCard
                entry={entry}
                onClick={onLoadDetail}
                onRemove={onRemove}
                onToggleFavorite={onToggleFavorite}
              />
            </div>
          );
        })}
      </div>
    </div>
  );
}

export function LibraryPanel() {
  const { t } = useI18n();
  const { entries, tags, loading, detail, query, search, updateQuery, loadDetail, remove, closeDetail, toggleFavorite, createTag, assignTag, unassignTag, openEpisode, refresh } = useLibrary();
  const [showStats, setShowStats] = useState(false);
  const { success, error } = useToast();

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
            placeholder={t("library.search")}
            aria-label={t("library.search")}
            className="w-full pl-9 pr-4 py-2 rounded-lg bg-[var(--card)] border border-[var(--border)] text-[var(--text)] text-sm focus:outline-none focus:border-[var(--accent)]"
          />
        </div>
        <select
          value={query.source || 'all'}
          onChange={(e) => updateQuery({ source: e.target.value === 'all' ? undefined : e.target.value })}
          aria-label="Filter by source"
          className="px-3 py-2 rounded-lg bg-[var(--card)] border border-[var(--border)] text-[var(--text)] text-sm"
        >
          <option value="all">{t("library.allSources")}</option>
          {sources.map(s => (
            <option key={s} value={s}>{s}</option>
          ))}
        </select>
      </div>

      {/* Toolbar */}
      <div className="flex items-center gap-2 flex-wrap" role="toolbar" aria-label={t("library.filtersAndSorting")}>
        <select
          value={query.sort || 'date_added'}
          onChange={(e) => updateQuery({ sort: e.target.value })}
          aria-label="Sort by"
          className="px-2 py-1 rounded bg-[var(--card)] border border-[var(--border)] text-[var(--text)] text-xs"
        >
          <option value="date_added">{t("library.dateAdded")}</option>
          <option value="title">{t("library.title")}</option>
          <option value="progress">{t("library.progress")}</option>
          <option value="source">{t("library.source")}</option>
          <option value="last_downloaded">{t("library.lastDownloaded")}</option>
        </select>

        <button
          onClick={() => updateQuery({ order: query.order === 'asc' ? 'desc' : 'asc' })}
          className="px-2 py-1 rounded bg-[var(--card)] border border-[var(--border)] text-[var(--text)] text-xs"
          title={query.order === 'asc' ? t("library.ascending") : t("library.descending")}
          aria-label={`${t("library.sortBy")}: ${query.order === 'asc' ? t("library.ascending") : t("library.descending")}`}
        >
          {query.order === 'asc' ? '↑' : '↓'}
        </button>

        <button
          onClick={() => updateQuery({ favoriteOnly: query.favoriteOnly ? undefined : true })}
          className={`px-2 py-1 rounded text-xs border ${query.favoriteOnly ? 'bg-yellow-500/20 border-yellow-500/40 text-yellow-400' : 'bg-[var(--card)] border-[var(--border)] text-[var(--text)]'}`}
          aria-label="Toggle favorites filter"
          aria-pressed={query.favoriteOnly || false}
        >
          {t("library.favorites")}
        </button>

        <select
          value={query.status || 'all'}
          onChange={(e) => updateQuery({ status: e.target.value === 'all' ? undefined : e.target.value })}
          aria-label="Filter by status"
          className="px-2 py-1 rounded bg-[var(--card)] border border-[var(--border)] text-[var(--text)] text-xs"
        >
          <option value="all">{t("library.allStatus")}</option>
          <option value="complete">{t("library.complete")}</option>
          <option value="in_progress">{t("library.inProgress")}</option>
          <option value="not_started">{t("library.notStarted")}</option>
        </select>

        <button
          onClick={() => setShowStats(!showStats)}
          className={`px-2 py-1 rounded text-xs border ${showStats ? 'bg-violet-500/20 border-violet-500/40 text-violet-400' : 'bg-[var(--card)] border-[var(--border)] text-[var(--text)]'}`}
          title={t("library.stats")}
          aria-label={t("library.stats")}
          aria-pressed={showStats}
        >
          <BarChart3 size={12} className="inline mr-1" />
          {t("library.stats")}
        </button>

        <div className="ml-auto">
          <ImportExport
            onImportComplete={(count) => {
              success(`Imported ${count} series`);
              refresh();
            }}
            onError={(msg) => error(msg)}
            onSuccess={(msg) => success(msg)}
          />
        </div>
      </div>

      {/* Stats Dashboard */}
      {showStats && <LibraryStats />}

      {/* Tag filter chips */}
      {tags.length > 0 && (
        <div className="flex gap-2 overflow-x-auto pb-1">
          <button
            onClick={() => updateQuery({ tagId: undefined })}
            className={`px-3 py-1 rounded-full text-xs whitespace-nowrap ${!query.tagId ? 'bg-[var(--accent)] text-white' : 'bg-[var(--card)] text-[var(--text)] border border-[var(--border)]'}`}
          >
            {t("library.all")}
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
              const name = prompt(`${t("library.tagName")}`);
              if (name?.trim()) createTag(name.trim());
            }}
            className="px-3 py-1 rounded-full text-xs whitespace-nowrap bg-[var(--card)] text-[var(--accent)] border border-dashed border-[var(--accent)]/40"
          >
            {t("library.newTag")}
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
          <p>{t("library.empty")}</p>
          <p className="text-sm mt-1">{t("library.emptyHint")}</p>
        </div>
      ) : entries.length > 50 ? (
        // Virtual scrolling for large lists
        <VirtualizedLibraryGrid
          entries={entries}
          onLoadDetail={loadDetail}
          onRemove={remove}
          onToggleFavorite={toggleFavorite}
        />
      ) : (
        // Regular grid for smaller lists
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

import { useState } from 'react';
import { useLibrary } from '../hooks/useLibrary';
import { LibraryCard } from './LibraryCard';
import { SeriesDetail } from './SeriesDetail';
import { Search, Plus, Loader2 } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

export function LibraryPanel() {
  const { entries, loading, detail, search, loadDetail, remove, closeDetail } = useLibrary();
  const [searchQuery, setSearchQuery] = useState('');
  const [filterSource, setFilterSource] = useState<string>('all');

  const handleSearch = (q: string) => {
    setSearchQuery(q);
    search(q);
  };

  const handleRefetch = async (id: number) => {
    try {
      await invoke('cmd_refetch_series', { libraryId: id });
      loadDetail(id); // Refresh detail view
    } catch (e) {
      console.error('Refetch failed:', e);
    }
  };

  const filtered = filterSource === 'all'
    ? entries
    : entries.filter(e => e.source === filterSource);

  const sources = [...new Set(entries.map(e => e.source))];

  if (detail) {
    return (
      <div className="p-4">
        <SeriesDetail
          detail={detail}
          onBack={closeDetail}
          onRemove={remove}
          onRefetch={handleRefetch}
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
            value={searchQuery}
            onChange={(e) => handleSearch(e.target.value)}
            placeholder="Search library..."
            className="w-full pl-9 pr-4 py-2 rounded-lg bg-[var(--card)] border border-[var(--border)] text-[var(--text)] text-sm focus:outline-none focus:border-[var(--accent)]"
          />
        </div>
        <select
          value={filterSource}
          onChange={(e) => setFilterSource(e.target.value)}
          className="px-3 py-2 rounded-lg bg-[var(--card)] border border-[var(--border)] text-[var(--text)] text-sm"
        >
          <option value="all">All sources</option>
          {sources.map(s => (
            <option key={s} value={s}>{s}</option>
          ))}
        </select>
      </div>

      {/* Library grid */}
      {loading ? (
        <div className="flex justify-center py-12">
          <Loader2 size={32} className="animate-spin text-[var(--accent)]" />
        </div>
      ) : filtered.length === 0 ? (
        <div className="text-center py-12 text-[var(--text)] opacity-40">
          <Plus size={48} className="mx-auto mb-3 opacity-30" />
          <p>No series in library yet</p>
          <p className="text-sm mt-1">Fetch a series to auto-add it here</p>
        </div>
      ) : (
        <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4">
          {filtered.map(entry => (
            <LibraryCard
              key={entry.id}
              entry={entry}
              onClick={loadDetail}
              onRemove={remove}
            />
          ))}
        </div>
      )}
    </div>
  );
}

export default LibraryPanel;

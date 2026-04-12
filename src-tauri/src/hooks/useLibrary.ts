import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { LibraryEntry, SeriesDetail } from '../types';

export function useLibrary() {
  const [entries, setEntries] = useState<LibraryEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [detail, setDetail] = useState<SeriesDetail | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const result = await invoke<LibraryEntry[]>('cmd_get_library');
      setEntries(result);
    } catch (e) {
      console.error('Failed to load library:', e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { refresh(); }, [refresh]);

  const search = useCallback(async (query: string) => {
    if (!query.trim()) { refresh(); return; }
    try {
      const result = await invoke<LibraryEntry[]>('cmd_search_library', { query });
      setEntries(result);
    } catch (e) {
      console.error('Search failed:', e);
    }
  }, [refresh]);

  const loadDetail = useCallback(async (id: number) => {
    try {
      const result = await invoke<SeriesDetail>('cmd_get_series_detail', { libraryId: id });
      setDetail(result);
    } catch (e) {
      console.error('Load detail failed:', e);
    }
  }, []);

  const remove = useCallback(async (id: number) => {
    try {
      await invoke('cmd_remove_from_library', { libraryId: id });
      setEntries(prev => prev.filter(e => e.id !== id));
      if (detail?.entry.id === id) setDetail(null);
    } catch (e) {
      console.error('Remove failed:', e);
    }
  }, [detail]);

  const closeDetail = useCallback(() => setDetail(null), []);

  return { entries, loading, detail, refresh, search, loadDetail, remove, closeDetail };
}

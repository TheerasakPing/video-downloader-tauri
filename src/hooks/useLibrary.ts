import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { LibraryEntry, LibraryTag, LibraryQuery, SeriesDetail } from '../types';

export function useLibrary() {
  const [entries, setEntries] = useState<LibraryEntry[]>([]);
  const [tags, setTags] = useState<LibraryTag[]>([]);
  const [loading, setLoading] = useState(false);
  const [detail, setDetail] = useState<SeriesDetail | null>(null);
  const [query, setQuery] = useState<LibraryQuery>({});

  const refresh = useCallback(async (q?: LibraryQuery) => {
    setLoading(true);
    try {
      const result = await invoke<LibraryEntry[]>('cmd_get_library', { query: q ?? query });
      setEntries(result);
    } catch (e) {
      console.error('Failed to load library:', e);
    } finally {
      setLoading(false);
    }
  }, [query]);

  const loadTags = useCallback(async () => {
    try {
      const result = await invoke<LibraryTag[]>('cmd_get_tags');
      setTags(result);
    } catch (e) {
      console.error('Failed to load tags:', e);
    }
  }, []);

  const updateQuery = useCallback((updates: Partial<LibraryQuery>) => {
    setQuery(prev => {
      const next = { ...prev, ...updates };
      refresh(next);
      return next;
    });
  }, [refresh]);

  useEffect(() => { refresh(); loadTags(); }, [refresh, loadTags]);

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

  const toggleFavorite = useCallback(async (id: number) => {
    try {
      const newState = await invoke<boolean>('cmd_toggle_favorite', { libraryId: id });
      setEntries(prev => prev.map(e => e.id === id ? { ...e, favorite: newState } : e));
      if (detail?.entry.id === id) {
        setDetail(d => d ? { ...d, entry: { ...d.entry, favorite: newState } } : null);
      }
    } catch (e) {
      console.error('Toggle favorite failed:', e);
    }
  }, [detail]);

  const createTag = useCallback(async (name: string) => {
    const id = await invoke<number>('cmd_create_tag', { name });
    await loadTags();
    return id;
  }, [loadTags]);

  const deleteTag = useCallback(async (tagId: number) => {
    await invoke('cmd_delete_tag', { tagId });
    await loadTags();
    await refresh();
  }, [loadTags, refresh]);

  const assignTag = useCallback(async (libraryId: number, tagId: number) => {
    await invoke('cmd_assign_tag', { libraryId, tagId });
    await refresh();
    if (detail?.entry.id === libraryId) await loadDetail(libraryId);
  }, [refresh, detail, loadDetail]);

  const unassignTag = useCallback(async (libraryId: number, tagId: number) => {
    await invoke('cmd_unassign_tag', { libraryId, tagId });
    await refresh();
    if (detail?.entry.id === libraryId) await loadDetail(libraryId);
  }, [refresh, detail, loadDetail]);

  const openEpisode = useCallback(async (libraryId: number, episodeNumber: number) => {
    try {
      await invoke('cmd_open_episode', { libraryId, episodeNumber });
    } catch (e) {
      console.error('Open episode failed:', e);
    }
  }, []);

  const closeDetail = useCallback(() => setDetail(null), []);

  const search = useCallback((q: string) => {
    updateQuery({ search: q || undefined });
  }, [updateQuery]);

  return {
    entries, tags, loading, detail, query,
    refresh, search, updateQuery, loadTags,
    loadDetail, remove, closeDetail,
    toggleFavorite, createTag, deleteTag, assignTag, unassignTag, openEpisode,
  };
}

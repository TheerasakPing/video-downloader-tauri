import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface PersistentQueueItem {
  id: number;
  url: string;
  status: string;
  seriesInfo?: string;
  error?: string;
  priority: number;
  createdAt: string;
  updatedAt: string;
}

interface QueueStats {
  pending: number;
  downloading: number;
  completed: number;
  failed: number;
}

export function usePersistentQueue() {
  const [items, setItems] = useState<PersistentQueueItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [stats, setStats] = useState<QueueStats>({
    pending: 0,
    downloading: 0,
    completed: 0,
    failed: 0,
  });

  // Load from backend on mount
  const loadQueue = useCallback(async () => {
    try {
      const result = await invoke<PersistentQueueItem[]>('get_persistent_queue');
      setItems(result);

      // Also load stats
      const statsResult = await invoke<QueueStats>('get_queue_stats');
      setStats(statsResult);
    } catch (e) {
      console.error('Failed to load persistent queue:', e);
    } finally {
      setLoading(false);
    }
  }, []);

  // Add item to persistent queue
  const addItem = useCallback(async (url: string): Promise<number> => {
    const id = await invoke<number>('persist_queue_item', { url });
    await loadQueue();
    return id;
  }, [loadQueue]);

  // Update item status
  const updateItem = useCallback(async (
    id: number,
    status: string,
    seriesInfo?: string,
    error?: string
  ) => {
    await invoke('update_queue_item', {
      id,
      status,
      seriesInfo: seriesInfo || null,
      error: error || null,
    });
    await loadQueue();
  }, [loadQueue]);

  // Remove item
  const removeItem = useCallback(async (id: number) => {
    await invoke('remove_queue_item', { id });
    await loadQueue();
  }, [loadQueue]);

  // Clear completed
  const clearCompleted = useCallback(async () => {
    await invoke('clear_queue_completed');
    await loadQueue();
  }, [loadQueue]);

  // Restore pending items
  const restoreQueue = useCallback(async () => {
    const items = await invoke<PersistentQueueItem[]>('restore_queue');
    return items;
  }, []);

  // Get stats
  const getStats = useCallback(async () => {
    const statsResult = await invoke<QueueStats>('get_queue_stats');
    setStats(statsResult);
    return statsResult;
  }, []);

  useEffect(() => {
    loadQueue();
  }, [loadQueue]);

  return {
    items,
    loading,
    stats,
    addItem,
    updateItem,
    removeItem,
    clearCompleted,
    restoreQueue,
    getStats,
    refresh: loadQueue,
  };
}

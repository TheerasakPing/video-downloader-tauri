import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface LibraryStats {
  totalSeries: number;
  totalEpisodes: number;
  completedEpisodes: number;
  totalSizeBytes: number;
  bySource: SourceStat[];
  byStatus: StatusStat;
  byMonth: MonthStat[];
  favoriteCount: number;
  tagCount: number;
}

export interface SourceStat {
  source: string;
  seriesCount: number;
  episodeCount: number;
}

export interface StatusStat {
  complete: number;
  inProgress: number;
  notStarted: number;
}

export interface MonthStat {
  month: string;
  count: number;
}

export function useLibraryStats() {
  const [stats, setStats] = useState<LibraryStats | null>(null);
  const [loading, setLoading] = useState(true);

  const loadStats = useCallback(async () => {
    setLoading(true);
    try {
      const result = await invoke<LibraryStats>('cmd_get_library_stats');
      setStats(result);
    } catch (e) {
      console.error('Failed to load stats:', e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { loadStats(); }, [loadStats]);

  return { stats, loading, refresh: loadStats };
}

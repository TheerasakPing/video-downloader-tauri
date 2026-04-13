import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface ScheduleEntry {
  id: number;
  name: string;
  url: string;
  outputDir: string;
  cronExpression: string;
  enabled: boolean;
  lastRun?: string;
  nextRun?: string;
  createdAt: string;
}

export interface CreateScheduleRequest {
  name: string;
  url: string;
  outputDir: string;
  cronExpression: string;
}

export interface UpdateScheduleRequest {
  id: number;
  name?: string;
  url?: string;
  outputDir?: string;
  cronExpression?: string;
}

export function useScheduler() {
  const [schedules, setSchedules] = useState<ScheduleEntry[]>([]);
  const [loading, setLoading] = useState(false);

  const loadSchedules = useCallback(async () => {
    setLoading(true);
    try {
      const result = await invoke<ScheduleEntry[]>('cmd_get_schedules');
      setSchedules(result);
    } catch (e) {
      console.error('Failed to load schedules:', e);
    } finally {
      setLoading(false);
    }
  }, []);

  const createSchedule = useCallback(async (req: CreateScheduleRequest) => {
    try {
      const result = await invoke<ScheduleEntry>('cmd_create_schedule', { req });
      setSchedules(prev => [...prev, result]);
      return result;
    } catch (e) {
      console.error('Create schedule failed:', e);
      throw e;
    }
  }, []);

  const updateSchedule = useCallback(async (req: UpdateScheduleRequest) => {
    try {
      const result = await invoke<ScheduleEntry>('cmd_update_schedule', { req });
      setSchedules(prev => prev.map(s => s.id === req.id ? result : s));
      return result;
    } catch (e) {
      console.error('Update schedule failed:', e);
      throw e;
    }
  }, []);

  const toggleSchedule = useCallback(async (id: number) => {
    try {
      const newState = await invoke<boolean>('cmd_toggle_schedule', { id });
      setSchedules(prev => prev.map(s => s.id === id ? { ...s, enabled: newState } : s));
    } catch (e) {
      console.error('Toggle schedule failed:', e);
      throw e;
    }
  }, []);

  const deleteSchedule = useCallback(async (id: number) => {
    try {
      await invoke('cmd_delete_schedule', { id });
      setSchedules(prev => prev.filter(s => s.id !== id));
    } catch (e) {
      console.error('Delete schedule failed:', e);
      throw e;
    }
  }, []);

  useEffect(() => {
    loadSchedules();
  }, [loadSchedules]);

  return {
    schedules,
    loading,
    loadSchedules,
    createSchedule,
    updateSchedule,
    toggleSchedule,
    deleteSchedule,
  };
}

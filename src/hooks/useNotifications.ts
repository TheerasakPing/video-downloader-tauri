import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface NotificationEntry {
  id: number;
  category: string;
  title: string;
  message: string;
  read: boolean;
  actionType: string | null;
  actionData: string | null;
  createdAt: string;
}

export function useNotifications() {
  const [notifications, setNotifications] = useState<NotificationEntry[]>([]);
  const [unreadCount, setUnreadCount] = useState(0);
  const [loading, setLoading] = useState(false);

  const loadNotifications = useCallback(async (limit: number = 50, unreadOnly: boolean = false) => {
    setLoading(true);
    try {
      const result = await invoke<NotificationEntry[]>('cmd_get_notifications', {
        limit,
        unreadOnly,
      });
      setNotifications(result);
    } catch (e) {
      console.error('Failed to load notifications:', e);
    } finally {
      setLoading(false);
    }
  }, []);

  const loadUnreadCount = useCallback(async () => {
    try {
      const count = await invoke<number>('cmd_get_unread_count');
      setUnreadCount(count);
    } catch (e) {
      console.error('Failed to load unread count:', e);
    }
  }, []);

  const markRead = useCallback(async (id: number) => {
    try {
      await invoke('cmd_mark_notification_read', { id });
      setNotifications(prev =>
        prev.map(n => (n.id === id ? { ...n, read: true } : n))
      );
      await loadUnreadCount();
    } catch (e) {
      console.error('Failed to mark notification as read:', e);
    }
  }, [loadUnreadCount]);

  const markAllRead = useCallback(async () => {
    try {
      await invoke('cmd_mark_all_read');
      setNotifications(prev => prev.map(n => ({ ...n, read: true })));
      await loadUnreadCount();
    } catch (e) {
      console.error('Failed to mark all as read:', e);
    }
  }, [loadUnreadCount]);

  const clearOld = useCallback(async (days: number = 30) => {
    try {
      await invoke('cmd_clear_notifications', { days });
      await loadNotifications();
      await loadUnreadCount();
    } catch (e) {
      console.error('Failed to clear notifications:', e);
    }
  }, [loadNotifications, loadUnreadCount]);

  const logNotification = useCallback(async (
    category: string,
    title: string,
    message: string,
    actionType?: string,
    actionData?: string
  ) => {
    try {
      await invoke('cmd_log_notification', {
        category,
        title,
        message,
        actionType: actionType || null,
        actionData: actionData || null,
      });
      await loadUnreadCount();
    } catch (e) {
      console.error('Failed to log notification:', e);
    }
  }, [loadUnreadCount]);

  // Poll for unread count every 30 seconds
  useEffect(() => {
    loadUnreadCount();
    const interval = setInterval(loadUnreadCount, 30000);
    return () => clearInterval(interval);
  }, [loadUnreadCount]);

  return {
    notifications,
    unreadCount,
    loading,
    loadNotifications,
    markRead,
    markAllRead,
    clearOld,
    logNotification,
  };
}

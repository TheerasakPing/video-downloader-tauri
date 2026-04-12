import { useState, useEffect, useRef } from 'react';
import { Bell, ArrowDownCircle, Clock, Settings, RefreshCw, Loader2 } from 'lucide-react';
import { useNotifications, NotificationEntry } from '../hooks/useNotifications';
import { useI18n } from '../hooks/useI18n';

interface NotificationCenterProps {
  onAction?: (actionType: string, actionData: string | null) => void;
}

export function NotificationCenter({ onAction }: NotificationCenterProps) {
  const { notifications, unreadCount, loading, loadNotifications, markRead, markAllRead, clearOld } = useNotifications();
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const { t, language } = useI18n();

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [isOpen]);

  // Load notifications when dropdown opens
  useEffect(() => {
    if (isOpen) {
      loadNotifications(50, false);
    }
  }, [isOpen, loadNotifications]);

  const handleNotificationClick = async (notification: NotificationEntry) => {
    if (!notification.read) {
      await markRead(notification.id);
    }
    if (notification.actionType && onAction) {
      onAction(notification.actionType, notification.actionData);
    }
    setIsOpen(false);
  };

  const handleMarkAllRead = async () => {
    await markAllRead();
  };

  const handleClearAll = async () => {
    if (confirm(t("notifications.clearConfirm"))) {
      await clearOld(30);
    }
  };

  const getCategoryIcon = (category: string) => {
    switch (category.toLowerCase()) {
      case 'download':
        return <ArrowDownCircle size={16} className="text-cyan-400" />;
      case 'schedule':
        return <Clock size={16} className="text-amber-400" />;
      case 'system':
        return <Settings size={16} className="text-slate-400" />;
      case 'update':
        return <RefreshCw size={16} className="text-emerald-400" />;
      default:
        return <Bell size={16} className="text-slate-400" />;
    }
  };

  const timeAgo = (dateStr: string): string => {
    const seconds = Math.floor((Date.now() - new Date(dateStr).getTime()) / 1000);
    if (seconds < 60) return t("notifications.justNow");
    if (seconds < 3600) return t("notifications.minutesAgo", { m: Math.floor(seconds / 60) });
    if (seconds < 86400) return t("notifications.hoursAgo", { h: Math.floor(seconds / 3600) });
    if (seconds < 604800) return t("notifications.daysAgo", { d: Math.floor(seconds / 86400) });
    return new Date(dateStr).toLocaleDateString(language === "th" ? "th-TH" : undefined);
  };

  return (
    <div className="relative" ref={dropdownRef}>
      {/* Bell icon button */}
      <button
        className="relative p-1.5 hover:bg-slate-700/50 rounded-md transition-colors"
        onClick={() => setIsOpen(!isOpen)}
        title={unreadCount > 0 ? t("notifications.unread", { count: unreadCount, plural: unreadCount > 1 ? "s" : "" }) : t("notifications.title")}
        aria-label={unreadCount > 0 ? t("notifications.unread", { count: unreadCount, plural: unreadCount > 1 ? "s" : "" }) : t("notifications.title")}
        aria-haspopup="true"
        aria-expanded={isOpen}
      >
        <Bell size={14} className="text-amber-400" />
        {unreadCount > 0 && (
          <span className="absolute -top-0.5 -right-0.5 bg-red-500 text-white text-[8px] font-bold rounded-full w-3.5 h-3.5 flex items-center justify-center" aria-hidden="true">
            {unreadCount > 9 ? '9+' : unreadCount}
          </span>
        )}
      </button>

      {/* Dropdown panel */}
      {isOpen && (
        <div className="absolute right-0 top-full mt-2 w-80 max-h-96 bg-slate-800/95 backdrop-blur-xl border border-slate-700/50 rounded-xl shadow-2xl overflow-hidden z-50">
          {/* Header */}
          <div className="flex items-center justify-between px-4 py-3 border-b border-slate-700/50">
            <div className="flex items-center gap-2">
              <h3 className="text-sm font-semibold text-white">{t("notifications.title")}</h3>
              {unreadCount > 0 && (
                <span className="px-1.5 py-0.5 bg-red-500/20 text-red-400 text-[10px] font-bold rounded-full">
                  {unreadCount}
                </span>
              )}
            </div>
            {unreadCount > 0 && (
              <button
                onClick={handleMarkAllRead}
                className="text-[10px] text-blue-400 hover:text-blue-300 transition-colors"
                aria-label={t("notifications.markAllReadAria")}
              >
                {t("notifications.markAllRead")}
              </button>
            )}
          </div>

          {/* Notifications list */}
          <div className="overflow-y-auto max-h-72">
            {loading ? (
              <div className="flex justify-center py-8">
                <Loader2 size={24} className="animate-spin text-blue-400" />
              </div>
            ) : notifications.length === 0 ? (
              <div className="text-center py-8 text-slate-400 text-sm">
                <Bell size={32} className="mx-auto mb-2 opacity-30" />
                <p>{t("notifications.noNotifications")}</p>
              </div>
            ) : (
              <div className="divide-y divide-slate-700/30" role="list" aria-label={t("notifications.notificationsAria")}>
                {notifications.map((notification) => (
                  <button
                    key={notification.id}
                    onClick={() => handleNotificationClick(notification)}
                    className={`w-full px-4 py-3 flex items-start gap-3 hover:bg-slate-700/30 transition-colors text-left ${
                      !notification.read ? 'bg-slate-700/20' : ''
                    }`}
                    role="listitem"
                    aria-label={`${notification.title}: ${notification.message}`}
                    aria-readonly={!notification.read ? 'true' : 'false'}
                  >
                    {/* Category icon */}
                    <div className="mt-0.5 flex-shrink-0" aria-hidden="true">
                      {getCategoryIcon(notification.category)}
                    </div>

                    {/* Content */}
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium text-white truncate">
                        {notification.title}
                      </p>
                      <p className="text-xs text-slate-400 truncate mt-0.5">
                        {notification.message}
                      </p>
                      <p className="text-[10px] text-slate-500 mt-1">
                        {timeAgo(notification.createdAt)}
                      </p>
                    </div>

                    {/* Unread dot */}
                    {!notification.read && (
                      <div className="mt-2 flex-shrink-0" aria-label={t("notifications.unreadLabel")}>
                        <div className="w-2 h-2 bg-blue-400 rounded-full" />
                      </div>
                    )}
                  </button>
                ))}
              </div>
            )}
          </div>

          {/* Footer */}
          {notifications.length > 0 && (
            <div className="px-4 py-2 border-t border-slate-700/50 flex justify-center">
              <button
                onClick={handleClearAll}
                className="text-[10px] text-slate-400 hover:text-slate-300 transition-colors"
                aria-label={t("notifications.clearAllAria")}
              >
                {t("notifications.clearAll")}
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

export default NotificationCenter;

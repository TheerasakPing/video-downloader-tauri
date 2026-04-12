import { useState } from 'react';
import { useScheduler, ScheduleEntry } from '../hooks/useScheduler';
import { Clock, Plus, Trash2, ToggleLeft, ToggleRight, Calendar, Loader2, ExternalLink, FolderOpen } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { useI18n } from '../hooks/useI18n';

const CRON_PRESETS_VALUES = [
  { labelKey: 'scheduler.daily', labelThKey: 'scheduler.daily', value: 'daily 02:00', time: '02:00' },
  { labelKey: 'scheduler.daily', value: 'daily 06:00', time: '06:00' },
  { labelKey: 'scheduler.daily', value: 'daily 12:00', time: '12:00' },
  { labelKey: 'scheduler.daily', value: 'daily 18:00', time: '18:00' },
  { labelKey: 'scheduler.weekly', value: 'weekly 0 02:00', day: '0', time: '02:00' },
  { labelKey: 'scheduler.weekly', value: 'weekly 1 02:00', day: '1', time: '02:00' },
  { labelKey: 'scheduler.weekly', value: 'weekly 5 22:00', day: '5', time: '22:00' },
  { labelKey: 'scheduler.hourly', value: 'hourly' },
];

function getCronLabel(t: (key: string, params?: Record<string, string | number>) => string, preset: typeof CRON_PRESETS_VALUES[0]): string {
  if (preset.value === 'hourly') return t('scheduler.hourly');
  if (preset.value.startsWith('daily ')) return t('scheduler.daily', { time: preset.time ?? "" });
  if (preset.value.startsWith('weekly ')) {
    const dayKeys = ['scheduler.sunday', 'scheduler.monday', 'scheduler.tuesday', 'scheduler.wednesday', 'scheduler.thursday', 'scheduler.friday', 'scheduler.saturday'];
    const dayIndex = parseInt(preset.day || '0');
    return t('scheduler.weekly', { day: t(dayKeys[dayIndex]), time: preset.time ?? "" });
  }
  return preset.value;
}

export function SchedulerPanel() {
  const { schedules, loading, createSchedule, toggleSchedule, deleteSchedule } = useScheduler();
  const [showForm, setShowForm] = useState(false);
  const [formData, setFormData] = useState({
    name: '',
    url: '',
    outputDir: '',
    cronExpression: 'daily 02:00',
  });
  const [formError, setFormError] = useState('');
  const { t, language } = useI18n();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setFormError('');

    if (!formData.name.trim()) {
      setFormError(t('scheduler.nameRequired'));
      return;
    }
    if (!formData.url.trim()) {
      setFormError(t('scheduler.urlRequired'));
      return;
    }
    if (!formData.outputDir.trim()) {
      setFormError(t('scheduler.outputDirRequired'));
      return;
    }

    try {
      await createSchedule({
        name: formData.name.trim(),
        url: formData.url.trim(),
        outputDir: formData.outputDir.trim(),
        cronExpression: formData.cronExpression,
      });
      setShowForm(false);
      setFormData({ name: '', url: '', outputDir: '', cronExpression: 'daily 02:00' });
    } catch (err) {
      setFormError(err as string);
    }
  };

  const handleSelectOutputDir = async () => {
    try {
      const dir = await invoke<string>('cmd_select_output_folder');
      if (dir) {
        setFormData(prev => ({ ...prev, outputDir: dir }));
      }
    } catch (e) {
      console.error('Failed to select directory:', e);
    }
  };

  const handleDelete = async (id: number, name: string) => {
    if (confirm(t('scheduler.deleteConfirm', { name }))) {
      try {
        await deleteSchedule(id);
      } catch (e) {
        console.error('Delete failed:', e);
      }
    }
  };

  const formatDateTime = (dateStr?: string) => {
    if (!dateStr) return t('scheduler.never');
    try {
      const date = new Date(dateStr);
      return date.toLocaleString(language === "th" ? "th-TH" : undefined);
    } catch {
      return dateStr;
    }
  };

  const formatCron = (expr: string) => {
    const preset = CRON_PRESETS_VALUES.find(p => p.value === expr);
    if (preset) return getCronLabel(t, preset);

    const expr_lower = expr.toLowerCase();
    if (expr_lower === 'hourly') return t('scheduler.hourly');
    if (expr_lower.startsWith('daily ')) return t('scheduler.daily', { time: expr.substring(6) });
    if (expr_lower.startsWith('weekly ')) {
      const parts = expr.substring(7).split(' ');
      const dayKeys = ['scheduler.sunday', 'scheduler.monday', 'scheduler.tuesday', 'scheduler.wednesday', 'scheduler.thursday', 'scheduler.friday', 'scheduler.saturday'];
      const dayIndex = parseInt(parts[0]) || 0;
      const day = t(dayKeys[dayIndex]);
      return t('scheduler.weekly', { day, time: parts[1] });
    }
    return expr;
  };

  return (
    <div className="p-4 space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Clock size={20} className="text-[var(--accent)]" />
          <h2 className="text-lg font-semibold text-[var(--text)]">{t("scheduler.title")}</h2>
        </div>
        <button
          onClick={() => setShowForm(!showForm)}
          className="flex items-center gap-2 px-3 py-2 rounded-lg bg-[var(--accent)] text-white text-sm hover:opacity-90 transition-opacity"
        >
          <Plus size={16} />
          {showForm ? t("scheduler.cancel") : t("scheduler.addSchedule")}
        </button>
      </div>

      {/* Add Schedule Form */}
      {showForm && (
        <form onSubmit={handleSubmit} className="p-4 rounded-lg bg-[var(--card)] border border-[var(--border)] space-y-4">
          <div>
            <label className="block text-sm font-medium text-[var(--text)] mb-1">{t("scheduler.name")}</label>
            <input
              type="text"
              value={formData.name}
              onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
              placeholder={t("scheduler.namePlaceholder")}
              aria-label={t("scheduler.scheduleNameAria")}
              className="w-full px-3 py-2 rounded-lg bg-[var(--bg)] border border-[var(--border)] text-[var(--text)] text-sm focus:outline-none focus:border-[var(--accent)]"
              autoFocus
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-[var(--text)] mb-1">{t("scheduler.url")}</label>
            <input
              type="text"
              value={formData.url}
              onChange={(e) => setFormData(prev => ({ ...prev, url: e.target.value }))}
              placeholder={t("scheduler.urlPlaceholder")}
              aria-label={t("scheduler.videoUrlAria")}
              className="w-full px-3 py-2 rounded-lg bg-[var(--bg)] border border-[var(--border)] text-[var(--text)] text-sm focus:outline-none focus:border-[var(--accent)]"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-[var(--text)] mb-1">{t("scheduler.outputDir")}</label>
            <div className="flex gap-2">
              <input
                type="text"
                value={formData.outputDir}
                onChange={(e) => setFormData(prev => ({ ...prev, outputDir: e.target.value }))}
                placeholder={t("scheduler.outputDirPlaceholder")}
                aria-label={t("scheduler.outputDirAria")}
                className="flex-1 px-3 py-2 rounded-lg bg-[var(--bg)] border border-[var(--border)] text-[var(--text)] text-sm focus:outline-none focus:border-[var(--accent)]"
              />
              <button
                type="button"
                onClick={handleSelectOutputDir}
                className="px-3 py-2 rounded-lg bg-[var(--bg)] border border-[var(--border)] text-[var(--text)] hover:bg-[var(--border)] transition-colors"
                title={t("scheduler.browseFolder")}
                aria-label={t("scheduler.browseOutputFolder")}
              >
                <FolderOpen size={16} />
              </button>
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium text-[var(--text)] mb-1">{t("scheduler.schedule")}</label>
            <select
              value={formData.cronExpression}
              onChange={(e) => setFormData(prev => ({ ...prev, cronExpression: e.target.value }))}
              aria-label={t("scheduler.selectPreset")}
              className="w-full px-3 py-2 rounded-lg bg-[var(--bg)] border border-[var(--border)] text-[var(--text)] text-sm focus:outline-none focus:border-[var(--accent)]"
            >
              {CRON_PRESETS_VALUES.map(preset => (
                <option key={preset.value} value={preset.value}>{getCronLabel(t, preset)}</option>
              ))}
            </select>
            <p className="mt-1 text-xs text-[var(--text)] opacity-50">
              {t("scheduler.customSchedule")}
            </p>
          </div>

          {formError && (
            <p className="text-sm text-red-400">{formError}</p>
          )}

          <div className="flex gap-2">
            <button
              type="submit"
              className="flex-1 px-4 py-2 rounded-lg bg-[var(--accent)] text-white text-sm font-medium hover:opacity-90 transition-opacity"
            >
              {t("scheduler.create")}
            </button>
            <button
              type="button"
              onClick={() => {
                setShowForm(false);
                setFormError('');
              }}
              className="px-4 py-2 rounded-lg bg-[var(--bg)] border border-[var(--border)] text-[var(--text)] text-sm hover:bg-[var(--border)] transition-colors"
            >
              {t("scheduler.cancel")}
            </button>
          </div>
        </form>
      )}

      {/* Schedules List */}
      {loading ? (
        <div className="flex justify-center py-12">
          <Loader2 size={32} className="animate-spin text-[var(--accent)]" />
        </div>
      ) : schedules.length === 0 ? (
        <div className="text-center py-12 text-[var(--text)] opacity-40">
          <Calendar size={48} className="mx-auto mb-3 opacity-30" />
          <p>{t("scheduler.noSchedules")}</p>
          <p className="text-sm mt-1">{t("scheduler.noSchedulesHint")}</p>
        </div>
      ) : (
        <div className="space-y-3">
          {schedules.map(schedule => (
            <ScheduleCard
              key={schedule.id}
              schedule={schedule}
              onToggle={() => toggleSchedule(schedule.id)}
              onDelete={() => handleDelete(schedule.id, schedule.name)}
              formatCron={formatCron}
              formatDateTime={formatDateTime}
            />
          ))}
        </div>
      )}
    </div>
  );
}

interface ScheduleCardProps {
  schedule: ScheduleEntry;
  onToggle: () => void;
  onDelete: () => void;
  formatCron: (expr: string) => string;
  formatDateTime: (dateStr?: string) => string;
}

function ScheduleCard({ schedule, onToggle, onDelete, formatCron, formatDateTime }: ScheduleCardProps) {
  const { t } = useI18n();

  return (
    <div className={`p-4 rounded-lg border transition-all ${
      schedule.enabled
        ? 'bg-[var(--card)] border-[var(--border)]'
        : 'bg-[var(--card)] border-[var(--border)] opacity-60'
    }`}>
      <div className="flex items-start justify-between gap-4">
        {/* Left side: Info */}
        <div className="flex-1 min-w-0 space-y-2">
          {/* Name + enabled badge */}
          <div className="flex items-center gap-2">
            <h3 className="font-medium text-[var(--text)] truncate">{schedule.name}</h3>
            {!schedule.enabled && (
              <span className="px-2 py-0.5 rounded text-xs bg-yellow-500/20 text-yellow-400 border border-yellow-500/30">
                {t("scheduler.paused")}
              </span>
            )}
          </div>

          {/* URL */}
          <div className="flex items-center gap-1 text-sm text-[var(--text)] opacity-70">
            <ExternalLink size={12} />
            <a
              href={schedule.url}
              target="_blank"
              rel="noopener noreferrer"
              className="truncate hover:underline"
            >
              {schedule.url}
            </a>
          </div>

          {/* Schedule pattern */}
          <div className="flex items-center gap-2 text-sm text-[var(--text)] opacity-80">
            <Clock size={14} className="text-[var(--accent)]" />
            <span>{formatCron(schedule.cronExpression)}</span>
          </div>

          {/* Output dir */}
          <div className="flex items-center gap-1 text-xs text-[var(--text)] opacity-60">
            <FolderOpen size={12} />
            <span className="truncate">{schedule.outputDir}</span>
          </div>

          {/* Run times */}
          <div className="flex gap-4 text-xs text-[var(--text)] opacity-60">
            <span>{t("scheduler.lastRun")} {formatDateTime(schedule.lastRun)}</span>
            <span>{t("scheduler.nextRun")} {formatDateTime(schedule.nextRun)}</span>
          </div>
        </div>

        {/* Right side: Actions */}
        <div className="flex flex-col gap-2">
          <button
            onClick={onToggle}
            className={`p-2 rounded-lg transition-colors ${
              schedule.enabled
                ? 'bg-green-500/20 text-green-400 hover:bg-green-500/30'
                : 'bg-[var(--bg)] text-[var(--text)] hover:bg-[var(--border)]'
            }`}
            title={schedule.enabled ? t("scheduler.disable") : t("scheduler.enable")}
            aria-label={schedule.enabled ? t("scheduler.disable") : t("scheduler.enable")}
            aria-pressed={schedule.enabled}
          >
            {schedule.enabled ? <ToggleRight size={20} /> : <ToggleLeft size={20} />}
          </button>
          <button
            onClick={onDelete}
            className="p-2 rounded-lg bg-red-500/20 text-red-400 hover:bg-red-500/30 transition-colors"
            title={t("scheduler.delete")}
            aria-label={t("scheduler.deleteConfirm", { name: schedule.name })}
          >
            <Trash2 size={18} />
          </button>
        </div>
      </div>
    </div>
  );
}

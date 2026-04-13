import { useLibraryStats } from '../hooks/useLibraryStats';
import { BookOpen, Film, HardDrive, Star, Loader2 } from 'lucide-react';
import { useI18n } from '../hooks/useI18n';

interface LibraryStatsProps {
  className?: string;
}

export function LibraryStats({ className = '' }: LibraryStatsProps) {
  const { stats, loading } = useLibraryStats();
  const { t } = useI18n();

  if (loading) {
    return (
      <div className={`flex items-center justify-center py-12 ${className}`}>
        <Loader2 size={32} className="animate-spin text-[var(--accent)]" />
      </div>
    );
  }

  if (!stats) {
    return (
      <div className={`text-center py-12 text-[var(--text)] opacity-40 ${className}`}>
        <p>{t("stats.noStats")}</p>
      </div>
    );
  }

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(i > 1 ? 1 : 0)} ${sizes[i]}`;
  };

  const totalStatus = stats.byStatus.complete + stats.byStatus.inProgress + stats.byStatus.notStarted;
  const completePercent = totalStatus > 0 ? (stats.byStatus.complete / totalStatus) * 100 : 0;
  const inProgressPercent = totalStatus > 0 ? (stats.byStatus.inProgress / totalStatus) * 100 : 0;
  const notStartedPercent = totalStatus > 0 ? (stats.byStatus.notStarted / totalStatus) * 100 : 0;

  const maxMonthCount = Math.max(...stats.byMonth.map(m => m.count), 1);

  const sourceColors = [
    'from-violet-500 to-violet-600',
    'from-cyan-500 to-cyan-600',
    'from-emerald-500 to-emerald-600',
    'from-amber-500 to-amber-600',
    'from-pink-500 to-pink-600',
    'from-indigo-500 to-indigo-600',
  ];

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Summary Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        {/* Total Series */}
        <div className="bg-slate-800/50 backdrop-blur border border-slate-700/50 rounded-xl p-4 hover:border-[var(--accent)]/50 transition-all group">
          <div className="flex items-start justify-between">
            <div>
              <p className="text-sm text-[var(--text)] opacity-60">{t("stats.totalSeries")}</p>
              <p className="text-2xl font-bold text-[var(--text)] mt-1 group-hover:text-[var(--accent)] transition-colors">
                {stats.totalSeries}
              </p>
            </div>
            <div className="p-2 rounded-lg bg-violet-500/20 group-hover:bg-violet-500/30 transition-all">
              <BookOpen size={20} className="text-violet-400" />
            </div>
          </div>
        </div>

        {/* Total Episodes */}
        <div className="bg-slate-800/50 backdrop-blur border border-slate-700/50 rounded-xl p-4 hover:border-[var(--accent)]/50 transition-all group">
          <div className="flex items-start justify-between">
            <div>
              <p className="text-sm text-[var(--text)] opacity-60">{t("stats.totalEpisodes")}</p>
              <p className="text-2xl font-bold text-[var(--text)] mt-1 group-hover:text-[var(--accent)] transition-colors">
                {stats.totalEpisodes}
              </p>
              <p className="text-xs text-[var(--text)] opacity-40 mt-1">
                {t("stats.completed", { count: stats.completedEpisodes })}
              </p>
            </div>
            <div className="p-2 rounded-lg bg-cyan-500/20 group-hover:bg-cyan-500/30 transition-all">
              <Film size={20} className="text-cyan-400" />
            </div>
          </div>
        </div>

        {/* Storage Used */}
        <div className="bg-slate-800/50 backdrop-blur border border-slate-700/50 rounded-xl p-4 hover:border-[var(--accent)]/50 transition-all group">
          <div className="flex items-start justify-between">
            <div>
              <p className="text-sm text-[var(--text)] opacity-60">{t("stats.storageUsed")}</p>
              <p className="text-2xl font-bold text-[var(--text)] mt-1 group-hover:text-[var(--accent)] transition-colors">
                {formatBytes(stats.totalSizeBytes)}
              </p>
            </div>
            <div className="p-2 rounded-lg bg-emerald-500/20 group-hover:bg-emerald-500/30 transition-all">
              <HardDrive size={20} className="text-emerald-400" />
            </div>
          </div>
        </div>

        {/* Favorites */}
        <div className="bg-slate-800/50 backdrop-blur border border-slate-700/50 rounded-xl p-4 hover:border-[var(--accent)]/50 transition-all group">
          <div className="flex items-start justify-between">
            <div>
              <p className="text-sm text-[var(--text)] opacity-60">{t("stats.favorites")}</p>
              <p className="text-2xl font-bold text-[var(--text)] mt-1 group-hover:text-[var(--accent)] transition-colors">
                {stats.favoriteCount}
              </p>
              <p className="text-xs text-[var(--text)] opacity-40 mt-1">
                {t("stats.tags", { count: stats.tagCount })}
              </p>
            </div>
            <div className="p-2 rounded-lg bg-amber-500/20 group-hover:bg-amber-500/30 transition-all">
              <Star size={20} className="text-amber-400" />
            </div>
          </div>
        </div>
      </div>

      {/* Charts Row */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Source Distribution */}
        <div className="bg-slate-800/50 backdrop-blur border border-slate-700/50 rounded-xl p-4">
          <h3 className="text-sm font-medium text-[var(--text)] mb-4">{t("stats.bySource")}</h3>
          <div className="space-y-3">
            {stats.bySource.map((source, idx) => {
              const maxCount = Math.max(...stats.bySource.map(s => s.seriesCount), 1);
              const widthPercent = (source.seriesCount / maxCount) * 100;
              const colorClass = sourceColors[idx % sourceColors.length];

              return (
                <div key={source.source} className="space-y-1">
                  <div className="flex items-center justify-between text-xs">
                    <span className="text-[var(--text)]">{source.source}</span>
                    <span className="text-[var(--text)] opacity-60">
                      {t("stats.sourceInfo", { series: source.seriesCount, episodes: source.episodeCount })}
                    </span>
                  </div>
                  <div className="h-2 bg-slate-700/50 rounded-full overflow-hidden">
                    <div
                      className={`h-full bg-gradient-to-r ${colorClass} rounded-full transition-all`}
                      style={{ width: `${widthPercent}%` }}
                    />
                  </div>
                </div>
              );
            })}
            {stats.bySource.length === 0 && (
              <p className="text-sm text-[var(--text)] opacity-40 text-center py-4">{t("stats.noData")}</p>
            )}
          </div>
        </div>

        {/* Status Breakdown */}
        <div className="bg-slate-800/50 backdrop-blur border border-slate-700/50 rounded-xl p-4">
          <h3 className="text-sm font-medium text-[var(--text)] mb-4">{t("stats.watchStatus")}</h3>

          {/* Stacked progress bar */}
          <div className="h-3 bg-slate-700/50 rounded-full overflow-hidden flex mb-4">
            {completePercent > 0 && (
              <div
                className="bg-emerald-500 h-full transition-all"
                style={{ width: `${completePercent}%` }}
                title={t("stats.completeTitle", { count: stats.byStatus.complete })}
              />
            )}
            {inProgressPercent > 0 && (
              <div
                className="bg-amber-500 h-full transition-all"
                style={{ width: `${inProgressPercent}%` }}
                title={t("stats.inProgressTitle", { count: stats.byStatus.inProgress })}
              />
            )}
            {notStartedPercent > 0 && (
              <div
                className="bg-slate-500 h-full transition-all"
                style={{ width: `${notStartedPercent}%` }}
                title={t("stats.notStartedTitle", { count: stats.byStatus.notStarted })}
              />
            )}
          </div>

          {/* Legend */}
          <div className="grid grid-cols-3 gap-2 text-center">
            <div className="bg-emerald-500/10 rounded-lg p-2 border border-emerald-500/20">
              <p className="text-lg font-bold text-emerald-400">{stats.byStatus.complete}</p>
              <p className="text-xs text-[var(--text)] opacity-60">{t("library.complete")}</p>
            </div>
            <div className="bg-amber-500/10 rounded-lg p-2 border border-amber-500/20">
              <p className="text-lg font-bold text-amber-400">{stats.byStatus.inProgress}</p>
              <p className="text-xs text-[var(--text)] opacity-60">{t("library.inProgress")}</p>
            </div>
            <div className="bg-slate-500/10 rounded-lg p-2 border border-slate-500/20">
              <p className="text-lg font-bold text-slate-400">{stats.byStatus.notStarted}</p>
              <p className="text-xs text-[var(--text)] opacity-60">{t("library.notStarted")}</p>
            </div>
          </div>
        </div>
      </div>

      {/* Monthly Trend */}
      <div className="bg-slate-800/50 backdrop-blur border border-slate-700/50 rounded-xl p-4">
        <h3 className="text-sm font-medium text-[var(--text)] mb-4">{t("stats.monthlyDownloads")}</h3>
        <div className="flex items-end justify-between gap-2 h-32">
          {stats.byMonth.length > 0 ? (
            stats.byMonth.map((month) => {
              const heightPercent = (month.count / maxMonthCount) * 100;

              return (
                <div key={month.month} className="flex-1 flex flex-col items-center gap-2">
                  <div className="w-full bg-slate-700/30 rounded-t-lg relative flex-1 flex items-end overflow-hidden">
                    <div
                      className="w-full bg-gradient-to-t from-[var(--accent)] to-[var(--accent)]/70 rounded-t-lg transition-all hover:from-[var(--accent)] hover:to-[var(--accent)]/90"
                      style={{ height: `${heightPercent}%` }}
                      title={`${month.count} series`}
                    />
                  </div>
                  <span className="text-xs text-[var(--text)] opacity-60 truncate w-full text-center">
                    {month.month.split('-')[1]}
                  </span>
                </div>
              );
            })
          ) : (
            <div className="w-full h-full flex items-center justify-center">
              <p className="text-sm text-[var(--text)] opacity-40">{t("stats.noMonthlyData")}</p>
            </div>
          )}
        </div>
        {stats.byMonth.length > 0 && (
          <div className="flex justify-between mt-2 text-xs text-[var(--text)] opacity-40">
            {stats.byMonth.map(m => (
              <span key={m.month} className="w-full text-center truncate">
                {m.month}
              </span>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

export default LibraryStats;

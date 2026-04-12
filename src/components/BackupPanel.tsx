import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { save, open } from "@tauri-apps/plugin-dialog";
import {
  Database,
  Download,
  Upload,
  Search,
  Trash2,
  Check,
  AlertCircle,
  Loader2,
} from "lucide-react";
import { Button } from "./Button";
import { useI18n } from "../hooks/useI18n";

interface LibraryEntry {
  id: number;
  title: string;
  source: string;
  posterPath?: string;
  totalEpisodes: number;
  completedCount: number;
  favorite: boolean;
  tags: Array<{ id: number; name: string }>;
}

interface DuplicateGroup {
  primary: LibraryEntry;
  duplicates: LibraryEntry[];
}

export function BackupPanel() {
  const [isCreatingBackup, setIsCreatingBackup] = useState(false);
  const [isRestoring, setIsRestoring] = useState(false);
  const [isScanning, setIsScanning] = useState(false);
  const [duplicates, setDuplicates] = useState<DuplicateGroup[]>([]);
  const [message, setMessage] = useState<{ type: "success" | "error"; text: string } | null>(null);
  const { t } = useI18n();

  const showMessage = (type: "success" | "error", text: string) => {
    setMessage({ type, text });
    setTimeout(() => setMessage(null), 5000);
  };

  const handleCreateBackup = async () => {
    try {
      setIsCreatingBackup(true);
      const filePath = await save({
        title: t("backup.saveTitle"),
        defaultPath: `backup_${new Date().toISOString().slice(0, 10)}.json`,
        filters: [{ name: "JSON", extensions: ["json"] }],
      });

      if (filePath) {
        await invoke("cmd_create_backup", { outputPath: filePath });
        showMessage("success", t("backup.created", { path: filePath }));
      }
    } catch (e) {
      showMessage("error", t("backup.createFailed", { error: String(e) }));
    } finally {
      setIsCreatingBackup(false);
    }
  };

  const handleRestoreBackup = async () => {
    try {
      const filePath = await open({
        title: t("backup.selectTitle"),
        multiple: false,
        filters: [{ name: "JSON", extensions: ["json"] }],
      });

      if (filePath && typeof filePath === "string") {
        if (confirm(t("backup.restoreConfirm"))) {
          setIsRestoring(true);
          const rowsRestored = await invoke<number>("cmd_restore_backup", { backupPath: filePath });
          showMessage("success", t("backup.restored", { count: rowsRestored }));
          // Reload page to refresh data
          setTimeout(() => window.location.reload(), 1500);
        }
      }
    } catch (e) {
      showMessage("error", t("backup.restoreFailed", { error: String(e) }));
    } finally {
      setIsRestoring(false);
    }
  };

  const handleFindDuplicates = async () => {
    try {
      setIsScanning(true);
      setDuplicates([]);
      const result = await invoke<[LibraryEntry, LibraryEntry][]>("cmd_find_duplicates");
      const duplicateGroups: DuplicateGroup[] = result.map(([primary, dupes]) => ({
        primary,
        duplicates: Array.isArray(dupes) ? dupes : [dupes],
      }));
      setDuplicates(duplicateGroups);

      if (duplicateGroups.length === 0) {
        showMessage("success", t("backup.noDuplicates"));
      } else {
        showMessage("success", t("backup.foundDuplicates", { count: duplicateGroups.length }));
      }
    } catch (e) {
      showMessage("error", t("backup.failedFindDuplicates", { error: String(e) }));
    } finally {
      setIsScanning(false);
    }
  };

  const handleRemoveDuplicate = async (id: number) => {
    if (confirm(t("backup.removeEntry"))) {
      try {
        await invoke("cmd_remove_from_library", { libraryId: id });
        setDuplicates((prev) =>
          prev.map((group) => ({
            ...group,
            duplicates: group.duplicates.filter((d) => d.id !== id),
          })).filter((group) => group.duplicates.length > 0 || group.primary)
        );
        showMessage("success", t("backup.entryRemoved"));
      } catch (e) {
        showMessage("error", t("backup.removeFailed", { error: String(e) }));
      }
    }
  };

  return (
    <div className="bg-slate-800/50 rounded-xl p-4 border border-slate-700">
      <h3 className="text-sm font-medium text-slate-300 mb-4 flex items-center gap-2">
        <span className="icon-glow icon-glow-sm icon-glow-cyan icon-glow-animated">
          <Database size={16} />
        </span>
        {t("backup.title")}
      </h3>

      {/* Message */}
      {message && (
        <div
          className={`mb-4 p-3 rounded-lg flex items-center gap-2 ${
            message.type === "success"
              ? "bg-emerald-500/20 text-emerald-300 border border-emerald-500/30"
              : "bg-red-500/20 text-red-300 border border-red-500/30"
          }`}
        >
          {message.type === "success" ? (
            <Check size={16} className="flex-shrink-0" />
          ) : (
            <AlertCircle size={16} className="flex-shrink-0" />
          )}
          <span className="text-sm">{message.text}</span>
        </div>
      )}

      {/* Backup & Restore Buttons */}
      <div className="grid grid-cols-2 gap-3 mb-6">
        <Button
          variant="secondary"
          leftIcon={<Download size={14} />}
          onClick={handleCreateBackup}
          disabled={isCreatingBackup}
          className="w-full"
        >
          {isCreatingBackup ? (
            <>
              <Loader2 size={14} className="animate-spin" />
              {t("backup.creating")}
            </>
          ) : (
            t("backup.create")
          )}
        </Button>

        <Button
          variant="secondary"
          leftIcon={<Upload size={14} />}
          onClick={handleRestoreBackup}
          disabled={isRestoring}
          className="w-full"
        >
          {isRestoring ? (
            <>
              <Loader2 size={14} className="animate-spin" />
              {t("backup.restoring")}
            </>
          ) : (
            t("backup.restore")
          )}
        </Button>
      </div>

      {/* Find Duplicates */}
      <div className="border-t border-slate-700 pt-4">
        <div className="flex items-center justify-between mb-3">
          <div>
            <h4 className="text-sm font-medium text-white flex items-center gap-2">
              <span className="icon-glow icon-glow-sm icon-glow-amber">
                <Search size={14} />
              </span>
              {t("backup.findDuplicates")}
            </h4>
            <p className="text-xs text-slate-500">
              {t("backup.findDuplicatesDesc")}
            </p>
          </div>
          <Button
            variant="secondary"
            size="sm"
            leftIcon={<Search size={14} />}
            onClick={handleFindDuplicates}
            disabled={isScanning}
          >
            {isScanning ? (
              <>
                <Loader2 size={14} className="animate-spin" />
                {t("backup.scanning")}
              </>
            ) : (
              t("backup.scan")
            )}
          </Button>
        </div>

        {/* Duplicates List */}
        {duplicates.length > 0 && (
          <div className="space-y-3 max-h-96 overflow-y-auto custom-scrollbar">
            {duplicates.map((group) => (
              <div
                key={group.primary.id}
                className="bg-slate-900/50 rounded-lg p-3 border border-slate-700"
              >
                <div className="flex items-start gap-3">
                  {/* Primary Entry */}
                  <div className="flex-1">
                    <div className="flex items-center gap-2 mb-2">
                      <span className="text-xs bg-emerald-500/20 text-emerald-300 px-2 py-0.5 rounded">
                        {t("backup.keep")}
                      </span>
                      <span className="text-xs text-slate-500">
                        {group.primary.source}
                      </span>
                    </div>
                    <p className="text-sm font-medium text-white">
                      {group.primary.title}
                    </p>
                    <p className="text-xs text-slate-500">
                      {t("backup.episodesCount", { completed: group.primary.completedCount, total: group.primary.totalEpisodes })}
                    </p>
                  </div>

                  {/* Duplicates */}
                  <div className="flex-1 space-y-2">
                    <span className="text-xs text-slate-500">{t("backup.duplicates")}</span>
                    {group.duplicates.map((dup) => (
                      <div
                        key={dup.id}
                        className="flex items-center justify-between bg-red-500/10 rounded px-2 py-1.5 border border-red-500/20"
                      >
                        <div className="flex-1 min-w-0">
                          <p className="text-xs text-slate-300 truncate">
                            {dup.title}
                          </p>
                          <p className="text-[10px] text-slate-500">
                            {dup.source} • {t("backup.episodesCount", { completed: dup.completedCount, total: dup.totalEpisodes })}
                          </p>
                        </div>
                        <Button
                          variant="ghost"
                          size="sm"
                          leftIcon={<Trash2 size={12} />}
                          onClick={() => handleRemoveDuplicate(dup.id)}
                          className="h-6 w-6 p-0 text-red-400 hover:text-red-300"
                        />
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}

        {duplicates.length === 0 && !isScanning && (
          <div className="text-center py-8 text-slate-600">
            <Database size={32} className="mx-auto mb-2 opacity-50" />
            <p className="text-sm">{t("backup.clickToScan")}</p>
          </div>
        )}
      </div>
    </div>
  );
}

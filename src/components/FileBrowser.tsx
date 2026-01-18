import { useState } from "react";
import {
  FolderOpen,
  Film,
  Trash2,
  Play,
  FileVideo,
  HardDrive,
  RefreshCw,
} from "lucide-react";
import { Button } from "./Button";

interface FileInfo {
  name: string;
  path: string;
  size: number;
  isEpisode: boolean;
  isMerged: boolean;
}

interface FileBrowserProps {
  outputDir: string;
  files: FileInfo[];
  onRefresh: () => void;
  onOpenFolder: () => void;
  onDelete: (paths: string[]) => void;
  onPlay: (path: string) => void;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

export function FileBrowser({
  outputDir,
  files,
  onRefresh,
  onOpenFolder,
  onDelete,
  onPlay,
}: FileBrowserProps) {
  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());

  const totalSize = files.reduce((sum, f) => sum + f.size, 0);
  const episodeFiles = files.filter((f) => f.isEpisode);
  const mergedFiles = files.filter((f) => f.isMerged);

  const toggleSelect = (path: string) => {
    setSelectedFiles((prev) => {
      const next = new Set(prev);
      if (next.has(path)) {
        next.delete(path);
      } else {
        next.add(path);
      }
      return next;
    });
  };

  const selectAll = () => {
    setSelectedFiles(new Set(files.map((f) => f.path)));
  };

  const deselectAll = () => {
    setSelectedFiles(new Set());
  };

  const deleteSelected = () => {
    if (selectedFiles.size > 0) {
      onDelete(Array.from(selectedFiles));
      setSelectedFiles(new Set());
    }
  };

  return (
    <div className="space-y-3 sm:space-y-4">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
        <div>
          <h3 className="text-xs sm:text-sm font-medium text-white flex items-center gap-2 truncate">
            <FolderOpen size={14} className="sm:w-4 sm:h-4 flex-shrink-0" />
            <span className="truncate">{outputDir}</span>
          </h3>
          <p className="text-[10px] sm:text-xs text-slate-500 mt-0.5 sm:mt-1">
            {files.length} files • {formatBytes(totalSize)}
          </p>
        </div>
        <div className="flex gap-2">
          <Button
            size="sm"
            variant="ghost"
            leftIcon={<RefreshCw size={14} />}
            onClick={onRefresh}
            className="hover-scale"
          >
            <span className="hidden sm:inline">Refresh</span>
            <span className="sm:hidden">🔄</span>
          </Button>
          <Button
            size="sm"
            variant="secondary"
            leftIcon={<FolderOpen size={14} />}
            onClick={onOpenFolder}
            className="hover-lift"
          >
            <span className="hidden sm:inline">Open Folder</span>
            <span className="sm:hidden">Open</span>
          </Button>
        </div>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-3 gap-2 sm:gap-3">
        <div className="stats-card-glow stats-card-violet rounded-lg p-2 sm:p-3">
          <div className="flex items-center gap-1 sm:gap-2 text-violet-400 text-[10px] sm:text-xs mb-0.5 sm:mb-1">
            <span className="icon-glow icon-glow-sm icon-glow-violet">
              <Film size={12} />
            </span>
            <span className="hidden sm:inline">Episodes</span>
            <span className="sm:hidden">Eps</span>
          </div>
          <div className="text-sm sm:text-lg font-bold text-white">
            {episodeFiles.length}
          </div>
        </div>
        <div className="stats-card-glow stats-card-emerald rounded-lg p-2 sm:p-3">
          <div className="flex items-center gap-1 sm:gap-2 text-emerald-400 text-[10px] sm:text-xs mb-0.5 sm:mb-1">
            <span className="icon-glow icon-glow-sm icon-glow-emerald">
              <FileVideo size={12} />
            </span>
            Merged
          </div>
          <div className="text-sm sm:text-lg font-bold text-white">
            {mergedFiles.length}
          </div>
        </div>
        <div className="stats-card-glow stats-card-blue rounded-lg p-2 sm:p-3">
          <div className="flex items-center gap-1 sm:gap-2 text-blue-400 text-[10px] sm:text-xs mb-0.5 sm:mb-1">
            <span className="icon-glow icon-glow-sm icon-glow-blue">
              <HardDrive size={12} />
            </span>
            <span className="hidden sm:inline">Total Size</span>
            <span className="sm:hidden">Size</span>
          </div>
          <div className="text-sm sm:text-lg font-bold text-white">
            {formatBytes(totalSize)}
          </div>
        </div>
      </div>

      {/* Actions */}
      {files.length > 0 && (
        <div className="flex flex-wrap items-center gap-2 animate-fade-in">
          <Button size="sm" variant="ghost" onClick={selectAll} className="hover-scale btn-ripple">
            Select All
          </Button>
          <Button size="sm" variant="ghost" onClick={deselectAll} className="hover-scale btn-ripple">
            Deselect
          </Button>
          {selectedFiles.size > 0 && (
            <Button
              size="sm"
              variant="danger"
              leftIcon={<Trash2 size={14} />}
              onClick={deleteSelected}
              className="hover-lift btn-ripple animate-scale-in"
            >
              Delete ({selectedFiles.size})
            </Button>
          )}
        </div>
      )}

      {/* File List */}
      <div className="panel-glow overflow-hidden">
        {files.length === 0 ? (
          <div className="p-6 sm:p-8 text-center text-slate-500 text-sm animate-fade-in">
            <div className="icon-glow icon-glow-lg icon-glow-slate mx-auto mb-3">
              <FolderOpen size={28} />
            </div>
            No files in output directory
          </div>
        ) : (
          <div className="max-h-48 sm:max-h-64 overflow-y-auto divide-y divide-slate-700/50 scrollbar-hide">
            {files.map((file, index) => (
              <div
                key={file.path}
                className={`flex items-center gap-2 sm:gap-3 px-3 sm:px-4 py-2 hover:bg-slate-700/30 transition-colors cursor-pointer card-interactive ${
                  selectedFiles.has(file.path) ? "bg-violet-600/20" : ""
                }`}
                onClick={() => toggleSelect(file.path)}
                style={{ animationDelay: `${Math.min(index * 30, 300)}ms` }}
              >
                <input
                  type="checkbox"
                  checked={selectedFiles.has(file.path)}
                  onChange={() => {}}
                  className="w-3.5 h-3.5 sm:w-4 sm:h-4 rounded bg-slate-700 border-slate-600 text-violet-600 focus-ring"
                />
                <div
                  className={`p-1 sm:p-1.5 rounded ${
                    file.isMerged
                      ? "bg-emerald-500/20 text-emerald-400"
                      : "bg-violet-500/20 text-violet-400"
                  }`}
                >
                  <FileVideo size={14} className="sm:w-4 sm:h-4" />
                </div>
                <div className="flex-1 min-w-0">
                  <div className="text-xs sm:text-sm text-white truncate">{file.name}</div>
                  <div className="text-[10px] sm:text-xs text-slate-500">
                    {formatBytes(file.size)}
                  </div>
                </div>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onPlay(file.path);
                  }}
                  className="p-1 sm:p-1.5 text-slate-400 hover:text-white hover:bg-slate-700 rounded-lg transition-colors hover-scale"
                >
                  <Play size={12} className="sm:w-3.5 sm:h-3.5" />
                </button>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

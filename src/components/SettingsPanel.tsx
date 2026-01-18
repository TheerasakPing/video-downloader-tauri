import {
  Moon,
  Sun,
  Monitor,
  Download,
  Bell,
  Volume2,
  Trash2,
  FolderOpen,
  Gauge,
  RotateCcw,
} from "lucide-react";
import { Settings as SettingsType } from "../hooks/useSettings";
import { Button } from "./Button";

interface SettingsPanelProps {
  settings: SettingsType;
  onUpdate: <K extends keyof SettingsType>(key: K, value: SettingsType[K]) => void;
  onReset: () => void;
  onOpenFolder: () => void;
}

export function SettingsPanel({
  settings,
  onUpdate,
  onReset,
  onOpenFolder,
}: SettingsPanelProps) {
  return (
    <div className="space-y-6">
      {/* Download Settings */}
      <section className="bg-slate-800/50 rounded-xl p-4 border border-slate-700">
        <h3 className="text-sm font-medium text-slate-300 mb-4 flex items-center gap-2">
          <Download size={16} />
          Download Settings
        </h3>

        <div className="space-y-4">
          {/* Concurrent Downloads */}
          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm text-white">Concurrent Downloads</label>
              <p className="text-xs text-slate-500">
                Number of episodes to download at once
              </p>
            </div>
            <select
              value={settings.concurrentDownloads}
              onChange={(e) =>
                onUpdate("concurrentDownloads", parseInt(e.target.value))
              }
              className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white"
            >
              {[1, 2, 3, 4, 5].map((n) => (
                <option key={n} value={n}>
                  {n}
                </option>
              ))}
            </select>
          </div>

          {/* Speed Limit */}
          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm text-white flex items-center gap-2">
                <Gauge size={14} />
                Speed Limit
              </label>
              <p className="text-xs text-slate-500">0 = Unlimited</p>
            </div>
            <div className="flex items-center gap-2">
              <input
                type="number"
                min="0"
                step="100"
                value={settings.speedLimit}
                onChange={(e) =>
                  onUpdate("speedLimit", parseInt(e.target.value) || 0)
                }
                className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white w-24"
              />
              <span className="text-xs text-slate-500">KB/s</span>
            </div>
          </div>

          {/* File Naming */}
          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm text-white">File Naming</label>
              <p className="text-xs text-slate-500">Format for episode files</p>
            </div>
            <select
              value={settings.fileNaming}
              onChange={(e) =>
                onUpdate("fileNaming", e.target.value as SettingsType["fileNaming"])
              }
              className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-white"
            >
              <option value="ep_001">ep_001.mp4</option>
              <option value="episode_1">episode_1.mp4</option>
              <option value="title_ep1">Title_EP1.mp4</option>
            </select>
          </div>

          {/* Auto Merge */}
          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm text-white">Auto Merge</label>
              <p className="text-xs text-slate-500">
                Merge videos after download
              </p>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={settings.autoMerge}
                onChange={(e) => onUpdate("autoMerge", e.target.checked)}
                className="sr-only peer"
              />
              <div className="w-11 h-6 bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-violet-600"></div>
            </label>
          </div>

          {/* Delete After Merge */}
          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm text-white flex items-center gap-2">
                <Trash2 size={14} />
                Delete After Merge
              </label>
              <p className="text-xs text-slate-500">
                Remove episode files after merging
              </p>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={settings.deleteAfterMerge}
                onChange={(e) => onUpdate("deleteAfterMerge", e.target.checked)}
                className="sr-only peer"
              />
              <div className="w-11 h-6 bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-violet-600"></div>
            </label>
          </div>
        </div>
      </section>

      {/* Notifications */}
      <section className="bg-slate-800/50 rounded-xl p-4 border border-slate-700">
        <h3 className="text-sm font-medium text-slate-300 mb-4 flex items-center gap-2">
          <Bell size={16} />
          Notifications
        </h3>

        <div className="space-y-4">
          {/* System Notifications */}
          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm text-white">System Notifications</label>
              <p className="text-xs text-slate-500">
                Show notification when download completes
              </p>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={settings.notificationsEnabled}
                onChange={(e) =>
                  onUpdate("notificationsEnabled", e.target.checked)
                }
                className="sr-only peer"
              />
              <div className="w-11 h-6 bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-violet-600"></div>
            </label>
          </div>

          {/* Sound */}
          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm text-white flex items-center gap-2">
                <Volume2 size={14} />
                Sound Alert
              </label>
              <p className="text-xs text-slate-500">Play sound when done</p>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={settings.soundEnabled}
                onChange={(e) => onUpdate("soundEnabled", e.target.checked)}
                className="sr-only peer"
              />
              <div className="w-11 h-6 bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-violet-600"></div>
            </label>
          </div>
        </div>
      </section>

      {/* Appearance */}
      <section className="bg-slate-800/50 rounded-xl p-4 border border-slate-700">
        <h3 className="text-sm font-medium text-slate-300 mb-4 flex items-center gap-2">
          <Sun size={16} />
          Appearance
        </h3>

        <div className="flex gap-2">
          <button
            onClick={() => onUpdate("theme", "light")}
            className={`flex-1 flex items-center justify-center gap-2 px-4 py-3 rounded-lg transition-all ${
              settings.theme === "light"
                ? "bg-violet-600 text-white"
                : "bg-slate-700 text-slate-400 hover:bg-slate-600"
            }`}
          >
            <Sun size={18} />
            Light
          </button>
          <button
            onClick={() => onUpdate("theme", "dark")}
            className={`flex-1 flex items-center justify-center gap-2 px-4 py-3 rounded-lg transition-all ${
              settings.theme === "dark"
                ? "bg-violet-600 text-white"
                : "bg-slate-700 text-slate-400 hover:bg-slate-600"
            }`}
          >
            <Moon size={18} />
            Dark
          </button>
          <button
            onClick={() => onUpdate("theme", "system")}
            className={`flex-1 flex items-center justify-center gap-2 px-4 py-3 rounded-lg transition-all ${
              settings.theme === "system"
                ? "bg-violet-600 text-white"
                : "bg-slate-700 text-slate-400 hover:bg-slate-600"
            }`}
          >
            <Monitor size={18} />
            System
          </button>
        </div>
      </section>

      {/* Actions */}
      <div className="flex gap-3">
        <Button
          variant="secondary"
          leftIcon={<FolderOpen size={16} />}
          onClick={onOpenFolder}
          className="flex-1"
        >
          Open Output Folder
        </Button>
        <Button
          variant="danger"
          leftIcon={<RotateCcw size={16} />}
          onClick={onReset}
        >
          Reset Settings
        </Button>
      </div>
    </div>
  );
}

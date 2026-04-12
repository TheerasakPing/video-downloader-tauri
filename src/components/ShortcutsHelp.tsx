import React, { useEffect, useState } from "react";
import { Keyboard, BookOpen, Download, Compass, Library, Settings } from "lucide-react";
import { SHORTCUTS, SHORTCUT_CATEGORIES } from "../hooks/useKeyboardShortcuts";
import { useI18n } from "../hooks/useI18n";

interface ShortcutsHelpProps {
  isOpen: boolean;
  onClose: () => void;
  language?: "en" | "th";
}

export const ShortcutsHelp: React.FC<ShortcutsHelpProps> = ({
  isOpen,
  onClose,
}) => {
  const { t, language } = useI18n();
  const [activeTab, setActiveTab] = useState<"shortcuts" | "guide">("shortcuts");
  const [selectedCategory, setSelectedCategory] = useState<string>("all");

  // Handle Escape key to close modal
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && isOpen) {
        onClose();
      }
    };

    if (isOpen) {
      document.addEventListener("keydown", handleKeyDown);
    }

    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const categories = [
    { id: "all", label: t("shortcuts.all"), icon: Keyboard },
    { id: "download", label: SHORTCUT_CATEGORIES.download[`label${language === "th" ? "Th" : ""}`], icon: Download },
    { id: "navigation", label: SHORTCUT_CATEGORIES.navigation[`label${language === "th" ? "Th" : ""}`], icon: Compass },
    { id: "library", label: SHORTCUT_CATEGORIES.library[`label${language === "th" ? "Th" : ""}`], icon: Library },
    { id: "ui", label: SHORTCUT_CATEGORIES.ui[`label${language === "th" ? "Th" : ""}`], icon: Settings },
  ];

  const filteredShortcuts = selectedCategory === "all"
    ? SHORTCUTS
    : SHORTCUTS.filter(s => s.category === selectedCategory);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/60 backdrop-blur-sm"
        onClick={onClose}
      />

      {/* Modal */}
      <div className="relative w-full max-w-lg glass rounded-2xl border border-slate-700/50 shadow-2xl animate-scale-in max-h-[90vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-4 border-b border-slate-700/50">
          <div className="flex items-center gap-3">
            <span className="icon-glow icon-glow-sm icon-glow-violet">
              <Keyboard size={18} />
            </span>
            <h2 className="text-lg font-semibold text-white">
              {t("shortcuts.help")}
            </h2>
          </div>
          <button
            onClick={onClose}
            className="text-slate-400 hover:text-white transition-colors"
            aria-label={t("shortcuts.closeShortcuts")}
          >
            ✕
          </button>
        </div>

        {/* Tabs */}
        <div className="flex border-b border-slate-700/50">
          <button
            onClick={() => setActiveTab("shortcuts")}
            className={`flex-1 py-3 text-sm font-medium transition-colors flex items-center justify-center gap-2 ${
              activeTab === "shortcuts"
                ? "text-violet-400 border-b-2 border-violet-400"
                : "text-slate-400 hover:text-slate-300"
            }`}
            aria-label={t("shortcuts.shortcuts")}
          >
            <Keyboard size={14} />
            {t("shortcuts.shortcuts")}
          </button>
          <button
            onClick={() => setActiveTab("guide")}
            className={`flex-1 py-3 text-sm font-medium transition-colors flex items-center justify-center gap-2 ${
              activeTab === "guide"
                ? "text-violet-400 border-b-2 border-violet-400"
                : "text-slate-400 hover:text-slate-300"
            }`}
            aria-label={t("shortcuts.gettingStarted")}
          >
            <BookOpen size={14} />
            {t("shortcuts.gettingStarted")}
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto custom-scrollbar">
          {activeTab === "shortcuts" ? (
            <div className="p-5">
              {/* Category Filter */}
              <div className="flex gap-2 overflow-x-auto pb-4 mb-4">
                {categories.map((cat) => (
                  <button
                    key={cat.id}
                    onClick={() => setSelectedCategory(cat.id)}
                    className={`px-3 py-1.5 text-xs font-medium rounded-full whitespace-nowrap transition-all flex items-center gap-1.5 ${
                      selectedCategory === cat.id
                        ? "bg-violet-500/20 text-violet-300 border border-violet-500/40"
                        : "bg-slate-700/50 text-slate-400 border border-slate-600 hover:text-slate-300"
                    }`}
                    aria-label={cat.label}
                  >
                    <cat.icon size={12} />
                    {cat.label}
                  </button>
                ))}
              </div>

              {/* Shortcuts List */}
              <div className="space-y-1">
                {filteredShortcuts.map((shortcut, index) => (
                  <div
                    key={index}
                    className="flex items-center justify-between py-2.5 px-3 rounded-lg hover:bg-slate-700/30 transition-colors"
                  >
                    <span className="text-slate-300 text-sm">
                      {language === "th" ? shortcut.descriptionTh : shortcut.description}
                    </span>
                    <kbd className="px-2.5 py-1 text-xs font-mono bg-slate-700 text-slate-300 rounded-md border border-slate-600">
                      {shortcut.key}
                    </kbd>
                  </div>
                ))}
              </div>
            </div>
          ) : (
            <div className="p-5 space-y-4">
              <GettingStartedGuide />
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="px-5 py-4 border-t border-slate-700/50">
          <button
            onClick={onClose}
            className="w-full py-2.5 text-sm font-medium text-slate-300 hover:text-white bg-slate-700/50 hover:bg-slate-700 rounded-lg transition-colors"
          >
            {t("shortcuts.close")}
          </button>
        </div>
      </div>
    </div>
  );
};

function GettingStartedGuide() {
  const { t } = useI18n();
  const steps = [
    { title: t("shortcuts.pasteUrl"), description: t("shortcuts.pasteUrlDesc"), shortcut: t("shortcuts.pasteUrlShortcut") },
    { title: t("shortcuts.selectEpisodes"), description: t("shortcuts.selectEpisodesDesc"), shortcut: t("shortcuts.selectEpisodesShortcut") },
    { title: t("shortcuts.startDownload"), description: t("shortcuts.startDownloadDesc"), shortcut: t("shortcuts.startDownloadShortcut") },
    { title: t("shortcuts.manageQueue"), description: t("shortcuts.manageQueueDesc"), shortcut: t("shortcuts.manageQueueShortcut") },
    { title: t("shortcuts.addToLibrary"), description: t("shortcuts.addToLibraryDesc"), shortcut: t("shortcuts.addToLibraryShortcut") },
  ];

  return (
    <div className="space-y-4">
      <h3 className="text-base font-semibold text-white mb-4">
        {t("shortcuts.gettingStarted")}
      </h3>
      {steps.map((step, index) => (
        <div key={index} className="flex gap-3 p-3 rounded-lg bg-slate-800/30 border border-slate-700/50">
          <div className="flex-shrink-0 w-6 h-6 rounded-full bg-violet-500/20 text-violet-400 flex items-center justify-center text-xs font-bold border border-violet-500/30">
            {index + 1}
          </div>
          <div className="flex-1">
            <h4 className="text-sm font-medium text-white mb-1">{step.title}</h4>
            <p className="text-xs text-slate-400 mb-2">{step.description}</p>
            <kbd className="px-2 py-0.5 text-xs font-mono bg-slate-700 text-slate-400 rounded border border-slate-600">
              {step.shortcut}
            </kbd>
          </div>
        </div>
      ))}
      <div className="mt-4 p-3 rounded-lg bg-violet-500/10 border border-violet-500/20">
        <p className="text-xs text-violet-300">
          {t("shortcuts.tip")}
        </p>
      </div>
    </div>
  );
}

export default ShortcutsHelp;

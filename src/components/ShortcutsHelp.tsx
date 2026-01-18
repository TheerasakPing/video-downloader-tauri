import React, { useEffect } from "react";
import { Keyboard } from "lucide-react";
import { SHORTCUTS } from "../hooks/useKeyboardShortcuts";

interface ShortcutsHelpProps {
  isOpen: boolean;
  onClose: () => void;
  language?: "en" | "th";
}

export const ShortcutsHelp: React.FC<ShortcutsHelpProps> = ({
  isOpen,
  onClose,
  language = "en",
}) => {
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

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/60 backdrop-blur-sm"
        onClick={onClose}
      />

      {/* Modal */}
      <div className="relative w-full max-w-md glass rounded-2xl border border-slate-700/50 shadow-2xl animate-scale-in">
        {/* Header */}
        <div className="flex items-center gap-3 px-5 py-4 border-b border-slate-700/50">
          <span className="icon-glow icon-glow-sm icon-glow-violet">
            <Keyboard size={18} />
          </span>
          <h2 className="text-lg font-semibold text-white">
            {language === "th" ? "ปุ่มลัด" : "Keyboard Shortcuts"}
          </h2>
        </div>

        {/* Shortcuts List */}
        <div className="p-5 space-y-2">
          {SHORTCUTS.map((shortcut, index) => (
            <div
              key={index}
              className="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-slate-700/30 transition-colors"
            >
              <span className="text-slate-300">
                {language === "th" ? shortcut.descriptionTh : shortcut.description}
              </span>
              <kbd className="px-2.5 py-1 text-xs font-mono bg-slate-700 text-slate-300 rounded-md border border-slate-600">
                {shortcut.key}
              </kbd>
            </div>
          ))}
        </div>

        {/* Footer */}
        <div className="px-5 py-4 border-t border-slate-700/50">
          <button
            onClick={onClose}
            className="w-full py-2.5 text-sm font-medium text-slate-300 hover:text-white bg-slate-700/50 hover:bg-slate-700 rounded-lg transition-colors"
          >
            {language === "th" ? "ปิด" : "Close"}
          </button>
        </div>
      </div>
    </div>
  );
};

export default ShortcutsHelp;

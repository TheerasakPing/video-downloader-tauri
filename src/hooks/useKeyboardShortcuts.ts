import { useEffect } from "react";

export const SHORTCUTS = [
  {
    key: "Ctrl + V",
    description: "Paste URL from clipboard",
    descriptionTh: "วางลิงก์จากคลิปบอร์ด",
  },
  {
    key: "Ctrl + D",
    description: "Start Download",
    descriptionTh: "เริ่มดาวน์โหลด",
  },
  {
    key: "Space",
    description: "Pause / Resume",
    descriptionTh: "พัก / ต่อการดาวน์โหลด",
  },
  {
    key: "Esc",
    description: "Cancel Download",
    descriptionTh: "ยกเลิกการดาวน์โหลด",
  },
  {
    key: "Ctrl + M",
    description: "Toggle Mini Mode",
    descriptionTh: "เปิด/ปิด โหมดย่อ",
  },
  {
    key: "Ctrl + Tab",
    description: "Next Tab",
    descriptionTh: "แท็บถัดไป",
  },
  {
    key: "Ctrl + Shift + Tab",
    description: "Previous Tab",
    descriptionTh: "แท็บก่อนหน้า",
  },
];

interface ShortcutHandlers {
  onPaste: () => void;
  onDownload: () => void;
  onPauseResume: () => void;
  onCancel: () => void;
  onToggleMiniMode: () => void;
  onNextTab: () => void;
  onPrevTab: () => void;
  isDownloading: boolean;
  isPaused: boolean;
}

export const useKeyboardShortcuts = ({
  onPaste,
  onDownload,
  onPauseResume,
  onCancel,
  onToggleMiniMode,
  onNextTab,
  onPrevTab,
  isDownloading,
  isPaused,
}: ShortcutHandlers) => {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ignore if user is typing in an input
      if (
        document.activeElement?.tagName === "INPUT" ||
        document.activeElement?.tagName === "TEXTAREA"
      ) {
        return;
      }

      const ctrlOrCmd = e.ctrlKey || e.metaKey;

      // Paste: Ctrl+V
      if (ctrlOrCmd && e.key.toLowerCase() === "v") {
        // Let default paste happen if relevant, but here we want to trigger app paste
        // We prevent default to handle it manually via clipboard API
        // e.preventDefault();
        onPaste();
      }

      // Download: Ctrl+D
      if (ctrlOrCmd && e.key.toLowerCase() === "d") {
        e.preventDefault();
        onDownload();
      }

      // Mini Mode: Ctrl+M
      if (ctrlOrCmd && e.key.toLowerCase() === "m") {
        e.preventDefault();
        onToggleMiniMode();
      }

      // Tab Navigation
      if (ctrlOrCmd && e.key === "Tab") {
        e.preventDefault();
        if (e.shiftKey) {
          onPrevTab();
        } else {
          onNextTab();
        }
      }

      // Space: Pause/Resume (only if downloading)
      if (e.code === "Space" && isDownloading) {
        e.preventDefault();
        onPauseResume();
      }

      // Esc: Cancel (only if downloading)
      if (e.key === "Escape") {
        if (isDownloading) {
          e.preventDefault();
          onCancel();
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [
    onPaste,
    onDownload,
    onPauseResume,
    onCancel,
    onToggleMiniMode,
    onNextTab,
    onPrevTab,
    isDownloading,
    isPaused,
  ]);
};

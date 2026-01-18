import { useEffect, useCallback } from "react";

interface KeyboardShortcuts {
  onPaste?: () => void;
  onDownload?: () => void;
  onPauseResume?: () => void;
  onCancel?: () => void;
  onToggleMiniMode?: () => void;
  onNextTab?: () => void;
  onPrevTab?: () => void;
  isDownloading?: boolean;
  isPaused?: boolean;
}

export function useKeyboardShortcuts({
  onPaste,
  onDownload,
  onPauseResume,
  onCancel,
  onToggleMiniMode,
  onNextTab,
  onPrevTab,
  isDownloading = false,
  isPaused = false,
}: KeyboardShortcuts) {
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      // Ignore if typing in input
      const target = e.target as HTMLElement;
      if (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.tagName === "SELECT") {
        // Allow Ctrl+V in inputs
        if (!(e.ctrlKey && e.key === "v") && !(e.metaKey && e.key === "v")) {
          return;
        }
      }

      // Ctrl/Cmd + V - Paste URL
      if ((e.ctrlKey || e.metaKey) && e.key === "v") {
        // Only handle if not in an input field
        if (target.tagName !== "INPUT" && target.tagName !== "TEXTAREA") {
          e.preventDefault();
          onPaste?.();
        }
      }

      // Ctrl/Cmd + D - Start Download
      if ((e.ctrlKey || e.metaKey) && e.key === "d") {
        e.preventDefault();
        onDownload?.();
      }

      // Space - Pause/Resume (only when downloading)
      if (e.key === " " && isDownloading && target.tagName !== "INPUT" && target.tagName !== "TEXTAREA" && target.tagName !== "BUTTON") {
        e.preventDefault();
        onPauseResume?.();
      }

      // Escape - Cancel download
      if (e.key === "Escape" && isDownloading) {
        e.preventDefault();
        onCancel?.();
      }

      // Ctrl/Cmd + M - Toggle Mini Mode
      if ((e.ctrlKey || e.metaKey) && e.key === "m") {
        e.preventDefault();
        onToggleMiniMode?.();
      }

      // Ctrl/Cmd + Tab - Next Tab
      if ((e.ctrlKey || e.metaKey) && e.key === "Tab" && !e.shiftKey) {
        e.preventDefault();
        onNextTab?.();
      }

      // Ctrl/Cmd + Shift + Tab - Previous Tab
      if ((e.ctrlKey || e.metaKey) && e.key === "Tab" && e.shiftKey) {
        e.preventDefault();
        onPrevTab?.();
      }
    },
    [onPaste, onDownload, onPauseResume, onCancel, onToggleMiniMode, onNextTab, onPrevTab, isDownloading, isPaused]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);
}

export const SHORTCUTS = [
  { key: "Ctrl+V", description: "Paste URL", descriptionTh: "วาง URL" },
  { key: "Ctrl+D", description: "Start Download", descriptionTh: "เริ่มดาวน์โหลด" },
  { key: "Space", description: "Pause/Resume", descriptionTh: "หยุด/ดำเนินต่อ" },
  { key: "Escape", description: "Cancel Download", descriptionTh: "ยกเลิกดาวน์โหลด" },
  { key: "Ctrl+M", description: "Toggle Mini Mode", descriptionTh: "สลับโหมดเล็ก" },
  { key: "Ctrl+Tab", description: "Next Tab", descriptionTh: "แท็บถัดไป" },
  { key: "Ctrl+Shift+Tab", description: "Previous Tab", descriptionTh: "แท็บก่อนหน้า" },
];

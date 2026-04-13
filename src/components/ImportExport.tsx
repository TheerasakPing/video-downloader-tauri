import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { Download, Upload } from "lucide-react";
import { Button } from "./Button";
import { useI18n } from "../hooks/useI18n";

interface ImportExportProps {
  onImportComplete?: (count: number) => void;
  onError?: (message: string) => void;
  onSuccess?: (message: string) => void;
}

export function ImportExport({ onImportComplete, onError, onSuccess }: ImportExportProps) {
  const { t } = useI18n();

  const handleExport = async () => {
    try {
      const jsonData = await invoke<string>("cmd_export_library");

      const filePath = await save({
        title: t("importExport.exportTitle"),
        defaultPath: "library-export.json",
        filters: [
          {
            name: "JSON",
            extensions: ["json"],
          },
        ],
      });

      if (filePath) {
        // Write file using Tauri invoke
        await invoke("write_file", {
          path: filePath,
          contents: jsonData,
        });
        onSuccess?.(t("library.exported", { path: filePath }));
      }
    } catch (e) {
      onError?.(t("library.exportFailed", { error: String(e) }));
    }
  };

  const handleImport = async () => {
    try {
      const filePath = await open({
        title: t("importExport.importTitle"),
        multiple: false,
        filters: [
          {
            name: "JSON",
            extensions: ["json"],
          },
        ],
      });

      if (filePath && typeof filePath === "string") {
        // Read file using Tauri invoke
        const jsonData = await invoke<string>("read_file", {
          path: filePath,
        });

        const count = await invoke<number>("cmd_import_library", {
          jsonData,
        });

        onSuccess?.(t("library.imported", { count }));
        onImportComplete?.(count);
      }
    } catch (e) {
      onError?.(t("library.importFailed", { error: String(e) }));
    }
  };

  return (
    <div className="flex items-center gap-2">
      <Button
        size="sm"
        variant="ghost"
        onClick={handleExport}
        leftIcon={<Download size={14} />}
        title={t("importExport.exportHint")}
      >
        {t("importExport.export")}
      </Button>
      <Button
        size="sm"
        variant="ghost"
        onClick={handleImport}
        leftIcon={<Upload size={14} />}
        title={t("importExport.importHint")}
      >
        {t("importExport.import")}
      </Button>
    </div>
  );
}

export default ImportExport;

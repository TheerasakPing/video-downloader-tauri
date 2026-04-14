import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { readText } from "@tauri-apps/plugin-clipboard-manager";
import { SeriesInfo } from "../types";
import { DomainSettings } from "../types";
import { BatchItem } from "./useBatchProcessor";

interface UseSeriesFetchDeps {
  domainSettings: DomainSettings;
  log: (msg: string) => void;
  success: (msg: string) => void;
  error: (msg: string) => void;
  onBatchStart: (items: BatchItem[]) => void;
  onSingleFetch: (series: SeriesInfo) => void;
}

export function useSeriesFetch(deps: UseSeriesFetchDeps) {
  const { domainSettings, log, success, error, onBatchStart, onSingleFetch } = deps;

  const [url, setUrl] = useState("");
  const [series, setSeries] = useState<SeriesInfo | null>(null);
  const [isFetching, setIsFetching] = useState(false);

  const isValidSeriesUrl = useCallback(
    (text: string): boolean => {
      if (!text) return false;
      if (!text.startsWith("http://") && !text.startsWith("https://")) return false;

      const checkDomains = (settingDomain: string) =>
        settingDomain.split(",").some((d) => {
          const domain = d.trim();
          return domain.length > 0 && text.includes(domain);
        });

      return (
        checkDomains(domainSettings.rongyokDomain) ||
        checkDomains(domainSettings.titanDomain) ||
        checkDomains(domainSettings.baanjeenDomain) ||
        checkDomains(domainSettings.njavtvDomain || "njavtv.com") ||
        checkDomains(domainSettings.javwowDomain || "javwow.com") ||
        checkDomains(domainSettings.avkuyDomain || "www2.avkuy.com") ||
        text.includes("rongyok.com") ||
        text.includes("thongyok.com") ||
        text.includes("51cg") ||
        text.includes("357ms") ||
        text.includes("xn--82c7abb4jua0l.com") ||
        text.includes("njavtv.com") ||
        text.includes("javwow.com") ||
        text.includes("avkuy.com") ||
        text.includes("av-kuy.com")
      );
    },
    [domainSettings],
  );

  const extractUrls = useCallback(
    (text: string) =>
      text
        .split(/[\n\s]+/)
        .map((l) => {
          const trimmed = l.trim();
          if (trimmed.startsWith("//")) return `https:${trimmed}`;
          return trimmed;
        })
        .filter((l) => l.length > 0 && isValidSeriesUrl(l)),
    [isValidSeriesUrl],
  );

  const fetchSeries = useCallback(
    async (fetchUrl: string) => {
      setIsFetching(true);
      try {
        const result = await invoke<SeriesInfo>("fetch_series", { url: fetchUrl });
        setSeries(result);
        onSingleFetch(result);
        return result;
      } catch (e) {
        error(`Failed to fetch: ${e}`);
        return null;
      } finally {
        setIsFetching(false);
      }
    },
    [error, onSingleFetch],
  );

  const handlePaste = useCallback(async () => {
    try {
      const text = await readText();
      if (!text) {
        error("Clipboard is empty");
        return;
      }

      const validUrls = text.split(/[\n\s]+/).map((l) => l.trim()).filter((l) => l.length > 0 && isValidSeriesUrl(l));

      if (validUrls.length > 1) {
        log(`Detected ${validUrls.length} URLs - Switching to Batch Mode`);
        const newBatch: BatchItem[] = validUrls.map((u) => ({ url: u, status: "pending" as const }));
        onBatchStart(newBatch);

        // Fetch info for each batch item
        for (let i = 0; i < newBatch.length; i++) {
          try {
            const result = await invoke<SeriesInfo>("fetch_series", { url: newBatch[i].url });
            newBatch[i] = { ...newBatch[i], status: "ready", info: { ...result, url: newBatch[i].url } };
          } catch (e) {
            newBatch[i] = { ...newBatch[i], status: "error", error: String(e) };
          }
        }

        success(`Ready to download ${validUrls.length} series`);
        return;
      }

      setUrl(text);
      log(`Pasted URL: ${text}`);

      if (isValidSeriesUrl(text)) {
        await fetchSeries(text);
      }
    } catch {
      error("Failed to read clipboard");
    }
  }, [log, error, success, isValidSeriesUrl, fetchSeries, onBatchStart]);

  const autoFetchFromClipboard = useCallback(async () => {
    try {
      const text = await readText();
      if (text && isValidSeriesUrl(text) && text !== url) {
        setUrl(text);
        log(`Auto-pasted URL: ${text}`);
        setIsFetching(true);
        try {
          const result = await invoke<SeriesInfo>("fetch_series", { url: text });
          setSeries(result);
          success(`Auto-loaded: ${result.title} (${result.totalEpisodes} episodes)`);
        } catch (e) {
          error(`Auto-fetch failed: ${e}`);
        } finally {
          setIsFetching(false);
        }
      }
    } catch {
      // Ignore clipboard errors
    }
  }, [url, isValidSeriesUrl, log, success, error]);

  return {
    url,
    setUrl,
    series,
    setSeries,
    isFetching,
    setIsFetching,
    isValidSeriesUrl,
    extractUrls,
    handlePaste,
    fetchSeries,
    autoFetchFromClipboard,
  };
}

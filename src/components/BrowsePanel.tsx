import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Search, Loader2, Compass } from "lucide-react";
import { Button } from "./";
import BrowseCard from "./BrowseCard";
import BrowseDetail from "./BrowseDetail";
import type { SearchResult, SiteCategory, SearchResponse } from "../types";

const SOURCE_FILTERS = [
  { id: "", label: "All" },
  { id: "rongyok", label: "Rongyok" },
  { id: "baanjeen", label: "BaanJeen" },
  { id: "titan", label: "Titan" },
  { id: "hsck", label: "Hsck" },
];

const SOURCE_COLORS: Record<string, string> = {
  rongyok: "border-violet-500/30 text-violet-300",
  baanjeen: "border-blue-500/30 text-blue-300",
  titan: "border-amber-500/30 text-amber-300",
  hsck: "border-emerald-500/30 text-emerald-300",
};

interface BrowsePanelProps {
  settings: any;
  ffmpegAvailable: boolean;
}

export default function BrowsePanel({ settings, ffmpegAvailable }: BrowsePanelProps) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResponse[]>([]);
  const [categories, setCategories] = useState<SiteCategory[]>([]);
  const [selectedSource, setSelectedSource] = useState("");
  const [activeCategory, setActiveCategory] = useState<{ source: string; id: string } | null>(null);
  const [detail, setDetail] = useState<SearchResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);

  useEffect(() => {
    invoke<SiteCategory[]>("get_browse_categories").then(setCategories).catch(() => {});
  }, []);

  const doSearch = useCallback(async (q: string, page: number = 1) => {
    if (!q.trim()) return;
    setIsLoading(true);
    try {
      const responses = await invoke<SearchResponse[]>("search_sites", { query: q, page });
      if (page === 1) {
        setResults(responses);
      } else {
        setResults((prev) => {
          const merged = [...prev];
          for (const resp of responses) {
            const existing = merged.find((r) => r.source === resp.source);
            if (existing) {
              existing.results = [...existing.results, ...resp.results];
              existing.hasMore = resp.hasMore;
            } else {
              merged.push(resp);
            }
          }
          return merged;
        });
      }
      setCurrentPage(page);
    } catch (e) {
      console.error("Search failed:", e);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const doBrowse = useCallback(async (source: string, category: string, page: number = 1) => {
    setIsLoading(true);
    setActiveCategory({ source, id: category });
    try {
      const response = await invoke<SearchResponse>("browse_category", { source, category, page });
      setResults([response]);
      setCurrentPage(page);
    } catch (e) {
      console.error("Browse failed:", e);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const allResults = results
    .filter((r) => !selectedSource || r.source === selectedSource)
    .flatMap((r) => r.results);

  const hasMore = results.some((r) => (!selectedSource || r.source === selectedSource) && r.hasMore);

  if (detail) {
    return (
      <div className="h-full overflow-y-auto custom-scrollbar">
        <BrowseDetail
          result={detail}
          onBack={() => setDetail(null)}
          settings={settings}
          ffmpegAvailable={ffmpegAvailable}
        />
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col overflow-hidden">
      {/* Search Bar */}
      <div className="p-3 space-y-2 border-b border-slate-700/50 bg-slate-900/50">
        <div className="flex gap-2">
          <div className="flex-1 relative">
            <Search size={14} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-slate-400" />
            <input
              type="text"
              placeholder="Search across all sites..."
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && doSearch(query)}
              className="w-full pl-8 pr-3 py-1.5 text-xs bg-slate-800 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:border-cyan-500/50"
            />
          </div>
          <Button size="sm" onClick={() => doSearch(query)} isLoading={isLoading}>
            <Search size={14} />
          </Button>
        </div>

        {/* Source Filters */}
        <div className="flex gap-1 overflow-x-auto pb-1">
          {SOURCE_FILTERS.map((f) => (
            <button
              key={f.id}
              onClick={() => setSelectedSource(f.id)}
              className={`px-2 py-0.5 text-[10px] font-bold rounded-full whitespace-nowrap transition-all ${
                selectedSource === f.id
                  ? "bg-cyan-500/20 text-cyan-300 border border-cyan-500/40"
                  : "bg-slate-800 text-slate-400 border border-slate-700 hover:text-white"
              }`}
            >
              {f.label}
            </button>
          ))}
        </div>

        {/* Category Pills */}
        {categories.length > 0 && (
          <div className="flex gap-1 overflow-x-auto pb-1">
            {categories.map((cat) => (
              <button
                key={`${cat.source}-${cat.id}`}
                onClick={() => doBrowse(cat.source, cat.id)}
                className={`px-2 py-0.5 text-[10px] rounded-full whitespace-nowrap border transition-all ${
                  activeCategory?.source === cat.source && activeCategory?.id === cat.id
                    ? `bg-slate-700 text-white ${SOURCE_COLORS[cat.source] || "border-slate-600"}`
                    : "bg-slate-800/50 text-slate-400 border-slate-700 hover:text-white"
                }`}
              >
                <span className="opacity-60">{cat.source}:</span> {cat.label}
              </button>
            ))}
          </div>
        )}
      </div>

      {/* Results Grid */}
      <div className="flex-1 overflow-y-auto custom-scrollbar p-3">
        {allResults.length === 0 && !isLoading && (
          <div className="h-full flex flex-col items-center justify-center text-slate-500 opacity-40">
            <Compass size={40} className="mb-3" />
            <p className="text-sm">Search or browse categories to discover content</p>
          </div>
        )}

        <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-2">
          {allResults.map((result, i) => (
            <BrowseCard key={`${result.source}-${i}`} result={result} onClick={setDetail} />
          ))}
        </div>

        {isLoading && (
          <div className="flex justify-center py-4">
            <Loader2 size={20} className="animate-spin text-cyan-400" />
          </div>
        )}

        {hasMore && !isLoading && (
          <div className="flex justify-center py-3">
            <Button size="sm" variant="ghost" onClick={() => {
              if (activeCategory) {
                doBrowse(activeCategory.source, activeCategory.id, currentPage + 1);
              } else {
                doSearch(query, currentPage + 1);
              }
            }}>
              Load more
            </Button>
          </div>
        )}
      </div>
    </div>
  );
}

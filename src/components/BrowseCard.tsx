import { Image as ImageIcon } from "lucide-react";
import type { SearchResult } from "../types";

const SOURCE_COLORS: Record<string, string> = {
  rongyok: "bg-violet-500/20 text-violet-300",
  baanjeen: "bg-blue-500/20 text-blue-300",
  titan: "bg-amber-500/20 text-amber-300",
  hsck: "bg-emerald-500/20 text-emerald-300",
};

interface BrowseCardProps {
  result: SearchResult;
  onClick: (result: SearchResult) => void;
}

export default function BrowseCard({ result, onClick }: BrowseCardProps) {
  return (
    <div
      className="group cursor-pointer rounded-lg border border-slate-700/50 bg-slate-800/30 overflow-hidden hover:border-slate-600 hover:bg-slate-800/60 transition-all"
      onClick={() => onClick(result)}
    >
      <div className="aspect-[2/3] bg-slate-900 relative overflow-hidden">
        {result.posterUrl ? (
          <img
            src={result.posterUrl}
            alt={result.title}
            className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
          />
        ) : (
          <div className="w-full h-full flex items-center justify-center">
            <ImageIcon size={24} className="text-slate-600" />
          </div>
        )}
        <span
          className={`absolute top-1.5 left-1.5 px-1.5 py-0.5 rounded text-[10px] font-bold ${
            SOURCE_COLORS[result.source] || "bg-slate-500/20 text-slate-300"
          }`}
        >
          {result.source}
        </span>
        {result.totalEpisodes && (
          <span className="absolute top-1.5 right-1.5 px-1.5 py-0.5 rounded text-[10px] bg-black/60 text-white">
            EP {result.totalEpisodes}
          </span>
        )}
      </div>
      <div className="p-2">
        <p className="text-xs text-slate-200 line-clamp-2 leading-tight">
          {result.title}
        </p>
      </div>
    </div>
  );
}

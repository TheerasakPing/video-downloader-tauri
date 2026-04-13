import { Image as ImageIcon, Star, Clock } from "lucide-react";
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
  const genres = result.genre?.split(",").map(g => g.trim()).filter(Boolean) ?? [];

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
        <div className="absolute top-1.5 right-1.5 flex flex-col gap-0.5 items-end">
          {result.totalEpisodes && (
            <span className="px-1.5 py-0.5 rounded text-[10px] bg-black/60 text-white">
              EP {result.totalEpisodes}
            </span>
          )}
          {result.rating != null && (
            <span className="px-1.5 py-0.5 rounded text-[10px] bg-amber-500/80 text-white flex items-center gap-0.5">
              <Star size={8} className="fill-white" /> {result.rating}
            </span>
          )}
          {result.year != null && (
            <span className="px-1.5 py-0.5 rounded text-[10px] bg-black/60 text-slate-200">
              {result.year}
            </span>
          )}
        </div>
        {result.duration && (
          <span className="absolute bottom-1.5 right-1.5 px-1.5 py-0.5 rounded text-[10px] bg-black/60 text-slate-300 flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
            <Clock size={8} /> {result.duration}
          </span>
        )}
      </div>
      <div className="p-2 space-y-1">
        <p className="text-xs text-slate-200 line-clamp-2 leading-tight">
          {result.title}
        </p>
        {genres.length > 0 && (
          <div className="flex flex-wrap gap-0.5">
            {genres.slice(0, 3).map(g => (
              <span key={g} className="px-1 py-px rounded text-[9px] bg-slate-700/60 text-slate-400">
                {g}
              </span>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

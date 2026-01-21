import { useMemo } from "react";
import { Zap, TrendingUp, Gauge } from "lucide-react";

interface SpeedDataPoint {
  time: number;
  speed: number;
}

interface SpeedGraphProps {
  data: SpeedDataPoint[];
  currentSpeed: number;
  avgSpeed: number;
  peakSpeed: number;
}

function formatSpeed(bytesPerSec: number): string {
  if (bytesPerSec < 1024) return `${bytesPerSec.toFixed(0)} B/s`;
  if (bytesPerSec < 1024 * 1024)
    return `${(bytesPerSec / 1024).toFixed(1)} KB/s`;
  return `${(bytesPerSec / 1024 / 1024).toFixed(1)} MB/s`;
}

export function SpeedGraph({
  data,
  currentSpeed,
  avgSpeed,
  peakSpeed,
}: SpeedGraphProps) {
  const maxSpeed = useMemo(() => {
    return Math.max(peakSpeed * 1.2, 1024 * 1024);
  }, [peakSpeed]);

  const pathD = useMemo(() => {
    if (data.length < 2) return "";
    const width = 100;
    const height = 100;
    const points = data.map((point, i) => {
      const x = (i / (data.length - 1)) * width;
      const y = height - (point.speed / maxSpeed) * height;
      return `${x},${y}`;
    });
    return `M ${points.join(" L ")}`;
  }, [data, maxSpeed]);

  const areaPathD = useMemo(() => {
    if (data.length < 2) return "";
    const width = 100;
    const height = 100;
    const points = data.map((point, i) => {
      const x = (i / (data.length - 1)) * width;
      const y = height - (point.speed / maxSpeed) * height;
      return `${x},${y}`;
    });
    return `M 0,${height} L ${points.join(" L ")} L 100,${height} Z`;
  }, [data, maxSpeed]);

  return (
    <div className="glass rounded-lg border border-slate-700/50 overflow-hidden">
      {/* Stats row */}
      <div className="flex items-center justify-between px-2 py-1.5 bg-slate-800/30 border-b border-slate-700/50">
        <div className="flex items-center gap-1 text-violet-400">
          <Zap size={10} className="drop-shadow-[0_0_4px_currentColor]" />
          <span className="text-[10px]">Current</span>
          <span className="text-xs font-bold text-white ml-0.5">{formatSpeed(currentSpeed)}</span>
        </div>
        <div className="flex items-center gap-1 text-emerald-400">
          <Gauge size={10} className="drop-shadow-[0_0_4px_currentColor]" />
          <span className="text-[10px]">Avg</span>
          <span className="text-xs font-bold text-white ml-0.5">{formatSpeed(avgSpeed)}</span>
        </div>
        <div className="flex items-center gap-1 text-amber-400">
          <TrendingUp size={10} className="drop-shadow-[0_0_4px_currentColor]" />
          <span className="text-[10px]">Peak</span>
          <span className="text-xs font-bold text-white ml-0.5">{formatSpeed(peakSpeed)}</span>
        </div>
      </div>

      {/* Graph */}
      <div className="h-16 relative bg-slate-900/50">
        {data.length < 2 ? (
          <div className="absolute inset-0 flex items-center justify-center text-slate-600 text-[10px]">
            Waiting...
          </div>
        ) : (
          <svg viewBox="0 0 100 100" preserveAspectRatio="none" className="w-full h-full">
            <line x1="0" y1="50" x2="100" y2="50" stroke="#374151" strokeWidth="0.5" />
            <path d={areaPathD} fill="url(#speedGradient)" opacity="0.3" />
            <path d={pathD} fill="none" stroke="#8b5cf6" strokeWidth="2" strokeLinecap="round" vectorEffect="non-scaling-stroke" />
            <defs>
              <linearGradient id="speedGradient" x1="0%" y1="0%" x2="0%" y2="100%">
                <stop offset="0%" stopColor="#8b5cf6" />
                <stop offset="100%" stopColor="#8b5cf6" stopOpacity="0" />
              </linearGradient>
            </defs>
          </svg>
        )}
      </div>
    </div>
  );
}

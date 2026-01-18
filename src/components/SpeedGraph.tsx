import { useMemo } from "react";
import { Activity, TrendingUp, Gauge, Zap } from "lucide-react";

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
    return Math.max(peakSpeed * 1.2, 1024 * 1024); // At least 1 MB/s scale
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
    <div className="glass rounded-xl border border-slate-700/50 overflow-hidden">
      <div className="px-3 sm:px-4 py-2 sm:py-3 bg-slate-800/30 border-b border-slate-700/50">
        <h3 className="text-xs sm:text-sm font-medium text-slate-300 flex items-center gap-2">
          <Activity size={14} className="sm:w-4 sm:h-4" />
          Download Speed
        </h3>
      </div>

      <div className="p-3 sm:p-4">
        {/* Stats */}
        <div className="grid grid-cols-3 gap-2 sm:gap-4 mb-3 sm:mb-4">
          <div className="text-center animate-fade-in stagger-1">
            <div className="flex items-center justify-center gap-1 text-violet-400 mb-0.5 sm:mb-1">
              <Zap size={12} className="sm:w-3.5 sm:h-3.5" />
              <span className="text-[10px] sm:text-xs">Current</span>
            </div>
            <div className="text-sm sm:text-lg font-bold text-white">
              {formatSpeed(currentSpeed)}
            </div>
          </div>
          <div className="text-center animate-fade-in stagger-2">
            <div className="flex items-center justify-center gap-1 text-emerald-400 mb-0.5 sm:mb-1">
              <Gauge size={12} className="sm:w-3.5 sm:h-3.5" />
              <span className="text-[10px] sm:text-xs">Average</span>
            </div>
            <div className="text-sm sm:text-lg font-bold text-white">
              {formatSpeed(avgSpeed)}
            </div>
          </div>
          <div className="text-center animate-fade-in stagger-3">
            <div className="flex items-center justify-center gap-1 text-amber-400 mb-0.5 sm:mb-1">
              <TrendingUp size={12} className="sm:w-3.5 sm:h-3.5" />
              <span className="text-[10px] sm:text-xs">Peak</span>
            </div>
            <div className="text-sm sm:text-lg font-bold text-white">
              {formatSpeed(peakSpeed)}
            </div>
          </div>
        </div>

        {/* Graph */}
        <div className="h-24 sm:h-32 relative bg-slate-900/50 rounded-lg overflow-hidden">
          {data.length < 2 ? (
            <div className="absolute inset-0 flex items-center justify-center text-slate-600 text-xs sm:text-sm animate-pulse">
              Waiting for data...
            </div>
          ) : (
            <svg
              viewBox="0 0 100 100"
              preserveAspectRatio="none"
              className="w-full h-full"
            >
              {/* Grid lines */}
              <line
                x1="0"
                y1="25"
                x2="100"
                y2="25"
                stroke="#374151"
                strokeWidth="0.5"
              />
              <line
                x1="0"
                y1="50"
                x2="100"
                y2="50"
                stroke="#374151"
                strokeWidth="0.5"
              />
              <line
                x1="0"
                y1="75"
                x2="100"
                y2="75"
                stroke="#374151"
                strokeWidth="0.5"
              />

              {/* Area fill */}
              <path
                d={areaPathD}
                fill="url(#speedGradient)"
                opacity="0.3"
              />

              {/* Line */}
              <path
                d={pathD}
                fill="none"
                stroke="#8b5cf6"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
                vectorEffect="non-scaling-stroke"
              />

              {/* Gradient definition */}
              <defs>
                <linearGradient
                  id="speedGradient"
                  x1="0%"
                  y1="0%"
                  x2="0%"
                  y2="100%"
                >
                  <stop offset="0%" stopColor="#8b5cf6" />
                  <stop offset="100%" stopColor="#8b5cf6" stopOpacity="0" />
                </linearGradient>
              </defs>
            </svg>
          )}

          {/* Y-axis labels */}
          <div className="absolute right-1 sm:right-2 top-0.5 sm:top-1 text-[10px] sm:text-xs text-slate-600">
            {formatSpeed(maxSpeed)}
          </div>
          <div className="absolute right-1 sm:right-2 bottom-0.5 sm:bottom-1 text-[10px] sm:text-xs text-slate-600">
            0
          </div>
        </div>
      </div>
    </div>
  );
}

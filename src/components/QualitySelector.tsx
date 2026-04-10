import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { QualityInfo } from '../types';
import { Monitor, ChevronDown } from 'lucide-react';

interface QualitySelectorProps {
  episodeUrl: string | undefined;
  onSelect: (quality: string | null) => void;
  defaultQuality: string;
}

export default function QualitySelector({ episodeUrl, onSelect, defaultQuality }: QualitySelectorProps) {
  const [qualityInfo, setQualityInfo] = useState<QualityInfo | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const [expanded, setExpanded] = useState(false);

  useEffect(() => {
    if (!episodeUrl || defaultQuality !== 'ask') {
      return;
    }
    invoke<QualityInfo>('get_quality_options', { url: episodeUrl })
      .then(info => {
        setQualityInfo(info);
        if (info.qualities.length > 0) {
          setSelected(null);
        }
      })
      .catch(() => setQualityInfo(null));
  }, [episodeUrl, defaultQuality]);

  const handleSelect = (quality: string | null) => {
    setSelected(quality);
    onSelect(quality);
    setExpanded(false);
  };

  if (defaultQuality === 'best' || !qualityInfo || qualityInfo.qualities.length <= 1) {
    return null;
  }

  return (
    <div className="flex items-center gap-2">
      <Monitor size={16} className="text-[var(--accent)]" />
      <div className="relative">
        <button
          onClick={() => setExpanded(!expanded)}
          className="flex items-center gap-1 px-3 py-1.5 rounded-lg bg-[var(--card)] border border-[var(--border)] text-sm hover:border-[var(--accent)] transition-colors"
        >
          {selected
            ? qualityInfo.qualities.find(q => q.resolution === selected)?.label || 'Best'
            : 'Best available'}
          <ChevronDown size={14} />
        </button>
        {expanded && (
          <div className="absolute top-full mt-1 left-0 z-50 bg-[var(--card)] border border-[var(--border)] rounded-lg shadow-xl min-w-[200px]">
            <button
              onClick={() => handleSelect(null)}
              className={`w-full text-left px-3 py-2 text-sm hover:bg-[var(--accent)]/10 ${!selected ? 'text-[var(--accent)]' : ''}`}
            >
              Best available (recommended)
            </button>
            {qualityInfo.qualities.map((q, i) => (
              <button
                key={i}
                onClick={() => handleSelect(q.resolution)}
                className={`w-full text-left px-3 py-2 text-sm hover:bg-[var(--accent)]/10 ${selected === q.resolution ? 'text-[var(--accent)]' : ''}`}
              >
                {q.label}
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

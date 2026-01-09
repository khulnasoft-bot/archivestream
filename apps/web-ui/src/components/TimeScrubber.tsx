"use client";

import React, { useEffect, useState, useRef } from "react";
import { Clock, ChevronLeft, ChevronRight, GitCompare } from "lucide-react";
import { format } from "date-fns";

interface TimelineSnapshot {
  timestamp: string;
  status: number;
  digest: string;
  intensity: number;
}

interface TimeScrubberProps {
  url: string;
  currentTimestamp: string;
  onNavigate: (timestamp: string) => void;
  onDiff: (from: string, to: string) => void;
  isDiffMode: boolean;
  toggleDiffMode: () => void;
}

export const TimeScrubber: React.FC<TimeScrubberProps> = ({
  url,
  currentTimestamp,
  onNavigate,
  onDiff,
  isDiffMode,
  toggleDiffMode,
}) => {
  const [snapshots, setSnapshots] = useState<TimelineSnapshot[]>([]);
  const [bookmarks, setBookmarks] = useState<string[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(-1);
  const [diffTargetIndex, setDiffTargetIndex] = useState(-1);
  const scrubberRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const savedBookmarks = JSON.parse(localStorage.getItem(`bookmarks_${url}`) || "[]");
    setBookmarks(savedBookmarks);
  }, [url]);

  useEffect(() => {
    const fetchTimeline = async () => {
      try {
        const apiUrl =
          process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001";
        const response = await fetch(
          `${apiUrl}/api/v1/timeline?url=${encodeURIComponent(url)}`,
        );
        const data = await response.json();
        setSnapshots(data.snapshots);

        const idx = data.snapshots.findIndex(
          (s: any) =>
            s.timestamp.replace(/[-:T]/g, "").split(".")[0] ===
            currentTimestamp,
        );
        setSelectedIndex(idx);
      } catch (e) {
        console.error("Failed to fetch timeline", e);
      }
    };
    fetchTimeline();
  }, [url, currentTimestamp]);

  const handleSnap = (index: number) => {
    if (index >= 0 && index < snapshots.length) {
      if (isDiffMode) {
        setDiffTargetIndex(index);
        const fromTs = snapshots[index].timestamp
          .replace(/[-:T]/g, "")
          .split(".")[0];
        const toTs = snapshots[selectedIndex].timestamp
          .replace(/[-:T]/g, "")
          .split(".")[0];
        onDiff(fromTs, toTs);
      } else {
        const ts = snapshots[index].timestamp
          .replace(/[-:T]/g, "")
          .split(".")[0];
        onNavigate(ts);
      }
    }
  };

  const toggleBookmark = () => {
    if (selectedIndex === -1) return;
    const ts = snapshots[selectedIndex].timestamp;
    let newBookmarks;
    if (bookmarks.includes(ts)) {
      newBookmarks = bookmarks.filter(b => b !== ts);
    } else {
      newBookmarks = [...bookmarks, ts];
    }
    setBookmarks(newBookmarks);
    localStorage.setItem(`bookmarks_${url}`, JSON.stringify(newBookmarks));
  };

  const handleShare = () => {
    const shareUrl = window.location.href;
    navigator.clipboard.writeText(shareUrl);
    alert("Replay URL copied to clipboard!");
  };

  return (
    <div className="bg-black/90 backdrop-blur-2xl border-b border-white/10 px-6 py-3 flex items-center gap-6 sticky top-0 z-[100] h-16">
      <div className="flex items-center gap-3 min-w-max">
        <div className="w-8 h-8 bg-primary-600 rounded-lg flex items-center justify-center font-bold text-lg">
          A
        </div>
        <div className="hidden lg:block">
          <div className="text-[10px] text-gray-500 font-bold uppercase tracking-widest">
            History Engine
          </div>
          <div className="text-xs font-bold truncate max-w-[150px]">{url}</div>
        </div>
      </div>

      <div className="flex items-center gap-2">
        <button
          onClick={toggleDiffMode}
          className={`flex items-center gap-2 px-3 py-1.5 rounded-lg border transition-all ${isDiffMode
              ? "bg-primary-500/20 border-primary-500 text-primary-400"
              : "bg-white/5 border-white/10 text-gray-400 hover:bg-white/10"
            }`}
        >
          <GitCompare size={16} />
          <span className="text-xs font-bold uppercase tracking-tight">
            Diff
          </span>
        </button>
        <button
          onClick={toggleBookmark}
          className={`p-1.5 rounded-lg border transition-all ${selectedIndex !== -1 && bookmarks.includes(snapshots[selectedIndex].timestamp)
              ? "bg-amber-500/20 border-amber-500 text-amber-400"
              : "bg-white/5 border-white/10 text-gray-400 hover:bg-white/10"
            }`}
          title="Bookmark Snapshot"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill={selectedIndex !== -1 && bookmarks.includes(snapshots[selectedIndex].timestamp) ? "currentColor" : "none"}
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <path d="m19 21-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v16z" />
          </svg>
        </button>
        <button
          onClick={handleShare}
          className="p-1.5 rounded-lg border bg-white/5 border-white/10 text-gray-400 hover:bg-white/10 transition-all"
          title="Share Replay"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <path d="M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8" />
            <polyline points="16 6 12 2 8 6" />
            <line x1="12" y1="2" x2="12" y2="15" />
          </svg>
        </button>
      </div>

      <div className="flex-1 flex items-center gap-4 px-4 bg-white/5 rounded-full h-10 border border-white/5 relative group">
        <button
          onClick={() => handleSnap(selectedIndex - 1)}
          disabled={selectedIndex <= 0 || isDiffMode}
          className="hover:text-primary-400 disabled:opacity-30 transition-colors"
        >
          <ChevronLeft size={20} />
        </button>

        <div
          className="flex-1 h-full relative flex items-center px-2"
          ref={scrubberRef}
        >
          <div className="absolute top-1/2 left-0 right-0 h-[1.5px] bg-white/10 -translate-y-1/2" />

          {snapshots.map((s, i) => {
            const pos = (i / (snapshots.length - 1 || 1)) * 100;
            const isCurrent = i === selectedIndex;
            const isTarget = i === diffTargetIndex;
            const isBookmarked = bookmarks.includes(s.timestamp);

            return (
              <button
                key={i}
                onClick={() => handleSnap(i)}
                className={`absolute top-1/2 -translate-y-1/2 w-2 h-2 rounded-full transition-all group/tick ${isCurrent
                    ? "bg-primary-500 scale-150 z-20 shadow-[0_0_12px_rgba(var(--primary-rgb),0.8)]"
                    : isTarget
                      ? "bg-orange-500 scale-150 z-20 shadow-[0_0_12px_rgba(249,115,22,0.8)]"
                      : isBookmarked
                        ? "bg-amber-400 scale-125 z-10 shadow-[0_0_8px_rgba(251,191,36,0.6)]"
                        : s.intensity > 0.5
                          ? "bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.4)]"
                          : s.intensity > 0
                            ? "bg-primary-400"
                            : "bg-white/20 hover:bg-white/50"
                  }`}
                style={{ left: `${pos}%` }}
              >
                {isBookmarked && (
                  <div className="absolute -top-3 left-1/2 -translate-x-1/2 text-amber-500">
                    <div className="w-1 h-1 bg-amber-500 rounded-full" />
                  </div>
                )}
                <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2 py-1 bg-black border border-white/10 rounded text-[10px] whitespace-nowrap opacity-0 group-hover/tick:opacity-100 transition-opacity pointer-events-none">
                  {isBookmarked && "★ "}{format(new Date(s.timestamp), "MMM dd, HH:mm")}
                </div>
              </button>
            );
          })}
        </div>

        <button
          onClick={() => handleSnap(selectedIndex + 1)}
          disabled={selectedIndex >= snapshots.length - 1 || isDiffMode}
          className="hover:text-primary-400 disabled:opacity-30 transition-colors"
        >
          <ChevronRight size={20} />
        </button>
      </div>

      <div className="flex items-center gap-4 min-w-max border-l border-white/10 pl-6 h-8">
        <div className="flex flex-col items-end">
          <div className="text-[10px] text-primary-500 font-bold uppercase tracking-widest leading-none mb-1 flex items-center gap-1">
            {isDiffMode ? "Temporal Delta" : "Point-in-Time"}
          </div>
          <div className="text-sm font-mono font-bold">
            {selectedIndex >= 0
              ? format(
                new Date(snapshots[selectedIndex].timestamp),
                "MMM dd, HH:mm",
              )
              : "..."}
          </div>
        </div>
      </div>
    </div>

  );
};

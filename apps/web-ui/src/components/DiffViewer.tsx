"use client";

import React from "react";
import { Plus, Minus, Hash } from "lucide-react";

interface DiffChange {
    tag: "added" | "removed" | "equal";
    value: string;
}

interface DiffResult {
    from_timestamp: string;
    to_timestamp: string;
    summary: {
        added: number;
        removed: number;
        unchanged: number;
    };
    changes: DiffChange[];
}

interface DiffViewerProps {
    data: DiffResult | null;
    loading: boolean;
}

export const DiffViewer: React.FC<DiffViewerProps> = ({ data, loading }) => {
    if (loading) {
        return (
            <div className="flex-1 flex items-center justify-center bg-[#0a0a0a]">
                <div className="flex flex-col items-center gap-4">
                    <div className="w-12 h-12 border-4 border-primary-500/20 border-t-primary-500 rounded-full animate-spin" />
                    <p className="font-mono text-xs tracking-widest uppercase text-gray-500">Computing Visual Differential...</p>
                </div>
            </div>
        );
    }

    if (!data) {
        return (
            <div className="flex-1 flex items-center justify-center bg-[#0a0a0a] text-gray-500 italic">
                Select a second snapshot on the timeline to compare
            </div>
        );
    }

    return (
        <div className="flex-1 flex flex-col bg-[#050505] overflow-hidden">
            {/* Stats Bar */}
            <div className="px-6 py-3 border-b border-white/5 flex items-center gap-8 bg-black/50">
                <div className="flex items-center gap-2">
                    <div className="w-2 h-2 rounded-full bg-green-500" />
                    <span className="text-xs font-bold text-gray-400 uppercase tracking-tighter">
                        {data.summary.added} Added
                    </span>
                </div>
                <div className="flex items-center gap-2">
                    <div className="w-2 h-2 rounded-full bg-red-500" />
                    <span className="text-xs font-bold text-gray-400 uppercase tracking-tighter">
                        {data.summary.removed} Removed
                    </span>
                </div>
                <div className="flex items-center gap-2 ml-auto">
                    <Hash size={14} className="text-gray-600" />
                    <span className="text-xs font-mono text-gray-500">
                        {data.summary.unchanged + data.summary.added + data.summary.removed} lines analyzed
                    </span>
                </div>
            </div>

            {/* Diff Content */}
            <div className="flex-1 overflow-y-auto p-8 font-mono text-sm leading-relaxed">
                <div className="max-w-4xl mx-auto space-y-1">
                    {data.changes.map((change, i) => {
                        const isAdded = change.tag === "added";
                        const isRemoved = change.tag === "removed";

                        return (
                            <div
                                key={i}
                                className={`flex gap-4 px-3 py-1 rounded transition-colors ${isAdded ? "bg-green-500/10 text-green-400 border-l-2 border-green-500" :
                                        isRemoved ? "bg-red-500/10 text-red-400 border-l-2 border-red-500" :
                                            "text-gray-500 opacity-60 hover:opacity-100"
                                    }`}
                            >
                                <div className="w-4 select-none flex-shrink-0">
                                    {isAdded ? "+" : isRemoved ? "-" : " "}
                                </div>
                                <div className="whitespace-pre-wrap">{change.value}</div>
                            </div>
                        );
                    })}
                </div>
            </div>
        </div>
    );
};

"use client";

import React, { useState } from "react";
import { Columns, Layers, Type, Maximize2 } from "lucide-react";

interface VisualDiffProps {
    url: string;
    fromTimestamp: string;
    toTimestamp: string;
}

export const VisualDiff: React.FC<VisualDiffProps> = ({ url, fromTimestamp, toTimestamp }) => {
    const [mode, setMode] = useState<"side-by-side" | "overlay">("side-by-side");
    const [sliderPos, setSliderPos] = useState(50);
    const apiUrl = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001";

    const fromSrc = `${apiUrl}/web/${fromTimestamp}/${url}`;
    const toSrc = `${apiUrl}/web/${toTimestamp}/${url}`;

    return (
        <div className="flex-1 flex flex-col bg-[#050505] overflow-hidden">
            {/* Mode Switcher */}
            <div className="px-6 py-3 border-b border-white/5 flex items-center gap-4 bg-black/50">
                <button
                    onClick={() => setMode("side-by-side")}
                    className={`flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-bold uppercase transition-all ${mode === "side-by-side" ? "bg-primary-500 text-white" : "hover:bg-white/5 text-gray-400"}`}
                >
                    <Columns size={14} />
                    Side-by-side
                </button>
                <button
                    onClick={() => setMode("overlay")}
                    className={`flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-bold uppercase transition-all ${mode === "overlay" ? "bg-primary-500 text-white" : "hover:bg-white/5 text-gray-400"}`}
                >
                    <Layers size={14} />
                    Slider Overlay
                </button>

                <div className="ml-auto flex items-center gap-2 text-[10px] text-gray-500 uppercase tracking-widest font-bold">
                    Comparing {fromTimestamp} → {toTimestamp}
                </div>
            </div>

            <div className="flex-1 relative overflow-hidden bg-white">
                {mode === "side-by-side" ? (
                    <div className="flex h-full w-full divide-x divide-black/20">
                        <div className="flex-1 relative group">
                            <div className="absolute top-4 left-4 z-10 px-2 py-1 bg-black/80 backdrop-blur rounded text-[10px] font-bold text-white uppercase border border-white/10 opacity-60 group-hover:opacity-100 transition-opacity">
                                Previous: {fromTimestamp}
                            </div>
                            <iframe src={fromSrc} className="w-full h-full border-none" title="From Version" />
                        </div>
                        <div className="flex-1 relative group">
                            <div className="absolute top-4 left-4 z-10 px-2 py-1 bg-primary-600/80 backdrop-blur rounded text-[10px] font-bold text-white uppercase border border-white/10 opacity-60 group-hover:opacity-100 transition-opacity">
                                Current: {toTimestamp}
                            </div>
                            <iframe src={toSrc} className="w-full h-full border-none" title="To Version" />
                        </div>
                    </div>
                ) : (
                    <div className="relative h-full w-full select-none" onMouseMove={(e) => {
                        if (e.buttons === 1) {
                            const rect = e.currentTarget.getBoundingClientRect();
                            const x = ((e.clientX - rect.left) / rect.width) * 100;
                            setSliderPos(Math.min(100, Math.max(0, x)));
                        }
                    }}>
                        {/* Bottom Layer (From) */}
                        <div className="absolute inset-0">
                            <iframe src={fromSrc} className="w-full h-full border-none" title="From Version" />
                        </div>

                        {/* Top Layer (To) */}
                        <div className="absolute inset-0 overflow-hidden" style={{ width: `${sliderPos}%` }}>
                            <iframe src={toSrc} className="w-full h-full border-none" style={{ width: `${100 / (sliderPos / 100)}%` }} title="To Version" />
                        </div>

                        {/* Slider Handle */}
                        <div
                            className="absolute top-0 bottom-0 w-1 bg-primary-500 shadow-[0_0_15px_rgba(var(--primary-rgb),0.5)] z-50 cursor-ew-resize flex items-center justify-center translate-x-[-2px]"
                            style={{ left: `${sliderPos}%` }}
                        >
                            <div className="w-8 h-8 rounded-full bg-primary-500 border-2 border-white flex items-center justify-center shadow-lg">
                                <Maximize2 size={14} className="text-white rotate-45" />
                            </div>
                        </div>

                        <div className="absolute bottom-4 left-4 z-10 flex gap-2">
                            <div className="px-2 py-1 bg-black/80 backdrop-blur rounded text-[10px] font-bold text-white uppercase border border-white/10">
                                {fromTimestamp}
                            </div>
                            <div className="px-2 py-1 bg-primary-600/80 backdrop-blur rounded text-[10px] font-bold text-white uppercase border border-white/10">
                                {toTimestamp}
                            </div>
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
};

"use client";

import React from "react";

interface FrontierMetric {
    domain: string;
    count: number;
    depth_range: [number, number];
}

interface Props {
    data: FrontierMetric[];
}

export const FrontierHeatmap: React.FC<Props> = ({ data }) => {
    const maxCount = Math.max(...data.map(d => d.count), 1);

    return (
        <div className="bg-white/5 border border-white/10 rounded-2xl p-6">
            <h3 className="text-sm font-bold uppercase tracking-widest text-gray-500 mb-6">Frontier Density (by Domain)</h3>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {data.map((item, i) => {
                    const intensity = item.count / maxCount;
                    return (
                        <div key={i} className="flex flex-col gap-2 p-4 rounded-xl bg-black/40 border border-white/5 hover:border-primary-500/50 transition-all group">
                            <div className="flex justify-between items-start">
                                <span className="text-xs font-bold truncate max-w-[150px] group-hover:text-primary-400 transition-colors">{item.domain}</span>
                                <span className="text-[10px] font-mono text-gray-500 bg-white/5 px-1.5 rounded">Depth {item.depth_range[0]}-{item.depth_range[1]}</span>
                            </div>
                            <div className="relative h-2 bg-white/5 rounded-full overflow-hidden">
                                <div
                                    className="absolute inset-y-0 left-0 bg-primary-500 transition-all duration-1000"
                                    style={{ width: `${intensity * 100}%`, opacity: 0.3 + (intensity * 0.7) }}
                                />
                            </div>
                            <div className="flex justify-between items-center text-[10px] font-mono">
                                <span className="text-gray-500">BACKLOG</span>
                                <span className="text-primary-500 font-bold">{item.count.toLocaleString()} URLs</span>
                            </div>
                        </div>
                    );
                })}
            </div>
        </div>
    );
};

"use client";

import React from "react";
import { CheckCircle2, AlertCircle, Timer, Activity } from "lucide-react";

interface OutcomeMetric {
    status: string;
    count: number;
}

interface Props {
    data: OutcomeMetric[];
}

export const CrawlOutcomes: React.FC<Props> = ({ data }) => {
    const total = data.reduce((acc, curr) => acc + curr.count, 0);
    const success = data.find(d => d.status === "success")?.count || 0;
    const errors = data.find(d => d.status === "error")?.count || 0;

    const successRate = total > 0 ? (success / total) * 100 : 0;

    return (
        <div className="bg-white/5 border border-white/10 rounded-2xl p-6">
            <h3 className="text-sm font-bold uppercase tracking-widest text-gray-500 mb-6">24h Crawl Outcomes</h3>

            <div className="grid grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
                <div className="flex flex-col gap-1">
                    <span className="text-[10px] text-gray-500 font-bold uppercase">Health Score</span>
                    <span className={`text-2xl font-bold ${successRate > 90 ? 'text-green-500' : 'text-orange-500'}`}>
                        {successRate.toFixed(1)}%
                    </span>
                </div>
                <div className="flex flex-col gap-1">
                    <span className="text-[10px] text-gray-500 font-bold uppercase">Total Fetches</span>
                    <span className="text-2xl font-bold text-white">{total.toLocaleString()}</span>
                </div>
                <div className="flex flex-col gap-1">
                    <span className="text-[10px] text-gray-500 font-bold uppercase">Successes</span>
                    <span className="text-2xl font-bold text-green-500">{success.toLocaleString()}</span>
                </div>
                <div className="flex flex-col gap-1">
                    <span className="text-[10px] text-gray-500 font-bold uppercase">Failures</span>
                    <span className="text-2xl font-bold text-red-500">{errors.toLocaleString()}</span>
                </div>
            </div>

            <div className="flex gap-2 h-12 w-full rounded-xl overflow-hidden border border-white/5">
                {data.map((item, i) => {
                    const width = (item.count / total) * 100;
                    const isSuccess = item.status === "success";
                    return (
                        <div
                            key={i}
                            className={`h-full transition-all duration-1000 ${isSuccess ? 'bg-green-500' : 'bg-red-500/50'}`}
                            style={{ width: `${width}%` }}
                            title={`${item.status}: ${item.count}`}
                        />
                    );
                })}
            </div>
        </div>
    );
};

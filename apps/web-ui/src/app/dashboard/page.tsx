"use client";

import React, { useEffect, useState } from "react";
import { FrontierHeatmap } from "@/components/dashboard/FrontierHeatmap";
import { CrawlOutcomes } from "@/components/dashboard/CrawlOutcomes";
import { Activity, Database, Shield, Zap } from "lucide-react";

export default function DashboardPage() {
    const [frontierData, setFrontierData] = useState([]);
    const [outcomeData, setOutcomeData] = useState([]);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        const fetchData = async () => {
            try {
                const apiUrl = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001";
                const [fRes, oRes] = await Promise.all([
                    fetch(`${apiUrl}/health/frontier`),
                    fetch(`${apiUrl}/health/outcomes`)
                ]);
                const [fData, oData] = await Promise.all([fRes.json(), oRes.json()]);
                setFrontierData(fData);
                setOutcomeData(oData);
            } catch (e) {
                console.error("Dashboard fetch failed", e);
            } finally {
                setLoading(false);
            }
        };
        fetchData();
        const interval = setInterval(fetchData, 10000); // Poll every 10s
        return () => clearInterval(interval);
    }, []);

    if (loading) {
        return (
            <div className="min-h-screen bg-[#050505] flex items-center justify-center">
                <div className="flex flex-col items-center gap-4">
                    <Zap className="text-primary-500 animate-pulse" size={48} />
                    <p className="font-mono text-xs tracking-widest text-gray-500 uppercase">Synchronizing Telemetry...</p>
                </div>
            </div>
        );
    }

    return (
        <div className="min-h-screen bg-[#050505] text-white p-8">
            <div className="max-w-7xl mx-auto space-y-12">
                {/* Header */}
                <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
                    <div>
                        <h1 className="text-4xl font-black tracking-tighter flex items-center gap-3 italic">
                            ARCHIVESTREAM <span className="text-primary-600 not-italic whitespace-nowrap">OBSERVE v0.1</span>
                        </h1>
                        <p className="text-gray-500 font-medium mt-1">Real-time distributed crawl health and frontier metrics.</p>
                    </div>
                    <div className="flex items-center gap-4">
                        <div className="px-4 py-2 bg-green-500/10 border border-green-500/20 rounded-xl flex items-center gap-2">
                            <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse" />
                            <span className="text-[10px] font-bold text-green-500 uppercase tracking-widest">System Operational</span>
                        </div>
                    </div>
                </div>

                {/* Global Summary Stats */}
                <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                    <div className="p-6 bg-white/5 border border-white/10 rounded-2xl flex items-center gap-6">
                        <div className="w-12 h-12 bg-primary-500/20 rounded-xl flex items-center justify-center text-primary-500">
                            <Database size={24} />
                        </div>
                        <div>
                            <div className="text-[10px] text-gray-500 font-bold uppercase tracking-widest">Shared Frontier</div>
                            <div className="text-2xl font-bold">{frontierData.reduce((acc: any, c: any) => acc + c.count, 0).toLocaleString()} <span className="text-sm font-normal text-gray-500">URLs</span></div>
                        </div>
                    </div>
                    <div className="p-6 bg-white/5 border border-white/10 rounded-2xl flex items-center gap-6">
                        <div className="w-12 h-12 bg-blue-500/20 rounded-xl flex items-center justify-center text-blue-500">
                            <Activity size={24} />
                        </div>
                        <div>
                            <div className="text-[10px] text-gray-500 font-bold uppercase tracking-widest">Crawl Velocity</div>
                            <div className="text-2xl font-bold">~{(outcomeData.reduce((acc: any, c: any) => acc + c.count, 0) / 86400).toFixed(2)} <span className="text-sm font-normal text-gray-500">req/s</span></div>
                        </div>
                    </div>
                    <div className="p-6 bg-white/5 border border-white/10 rounded-2xl flex items-center gap-6">
                        <div className="w-12 h-12 bg-purple-500/20 rounded-xl flex items-center justify-center text-purple-500">
                            <Shield size={24} />
                        </div>
                        <div>
                            <div className="text-[10px] text-gray-500 font-bold uppercase tracking-widest">Active Domains</div>
                            <div className="text-2xl font-bold">{frontierData.length}</div>
                        </div>
                    </div>
                </div>

                {/* Major Charts */}
                <div className="grid grid-cols-1 gap-12">
                    <CrawlOutcomes data={outcomeData} />
                    <FrontierHeatmap data={frontierData} />
                </div>

                {/* Footer */}
                <div className="pt-12 border-t border-white/5 text-center">
                    <p className="text-[10px] font-mono text-gray-600 uppercase tracking-[0.2em]">ArchiveStream Control Plane &bull; Node: archivestream-api-01 &bull; {new Date().toISOString()}</p>
                </div>
            </div>
        </div>
    );
}

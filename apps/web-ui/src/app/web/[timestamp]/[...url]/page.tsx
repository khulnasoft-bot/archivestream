"use client";

import { useParams, useRouter } from "next/navigation";
import { TimeScrubber } from "@/components/TimeScrubber";
import { DiffViewer } from "@/components/DiffViewer";
import { VisualDiff } from "@/components/VisualDiff";
import { useState, useEffect } from "react";

export default function ReplayPage() {
    const params = useParams();
    const router = useRouter();
    const timestamp = params.timestamp as string;
    const urlParts = params.url as string[];
    const originalUrl = urlParts.join("/");

    const [iframeSrc, setIframeSrc] = useState("");
    const [isDiffMode, setIsDiffMode] = useState(false);
    const [diffType, setDiffType] = useState<"visual" | "text">("visual");
    const [diffTimestamps, setDiffTimestamps] = useState<{ from: string, to: string } | null>(null);
    const [diffData, setDiffData] = useState<any>(null);
    const [diffLoading, setDiffLoading] = useState(false);

    useEffect(() => {
        const apiUrl = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001";
        setIframeSrc(`${apiUrl}/web/${timestamp}/${originalUrl}`);
    }, [timestamp, originalUrl]);

    const handleNavigate = (newTimestamp: string) => {
        setIsDiffMode(false);
        setDiffData(null);
        setDiffTimestamps(null);
        router.push(`/web/${newTimestamp}/${originalUrl}`);
    };

    const handleDiff = async (from: string, to: string) => {
        setDiffTimestamps({ from, to });
        setDiffLoading(true);
        try {
            const apiUrl = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001";
            const response = await fetch(`${apiUrl}/api/v1/diff?url=${encodeURIComponent(originalUrl)}&from=${from}&to=${to}`);
            const data = await response.json();
            setDiffData(data);
        } catch (e) {
            console.error("Diff failed", e);
        } finally {
            setDiffLoading(false);
        }
    };

    return (
        <div className="flex flex-col h-screen bg-[#0a0a0a] overflow-hidden">
            {/* Time Travel Header */}
            <TimeScrubber
                url={originalUrl}
                currentTimestamp={timestamp}
                onNavigate={handleNavigate}
                onDiff={handleDiff}
                isDiffMode={isDiffMode}
                toggleDiffMode={() => setIsDiffMode(!isDiffMode)}
            />

            {/* Dynamic Content Area */}
            <div className="flex-1 w-full bg-white relative overflow-hidden flex flex-col">
                {isDiffMode ? (
                    <div className="flex-1 flex flex-col overflow-hidden">
                        {/* Internal Toggle */}
                        <div className="absolute top-4 right-6 z-[60] flex bg-black/80 backdrop-blur rounded-full border border-white/10 p-1">
                            <button
                                onClick={() => setDiffType("visual")}
                                className={`px-4 py-1.5 rounded-full text-[10px] font-bold uppercase transition-all ${diffType === "visual" ? "bg-primary-500 text-white" : "text-gray-500 hover:text-white"}`}
                            >
                                Visual
                            </button>
                            <button
                                onClick={() => setDiffType("text")}
                                className={`px-4 py-1.5 rounded-full text-[10px] font-bold uppercase transition-all ${diffType === "text" ? "bg-primary-500 text-white" : "text-gray-500 hover:text-white"}`}
                            >
                                Code
                            </button>
                        </div>

                        {diffType === "visual" && diffTimestamps ? (
                            <VisualDiff
                                url={originalUrl}
                                fromTimestamp={diffTimestamps.from}
                                toTimestamp={diffTimestamps.to}
                            />
                        ) : (
                            <DiffViewer data={diffData} loading={diffLoading} />
                        )}
                    </div>
                ) : (
                    <div className="flex-1 relative">
                        {!iframeSrc ? (
                            <div className="absolute inset-0 flex items-center justify-center bg-[#0a0a0a] text-gray-500">
                                <div className="flex flex-col items-center gap-4">
                                    <div className="w-12 h-12 border-4 border-primary-500/20 border-t-primary-500 rounded-full animate-spin" />
                                    <p className="font-mono text-sm tracking-widest uppercase">Initializing Replay Session...</p>
                                </div>
                            </div>
                        ) : (
                            <iframe
                                src={iframeSrc}
                                className="w-full h-full border-none shadow-2xl"
                                title="ArchiveStream Replay"
                            />
                        )}
                    </div>
                )}
            </div>

            {/* Footer Branding */}
            <div className="px-4 py-1.5 border-t border-white/5 bg-black/95 flex items-center justify-between text-[10px] text-gray-500 font-mono">
                <div>PROXIED VIA ARCHIVESTREAM FAITHFUL REPLAY ENGINE v0.1.0</div>
                <div className="flex items-center gap-4">
                    <span className="flex items-center gap-1 text-green-500/80">
                        <span className="w-1.5 h-1.5 bg-green-500 rounded-full animate-pulse" /> {isDiffMode ? "ANALYZING TEMPORAL DELTA" : "SSL SECURE ARCHIVE"}
                    </span>
                    <span>DOM EXTRACTED: {timestamp}</span>
                </div>
            </div>
        </div>
    );
}

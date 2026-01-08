"use client";

import { useState } from "react";
import { Search, Globe, Clock, Shield, Github } from "lucide-react";

export default function Home() {
    const [searchUrl, setSearchUrl] = useState("");
    const [results, setResults] = useState<any[]>([]);

    const handleSearch = async () => {
        if (searchUrl) {
            try {
                const apiUrl = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001";
                const response = await fetch(`${apiUrl}/search?q=${encodeURIComponent(searchUrl)}`);
                const data = await response.json();

                setResults(data.map((res: any) => ({
                    ...res,
                    displayDate: new Date(res.timestamp).toLocaleString(),
                    replayUrl: `/web/${res.timestamp.replace(/[-:T]/g, "").split(".")[0]}/${res.url}`
                })));
            } catch (error) {
                console.error("Search failed:", error);
            }
        }
    };

    return (
        <main className="min-h-screen bg-[#0a0a0a] text-white selection:bg-primary-500 selection:text-white">
            {/* Navigation */}
            <nav className="border-b border-white/10 px-6 py-4 flex items-center justify-between backdrop-blur-md bg-black/50 sticky top-0 z-50">
                <div className="flex items-center gap-2">
                    <div className="w-8 h-8 bg-primary-600 rounded-lg flex items-center justify-center font-bold text-xl">A</div>
                    <span className="text-xl font-bold tracking-tight">ArchiveStream</span>
                </div>
                <div className="flex items-center gap-6">
                    <a href="#" className="text-sm text-gray-400 hover:text-white transition-colors">Documentation</a>
                    <a href="https://github.com/ArchiveStream/ArchiveStream" className="flex items-center gap-2 bg-white/5 hover:bg-white/10 px-4 py-2 rounded-full border border-white/10 transition-all">
                        <Github size={18} />
                        <span className="text-sm font-medium">GitHub</span>
                    </a>
                </div>
            </nav>

            {/* Hero Section */}
            <section className="relative pt-32 pb-20 px-6 overflow-hidden">
                <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[1000px] h-[600px] bg-primary-900/20 blur-[120px] rounded-full -z-10" />

                <div className="max-w-4xl mx-auto text-center space-y-8">
                    <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-primary-500/10 border border-primary-500/20 text-primary-400 text-sm font-medium">
                        <span className="relative flex h-2 w-2">
                            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-primary-400 opacity-75"></span>
                            <span className="relative inline-flex rounded-full h-2 w-2 bg-primary-500"></span>
                        </span>
                        v0.1.0 Alpha is now live
                    </div>

                    <h1 className="text-6xl md:text-7xl font-bold tracking-tight leading-tight">
                        The Open Source <br />
                        <span className="text-transparent bg-clip-text bg-gradient-to-r from-primary-400 to-primary-600">Web Memory Engine</span>
                    </h1>

                    <p className="text-xl text-gray-400 max-w-2xl mx-auto">
                        Crawl, archive, and replay the web with a modern, high-performance system built in Rust. Self-hostable, WARC-compliant, and truly open.
                    </p>

                    <div className="max-w-2xl mx-auto mt-12 bg-white/5 p-2 rounded-2xl border border-white/10 backdrop-blur-xl shadow-2xl">
                        <div className="relative">
                            <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-500" size={20} />
                            <input
                                type="text"
                                value={searchUrl}
                                onChange={(e) => setSearchUrl(e.target.value)}
                                onKeyDown={(e) => e.key === "Enter" && handleSearch()}
                                placeholder="Paste a URL to archive or search the history..."
                                className="w-full bg-black/40 border border-white/10 rounded-xl py-4 pl-12 pr-32 focus:outline-none focus:ring-2 focus:ring-primary-500/50 transition-all text-lg"
                            />
                            <button
                                onClick={handleSearch}
                                className="absolute right-2 top-1/2 -translate-y-1/2 bg-primary-600 hover:bg-primary-500 text-white px-6 py-2.5 rounded-lg font-bold transition-all"
                            >
                                Archive Now
                            </button>
                        </div>
                    </div>

                    {results.length > 0 && (
                        <div className="mt-12 text-left max-w-2xl mx-auto animate-in fade-in slide-in-from-bottom-4 duration-500">
                            <h2 className="text-lg font-semibold mb-4 text-gray-300">Snapshots for {results[0].url}</h2>
                            <div className="grid gap-3">
                                {results.map((res, i) => (
                                    <a
                                        key={i}
                                        href={res.replayUrl}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        className="flex items-center justify-between p-4 rounded-xl bg-white/5 border border-white/10 hover:border-primary-500/50 hover:bg-white/10 transition-all group"
                                    >
                                        <div className="flex items-center gap-4">
                                            <Clock size={18} className="text-primary-500" />
                                            <div>
                                                <div className="font-medium text-white group-hover:text-primary-400 transition-colors">{res.displayDate}</div>
                                                <div className="text-xs text-gray-500">{res.timestamp}</div>
                                            </div>
                                        </div>
                                        <div className="text-primary-500 opacity-0 group-hover:opacity-100 transition-all translate-x-[-10px] group-hover:translate-x-0">
                                            Replay History →
                                        </div>
                                    </a>
                                ))}
                            </div>
                        </div>
                    )}
                </div>
            </section>

            {/* Features */}
            <section className="py-24 px-6 max-w-7xl mx-auto">
                <div className="grid md:grid-cols-3 gap-8">
                    <div className="p-8 rounded-3xl bg-white/5 border border-white/10 hover:border-primary-500/50 transition-all group">
                        <div className="w-12 h-12 bg-primary-500/10 rounded-xl flex items-center justify-center text-primary-500 mb-6 group-hover:scale-110 transition-transform">
                            <Globe size={24} />
                        </div>
                        <h3 className="text-xl font-bold mb-4">Distributed Crawling</h3>
                        <p className="text-gray-400 leading-relaxed">
                            Highly efficient Rust crawler that respects robots.txt and scales horizontally.
                        </p>
                    </div>

                    <div className="p-8 rounded-3xl bg-white/5 border border-white/10 hover:border-primary-500/50 transition-all group">
                        <div className="w-12 h-12 bg-primary-500/10 rounded-xl flex items-center justify-center text-primary-500 mb-6 group-hover:scale-110 transition-transform">
                            <Shield size={24} />
                        </div>
                        <h3 className="text-xl font-bold mb-4">Immutable Storage</h3>
                        <p className="text-gray-400 leading-relaxed">
                            Industry-standard WARC format storage with content-addressable deduplication.
                        </p>
                    </div>

                    <div className="p-8 rounded-3xl bg-white/5 border border-white/10 hover:border-primary-500/50 transition-all group">
                        <div className="w-12 h-12 bg-primary-500/10 rounded-xl flex items-center justify-center text-primary-500 mb-6 group-hover:scale-110 transition-transform">
                            <Clock size={24} />
                        </div>
                        <h3 className="text-xl font-bold mb-4">Faithful Replay</h3>
                        <p className="text-gray-400 leading-relaxed">
                            Advanced URL rewriting and proxying to replay historical sites exactly as they were.
                        </p>
                    </div>
                </div>
            </section>

            {/* Footer */}
            <footer className="border-t border-white/10 py-12 px-6 text-center text-gray-500 text-sm">
                <p>© 2026 ArchiveStream. Built with Rust & Next.js. Liberated under AGPL-3.0.</p>
            </footer>
        </main>
    );
}

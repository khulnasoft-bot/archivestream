"use client";

import React, { useState, useEffect, useCallback } from "react";
import { Columns, Layers, Maximize2, ImageIcon, Code } from "lucide-react";
import html2canvas from "html2canvas";
import pixelmatch from "pixelmatch";
import { Diff as DomDiff } from "diff-dom"; // Import Diff

interface VisualDiffProps {
  url: string;
  fromTimestamp: string;
  toTimestamp: string;
}

type Mode = "side-by-side" | "overlay" | "pixel-diff" | "dom-diff" | "cross-fade";

export const VisualDiff: React.FC<VisualDiffProps> = ({
  url,
  fromTimestamp,
  toTimestamp,
}) => {
  const [mode, setMode] = useState<Mode>("side-by-side");
  const [sliderPos, setSliderPos] = useState(50);
  const [opacity, setOpacity] = useState(50);
  const [diffImage, setDiffImage] = useState<string | null>(null);
  const [isDiffing, setIsDiffing] = useState(false);
  const [domHtmlFrom, setDomHtmlFrom] = useState<string | null>(null);
  const [domHtmlTo, setDomHtmlTo] = useState<string | null>(null);
  const [isDomDiffing, setIsDomDiffing] = useState(false);
  const apiUrl = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001";

  const fromSrc = `${apiUrl}/web/${fromTimestamp}/${url}`;
  const toSrc = `${apiUrl}/web/${toTimestamp}/${url}`;

  // Pixel Diff generation
  const generatePixelDiff = useCallback(async () => {
    setIsDiffing(true);
    setDiffImage(null);

    const screenshot = (src: string): Promise<HTMLCanvasElement> => {
      return new Promise((resolve, reject) => {
        const iframe = document.createElement("iframe");
        iframe.src = src;
        iframe.style.position = "absolute";
        iframe.style.left = "-9999px";
        iframe.style.top = "-9999px";
        iframe.style.width = "1920px";
        iframe.style.height = "1080px";
        iframe.onload = async () => {
          try {
            const canvas = await html2canvas(iframe.contentWindow!.document.body, {
              useCORS: true,
              width: 1920,
              height: 1080,
              scrollX: 0,
              scrollY: 0,
            });
            document.body.removeChild(iframe);
            resolve(canvas);
          } catch (e) {
            document.body.removeChild(iframe);
            reject(e);
          }
        };
        iframe.onerror = (e) => {
          document.body.removeChild(iframe);
          reject(e);
        };
        document.body.appendChild(iframe);
      });
    };

    try {
      const [fromCanvas, toCanvas] = await Promise.all([
        screenshot(fromSrc),
        screenshot(toSrc),
      ]);

      const width = Math.max(fromCanvas.width, toCanvas.width);
      const height = Math.max(fromCanvas.height, toCanvas.height);

      const diffCanvas = document.createElement("canvas");
      diffCanvas.width = width;
      diffCanvas.height = height;
      const diffCtx = diffCanvas.getContext("2d")!;

      const fromCtx = fromCanvas.getContext("2d")!;
      const toCtx = toCanvas.getContext("2d")!;

      const fromData = fromCtx.getImageData(0, 0, width, height);
      const toData = toCtx.getImageData(0, 0, width, height);
      const diffData = diffCtx.createImageData(width, height);

      pixelmatch(fromData.data, toData.data, diffData.data, width, height, {
        threshold: 0.1,
      });

      diffCtx.putImageData(diffData, 0, 0);
      setDiffImage(diffCanvas.toDataURL());
    } catch (error) {
      console.error("Error generating pixel diff:", error);
    } finally {
      setIsDiffing(false);
    }
  }, [fromSrc, toSrc]);

  useEffect(() => {
    if (mode === "pixel-diff" && !diffImage && !isDiffing) {
      generatePixelDiff();
    }
  }, [mode, diffImage, isDiffing, generatePixelDiff]);

  // DOM Diff generation
  const generateDomContent = useCallback(async () => {
    setIsDomDiffing(true);
    setDomHtmlFrom(null);
    setDomHtmlTo(null);

    try {
      const [fromResponse, toResponse] = await Promise.all([
        fetch(`${apiUrl}/api/v1/snapshot_content/${fromTimestamp}/${url}`),
        fetch(`${apiUrl}/api/v1/snapshot_content/${toTimestamp}/${url}`),
      ]);

      const fromHtml = await fromResponse.text();
      const toHtml = await toResponse.text();

      setDomHtmlFrom(fromHtml);
      setDomHtmlTo(toHtml);

    } catch (error) {
      console.error("Error fetching DOM content:", error);
    } finally {
      setIsDomDiffing(false);
    }
  }, [apiUrl, fromTimestamp, toTimestamp, url]);

  useEffect(() => {
    if (mode === "dom-diff" && (!domHtmlFrom || !domHtmlTo) && !isDomDiffing) {
      generateDomContent();
    }
  }, [mode, domHtmlFrom, domHtmlTo, isDomDiffing, generateDomContent]);

  return (
    <div className="flex-1 flex flex-col bg-[#050505] overflow-hidden">
      {/* Mode Switcher */}
      <div className="px-6 py-3 border-b border-white/5 flex items-center gap-4 bg-black/50 overflow-x-auto no-scrollbar">
        <button
          onClick={() => setMode("side-by-side")}
          className={`flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-bold uppercase transition-all whitespace-nowrap ${mode === "side-by-side" ? "bg-primary-500 text-white" : "hover:bg-white/5 text-gray-400"}`}
        >
          <Columns size={14} />
          Side-by-side
        </button>
        <button
          onClick={() => setMode("overlay")}
          className={`flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-bold uppercase transition-all whitespace-nowrap ${mode === "overlay" ? "bg-primary-500 text-white" : "hover:bg-white/5 text-gray-400"}`}
        >
          <Layers size={14} />
          Slider
        </button>
        <button
          onClick={() => setMode("cross-fade")}
          className={`flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-bold uppercase transition-all whitespace-nowrap ${mode === "cross-fade" ? "bg-primary-500 text-white" : "hover:bg-white/5 text-gray-400"}`}
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10" /><path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20" /></svg>
          Cross-fade
        </button>
        <button
          onClick={() => setMode("pixel-diff")}
          className={`flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-bold uppercase transition-all whitespace-nowrap ${mode === "pixel-diff" ? "bg-primary-500 text-white" : "hover:bg-white/5 text-gray-400"}`}
        >
          <ImageIcon size={14} />
          Pixel Diff
        </button>
        <button
          onClick={() => setMode("dom-diff")}
          className={`flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-bold uppercase transition-all whitespace-nowrap ${mode === "dom-diff" ? "bg-primary-500 text-white" : "hover:bg-white/5 text-gray-400"}`}
        >
          <Code size={14} />
          DOM Diff
        </button>

        <div className="ml-auto hidden md:flex items-center gap-2 text-[10px] text-gray-500 uppercase tracking-widest font-bold whitespace-nowrap">
          {fromTimestamp} → {toTimestamp}
        </div>
      </div>

      <div className="flex-1 relative overflow-hidden bg-white">
        <style jsx global>{`
          iframe {
            animation: fadeIn 0.5s ease-out;
          }
          @keyframes fadeIn {
            from { opacity: 0; }
            to { opacity: 1; }
          }
        `}</style>

        {mode === 'pixel-diff' ? (
          <div className="w-full h-full flex items-center justify-center bg-black/20">
            {isDiffing && (
              <div className="flex flex-col items-center gap-3">
                <div className="w-10 h-10 border-2 border-primary-500 border-t-transparent rounded-full animate-spin" />
                <div className="text-white text-[10px] font-bold uppercase tracking-wider">Analyzing Pixels...</div>
              </div>
            )}
            {diffImage && <img src={diffImage} alt="Pixel Diff" className="max-w-full max-h-full object-contain animate-in fade-in duration-500" />}
          </div>
        ) : mode === 'dom-diff' ? (
          <div className="flex h-full w-full divide-x divide-black/20">
            <div className="flex-1 relative group">
              <div className="absolute top-4 left-4 z-10 px-2 py-1 bg-black/80 backdrop-blur rounded text-[10px] font-bold text-white uppercase border border-white/10 opacity-60 group-hover:opacity-100 transition-opacity">
                Previous: {fromTimestamp}
              </div>
              {isDomDiffing && <div className="text-white absolute inset-0 flex items-center justify-center bg-black/40">Loading...</div>}
              {domHtmlFrom && (
                <iframe
                  srcDoc={domHtmlFrom}
                  className="w-full h-full border-none"
                  title="From DOM Version"
                />
              )}
            </div>
            <div className="flex-1 relative group">
              <div className="absolute top-4 left-4 z-10 px-2 py-1 bg-primary-600/80 backdrop-blur rounded text-[10px] font-bold text-white uppercase border border-white/10 opacity-60 group-hover:opacity-100 transition-opacity">
                Current: {toTimestamp}
              </div>
              {isDomDiffing && <div className="text-white absolute inset-0 flex items-center justify-center bg-black/40">Loading...</div>}
              {domHtmlTo && (
                <iframe
                  srcDoc={domHtmlTo}
                  className="w-full h-full border-none"
                  title="To DOM Version"
                />
              )}
            </div>
          </div>
        ) : mode === "side-by-side" ? (
          <div className="flex h-full w-full divide-x divide-black/20">
            <div className="flex-1 relative group">
              <div className="absolute top-4 left-4 z-10 px-2 py-1 bg-black/80 backdrop-blur rounded text-[10px] font-bold text-white uppercase border border-white/10 opacity-60 group-hover:opacity-100 transition-opacity">
                Previous: {fromTimestamp}
              </div>
              <iframe
                src={fromSrc}
                className="w-full h-full border-none"
                title="From Version"
              />
            </div>
            <div className="flex-1 relative group">
              <div className="absolute top-4 left-4 z-10 px-2 py-1 bg-primary-600/80 backdrop-blur rounded text-[10px] font-bold text-white uppercase border border-white/10 opacity-60 group-hover:opacity-100 transition-opacity">
                Current: {toTimestamp}
              </div>
              <iframe
                src={toSrc}
                className="w-full h-full border-none"
                title="To Version"
              />
            </div>
          </div>
        ) : mode === "cross-fade" ? (
          <div className="relative h-full w-full bg-white select-none">
            {/* Bottom Layer (From) */}
            <div className="absolute inset-0">
              <iframe
                src={fromSrc}
                className="w-full h-full border-none"
                title="From Version"
              />
            </div>

            {/* Top Layer (To) */}
            <div
              className="absolute inset-0 transition-opacity duration-300"
              style={{ opacity: opacity / 100 }}
            >
              <iframe
                src={toSrc}
                className="w-full h-full border-none"
                title="To Version"
              />
            </div>

            {/* Opacity Control */}
            <div className="absolute bottom-8 left-1/2 -translate-x-1/2 z-50 w-64 px-4 py-3 bg-black/80 backdrop-blur-xl rounded-2xl border border-white/10 flex flex-col gap-2">
              <div className="flex justify-between text-[10px] font-bold text-white uppercase tracking-widest">
                <span>Previous</span>
                <span>Current</span>
              </div>
              <input
                type="range"
                min="0"
                max="100"
                value={opacity}
                onChange={(e) => setOpacity(parseInt(e.target.value))}
                className="w-full h-1.5 bg-white/10 rounded-lg appearance-none cursor-pointer accent-primary-500"
              />
            </div>
          </div>
        ) : (
          <div
            className="relative h-full w-full select-none"
            onMouseMove={(e) => {
              if (e.buttons === 1) {
                const rect = e.currentTarget.getBoundingClientRect();
                const x = ((e.clientX - rect.left) / rect.width) * 100;
                setSliderPos(Math.min(100, Math.max(0, x)));
              }
            }}
          >
            {/* Bottom Layer (From) */}
            <div className="absolute inset-0">
              <iframe
                src={fromSrc}
                className="w-full h-full border-none"
                title="From Version"
              />
            </div>

            {/* Top Layer (To) */}
            <div
              className="absolute inset-0 overflow-hidden"
              style={{ width: `${sliderPos}%` }}
            >
              <iframe
                src={toSrc}
                className="w-full h-full border-none shadow-[4px_0_24px_rgba(0,0,0,0.5)]"
                style={{ width: `${100 / (sliderPos / 100)}%` }}
                title="To Version"
              />
            </div>

            {/* Slider Handle */}
            <div
              className="absolute top-0 bottom-0 w-[2px] bg-primary-500 shadow-[0_0_20px_rgba(var(--primary-rgb),0.8)] z-50 cursor-ew-resize flex items-center justify-center translate-x-[-1px]"
              style={{ left: `${sliderPos}%` }}
            >
              <div className="w-10 h-10 rounded-full bg-primary-500 border-4 border-[#050505] flex items-center justify-center shadow-2xl transition-transform hover:scale-110 active:scale-95">
                <div className="flex gap-0.5">
                  <div className="w-0.5 h-3 bg-white/50 rounded-full" />
                  <div className="w-0.5 h-3 bg-white rounded-full" />
                  <div className="w-0.5 h-3 bg-white/50 rounded-full" />
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

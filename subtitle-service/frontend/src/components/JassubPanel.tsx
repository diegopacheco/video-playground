import { useEffect, useRef, useState } from "react";
import JASSUB from "jassub";
import { SOURCE_URL, assUrl } from "../api/client";
import { fontsFor, formatBytes } from "../lib/format";
import type { TrackInfo } from "../types";
import { Panel } from "./Panel";

interface Props {
  track: TrackInfo;
  videoRef: (el: HTMLVideoElement | null) => void;
}

const AVAILABLE_FONTS: Record<string, string> = {
  "noto sans": "/media/fonts/NotoSans-Regular.ttf",
  "noto serif": "/media/fonts/NotoSerif-Regular.ttf",
  "noto sans cjk jp": "/media/fonts/NotoSansCJK-Regular.ttc",
};

export function JassubPanel({ track, videoRef }: Props) {
  const local = useRef<HTMLVideoElement | null>(null);
  const [status, setStatus] = useState("loading libass wasm");

  useEffect(() => {
    const video = local.current;
    if (!video) return;
    const started = performance.now();
    setStatus("loading libass wasm");
    const instance = new JASSUB({
      video,
      subUrl: assUrl(track.locale),
      fonts: fontsFor(track.locale),
      availableFonts: { ...AVAILABLE_FONTS },
      defaultFont: "noto sans",
    });
    let live = true;
    instance.ready.then(
      () => live && setStatus(`ready in ${Math.round(performance.now() - started)} ms`),
      (e: unknown) => live && setStatus(`failed: ${String(e)}`),
    );
    return () => {
      live = false;
      void instance.destroy();
    };
  }, [track.locale]);

  return (
    <Panel
      title="JASSUB (libass wasm)"
      subtitle="raw ASS rendered client-side into a canvas over the video"
      footer={
        <span>
          {formatBytes(track.assBytes)} ASS, full fidelity, {status}
        </span>
      }
    >
      <video
        ref={(el) => {
          local.current = el;
          videoRef(el);
        }}
        src={SOURCE_URL}
        muted
        playsInline
        loop
        preload="auto"
      />
    </Panel>
  );
}

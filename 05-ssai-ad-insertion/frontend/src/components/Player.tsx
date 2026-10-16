import Hls from "hls.js";
import { useEffect, useRef, useState } from "react";

interface Props {
  src: string;
}

export function Player({ src }: Props) {
  const video = useRef<HTMLVideoElement>(null);
  const [status, setStatus] = useState("loading");

  useEffect(() => {
    const element = video.current;
    if (!element) return;
    if (!Hls.isSupported()) {
      element.src = src;
      setStatus("native hls");
      return;
    }
    const hls = new Hls();
    hls.on(Hls.Events.MANIFEST_PARSED, (_, data) => setStatus(`manifest parsed, ${data.levels.length} level`));
    hls.on(Hls.Events.FRAG_CHANGED, (_, data) => setStatus(`playing ${data.frag.relurl ?? ""}`));
    hls.on(Hls.Events.ERROR, (_, data) => setStatus(`error ${data.details}`));
    hls.loadSource(src);
    hls.attachMedia(element);
    element.muted = true;
    element.play().catch(() => undefined);
    return () => hls.destroy();
  }, [src]);

  return (
    <div className="player">
      <video ref={video} controls muted playsInline />
      <div className="status">{status}</div>
    </div>
  );
}

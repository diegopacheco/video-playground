import { useEffect, useState } from "react";

function timeOfDay(ms: number): string {
  return new Date(ms).toISOString().slice(11, 23);
}

export function WallClock() {
  const [now, setNow] = useState(Date.now());

  useEffect(() => {
    let frame = 0;
    const tick = () => {
      setNow(Date.now());
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame);
  }, []);

  return (
    <div className="clock">
      <span className="label">Browser wall clock (UTC)</span>
      <span className="digits">{timeOfDay(now)}</span>
    </div>
  );
}

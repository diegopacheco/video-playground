import type { Segment } from "../types/search";

export function Highlighted({ segments }: { segments: Segment[] }) {
  return (
    <>
      {segments.map((s, i) => (s.hit ? <mark key={i}>{s.text}</mark> : <span key={i}>{s.text}</span>))}
    </>
  );
}

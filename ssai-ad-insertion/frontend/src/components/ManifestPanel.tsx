import { classifyLine, countKind } from "../manifest";

interface Props {
  title: string;
  manifest: string;
}

export function ManifestPanel({ title, manifest }: Props) {
  const lines = manifest.split("\n").filter((l) => l.length > 0);
  return (
    <section className="manifest">
      <header>
        <h3>{title}</h3>
        <span className="counts">
          {countKind(manifest, "cue")} cue · {countKind(manifest, "discontinuity")} discontinuity · {countKind(manifest, "ad")} ad seg ·{" "}
          {countKind(manifest, "slate")} slate seg
        </span>
      </header>
      <pre>
        {lines.map((line, i) => (
          <div key={i} className={`line ${classifyLine(line)}`}>
            <span className="no">{i + 1}</span>
            {line}
          </div>
        ))}
      </pre>
    </section>
  );
}

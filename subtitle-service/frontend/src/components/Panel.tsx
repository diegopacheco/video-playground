import type { ReactNode } from "react";

interface Props {
  title: string;
  subtitle: string;
  children: ReactNode;
  footer: ReactNode;
}

export function Panel({ title, subtitle, children, footer }: Props) {
  return (
    <section className="panel">
      <header>
        <h2>{title}</h2>
        <p>{subtitle}</p>
      </header>
      <div className="stage">{children}</div>
      <footer>{footer}</footer>
    </section>
  );
}

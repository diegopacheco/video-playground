interface Item {
  label: string;
  color: string;
  dashed?: boolean;
}

export function Legend({ items }: { items: Item[] }) {
  return (
    <div className="legend">
      {items.map((item) => (
        <span key={item.label} className="legend-item">
          <span className={item.dashed ? "swatch dashed" : "swatch"} style={{ borderColor: item.color, background: item.dashed ? "transparent" : item.color }} />
          {item.label}
        </span>
      ))}
    </div>
  );
}

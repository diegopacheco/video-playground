const PALETTE: Record<string, string> = {
  throughput: "#2563eb",
  bola: "#d97706",
  dynamic: "#059669",
};

export const BANDWIDTH_COLOR = "#94a3b8";
export const STALL_COLOR = "#dc2626";

export function algorithmColor(id: string): string {
  return PALETTE[id] ?? "#7c3aed";
}

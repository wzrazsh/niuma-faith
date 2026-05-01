export function formatNumber(n: number | null | undefined): string {
  if (n == null) return "0";
  return n.toLocaleString("en-US");
}

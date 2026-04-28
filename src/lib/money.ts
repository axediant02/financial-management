export function pesosFromCents(cents: number): number {
  return cents / 100;
}

export function centsFromPesos(amount: string): number {
  const normalized = amount.replace(/,/g, "").trim();
  if (!normalized) return 0;
  const value = Number(normalized);
  if (!Number.isFinite(value)) return 0;
  return Math.round(value * 100);
}

export function formatPHPFromCents(cents: number): string {
  const value = pesosFromCents(cents);
  return new Intl.NumberFormat("en-PH", {
    style: "currency",
    currency: "PHP",
    maximumFractionDigits: 2,
  }).format(value);
}


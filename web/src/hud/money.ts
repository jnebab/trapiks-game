export function formatPesos(amount: number): string {
  const sign = amount < 0 ? '−' : '';
  return `${sign}₱${Math.abs(amount).toLocaleString('en-US')}`;
}

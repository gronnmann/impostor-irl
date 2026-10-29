export function wirePoint(x: number, y: number, rect: { left: number; top: number; width: number; height: number }): { x: number; y: number } {
  return { x: (x - rect.left) / (rect.width || 1) * 100, y: (y - rect.top) / (rect.height || 1) * 100 };
}
export function batteryPosition(elapsedMs: number): number {
  return (Math.sin(elapsedMs / 620 - Math.PI / 2) + 1) * 50;
}
export function rotateMask(mask: number, turns: number): number {
  const n = ((turns % 4) + 4) % 4;
  return ((mask << n) | (mask >> (4 - n))) & 15;
}
export function circuitConnected(masks: number[], rotations: number[]): boolean {
  if (masks.length !== 9 || rotations.length !== 9) return false;
  const board = masks.map((m, i) => rotateMask(m, rotations[i]));
  if (!(board[0] & 8) || !(board[8] & 2)) return false;
  const seen = new Set<number>([0]); const queue = [0];
  const directions = [[-3, 1, 4], [1, 2, 8], [3, 4, 1], [-1, 8, 2]];
  while (queue.length) {
    const i = queue.shift()!;
    for (let d = 0; d < directions.length; d++) {
      const [step, bit, opposite] = directions[d]; const j = i + step;
      if (j < 0 || j > 8 || (d === 1 && i % 3 === 2) || (d === 3 && i % 3 === 0)) continue;
      if ((board[i] & bit) && (board[j] & opposite) && !seen.has(j)) { seen.add(j); queue.push(j); }
    }
  }
  return seen.has(8);
}
export function parseQrTarget(text: string): { type: 'player' | 'station'; id: string } | null {
  const match = text.trim().match(/^impostor:(player|station):([\w-]+)$/);
  if (match) return { type: match[1] as 'player' | 'station', id: match[2] };
  try {
    const url = new URL(text); const type = url.searchParams.get('target_type'); const id = url.searchParams.get('target');
    if ((type === 'player' || type === 'station') && id) return { type, id };
  } catch { /* a printed code uses the compact format */ }
  return null;
}

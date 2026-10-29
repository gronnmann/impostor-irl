import type { Player } from './types';
export function Avatar({ player, small = false }: { player: Player; small?: boolean }) {
  const palette = ['#34d5e7','#f56575','#f5bf4f','#ab83ff','#65d99b','#f299ce'];
  let index = palette.indexOf(player.color); if (index < 0) index = 0;
  return <span className={`avatar operative-avatar ${small ? 'small' : ''}`} role="img" aria-label={`${player.name}'s character`} style={{ '--avatar-color': player.color || '#34d5e7', '--avatar-x': `${index%3*50}%`, '--avatar-y': `${Math.floor(index/3)*100}%`, '--avatar-hue': player.color?.startsWith('hsl(') ? `${parseInt(player.color.slice(4))-185}deg` : '0deg' } as React.CSSProperties}><span>{player.name.slice(0, 1)}</span></span>;
}

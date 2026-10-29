import type { Kind, Station } from './types';
export const activities: (Station & { zone: string; color: string; tag: string })[] = [
  { id: 'wires', kind: 'wires', name: 'Connect Wires', description: 'A few loose ends. Connect matching terminals to restore power.', duration: '10–20 sec', zone: 'Electrical', color: 'cyan', tag: 'CONNECTION' },
  { id: 'code', kind: 'code', name: 'Enter Code', description: 'One quick look. Remember the access code before it disappears.', duration: '10–15 sec', zone: 'Security', color: 'violet', tag: 'MEMORY' },
  { id: 'signal', kind: 'signal', name: 'Calibrate Signal', description: 'Cut through the noise. Tune the receiver to a stable frequency.', duration: '15–25 sec', zone: 'Communications', color: 'blue', tag: 'PRECISION' },
  { id: 'battery', kind: 'battery', name: 'Charge Battery', description: 'Time it right. Capture energy inside the charging window.', duration: '10–20 sec', zone: 'Power core', color: 'amber', tag: 'TIMING' },
  { id: 'memory', kind: 'memory', name: 'Pattern Memory', description: 'Watch the sequence. Then show the system you were listening.', duration: '15–25 sec', zone: 'Navigation', color: 'pink', tag: 'SEQUENCE' },
  { id: 'anomaly', kind: 'anomaly', name: 'Find Anomaly', description: 'Something is out of place. Find the symbol that does not belong.', duration: '10–15 sec', zone: 'Laboratory', color: 'green', tag: 'OBSERVATION' },
  { id: 'circuits', kind: 'circuits', name: 'Align Circuits', description: 'Make a connection. Rotate the tiles to complete the circuit.', duration: '20–30 sec', zone: 'Engineering', color: 'orange', tag: 'LOGIC' },
  { id: 'password', kind: 'password', name: 'Crack Password', description: 'Follow the clues. Three digits stand between you and access.', duration: '15–30 sec', zone: 'Archives', color: 'indigo', tag: 'DEDUCTION' },
  { id: 'mixing', kind: 'mixing', name: 'Chemical Mixing', description: 'Match both ingredients.', duration: '15–30 sec', zone: 'Laboratory', color: 'cyan', tag: 'CHALLENGE' },
  { id: 'transfer', kind: 'transfer', name: 'Data Transfer', description: 'Keep the cursor inside the moving upload window.', duration: '15–30 sec', zone: 'Data hub', color: 'cyan', tag: 'CHALLENGE' },
  { id: 'fault', kind: 'fault', name: 'Find the Fault', description: 'Repair the irregular flashing light.', duration: '15–30 sec', zone: 'Maintenance', color: 'cyan', tag: 'CHALLENGE' },
  { id: 'frequency', kind: 'frequency', name: 'Frequency Hunt', description: 'Follow the pulses to the hidden frequency.', duration: '15–30 sec', zone: 'Radio room', color: 'cyan', tag: 'CHALLENGE' },
  { id: 'breaker', kind: 'breaker', name: 'Code Breaker', description: 'Guess the PIN using digit clues.', duration: '15–30 sec', zone: 'Security', color: 'cyan', tag: 'CHALLENGE' },
  { id: 'timing', kind: 'timing', name: 'Timing Challenge', description: 'Catch the light inside the narrow target.', duration: '15–30 sec', zone: 'Reactor', color: 'cyan', tag: 'CHALLENGE' },
  { id: 'route_power', kind: 'route_power', name: 'Route the Power', description: 'Rotate circuits to route the power.', duration: '15–30 sec', zone: 'Engineering', color: 'cyan', tag: 'CHALLENGE' },
  { id: 'switches', kind: 'switches', name: 'Switch Sequence', description: 'Flip switches in the displayed order.', duration: '15–30 sec', zone: 'Control room', color: 'cyan', tag: 'CHALLENGE' },
  { id: 'cargo', kind: 'cargo', name: 'Cargo Sorting', description: 'Swipe deliveries into the correct bay.', duration: '15–30 sec', zone: 'Cargo bay', color: 'cyan', tag: 'CHALLENGE' },
];
export const defaultSettings = { activities_per_player: null as number | null, player_count: null as number | null, impostors: 1, kill_cooldown_secs: 30, emergency_cooldown_secs: 30, meeting_duration_secs: 90, kill_mode: 'either' as const, task_mode: 'either' as const };
export function practiceChallenge(kind: Kind): Record<string, any> {
  const code = String(Math.floor(10000 + Math.random() * 90000));
  const sequence = Array.from({ length: 5 }, () => Math.floor(Math.random() * 4));
  const odd = Math.floor(Math.random() * 20);
  const solution = [0, 0, 0, 0, 0, 0, 0, 0, 0];
  return {
    wires: { colors: ['cyan', 'red', 'amber', 'violet'], right_order: [2, 0, 3, 1] },
    code: { code, reveal_ms: 2500 }, signal: { target: 35 + Math.floor(Math.random() * 45), tolerance: 3, hold_ms: 1500 },
    battery: { zone_start: 40, zone_end: 60, required_hits: 3 }, memory: { sequence },
    anomaly: { symbols: Array.from({ length: 20 }, (_, i) => i === odd ? '◈' : '◇'), odd_index: odd },
    circuits: { rotations: [1, 1, 1, 3, 1, 2, 2, 1, 2], solution, masks: [10, 10, 12, 6, 10, 9, 3, 10, 10] },
    mixing: {target_a:35,target_b:65,tolerance:3,hold_ms:1000}, transfer: {required_ms:5000,tolerance:14}, fault: {fault_index:2,lights:4}, frequency: {target:63,tolerance:2,hold_ms:1000}, breaker: {code:'583'}, timing: {zone_start:46,zone_end:54,required_hits:3}, route_power: {rotations:[1,1,1,3,1,2,2,1,2],solution,masks:[10,10,12,6,10,9,3,10,10]}, switches: {order:[2,0,3,1]}, cargo: {categories:['Tools','Supplies'],items:[{label:'Wrench',category:0},{label:'Food',category:1},{label:'Drill',category:0},{label:'Water',category:1}]},
    password: { clues: ['The first digit is 3.', 'The second digit is 2 more than the first.', 'The last digit is 2 more than the second.'], answer: '357' },
  }[kind];
}

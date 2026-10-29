// Integration checks use an isolated server; they never reset the preview game.
import assert from 'node:assert/strict';
import { createServer } from 'node:net';
import { run, start, executable } from './process.mjs';
await run('cargo', ['build', '-p', 'server']);
const reservation = createServer();
await new Promise(resolve => reservation.listen(0, '127.0.0.1', resolve));
const port = reservation.address().port;
await new Promise(resolve => reservation.close(resolve));
const child = start(executable(), [], { env: { ...process.env, PORT: String(port) }, stdio: ['ignore', 'pipe', 'pipe'] });
let logs = ''; child.stdout.on('data', d => logs += d); child.stderr.on('data', d => logs += d);
const origin = `http://127.0.0.1:${port}`;
const wait = ms => new Promise(resolve => setTimeout(resolve, ms));
const command = async (type, fields = {}, token) => {
  const result = await (await fetch(`${origin}/api/command`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ command: { type, ...fields }, ...(token ? { token } : {}) }) })).json();
  return result;
};
const ok = result => { assert.equal(result.ok, true, result.error); return result; };
const state = async token => (await fetch(`${origin}/api/state?${token ? `token=${token}` : 'view=host'}`)).json();
function answer(kind, c) {
  return {
    wires: { connections: c.right_order?.map((_, i) => c.right_order.indexOf(i)) },
    code: { answer: c.code }, signal: { value: c.target, held_ms: c.hold_ms },
    battery: { hits: Array(c.required_hits || 3).fill((c.zone_start + c.zone_end) / 2) },
    memory: { sequence: c.sequence }, anomaly: { index: c.odd_index },
    mixing:{a:c.target_a,b:c.target_b,held_ms:c.hold_ms}, transfer:{progress:100,held_ms:c.required_ms},fault:{index:c.fault_index}, frequency:{value:c.target,held_ms:c.hold_ms},breaker:{answer:c.code},timing:{hits:Array(c.required_hits || 3).fill((c.zone_start+c.zone_end)/2)},route_power:{rotations:c.solution},switches:{sequence:c.order},cargo:{choices:c.items?.map(x=>x.category)},
    circuits: { rotations: c.solution }, password: { answer: c.answer },
  }[kind];
}
try {
  let ready = false;
  for (let i = 0; i < 60; i++) { try { ready = (await fetch(`${origin}/api/health`)).ok; if (ready) break; } catch {} await wait(100); }
  assert.ok(ready, logs || 'Server did not start');
  assert.equal((await fetch(`${origin}/play`)).status, 200, 'Built SPA player route');
  assert.equal((await fetch(`${origin}/manifest.webmanifest`)).status, 200);
  assert.equal((await fetch(`${origin}/sw.js`)).status, 200);
  ok(await command('set_station_location', { station_id: 'wires', location: ' Scenerommet ' }));
  assert.equal((await state()).stations.find(s => s.id === 'wires').location, 'Scenerommet');
  const placed = ok(await command('add_station',{kind:'wires',location:'Kitchen'})).state;
  const duplicate = placed.stations.find(s=>s.kind==='wires' && s.id!=='wires');
  assert.ok(duplicate && duplicate.location==='Kitchen');
  let demo = ok(await command('demo')).state;
  assert.equal(demo.stations.find(s => s.id === 'wires').location, 'Scenerommet', 'Venue locations survive demo setup');
  const [killer, victim, witness] = demo.players;
  const tokens = {};
  for (const p of demo.players) tokens[p.id] = ok(await command('demo_session', { player_id: p.id })).token;
  const crewToken = tokens[witness.id];
  ok(await command('rename_player', { player_id: witness.id, name: 'Captain Test' }));
  const reclaimed = ok(await command('join', { name: ' captain test ', room_code: demo.room_code }));
  assert.equal(reclaimed.token, crewToken, 'Name and room code reclaim the same player');
  assert.equal(reclaimed.state.me.id, witness.id);
  assert.equal((await state()).players.length, 6);
  for (const {kind,id} of demo.stations) {
    const attempt = ok(await command('start_task', { station_id: id, interaction: 'qr' }, crewToken)).state.active_task;
    const done = ok(await command('submit_task', { session_id: attempt.id, result: answer(kind, attempt.challenge) }, crewToken));
    assert.ok(done.state.tasks.find(t => t.station_id === id).completed, kind);
  }
  assert.equal((await state()).task_progress.completed, demo.stations.length);
  const fake = ok(await command('start_task', { station_id: 'code', interaction: 'qr' }, tokens[killer.id])).state.active_task;
  ok(await command('submit_task', { session_id: fake.id, result: { answer: fake.challenge.code } }, tokens[killer.id]));
  assert.equal((await state()).task_progress.completed, demo.stations.length, 'Fake tasks do not advance crew');
  assert.equal((await command('kill', { target_id: victim.id, interaction: 'proximity' }, tokens[killer.id])).ok, false);
  ok(await command('presence', { first: killer.id, second: victim.id, near: true, valid_for_ms: 50 }));
  await wait(80);
  assert.equal((await command('kill', { target_id: victim.id, interaction: 'proximity' }, tokens[killer.id])).ok, false, 'Expired presence rejected');
  ok(await command('presence', { first: killer.id, second: victim.id, near: true, valid_for_ms: 5000 }));
  ok(await command('kill', { target_id: victim.id, interaction: 'proximity' }, tokens[killer.id]));
  const hidden = await state(crewToken);
  assert.equal(hidden.players.find(p => p.id === victim.id).alive, undefined);
  assert.equal(hidden.players.find(p => p.id === killer.id).role, undefined);
  assert.ok(!JSON.stringify(hidden.events).includes('was killed'));
  assert.equal((await state(tokens[victim.id])).me.alive, false);
  ok(await command('report', { target_id: victim.id }, crewToken));
  const meeting = await state(crewToken);
  assert.equal(meeting.phase, 'meeting');
  assert.equal(meeting.players.find(p => p.id === victim.id).alive, false);
  assert.ok((await state()).feedback.some(f => f.target === 'horn'));
  assert.equal((await command('vote', { target_id: killer.id }, tokens[victim.id])).ok, false);
  for (const p of demo.players.filter(p => p.id !== victim.id)) ok(await command('vote', { target_id: killer.id }, tokens[p.id]));
  assert.equal((await state()).winner, 'crew');
  ok(await command('demo'));
  ok(await command('emergency'));
  assert.equal((await state()).phase, 'meeting', 'Shared emergency button');
  ok(await command('end_meeting'));
  assert.equal((await command('emergency')).ok, false, 'Shared cooldown');
  ok(await command('reset'));
  assert.equal((await state()).phase, 'lobby');
  assert.equal((await state()).stations.find(s => s.id === 'wires').location, 'Scenerommet', 'Venue locations survive reset');
  assert.equal((await command('demo_session', { player_id: killer.id })).ok, false, 'Demo switching disabled in real lobby');
  const room = (await state()).room_code;
  const joined = ok(await command('join', { name: 'Real player', room_code: room }));
  assert.equal((await state(joined.token)).me.name, 'Real player');
  const before = (await state()).players.length;
  ok(await command('join', { name: 'Real player', room_code: room }, joined.token));
  assert.equal((await state()).players.length, before, 'Reconnect preserves identity');
  ok(await command('start'));
  assert.equal((await state()).phase, 'playing');
  const recovered = ok(await command('join', { name: ' REAL PLAYER ', room_code: room }));
  assert.equal(recovered.token, joined.token, 'Rejoin during an active round');
  assert.equal((await state()).players.length, before);
  assert.equal((await command('demo_session', { player_id: killer.id })).ok, false);
  ok(await command('reset'));
  const setup = await state();
  ok(await command('configure',{settings:{...setup.settings,player_count:80,impostors:12}}));
  assert.equal((await command('start')).ok,false,'Wait for expected roster');
  for(let i=setup.players.length;i<80;i++)ok(await command('join',{name:`Group member ${i+1}`,room_code:setup.room_code}));
  assert.equal((await state()).players.length,80,'No legacy player cap');
  assert.equal((await command('join',{name:'Overflow seat',room_code:setup.room_code})).ok,false,'Host capacity respected');
  const large=ok(await command('start')).state;
  assert.equal(large.players.filter(p=>p.role==='impostor').length,12);
  assert.equal(large.players.filter(p=>p.role==='crew').length,68);
  assert.equal(large.tasks.length,68*large.stations.length);
  console.log('PASS: 80 players / 12 impostors, repeated station IDs, all seventeen activities, fake tasks, proximity expiry, kill privacy, reports, meeting targets, horn, voting/victory, emergency cooldown, reconnect, normal start, and production SPA routes.');
} finally { child.kill(); }

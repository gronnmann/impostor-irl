import { existsSync } from 'node:fs';
import { root, run, start, executable } from './process.mjs';
if (!existsSync(`${root}web/node_modules/vite/bin/vite.js`)) {
  console.error('Install frontend dependencies first: npm --prefix web install'); process.exit(1);
}
await run('cargo', ['build', '-p', 'server']);
const children = [start(executable()), start(process.execPath, [`${root}web/node_modules/vite/bin/vite.js`, '--host', '0.0.0.0'], { cwd: `${root}web` })];
let stopping = false;
function stop(code = 0) { if (stopping) return; stopping = true; children.forEach(c => c.kill()); setTimeout(() => process.exit(code), 150); }
children.forEach(child => { child.on('error', error => { console.error(error.message); stop(1); }); child.on('exit', code => { if (!stopping) stop(code || 0); }); });
process.on('SIGINT', () => stop()); process.on('SIGTERM', () => stop());
console.log('\nMission control: http://localhost:5173\nCtrl+C stops both services.\n');

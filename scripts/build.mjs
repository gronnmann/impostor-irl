import { root, run } from './process.mjs';
await run(process.execPath, [`${root}web/node_modules/typescript/bin/tsc`, '-b'], { cwd: `${root}web` });
await run(process.execPath, [`${root}web/node_modules/vite/bin/vite.js`, 'build'], { cwd: `${root}web` });
await run('cargo', ['build', '--release', '-p', 'server']);
console.log('Built web/dist and the release server. Run npm start to serve both on port 3001.');

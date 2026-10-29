import { existsSync } from 'node:fs';
import { run, executable } from './process.mjs';
if (!existsSync(executable('release'))) { console.error('Build first: npm run build'); process.exit(1); }
await run(executable('release'));

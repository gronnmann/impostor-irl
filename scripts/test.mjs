import { root, run } from './process.mjs';
await run('cargo', ['test', '--workspace']);
await run(process.execPath, ['--test', `${root}web/tests/puzzles.test.mjs`]);

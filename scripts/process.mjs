import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';
export const root = fileURLToPath(new URL('../', import.meta.url));
export function run(command, args = [], options = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, { cwd: root, stdio: 'inherit', windowsHide: true, ...options });
    child.on('error', reject);
    child.on('exit', code => code === 0 ? resolve() : reject(new Error(`${command} exited with ${code}`)));
  });
}
export function start(command, args = [], options = {}) {
  return spawn(command, args, { cwd: root, stdio: 'inherit', windowsHide: true, ...options });
}
export const executable = (mode = 'debug') => `${root}target/${mode}/server${process.platform === 'win32' ? '.exe' : ''}`;

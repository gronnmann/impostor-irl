import { test } from 'node:test';
import assert from 'node:assert/strict';
import QRCode from 'qrcode';
import jsQR from 'jsqr';
import pngjs from 'pngjs';
import { batteryPosition, wirePoint, rotateMask, circuitConnected, parseQrTarget } from '../src/puzzles.ts';

test('wire coordinates follow actual board bounds, including resize and outside drags', () => {
  assert.deepEqual(wirePoint(140, 250, { left: 40, top: 50, width: 200, height: 400 }), { x: 50, y: 50 });
  assert.deepEqual(wirePoint(140, 250, { left: 40, top: 50, width: 400, height: 200 }), { x: 25, y: 100 });
  assert.deepEqual(wirePoint(20, 550, { left: 40, top: 50, width: 200, height: 400 }), { x: -10, y: 125 });
  assert.deepEqual(wirePoint(0, 0, { left: 0, top: 0, width: 0, height: 0 }), { x: 0, y: 0 });
});

test('battery clock starts each fresh pass outside the capture window', () => {
  const hitAt = 620 * Math.PI / 2;
  assert.ok(Math.abs(batteryPosition(hitAt) - 50) < 1e-10);
  assert.equal(batteryPosition(hitAt - hitAt), 0);
  for (const elapsed of [1, 20, 100]) assert.ok(batteryPosition(elapsed) < 35);
  assert.ok(Math.abs(batteryPosition(620 * Math.PI) - 100) < 1e-10);
});

test('pipe rotation follows clockwise N/E/S/W edges', () => {
  assert.equal(rotateMask(1, 1), 2); assert.equal(rotateMask(1, 2), 4);
  assert.equal(rotateMask(1, 3), 8); assert.equal(rotateMask(1, 4), 1);
  assert.equal(rotateMask(10, 1), 5);
});
test('circuits require a reciprocal path between the two outside terminals', () => {
  const masks = [10,10,12,6,10,9,3,10,10];
  assert.equal(circuitConnected(masks, Array(9).fill(0)), true);
  const broken = Array(9).fill(0); broken[2] = 1;
  assert.equal(circuitConnected(masks, broken), false);
  assert.equal(circuitConnected([2,...masks.slice(1)], Array(9).fill(0)), false);
  assert.equal(circuitConnected([], []), false);
});
test('QR parsing supports both compact printed codes and target links', () => {
  assert.deepEqual(parseQrTarget('impostor:station:code'), { type: 'station', id: 'code' });
  assert.deepEqual(parseQrTarget('impostor:player:player-12'), { type: 'player', id: 'player-12' });
  assert.deepEqual(parseQrTarget('https://example.test/play?target_type=station&target=signal'), { type: 'station', id: 'signal' });
  assert.equal(parseQrTarget('not-a-game-code'), null);
});
test('the actual QR generator produces codes readable by the camera decoder', async () => {
  const value = 'impostor:station:circuits';
  const buffer = await QRCode.toBuffer(value, { width: 320, margin: 2 });
  const png = pngjs.PNG.sync.read(buffer);
  const decoded = jsQR(new Uint8ClampedArray(png.data), png.width, png.height);
  assert.equal(decoded?.data, value);
  assert.deepEqual(parseQrTarget(decoded.data), { type: 'station', id: 'circuits' });
});

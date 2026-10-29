import { Mixing, Transfer, Fault, Frequency, Breaker, Timing, Switches, Cargo } from './AdditionalActivities';
import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import { ArrowLeft, Check, CheckCircle2, ChevronRight, Delete, LockKeyhole, RefreshCw, ShieldCheck, Sparkles, Volume2, X, Zap } from 'lucide-react';
import type { ActivitySession, Kind } from './types';
import { activities } from './catalog';
import { ActivityIcon } from './icons';
import { batteryPosition, circuitConnected, rotateMask, wirePoint } from './puzzles';

type Result = Record<string, unknown>;
export function ActivityModal({ session, practice, stationLocation, onSubmit, onClose }: { session: ActivitySession; practice: boolean; stationLocation?: string; onSubmit: (result: Result) => Promise<void>; onClose: () => void }) {
  const [done, setDone] = useState(false); const [busy, setBusy] = useState(false); const [error, setError] = useState('');
  const [pendingResult, setPendingResult] = useState<Result | null>(null);
  const base = activities.find(a => a.kind === session.kind)!;
  const meta = { ...base, name: stationLocation ? `${base.name} (${stationLocation})` : base.name, zone: stationLocation || base.zone };
  const complete = async (result: Result) => { if (busy || done) return; setPendingResult(result); setBusy(true); setError(''); try { await onSubmit(result); setDone(true); } catch (e) { setError((e as Error).message); } finally { setBusy(false); } };
  useEffect(() => { const listener = (e: KeyboardEvent) => { if (e.key === 'Escape') onClose(); }; window.addEventListener('keydown', listener); return () => window.removeEventListener('keydown', listener); }, [onClose]);
  const Puzzle = { wires: Wires, code: Code, signal: Signal, battery: Battery, memory: Memory, anomaly: Anomaly, circuits: Circuits, password: Password, mixing: Mixing, transfer: Transfer, fault: Fault, frequency: Frequency, breaker: Breaker, timing: Timing, route_power: Circuits, switches: Switches, cargo: Cargo }[session.kind];
  return <div className="modal-backdrop"><section className={`activity-modal ${meta.color}`} role="dialog" aria-modal="true" aria-label={meta.name}>
    <header className="modal-header"><span className="eyebrow">{practice ? 'ACTIVITY PRACTICE' : `${meta.zone.toUpperCase()} / ACTIVE TASK`}</span><button className="icon-button" onClick={onClose} aria-label="Close activity"><X size={20}/></button></header>
    {done ? <div className="task-success"><div className="success-orbit"><CheckCircle2 size={56}/></div><span className="eyebrow">{practice ? 'NICE WORK, OPERATIVE' : 'PROGRESS SYNCED'}</span><h2>{practice ? 'Practice complete.' : 'Task complete.'}</h2><p>{practice ? 'Ready for the real mission? Your practice does not affect the round.' : 'One step closer. Keep the crew moving.'}</p><button className="button dark" onClick={onClose}>Back to mission <ChevronRight size={16}/></button></div>
    : <><div className="puzzle-heading"><span className={`activity-symbol ${meta.color}`}><ActivityIcon kind={session.kind} size={30}/></span><div><h2>{meta.name}</h2><p>{meta.description}</p></div></div><div className="puzzle-body" aria-busy={busy}><Puzzle challenge={session.challenge} complete={complete}/></div>{error && <div className="inline-error" role="alert"><p>{error}</p>{pendingResult && <button className="button outline small" disabled={busy} onClick={() => complete(pendingResult)}>Retry submission <RefreshCw size={13}/></button>}</div>}<footer className="puzzle-footer"><span><ShieldCheck size={14}/> {practice ? 'Practice mode · no round progress' : 'Connected to mission control'}</span><span>{meta.duration}</span></footer></>}
  </section></div>;
}
type PuzzleProps = { challenge: Record<string, any>; complete: (result: Result) => Promise<void> };
const wireColors = ['#00bdd4', '#ff3b5c', '#f7ae33', '#9771f3'];
function Wires({ challenge: c, complete }: PuzzleProps) {
  const order: number[] = c.right_order; const [selected, setSelected] = useState<number | null>(null); const [connections, setConnections] = useState<number[]>([-1, -1, -1, -1]); const [wrong, setWrong] = useState(-1);
  const board = useRef<HTMLDivElement>(null); const svg = useRef<SVGSVGElement>(null);
  type Point = { x: number; y: number };
  type Drag = { left: number; pointerId: number; originX: number; originY: number; point: Point; moved: boolean };
  const [sockets, setSockets] = useState<{ left: Point[]; right: Point[] }>({ left: [], right: [] });
  const [drag, setDrag] = useState<Drag | null>(null); const activeDrag = useRef<Drag | null>(null); const suppressClick = useRef(false);
  const wrongTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  useLayoutEffect(() => {
    const measure = () => {
      const rect = svg.current!.getBoundingClientRect();
      const centers = (side: string) => Array.from(board.current!.querySelectorAll(`[data-${side}] i`), socket => { const r = socket.getBoundingClientRect(); return wirePoint(r.left + r.width / 2, r.top + r.height / 2, rect); });
      setSockets({ left: centers('left'), right: centers('right') });
    };
    measure();
    const observer = new ResizeObserver(measure); observer.observe(board.current!);
    board.current!.querySelectorAll('.terminal-column, .terminal, .terminal i').forEach(el => observer.observe(el));
    window.addEventListener('resize', measure);
    return () => { observer.disconnect(); window.removeEventListener('resize', measure); clearTimeout(wrongTimer.current); };
  }, []);
  const connect = (left: number, right: number) => { if (order[right] !== left) { setWrong(right); clearTimeout(wrongTimer.current); wrongTimer.current = setTimeout(() => setWrong(-1), 450); return; } const next = [...connections]; next[left] = right; setConnections(next); setSelected(null); setWrong(-1); if (next.every(n => n >= 0)) complete({ connections: next }); };
  const cancelDrag = () => { activeDrag.current = null; setDrag(null); setSelected(null); suppressClick.current = true; };
  const path = (from: Point, to: Point) => { const bend = Math.max(12, Math.abs(to.x - from.x) * .45); return `M ${from.x} ${from.y} C ${from.x + bend} ${from.y}, ${to.x - bend} ${to.y}, ${to.x} ${to.y}`; };
  return <><p className="instruction">Drag each wire to its matching socket</p><div ref={board} className={`wire-board ${drag ? 'dragging' : ''}`} style={{ touchAction: 'none' }}
    onPointerMove={e => { const current = activeDrag.current; if (!current || current.pointerId !== e.pointerId) return; const next = { ...current, point: wirePoint(e.clientX, e.clientY, svg.current!.getBoundingClientRect()), moved: current.moved || Math.hypot(e.clientX - current.originX, e.clientY - current.originY) > 6 }; activeDrag.current = next; setDrag(next); }}
    onPointerUp={e => {
      const current = activeDrag.current; if (!current || current.pointerId !== e.pointerId) return;
      const moved = current.moved || Math.hypot(e.clientX - current.originX, e.clientY - current.originY) > 6;
      activeDrag.current = null; setDrag(null); suppressClick.current = moved;
      const target = document.elementFromPoint(e.clientX, e.clientY)?.closest('[data-right]');
      if (target && board.current!.contains(target)) { suppressClick.current = true; connect(current.left, Number(target.getAttribute('data-right'))); }
      else if (moved) setSelected(null);
    }}
    onPointerCancel={e => { if (activeDrag.current?.pointerId === e.pointerId) cancelDrag(); }}
    onLostPointerCapture={e => { if (activeDrag.current?.pointerId === e.pointerId) cancelDrag(); }}>
    <svg ref={svg} viewBox="0 0 100 100" preserveAspectRatio="none" className="wire-lines" aria-hidden="true" style={{ pointerEvents: 'none', overflow: 'visible' }}>
      {connections.map((right, left) => right >= 0 && drag?.left !== left && sockets.left[left] && sockets.right[right] && <path key={left} d={path(sockets.left[left], sockets.right[right])} stroke={wireColors[left]} fill="none"/>)}
      {drag && sockets.left[drag.left] && <g className="loose-wire"><path d={path(sockets.left[drag.left], drag.point)} stroke={wireColors[drag.left]} fill="none"/><circle className="drag-plug" cx={drag.point.x} cy={drag.point.y} r="2" fill={wireColors[drag.left]} stroke="white" strokeWidth=".7"/></g>}
    </svg>
    <div className="terminal-column">{c.colors.map((_: string, i: number) => <button key={i} data-left={i} className={`terminal ${selected === i ? 'selected' : ''} ${drag?.left === i ? 'dragging' : ''} ${connections[i] >= 0 ? 'connected' : ''}`} style={{ '--wire': wireColors[i], touchAction: 'none' } as React.CSSProperties}
      onPointerDown={e => { if (!e.isPrimary || e.button !== 0 || activeDrag.current) return; e.currentTarget.setPointerCapture(e.pointerId); suppressClick.current = false; setSelected(i); const next = { left: i, pointerId: e.pointerId, originX: e.clientX, originY: e.clientY, point: wirePoint(e.clientX, e.clientY, svg.current!.getBoundingClientRect()), moved: false }; activeDrag.current = next; setDrag(next); }}
      onClick={e => { if (e.detail === 0 || !suppressClick.current) setSelected(i); }} aria-pressed={selected === i} aria-label={`Left terminal ${i + 1}`}><span>{['△', '◇', '○', '□'][i]}</span><i/></button>)}</div>
    <div className="terminal-column">{order.map((color, i) => <button key={i} data-right={i} className={`terminal right ${wrong === i ? 'shake wrong' : ''} ${connections.includes(i) ? 'connected' : ''}`} style={{ '--wire': wireColors[color] } as React.CSSProperties} onClick={() => selected !== null && connect(selected, i)} aria-label={`Right terminal ${i + 1}`}><i/><span>{['△', '◇', '○', '□'][color]}</span></button>)}</div>
  </div><div className="puzzle-status" role="status">{wrong >= 0 ? 'Wrong socket. Match the color and symbol.' : `${connections.filter(n => n >= 0).length} / 4 connections restored`}</div><p className="puzzle-hint">You can also tap a wire, then its socket.</p></>;
}
function NumberPad({ answer, setAnswer, onSubmit, length = 6 }: { answer: string; setAnswer: (s: string) => void; onSubmit: () => void; length?: number }) {
  return <><div className="code-entry" aria-label="Entered digits">{Array.from({ length }, (_, i) => <span key={i} className={answer[i] ? 'filled' : ''}>{answer[i] || '·'}</span>)}</div><div className="number-pad">{['1','2','3','4','5','6','7','8','9','back','0','go'].map(key => <button key={key} aria-label={key === 'back' ? 'Delete digit' : key === 'go' ? 'Submit code' : `Digit ${key}`} className={key === 'go' ? 'submit-key' : ''} onClick={() => key === 'back' ? setAnswer(answer.slice(0, -1)) : key === 'go' ? onSubmit() : answer.length < length && setAnswer(answer + key)}>{key === 'back' ? <Delete size={20}/> : key === 'go' ? <ChevronRight size={22}/> : key}</button>)}</div></>;
}
function Code({ challenge: c, complete }: PuzzleProps) {
  const [revealed, setRevealed] = useState(true); const [answer, setAnswer] = useState(''); const [error, setError] = useState('');
  useEffect(() => { const t = setTimeout(() => setRevealed(false), c.reveal_ms); return () => clearTimeout(t); }, [c]);
  const submit = () => { if (answer === c.code) complete({ answer }); else { setError('Not quite. Take another look and try again.'); setAnswer(''); } };
  return <>{revealed ? <div className="code-reveal"><LockKeyhole size={26}/><p>MEMORIZE THIS ACCESS CODE</p><strong>{c.code}</strong><span className="reveal-timer" style={{ animationDuration: `${c.reveal_ms}ms` }}/></div> : <><p className="instruction">Enter the code you just saw.</p><NumberPad answer={answer} setAnswer={setAnswer} onSubmit={submit} length={String(c.code).length}/></>}{error && <p className="puzzle-hint">{error}</p>}<button className="text-button center" onClick={() => { setRevealed(true); setError(''); setTimeout(() => setRevealed(false), c.reveal_ms); }}><RefreshCw size={14}/> Show code again</button></>;
}
function Signal({ challenge: c, complete }: PuzzleProps) {
  const [value, setValue] = useState(20); const [held, setHeld] = useState(0); const finished = useRef(false); const inRange = Math.abs(value - c.target) <= c.tolerance;
  const completeRef = useRef(complete); completeRef.current = complete;
  useEffect(() => { if (!inRange) { setHeld(0); return; } const start = performance.now(); const timer = setInterval(() => { const ms = performance.now() - start; setHeld(ms); if (ms >= c.hold_ms && !finished.current) { finished.current = true; completeRef.current({ value, held_ms: c.hold_ms }).catch(() => { finished.current = false; }); } }, 50); return () => clearInterval(timer); }, [inRange, value, c.hold_ms]);
  return <><div className={`signal-display ${inRange ? 'locked' : ''}`}><div className="waveform">{Array.from({ length: 37 }, (_, i) => <i key={i} style={{ height: `${15 + Math.abs(Math.sin(i * 1.7)) * 65}%`, animationDelay: `${i * .04}s` }}/>)}</div><div className="frequency"><strong>{value.toFixed(1)}</strong><span>MHz</span></div><span className="signal-state">{inRange ? 'SIGNAL LOCKED · HOLD STEADY' : 'SEARCHING FOR FREQUENCY'}</span></div><p className="instruction">Tune to <strong>{c.target} MHz</strong> and hold for {(c.hold_ms / 1000).toFixed(1)} seconds.</p><input aria-label="Signal frequency" className="signal-slider" type="range" min="0" max="100" step="1" value={value} onChange={e => setValue(Number(e.target.value))}/><div className="range-labels"><span>0 MHz</span><span>100 MHz</span></div><div className="progress-track"><i style={{ width: `${Math.min(100, held / c.hold_ms * 100)}%` }}/></div></>;
}
function Battery({ challenge: c, complete }: PuzzleProps) {
  const [position, setPosition] = useState(0); const [hits, setHits] = useState<number[]>([]); const [miss, setMiss] = useState(false); const start = useRef(performance.now());
  useEffect(() => { let frame: number; const tick = () => { setPosition(batteryPosition(performance.now() - start.current)); frame = requestAnimationFrame(tick); }; frame = requestAnimationFrame(tick); return () => cancelAnimationFrame(frame); }, []);
  const stop = () => { if (hits.length >= c.required_hits) return; const now = performance.now(); const captured = batteryPosition(now - start.current); if (captured >= c.zone_start && captured <= c.zone_end) { const next = [...hits, captured]; start.current = now; setPosition(0); setHits(next); setMiss(false); if (next.length >= c.required_hits) complete({ hits: next }); } else { setMiss(true); setTimeout(() => setMiss(false), 600); } };
  return <><div className={`battery-visual ${miss ? 'shake' : ''}`}><div className="battery-cap"/><div className="battery-body">{Array.from({ length: c.required_hits }, (_, i) => <i key={i} className={hits.length > i ? 'charged' : ''}/>)}</div><Zap size={38}/></div><p className="instruction">Stop the indicator inside the cyan zone.</p><div className="timing-track"><div className="target-zone" style={{ left: `${c.zone_start}%`, width: `${c.zone_end - c.zone_start}%` }}/><i style={{ left: `${position}%` }}/></div><button className="button dark wide" onClick={stop}><Zap size={18}/> Capture charge</button><div className="puzzle-status">{miss ? 'Missed the window. Try again.' : `${hits.length} / ${c.required_hits} cells charged`}</div></>;
}
function Memory({ challenge: c, complete }: PuzzleProps) {
  const [watch, setWatch] = useState(true); const [flash, setFlash] = useState(-1); const [entered, setEntered] = useState<number[]>([]); const [attempt, setAttempt] = useState(0); const [message, setMessage] = useState('Watch the sequence.');
  useEffect(() => { setWatch(true); setEntered([]); const timers: ReturnType<typeof setTimeout>[] = []; c.sequence.forEach((n: number, i: number) => { timers.push(setTimeout(() => setFlash(n), 650 + i * 700)); timers.push(setTimeout(() => setFlash(-1), 1100 + i * 700)); }); timers.push(setTimeout(() => { setWatch(false); setMessage('Your turn. Repeat the sequence.'); }, 650 + c.sequence.length * 700)); return () => timers.forEach(clearTimeout); }, [c, attempt]);
  const press = (n: number) => { if (watch) return; setFlash(n); setTimeout(() => setFlash(-1), 180); if (c.sequence[entered.length] !== n) { setMessage('Sequence missed. Watch and try again.'); setAttempt(a => a + 1); return; } const next = [...entered, n]; setEntered(next); if (next.length === c.sequence.length) complete({ sequence: next }); };
  return <><p className="instruction"><Volume2 size={16}/> {message}</p><div className="memory-pad">{['△','◇','○','□'].map((symbol, i) => <button key={i} aria-label={`Memory pad ${i + 1}`} disabled={watch} onClick={() => press(i)} className={`memory-button pad-${i} ${flash === i ? 'flash' : ''}`}>{symbol}</button>)}</div><div className="sequence-dots">{c.sequence.map((_: number, i: number) => <i key={i} className={entered.length > i ? 'filled' : ''}/>)}</div><button className="text-button center" onClick={() => { setMessage('Watch the sequence.'); setAttempt(a => a + 1); }}><RefreshCw size={14}/> Replay sequence</button></>;
}
function Anomaly({ challenge: c, complete }: PuzzleProps) {
  const [wrong, setWrong] = useState(-1);
  return <><p className="instruction">One signal is different. Find the anomaly.</p><div className="anomaly-grid">{c.symbols.map((symbol: string, i: number) => <button key={i} aria-label={`Symbol ${i + 1}`} className={wrong === i ? 'shake incorrect' : ''} onClick={() => i === c.odd_index ? complete({ index: i }) : (setWrong(i), setTimeout(() => setWrong(-1), 450))}>{symbol}</button>)}</div><div className="puzzle-status">Look closely. Every detail matters.</div></>;
}
function Circuits({ challenge: c, complete }: PuzzleProps) {
  const [rotations, setRotations] = useState<number[]>(c.rotations);
  const turn = (index: number) => { const next = [...rotations]; next[index] = (next[index] + 1) % 4; setRotations(next); if (circuitConnected(c.masks, next)) complete({ rotations: next }); };
  return <><p className="instruction">Rotate tiles. Connect IN at the top left to OUT at the bottom right.</p><div className="circuit-frame"><span className="circuit-in">IN →</span><div className="circuit-grid">{c.masks.map((mask: number, i: number) => { const rotated = rotateMask(mask, rotations[i]); return <button key={i} onClick={() => turn(i)} aria-label={`Rotate circuit tile ${i + 1}`}><svg viewBox="0 0 100 100"><circle cx="50" cy="50" r="8"/>{[[1,50,0],[2,100,50],[4,50,100],[8,0,50]].map(([bit,x,y]) => rotated & bit ? <line key={bit} x1="50" y1="50" x2={x} y2={y}/> : null)}</svg></button>; })}</div><span className="circuit-out">→ OUT</span></div><button className="text-button center" onClick={() => setRotations([...c.rotations])}><RefreshCw size={14}/> Reset circuit</button></>;
}
function Password({ challenge: c, complete }: PuzzleProps) {
  const [answer, setAnswer] = useState(''); const [error, setError] = useState(false);
  return <><div className="clue-list">{c.clues.map((clue: string, i: number) => <div key={i}><span>0{i + 1}</span><p>{clue}</p></div>)}</div><NumberPad answer={answer} setAnswer={setAnswer} length={3} onSubmit={() => { if (answer === c.answer) complete({ answer }); else { setError(true); setAnswer(''); } }}/>{error && <p className="puzzle-hint">Access denied. Check the clues and try again.</p>}</>;
}

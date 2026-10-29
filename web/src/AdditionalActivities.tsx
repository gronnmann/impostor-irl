import { useEffect, useRef, useState } from 'react';
import { batteryPosition } from './puzzles';
import './AdditionalActivities.css';

type Props = { challenge: Record<string, any>; complete: (result: Record<string, unknown>) => Promise<void> };

function useSubmission(complete: Props['complete']) {
  const latest = useRef(complete); latest.current = complete;
  const sent = useRef(false);
  const [busy, setBusy] = useState(false);
  const submit = useRef((result: Record<string, unknown>) => {
    if (sent.current) return;
    sent.current = true; setBusy(true);
    // The modal owns submission errors and its explicit retry button.
    void latest.current(result).catch(() => { sent.current = false; setBusy(false); });
  });
  return { submit: submit.current, busy };
}

function Meter({ value, label }: { value: number; label: string }) {
  return <div className="progress-track" role="progressbar" aria-label={label} aria-valuemin={0} aria-valuemax={100} aria-valuenow={Math.round(Math.min(100, value))}><i style={{ width: `${Math.min(100, value)}%` }}/></div>;
}

function useHold(active: boolean, duration: number, setting: number, finish: () => void) {
  const latest = useRef(finish); latest.current = finish;
  const [held, setHeld] = useState(0);
  useEffect(() => {
    setHeld(0);
    if (!active) return;
    let start = performance.now();
    const reset = () => { start = performance.now(); setHeld(0); };
    document.addEventListener('visibilitychange', reset);
    const timer = window.setInterval(() => {
      if (document.hidden) { start = performance.now(); setHeld(0); return; }
      const elapsed = performance.now() - start;
      setHeld(Math.min(duration, elapsed));
      if (elapsed >= duration) { clearInterval(timer); latest.current(); }
    }, 40);
    return () => { clearInterval(timer); document.removeEventListener('visibilitychange', reset); };
  }, [active, duration, setting]);
  return held;
}

export function Mixing({ challenge: c, complete }: Props) {
  const [a, setA] = useState(50); const [b, setB] = useState(50);
  const { submit, busy } = useSubmission(complete);
  const balanced = Math.abs(a - c.target_a) <= c.tolerance && Math.abs(b - c.target_b) <= c.tolerance;
  const held = useHold(balanced, c.hold_ms, a * 101 + b, () => submit({ a, b, held_ms: c.hold_ms }));
  return <div className="extra-puzzle"><p className="instruction">Blend A and B, then keep the mixture steady.</p>
    <div className={`instrument mixing-panel ${balanced ? 'is-locked' : ''}`}>
      <span className="instrument-label">REACTION CHAMBER</span>
      <svg viewBox="0 0 300 170" aria-hidden="true"><path d="M65 18h36m-30 0v42l-40 71q-10 23 17 23h71q27 0 17-23L95 60V18M199 18h36m-30 0v42l-40 71q-10 23 17 23h71q27 0 17-23l-41-71V18" fill="#0b253a" stroke="#94b6ce" strokeWidth="4"/>
        <path d="M42 124h82l6 12q4 10-10 10H49q-15 0-11-10Z" fill="#00d6eb"/><path d="M175 112h82l13 24q4 10-10 10h-78q-15 0-11-10Z" fill="#bd91ff"/>
        <g fill="#b7faff"><circle cx="69" cy="121" r="4"/><circle cx="95" cy="133" r="3"/></g><g fill="#eedfff"><circle cx="212" cy="126" r="4"/><circle cx="235" cy="119" r="3"/></g>
        <text x="84" y="98" textAnchor="middle" fill="#64edff" fontSize="22">A</text><text x="219" y="87" textAnchor="middle" fill="#dabdff" fontSize="22">B</text></svg>
      <div className="mix-readings"><strong>A {a}% <small>target {c.target_a}%</small></strong><strong>B {b}% <small>target {c.target_b}%</small></strong></div>
    </div>
    <label className="tuner-label" htmlFor="mix-ratio">Ingredient A · {a}%</label><input id="mix-ratio" className="signal-slider" type="range" min="0" max="100" step="1" value={a} disabled={busy} onChange={e => setA(Number(e.target.value))}/>
    <label className="tuner-label" htmlFor="mix-b">Ingredient B · {b}%</label><input id="mix-b" className="signal-slider" type="range" min="0" max="100" step="1" value={b} disabled={busy} onChange={e=>setB(Number(e.target.value))}/><div className="range-labels"><span>0%</span><span>100%</span></div><Meter value={held / c.hold_ms * 100} label="Mixture stabilization"/>
    <p className="puzzle-status" role="status">{busy ? 'Mixture stabilized.' : balanced ? `Hold steady for ${c.hold_ms / 1000} seconds…` : `Match both targets within ±${c.tolerance}%.`}</p>
  </div>;
}

export function Transfer({ challenge: c, complete }: Props) {
  const [cursor, setCursor] = useState(50); const cursorRef = useRef(cursor); cursorRef.current = cursor;
  const [target, setTarget] = useState(50); const [held, setHeld] = useState(0);
  const { submit, busy } = useSubmission(complete);
  useEffect(() => {
    let frame = 0; let progress = 0; let previous = performance.now(); const start = previous;
    const tick = (now: number) => {
      const position = 50 + Math.sin((now - start) / 1100) * 32 + Math.sin((now - start) / 470) * 4;
      const delta = Math.min(100, now - previous); previous = now; setTarget(position);
      if (!document.hidden && Math.abs(cursorRef.current - position) <= c.tolerance) progress += delta;
      setHeld(Math.min(progress, c.required_ms));
      if (progress >= c.required_ms) { submit({ progress: 100, held_ms: c.required_ms }); return; }
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick); return () => cancelAnimationFrame(frame);
  }, [c.required_ms, c.tolerance, submit]);
  const inRange = Math.abs(cursor - target) <= c.tolerance;
  return <div className="extra-puzzle"><p className="instruction">Follow the moving receiver. Transfer advances only while your cursor is inside.</p>
    <div className={`instrument transfer-panel ${inRange ? 'is-locked' : ''}`}><span className="instrument-label">UPLINK / LIVE TRACKING</span><div className="packet-stream" aria-hidden="true">{Array.from({ length: 7 }, (_, i) => <i key={i} style={{ animationDelay: `${i * .25}s` }}/>)}</div>
      <div className="transfer-track" aria-hidden="true"><span className="receiver" style={{ left: `${Math.max(0, target - c.tolerance)}%`, width: `${Math.min(100, target + c.tolerance) - Math.max(0, target - c.tolerance)}%` }}/><i className="transfer-cursor" style={{ left: `${cursor}%` }}/></div>
      <strong className="transfer-percent">{Math.floor(held / c.required_ms * 100)}<small>%</small></strong><span>{inRange ? 'TRANSMITTING' : 'ALIGN CURSOR WITH RECEIVER'}</span>
    </div><label className="tuner-label" htmlFor="transfer-cursor">Your cursor · {cursor}%</label><input id="transfer-cursor" type="range" className="signal-slider" min="0" max="100" value={cursor} disabled={busy} onChange={e => setCursor(Number(e.target.value))} aria-describedby="transfer-help"/>
    <div className="range-labels"><span>← Left</span><span>Right →</span></div><Meter value={held / c.required_ms * 100} label="Data transferred"/><p id="transfer-help" className="puzzle-status">{(held / 1000).toFixed(1)} / {(c.required_ms / 1000).toFixed(1)} seconds in range · receiver {Math.round(target)}%</p>
  </div>;
}

export function Fault({ challenge: c, complete }: Props) {
  const [time, setTime] = useState(0); const [message, setMessage] = useState('Watch several cycles, then select the irregular light.');
  const { submit, busy } = useSubmission(complete);
  useEffect(() => { const start = performance.now(); const timer = setInterval(() => setTime(performance.now() - start), 80); return () => clearInterval(timer); }, []);
  return <div className="extra-puzzle"><p className="instruction">One relay has an uneven heartbeat.</p><div className="instrument"><span className="instrument-label">RELAY DIAGNOSTICS</span><div className="fault-grid">{Array.from({ length: c.lights }, (_, i) => {
    const on = i === c.fault_index ? time % 3600 < 420 || (time % 3600 >= 1050 && time % 3600 < 1700) || (time % 3600 >= 2650 && time % 3600 < 2950) : time % 1800 < 900;
    return <button key={i} className={`relay ${on ? 'lit' : ''}`} disabled={busy} aria-label={`Relay ${i + 1}, ${on ? 'light on' : 'light off'}. Select as faulty relay.`} onClick={() => i === c.fault_index ? submit({ index: i }) : setMessage(`Relay ${i + 1} is regular. Watch the other lights.`)}><i aria-hidden="true"/><strong>0{i + 1}</strong><span>RELAY</span></button>;
  })}</div></div><p className="puzzle-status" role="status">{message}</p></div>;
}

export function Frequency({ challenge: c, complete }: Props) {
  const [value, setValue] = useState(10); const { submit, busy } = useSubmission(complete);
  const distance = Math.abs(value - c.target); const locked = distance <= c.tolerance;
  const held = useHold(locked, c.hold_ms, value, () => submit({ value, held_ms: c.hold_ms }));
  const blinkMs = 600 + distance * 38;
  return <div className="extra-puzzle"><p className="instruction">Tune by the beacon: faster blinking means closer. Hold when the signal locks.</p>
    <div className={`instrument hunt-panel ${locked ? 'is-locked' : ''}`}><span className="instrument-label">DEEP SPACE / FREQUENCY SCAN</span><div className="radar" aria-hidden="true"><i/><i/><i/><span className="beacon" style={{ animationDuration: `${blinkMs}ms` }}/><svg viewBox="0 0 160 160"><path d="M80 80L140 30M80 15v130M15 80h130" stroke="#64edff" strokeWidth="1" fill="none"/></svg></div><strong>{value}<small> MHz</small></strong><span>{locked ? 'SIGNAL LOCKED · HOLD STEADY' : distance < 10 ? 'STRONG SIGNAL' : distance < 25 ? 'SIGNAL DETECTED' : 'FAINT SIGNAL'}</span></div>
    <label className="tuner-label" htmlFor="frequency-tuner">Receiver frequency</label><input id="frequency-tuner" className="signal-slider" type="range" min="0" max="100" step="1" value={value} disabled={busy} onChange={e => setValue(Number(e.target.value))}/><div className="range-labels"><span>0 MHz</span><span>100 MHz</span></div><Meter value={held / c.hold_ms * 100} label="Frequency lock"/><p className="puzzle-status" role="status">{locked ? `Keep steady for ${c.hold_ms / 1000} seconds.` : 'Search in either direction. The target frequency is hidden.'}</p>
  </div>;
}

function scoreGuess(guess: string, code: string) {
  let exact = 0; let misplaced = 0; const remaining = code.split('');
  guess.split('').forEach((digit, i) => { if (digit === code[i]) { exact++; remaining[i] = ''; } });
  guess.split('').forEach((digit, i) => { if (digit === code[i]) return; const index = remaining.indexOf(digit); if (index >= 0) { misplaced++; remaining[index] = ''; } });
  return { exact, misplaced };
}

export function Breaker({ challenge: c, complete }: Props) {
  const [answer, setAnswer] = useState(''); const [guesses, setGuesses] = useState<{ answer: string; exact: number; misplaced: number }[]>([]);
  const { submit, busy } = useSubmission(complete);
  const guess = () => { if (answer.length !== 3 || busy) return; const score = scoreGuess(answer, c.code); setGuesses(old => [...old, { answer, ...score }]); if (score.exact === 3) submit({ answer }); else setAnswer(''); };
  return <div className="extra-puzzle"><p className="instruction">Crack the three-digit lock. Digits may repeat.</p><div className="instrument lock-panel"><span className="instrument-label">SECURE VAULT / 3 DIGIT CIPHER</span><svg viewBox="0 0 100 70" aria-hidden="true"><path d="M30 30V23a20 20 0 0 1 40 0v7" fill="none" stroke="#91b9d0" strokeWidth="7"/><rect x="20" y="30" width="60" height="36" rx="8" fill="#00c8df"/><circle cx="50" cy="46" r="5" fill="#08243a"/><path d="M50 46v10" stroke="#08243a" strokeWidth="4"/></svg><div className="code-entry" aria-label={`Entered code ${answer || 'empty'}`}>{Array.from({ length: 3 }, (_, i) => <span key={i} className={answer[i] ? 'filled' : ''}>{answer[i] || '·'}</span>)}</div></div>
    <div className="number-pad">{['1','2','3','4','5','6','7','8','9','back','0','go'].map(key => <button key={key} className={key === 'go' ? 'submit-key' : ''} disabled={busy || (key === 'go' && answer.length !== 3)} aria-label={key === 'back' ? 'Delete digit' : key === 'go' ? 'Test code' : `Digit ${key}`} onClick={() => key === 'back' ? setAnswer(answer.slice(0, -1)) : key === 'go' ? guess() : answer.length < 3 && setAnswer(answer + key)}>{key === 'back' ? '⌫' : key === 'go' ? '↵' : key}</button>)}</div>
    <div className="guess-history" aria-label="Previous guesses"><p className="puzzle-hint">Exact = correct digit and place. Misplaced = correct digit, wrong place.</p>{guesses.length === 0 ? <p className="puzzle-status">Enter any three digits to begin.</p> : <><p className="puzzle-status" role="status">{guesses.at(-1)!.answer}: {guesses.at(-1)!.exact} exact, {guesses.at(-1)!.misplaced} misplaced</p><ol>{guesses.map((g, i) => <li key={i}><strong>{g.answer}</strong><span>{g.exact} exact</span><span>{g.misplaced} misplaced</span></li>)}</ol></>}</div>
  </div>;
}

export function Timing({ challenge: c, complete }: Props) {
  const [position, setPosition] = useState(0); const [hits, setHits] = useState<number[]>([]); const [message, setMessage] = useState('Catch the white marker in the narrow cyan window.');
  const start = useRef(performance.now()); const capturedHits = useRef<number[]>([]); const { submit, busy } = useSubmission(complete);
  useEffect(() => { let frame: number; const tick = () => { setPosition(batteryPosition(performance.now() - start.current)); frame = requestAnimationFrame(tick); }; frame = requestAnimationFrame(tick); return () => cancelAnimationFrame(frame); }, []);
  const capture = () => {
    if (busy || capturedHits.current.length >= c.required_hits) return;
    const now = performance.now(); const hit = batteryPosition(now - start.current);
    if (hit < c.zone_start || hit > c.zone_end) { setMessage('Missed. Wait for the next pass.'); return; }
    const next = [...capturedHits.current, hit]; capturedHits.current = next; setHits(next); start.current = now; setPosition(0);
    setMessage(`Hit ${next.length} of ${c.required_hits}. ${next.length < c.required_hits ? 'Clock reset. Catch the next pass.' : 'Timing synchronized.'}`);
    if (next.length === c.required_hits) submit({ hits: next });
  };
  return <div className="extra-puzzle"><div className="instrument timing-panel"><span className="instrument-label">PRECISION SYNCHRONIZER</span><svg viewBox="0 0 200 130" aria-hidden="true"><path d="M25 100a75 75 0 0 1 150 0" fill="none" stroke="#29475b" strokeWidth="12"/><path d="M25 100a75 75 0 0 1 150 0" fill="none" stroke="#00d9ed" strokeWidth="12" strokeDasharray={`${Math.PI * 75 * (c.zone_end - c.zone_start) / 100} 300`} strokeDashoffset={-Math.PI * 75 * c.zone_start / 100}/><path d="M100 100V35" stroke="#fff" strokeWidth="3" transform={`rotate(${position * 1.8 - 90} 100 100)`}/><circle cx="100" cy="100" r="8" fill="#00d9ed"/></svg><div className="hit-lamps">{Array.from({ length: c.required_hits }, (_, i) => <i key={i} className={hits.length > i ? 'lit' : ''}/>)}</div></div>
    <p className="instruction">Stop inside the cyan zone {c.required_hits} times.</p><div className="timing-track" aria-hidden="true"><div className="target-zone" style={{ left: `${c.zone_start}%`, width: `${c.zone_end - c.zone_start}%` }}/><i style={{ left: `${position}%` }}/></div><button className="button dark wide" disabled={busy} onClick={capture} aria-label="Capture timing marker">Capture pulse</button><p className="puzzle-status" role="status">{message}</p>
  </div>;
}

export function Switches({ challenge: c, complete }: Props) {
  const order: number[] = c.order; const [sequence, setSequence] = useState<number[]>([]); const [message, setMessage] = useState('Activate switches in the order shown.'); const { submit, busy } = useSubmission(complete);
  const toggle = (index: number) => {
    if (busy) return;
    if (index !== order[sequence.length]) { setSequence([]); setMessage('Wrong order. All switches reset; start at step 1.'); return; }
    const next = [...sequence, index]; setSequence(next); setMessage(`Step ${next.length} of ${order.length} complete.`);
    if (next.length === order.length) submit({ sequence: next });
  };
  return <div className="extra-puzzle"><p className="instruction">Follow the activation order from left to right.</p><ol className="switch-order" aria-label="Switch activation order">{order.map((n, i) => <li key={i} className={sequence.length > i ? 'done' : ''}><small>STEP {i + 1}</small><strong>{n + 1}</strong></li>)}</ol><div className="instrument"><span className="instrument-label">POWER SEQUENCER</span><div className="switch-bank">{Array.from({ length: Math.max(...order) + 1 }, (_, i) => <button key={i} role="switch" aria-checked={sequence.includes(i)} aria-label={`Switch ${i + 1}`} disabled={busy || sequence.includes(i)} onClick={() => toggle(i)} className={sequence.includes(i) ? 'on' : ''}><span className="switch-led"/><span className="switch-lever"><i/></span><strong>{i + 1}</strong><small>{sequence.includes(i) ? 'ON' : 'OFF'}</small></button>)}</div></div><p className="puzzle-status" role="status">{message}</p></div>;
}

export function Cargo({ challenge: c, complete }: Props) {
  const items: { label: string; category: number }[] = c.items; const categories: string[] = c.categories;
  const [choices, setChoices] = useState<number[]>([]); const [message, setMessage] = useState('Swipe the crate toward its bay, or use the buttons.'); const [drag, setDrag] = useState(0);
  const gesture = useRef<{ id: number; x: number; y: number } | null>(null); const { submit, busy } = useSubmission(complete); const item = items[choices.length];
  const sort = (category: number) => {
    if (!item || busy) return;
    if (category !== item.category) { setMessage(`${item.label} does not belong in ${categories[category]}. Try the other bay.`); return; }
    const next = [...choices, category]; setChoices(next); setMessage(`${item.label} sorted. ${next.length} of ${items.length} loaded.`);
    if (next.length === items.length) submit({ choices: next });
  };
  return <div className="extra-puzzle"><div className="cargo-bays" aria-hidden="true"><span>← {categories[0]}</span><span>{categories[1]} →</span></div><div className="instrument cargo-panel"><span className="instrument-label">CARGO MANIFEST / {Math.min(choices.length + 1, items.length)} OF {items.length}</span>
    <div className="cargo-crate" style={{ transform: `translateX(${drag}px) rotate(${drag / 12}deg)` }} onPointerDown={e => { if (!item || busy || !e.isPrimary || e.button !== 0) return; gesture.current = { id: e.pointerId, x: e.clientX, y: e.clientY }; e.currentTarget.setPointerCapture(e.pointerId); }} onPointerMove={e => { const g = gesture.current; if (g?.id === e.pointerId) setDrag(Math.max(-70, Math.min(70, e.clientX - g.x))); }} onPointerUp={e => { const g = gesture.current; gesture.current = null; setDrag(0); if (g?.id === e.pointerId && Math.abs(e.clientX - g.x) >= 40 && Math.abs(e.clientX - g.x) > Math.abs(e.clientY - g.y)) sort(e.clientX < g.x ? 0 : 1); }} onPointerCancel={() => { gesture.current = null; setDrag(0); }} onLostPointerCapture={() => { gesture.current = null; setDrag(0); }}>
      <svg viewBox="0 0 200 160" aria-hidden="true"><path d="M25 40L100 10l75 30v85l-75 27-75-27Z" fill="#b68046" stroke="#f6c887" strokeWidth="3"/><path d="M25 40l75 29 75-29M100 69v83M65 25l75 29v45l-20 8V62L45 33" fill="#e6b574" stroke="#754e30" strokeWidth="2"/><path d="M43 94l33 12m-33 3l24 9" stroke="#573c2c" strokeWidth="4"/></svg><strong>{item?.label || 'All cargo loaded'}</strong><small>{item ? '← SWIPE TO SORT →' : 'MANIFEST COMPLETE'}</small>
    </div></div><div className="cargo-controls">{categories.map((category, i) => <button key={i} className="button outline" disabled={busy || !item} onClick={() => sort(i)} aria-label={`Sort ${item?.label || 'cargo'} into ${category}`}>{i === 0 ? '← ' : ''}{category}{i === 1 ? ' →' : ''}</button>)}</div><Meter value={choices.length / items.length * 100} label="Cargo sorted"/><p className="puzzle-status" role="status">{message}</p>
  </div>;
}

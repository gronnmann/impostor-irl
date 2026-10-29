import { useEffect, useRef, useState } from 'react';
import jsQR from 'jsqr';
import QRCode from 'qrcode';
import { Camera, CameraOff, Check, ScanLine, X } from 'lucide-react';
import { parseQrTarget } from './puzzles';

export function QrImage({ value, size = 180, downloadable = false }: { value: string; size?: number; downloadable?: boolean }) {
  const [src, setSrc] = useState('');
  useEffect(() => { let active = true; QRCode.toDataURL(value, { width: size * 2, margin: 2, color: { dark: '#0B132B', light: '#ffffff' } }).then(url => active && setSrc(url)); return () => { active = false; }; }, [value, size]);
  return src ? <><img className="qr-image" src={src} width={size} height={size} alt={`QR code: ${value}`}/>{downloadable && <a className="button outline small no-print qr-download" href={src} download={`${value.replace(/[^a-zA-Z0-9-]/g,'-').slice(0,90)}.png`}>Download PNG</a>}</> : <div style={{ width: size, height: size }}/>;
}
export function Scanner({ onScan, onClose, expected }: { onScan: (target: { type: 'player' | 'station'; id: string }) => void; onClose: () => void; expected?: 'player' | 'station' }) {
  const video = useRef<HTMLVideoElement>(null); const [message, setMessage] = useState('Starting camera…'); const [manual, setManual] = useState(''); const [error, setError] = useState(''); const scanned = useRef(false); const onScanRef = useRef(onScan); onScanRef.current = onScan;
  const accept = (text: string) => { const target = parseQrTarget(text); if (!target) { setError('Use an Impostor IRL player or station code.'); return; } if (expected && target.type !== expected) { setError(`Scan a ${expected} code for this action.`); return; } if (!scanned.current) { scanned.current = true; onScanRef.current(target); } };
  const acceptRef = useRef(accept); acceptRef.current = accept;
  useEffect(() => {
    let stopped = false; let stream: MediaStream | undefined; let frame: number;
    const canvas = document.createElement('canvas'); const context = canvas.getContext('2d', { willReadFrequently: true });
    const scan = () => { if (stopped || scanned.current) return; const element = video.current; if (element && element.readyState >= 2 && context) { canvas.width = Math.min(640, element.videoWidth); canvas.height = element.videoHeight * canvas.width / element.videoWidth; context.drawImage(element, 0, 0, canvas.width, canvas.height); const data = context.getImageData(0, 0, canvas.width, canvas.height); const code = jsQR(data.data, data.width, data.height, { inversionAttempts: 'dontInvert' }); if (code) acceptRef.current(code.data); } frame = requestAnimationFrame(scan); };
    (async () => { try { if (!navigator.mediaDevices?.getUserMedia) throw new Error('Camera access needs HTTPS or localhost. You can enter the code below.'); stream = await navigator.mediaDevices.getUserMedia({ video: { facingMode: 'environment' }, audio: false }); if (stopped) { stream.getTracks().forEach(t => t.stop()); return; } if (video.current) { video.current.srcObject = stream; await video.current.play(); setMessage('Point your camera at an Impostor IRL code.'); scan(); } } catch (e) { if (!stopped) setMessage((e as Error).message || 'Camera unavailable. Enter the code below.'); } })();
    return () => { stopped = true; cancelAnimationFrame(frame); stream?.getTracks().forEach(t => t.stop()); };
  }, []);
  return <div className="modal-backdrop"><section className="small-modal" role="dialog" aria-modal="true" aria-label="Scan QR code"><header><span className="eyebrow">FIELD SCANNER</span><button className="icon-button" onClick={onClose} aria-label="Close scanner"><X size={20}/></button></header><h2>Identify your {expected || 'target'}.</h2><div className="camera-view"><video ref={video} muted playsInline/><div className="scan-reticle"><ScanLine size={60}/></div></div><p className="muted">{message}</p><form onSubmit={e => { e.preventDefault(); accept(manual); }}><label className="field">Or enter the printed code<input value={manual} onChange={e => setManual(e.target.value)} placeholder={`impostor:${expected || 'station'}:code`} autoCapitalize="none"/></label><button className="button dark wide" type="submit">Use code <Check size={16}/></button></form>{error && <p className="inline-error">{error}</p>}</section></div>;
}

import { createRoot } from 'react-dom/client';
import App from './App';
import './style.css';
import './game.css';
createRoot(document.getElementById('root')!).render(<App />);
if (import.meta.env?.PROD && 'serviceWorker' in navigator) navigator.serviceWorker.register('/sw.js').catch(() => {});

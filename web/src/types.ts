export type Kind = 'wires' | 'code' | 'signal' | 'battery' | 'memory' | 'anomaly' | 'circuits' | 'password' | 'mixing' | 'transfer' | 'fault' | 'frequency' | 'breaker' | 'timing' | 'route_power' | 'switches' | 'cargo';
export type Mode = 'qr' | 'proximity' | 'either';
export interface Settings { activities_per_player: number | null; player_count: number | null; impostors: number; kill_cooldown_secs: number; emergency_cooldown_secs: number; meeting_duration_secs: number; kill_mode: Mode; task_mode: Mode }
export interface Player { id: string; name: string; color: string; connected: boolean; alive?: boolean; role?: 'crew' | 'impostor' }
export interface Station { id: string; name: string; kind: Kind; description: string; duration: string; location?: string }
export interface Task { id: string; player_id: string; station_id: string; kind: Kind; completed: boolean }
export interface ActivitySession { id: string; task_id: string; station_id?: string; kind: Kind; challenge: Record<string, any> }
export interface Feedback { id: string; at: number; target: string; kind: string; message: string }
export interface Snapshot {
  game_id: string; room_code: string; phase: 'lobby' | 'playing' | 'meeting' | 'ended'; now: number; settings: Settings;
  players: Player[]; me: null | { id: string; name: string; role: 'crew' | 'impostor' | null; alive: boolean; waiting_for_meeting: boolean; kill_ready_at: number };
  stations: Station[]; available_activities?: Station[]; tasks: Task[]; active_task: ActivitySession | null;
  meeting: null | { reason: 'emergency' | 'body'; victim_id?: string; ends_at: number; voted_player_ids: string[]; votes?: Record<string, string | null> };
  last_meeting: null | { ejected_id: string | null; skipped: boolean; tally: Record<string, number> };
  winner: null | 'crew' | 'impostor'; win_reason: string | null; task_progress: { completed: number; total: number };
  emergency_ready_at: number; presence: { first: string; second: string; near: boolean; expires_at: number }[];
  feedback: Feedback[]; events: Feedback[]; demo: boolean;
}
export interface CommandResult { ok: boolean; error?: string; token?: string; player_id?: string; state: Snapshot }

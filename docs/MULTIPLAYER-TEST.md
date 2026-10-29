# Multiplayer playthrough — 2026-10-29

Played through the production frontend against an isolated Rust server on port 3137. Five simultaneous, non-demo player tabs joined as Ada, Bo, Cleo, Dax and Eli. Separate host and emergency-station tabs remained open. Each player kept a distinct session token and used the actual join, scanner/manual-code, puzzle, report and vote controls. Host snapshots were inspected to verify authoritative results.

The normal preview room on port 3001 was not reset by these playthroughs.

## Completed rounds

| Round | Actions | Observed result |
| --- | --- | --- |
| 1: one impostor, four crew | Emergency interrupts an active wires puzzle; all skip. Impostor kills Ada, Bo reports her body, living players skip. Ada completes a ghost task. Impostor kills Bo and Cleo. | Game continues at one impostor/two crew; ends immediately at one impostor/one crew. All five tabs display IMPOSTORS WIN. Ghost task counted after body cleared. |
| 2: one impostor, four crew | Reset through the host UI, start again, call emergency, all players vote for the impostor. | Impostor is ejected; CREW WINS, reason All impostors were eliminated. |
| 3: one impostor, four crew | One anomaly placement, one assignment per crewmate. Three-second emergency cooldown, five-second meeting timeout. Impostor completes a fake task, then each crewmate completes the actual puzzle. | Emergency starts disabled, unlocks, times out without votes and resumes play with no ejection. Fake task contributes zero progress. Crew work advances 1/4, 2/4, 3/4, 4/4, then CREW WINS, with all crew alive. |
| 4: two impostors, three crew | Kill cooldown initially disables kill. First meeting votes split 2/2/1 skip. Second meeting ejects a crewmate. | Tie ejects nobody; cooldown unlocks. Ejecting crew leaves two impostors/two crew, immediately producing IMPOSTORS WIN. |

## Cross-session checks

- All five tabs receive the same meeting and result transitions. Ordinary players have no host navigation or demo character selector.
- Starting emergency closes an active puzzle; the shared emergency button disables during a meeting.
- Enabling the horn starts its test sound. A subsequent meeting invokes another oscillator. After disabling it, later body-report/emergency events invoke none. This verifies browser audio invocation, not physical speaker volume.
- A victim sees Ghost. Other players' pre-report snapshots omit the victim's alive/dead flag and role. Reporting reveals the meeting roster; the victim is absent from vote targets and has no voting controls.
- Skip votes resume play. Duplicate votes are rejected with You have already voted.
- The server blocks ghost task starts before a meeting clears the body. Tasks work afterward and contribute to the crew total.
- A wrong station QR is rejected without opening a puzzle.
- Refreshing during a task restores the same player and the same task session ID.
- Removing the saved token and rejoining with a trimmed/case-varied name and correct room code recovers the same player ID, role and roster size during play.
- After game end, kill, task-start and emergency commands are rejected with Play is not active.

## Fix found during play

The server correctly blocked a newly killed player from starting tasks before a meeting, but the UI still offered enabled task buttons. Added a private `me.waiting_for_meeting` flag: the victim now sees disabled tasks and a wait message. After the body is cleared, tasks unlock and the instructions change to finishing tasks silently. This field is not exposed in the other-player roster. A Rust regression covers the private flag, clearing it during a meeting, and starting ghost work afterward.

## Limits

QR entry was exercised through the scanner's manual-code fallback; the image generator/decoder pair is covered by utility tests. Real phone cameras, physical touch feel, speaker volume, deployment through an actual tunnel, and future beacon hardware still need device/venue testing. This playthrough does not establish an unlimited player capacity or a load benchmark.

# Guandan Actor

Source: https://github.com/746505972/guandan/tree/7d37216369d8b003e94a2779c82f497a621f6f7b
Checkpoint: models/master.pth
License: MIT, see LICENSE-Guandan-Actor.

The model weights are downloaded separately. The Rust adapter uses the upstream
3049-input / 375-action ActorNet, preserving its absolute-seat feature layout.
The bundled actions.json vocabulary is derived from that version's doudizhu_actions.json.
The game's own rules validate and restrict every candidate action.

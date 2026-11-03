DouZero-WP pretrained models, from kwai/DouZero (Apache-2.0).
Original project: https://github.com/kwai/DouZero
Authors: DouZero team, AI Platform, Kwai Inc. and Texas A&M University.
Paper: https://proceedings.mlr.press/v139/zha21a.html
License: LICENSE-DouZero in this directory.

Checkpoint mirror: https://huggingface.co/palemoky/douzero-baselines
Pinned revision: 57b3914046c2a0877016b8b8830fd07cf5b0ba08
Source files: checkpoints/douzero_WP/{landlord,landlord_down,landlord_up}.ckpt
Hashes, exact byte sizes and conversion errors: manifest.json.

LeoCard modifications: checkpoints converted to little-endian IEEE f16 in
PyTorch state_dict order with the header DZERO01 followed by a null byte.
The runtime expands weights to f32 and implements the published architecture
using ndarray, without Python, PyTorch, ONNX Runtime, or a model download.
The original checkpoint tensor order and shapes follow the published DouZero
LSTM/MLP architecture. Conversion changes storage precision, not the architecture.
The .fixture files are CPU reference outputs from the rounded PyTorch model and
are included only by Rust tests, not the shipped executable.

Stored weights: 8,973,342 bytes (8.56 MiB) for all three roles.
Decoded parameters: 17.12 MiB if all three roles are used, loaded lazily.
The model applies only to three-player play. Bidding retains the existing
heuristic; four-player robots use sampled-deal rollouts in the four_player module.
Human auto-play and disconnected takeover never use the model or rollout search.
Only public history, public hand counts, and the acting robot's hand enter the
model. The merged unknown-card pool is computed from the deck minus seen cards;
the individual hands of other players are never supplied to inference.

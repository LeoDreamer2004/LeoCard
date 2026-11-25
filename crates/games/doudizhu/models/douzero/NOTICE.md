DouZero-WP pretrained models, from kwai/DouZero (Apache-2.0).
Original project: https://github.com/kwai/DouZero
Authors: DouZero team, AI Platform, Kwai Inc. and Texas A&M University.
Paper: https://proceedings.mlr.press/v139/zha21a.html
License: LICENSE-DouZero in this directory.

Checkpoint mirror: https://huggingface.co/palemoky/douzero-baselines
Pinned revision: 57b3914046c2a0877016b8b8830fd07cf5b0ba08
Source files: checkpoints/douzero_WP/{landlord,landlord_down,landlord_up}.ckpt
Hashes and exact source byte sizes: leocard-ai's model catalog.

LeoCard downloads checkpoints explicitly from the AI settings page, verifies the
pinned SHA-256, and converts them to the shared LCAI f16 tensor archive in Rust.
Weights are not embedded in the application. The LSTM runs on the CPU; dense
layers use the shared CPU/GPU backend. No Python, PyTorch or ONNX Runtime is needed.
Source URLs, sizes and hashes are maintained in leocard-ai's model catalog.
The .fixture files are CPU reference outputs from the rounded PyTorch model and
are included only by Rust tests, not the shipped executable.

Reference fixtures below use the original checkpoints rounded to f16.
The model applies only to three-player play. Bidding retains the existing
heuristic; four-player robots use sampled-deal rollouts in the four_player module.
Human auto-play and disconnected takeover never use the model or rollout search.
Only public history, public hand counts, and the acting robot's hand enter the
model. The merged unknown-card pool is computed from the deck minus seen cards;
the individual hands of other players are never supplied to inference.

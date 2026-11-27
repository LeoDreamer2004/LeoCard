#!/usr/bin/env python3
"""Download pinned public checkpoints and prepare independent PyTorch references.

Development only. The application downloads and converts weights in Rust.
Usage: python crates/games/ai/tools/prepare_reference.py /tmp/leocard-ai-models
"""
import concurrent.futures
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import sys
import urllib.request
import numpy as np
import torch
import torch.nn.functional as F


def download(item):
    model, file = item
    directory = output / model["key"]
    directory.mkdir(parents=True, exist_ok=True)
    path = directory / (file["name"] + ".source")
    if path.exists() and hashlib.sha256(path.read_bytes()).hexdigest() == file["sha256"]:
        return model, file, path
    with urllib.request.urlopen(file["url"], timeout=180) as response:
        data = response.read(file["bytes"] + 1)
    if len(data) != file["bytes"] or hashlib.sha256(data).hexdigest() != file["sha256"]:
        raise ValueError("checkpoint checksum failed: " + str(path))
    path.write_bytes(data)
    print("downloaded", model["key"], file["name"], flush=True)
    return model, file, path


def reference(model, file, path):
    torch.manual_seed(2014)
    if file["format"] == "Json":
        return
    if file["format"] == "Numpy":
        archive = np.load(path, allow_pickle=False)
        weights = {key: torch.from_numpy(archive[key].astype(np.float32)) for key in archive.files}
        inputs = torch.randint(0, 2, (1, 38, 4, 9)).float()
        x = F.relu(F.conv2d(inputs, weights["stem.weight"], weights["stem.bias"], padding=1))
        for i in range(40):
            prefix = "body.%d" % i
            y = F.relu(F.conv2d(x, weights[prefix + ".c1.weight"], weights[prefix + ".c1.bias"], padding=1))
            y = F.conv2d(y, weights[prefix + ".c2.weight"], weights[prefix + ".c2.bias"], padding=1)
            x = F.relu(x + y)
        x = F.relu(F.linear(x.flatten(1), weights["foot.1.weight"], weights["foot.1.bias"]))
        logits = F.linear(x, weights["foot.3.weight"], weights["foot.3.bias"])
    else:
        weights = torch.load(path, map_location="cpu", weights_only=True)
        if file.get("group"):
            weights = weights[file["group"]]
        weights = {key: value.half().float() for key, value in weights.items()}
        if model["id"] == "BlackjackTransformer":
            values = []
            for length in [2, 8, 31, 127]:
                tokens = torch.randint(1, 11, (length,))
                tokens[0], tokens[-1] = 11, 16
                positions = torch.arange(256 - length, 256)
                x = weights["token_embedding.weight"][tokens] + weights["position_embedding.weight"][positions]
                for index in range(4):
                    prefix = f"blocks.{index}"
                    y = F.layer_norm(x, (128,), weights[prefix + ".attention_norm.weight"], weights[prefix + ".attention_norm.bias"])
                    q, k, v = F.linear(y, weights[prefix + ".attention.query_key_value.weight"]).chunk(3, dim=-1)
                    q, k, v = [part.reshape(length, 4, 32).transpose(0, 1) for part in [q, k, v]]
                    scores = (q @ k.transpose(1, 2)) / (32 ** 0.5)
                    scores = scores.masked_fill(~torch.ones(length, length, dtype=torch.bool).tril(), -float("inf"))
                    y = (scores.softmax(-1) @ v).transpose(0, 1).reshape(length, 128)
                    x = x + F.linear(y, weights[prefix + ".attention.output.weight"])
                    y = F.layer_norm(x, (128,), weights[prefix + ".feed_forward_norm.weight"], weights[prefix + ".feed_forward_norm.bias"])
                    y = F.gelu(F.linear(y, weights[prefix + ".feed_forward.input.weight"], weights[prefix + ".feed_forward.input.bias"]))
                    x = x + F.linear(y, weights[prefix + ".feed_forward.output.weight"], weights[prefix + ".feed_forward.output.bias"])
                x = F.layer_norm(x, (128,), weights["final_norm.weight"], weights["final_norm.bias"])
                logits = F.linear(x, weights["language_model_head.weight"])
                values.extend([float(length)] + tokens.float().tolist() + logits.flatten().tolist())
            path.with_suffix(".fixture").write_bytes(struct.pack("<%df" % len(values), *values))
            print("reference", model["key"], file["name"], flush=True)
            return
        elif model["id"] == "TexasPpo":
            inputs = torch.rand(7, 84)
            inputs[:, 52:54] *= 100
            x = inputs
            for dense, norm in [("backbone.0", "backbone.1"), ("backbone.3", "backbone.4")]:
                x = F.linear(x, weights[dense + ".weight"], weights[dense + ".bias"])
                x = F.relu(F.layer_norm(x, (128,), weights[norm + ".weight"], weights[norm + ".bias"]))
            logits = F.linear(x, weights["head.weight"], weights["head.bias"])
        elif model["id"] == "Douzero":
            history = torch.randint(0, 2, (1, 5, 162)).float()
            lstm = torch.nn.LSTM(162, 128, batch_first=True)
            lstm.load_state_dict({key.removeprefix("lstm."): value for key, value in weights.items()
                                  if key.startswith("lstm.")})
            hidden = lstm(history)[0][:, -1, :]
            width = weights["dense1.weight"].shape[1] - 128
            rows = torch.randint(0, 2, (32, width)).float()
            x = torch.cat((hidden.expand(32, -1), rows), dim=1)
            for layer in range(1, 7):
                x = F.linear(x, weights[f"dense{layer}.weight"], weights[f"dense{layer}.bias"])
                if layer < 6:
                    x = F.relu(x)
            inputs = torch.cat((history.flatten(), rows.flatten()))
            logits = x
        else:
            inputs = (torch.rand(1, 3049) > 0.8).float()
            x = F.relu(F.linear(inputs, weights["net.0.weight"], weights["net.0.bias"]))
            x = F.relu(F.linear(x, weights["net.2.weight"], weights["net.2.bias"]))
            logits = F.linear(x, weights["net.4.weight"], weights["net.4.bias"])
    values = inputs.flatten().tolist() + logits.flatten().tolist()
    path.with_suffix(".fixture").write_bytes(struct.pack("<%df" % len(values), *values))
    print("reference", model["key"], file["name"], flush=True)


if __name__ == "__main__":
    output = Path(sys.argv[1])
    root = Path(__file__).resolve().parents[4]
    result = subprocess.run(["cargo", "run", "--offline", "--quiet", "-p", "leocard-ai",
                             "--example", "ai_convert", "--", "--catalog"],
                            cwd=root, check=True, capture_output=True, text=True)
    catalog = json.loads(result.stdout)
    torch.set_num_threads(1)
    with concurrent.futures.ThreadPoolExecutor(max_workers=3) as executor:
        downloads = list(executor.map(download, [(model, file) for model in catalog for file in model["files"]]))
    with torch.no_grad():
        for model, file, path in downloads:
            reference(model, file, path)

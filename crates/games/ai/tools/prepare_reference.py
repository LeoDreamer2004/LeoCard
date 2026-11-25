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
    if model["id"] == "Douzero":
        return  # Existing fixtures were generated from rounded PyTorch weights.
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
        weights = {key: value.half().float() for key, value in weights.items()}
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

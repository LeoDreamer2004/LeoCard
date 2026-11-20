"""Deterministic, content-addressed resource packs for the web client."""

import base64
import gzip
import hashlib
import json
import struct
from collections import defaultdict

PACK_LIMIT = 4 * 1024 * 1024


def write_compressed(output, stem, data):
    compressed = gzip.compress(data, compresslevel=6, mtime=0)
    digest = hashlib.sha256(compressed).digest()
    relative = f"{stem}-{digest.hex()}.pack"
    destination = output / relative
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(compressed)
    return {
        "path": relative,
        "size": len(compressed),
        "unpacked_size": len(data),
        "integrity": "sha256-" + base64.b64encode(digest).decode("ascii"),
    }


def asset_group(relative):
    parts = relative.parts
    if parts[0] == "fonts":
        return f"fonts-{relative.stem}"
    depth = 3 if parts[0] == "vendor" else 2
    return "-".join(parts[:min(depth, len(parts) - 1)])


def write_asset_packs(output, root, files):
    groups = defaultdict(list)
    for source in sorted(files):
        groups[asset_group(source.relative_to(root))].append(source)
    packs = []
    for group, sources in sorted(groups.items()):
        entries = []
        contents = bytearray()
        part = 0

        def flush():
            header = json.dumps(entries, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
            data = struct.pack("<I", len(header)) + header + contents
            pack = write_compressed(output, f"packs/{group}-{part}", data)
            packs.append({**pack, "asset_count": len(entries)})

        for source in sources:
            data = source.read_bytes()
            if entries and len(contents) + len(data) > PACK_LIMIT:
                flush()
                entries = []
                contents = bytearray()
                part += 1
            entries.append([source.relative_to(root).as_posix(), len(contents), len(data)])
            contents.extend(data)
        if entries:
            flush()
    return packs

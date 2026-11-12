#!/usr/bin/env python3
"""Download official Java panoramas and pack Q95 WebP cubemaps without resizing.

Requires Python 3.10+ and Pillow with WebP support: python3 -m pip install Pillow

Download and register a version (independent of the working directory):
    python3 panorama.py download 1.21
    python3 panorama.py download 1.21.4 --name 1.21 --force
Convert local six-face PNG strips and update the bundled version index:
    python3 panorama.py convert --remove-source
    python3 panorama.py convert path/to/cubemap.png --quality 95

Mojang faces are reordered to +X,-X,+Y,-Y,+Z,-Z and flipped vertically.
The original PNGs are removed only with --remove-source, after verified output.
Only outputs inside this script's directory update ../versions.tsv automatically.
"""

import argparse
import json
import re
import struct
import tempfile
import zipfile
import zlib
from hashlib import sha1
from io import BytesIO
from pathlib import Path
from urllib.request import Request, urlopen

from PIL import Image, features

DIRECTORY = Path(__file__).resolve().parent
MANIFEST_URL = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"
FACE_PATTERN = re.compile(r"(?:^|/)panorama_?([0-5])\.png$")
FACE_ORDER = (1, 3, 5, 4, 0, 2)
ARCHIVE_OVERRIDES = {"24w14potato", "25w14craftmine", "26w14a"}


def fetch(url, byte_range=None, checksum=None):
    headers = {"User-Agent": "LeoCard-panorama/1.0"}
    if byte_range is not None:
        start, end = byte_range
        headers["Range"] = f"bytes={start}-{end}"
    with urlopen(Request(url, headers=headers), timeout=60) as response:
        if byte_range is not None:
            expected = f"bytes {start}-{end}/"
            if response.status != 206 or not response.headers.get("Content-Range", "").startswith(expected):
                raise ValueError(f"Server did not honor the archive byte range: {url}")
        data = response.read()
    if byte_range is not None and len(data) != end - start + 1:
        raise ValueError(f"Incomplete archive byte range: {url}")
    if checksum is not None and sha1(data).hexdigest() != checksum:
        raise ValueError(f"SHA-1 mismatch: {url}")
    return data


class RemoteArchive:
    """Read only the ZIP directory and six faces, without downloading the full JAR."""

    def __init__(self, source):
        self.url = source["url"]
        size = source["size"]
        start = max(0, size - 65558)
        suffix = fetch(self.url, (start, size - 1))
        eocd = suffix.rfind(b"PK\x05\x06")
        if eocd < 0:
            raise ValueError("Client JAR has no ZIP directory")
        central_offset = struct.unpack_from("<4s4H2LH", suffix, eocd)[6]
        if central_offset < start:
            start = central_offset
            suffix = fetch(self.url, (start, size - 1))
        with zipfile.ZipFile(BytesIO(suffix)) as archive:
            self.faces = {
                int(match[1]): entry
                for entry in archive.infolist()
                if (match := FACE_PATTERN.search(entry.filename))
            }
        if set(self.faces) != set(range(6)):
            raise ValueError("This client JAR does not contain a six-face panorama")
        for entry in self.faces.values():
            entry.header_offset += start

    def read(self, index):
        entry = self.faces[index]
        offset = entry.header_offset
        header = fetch(self.url, (offset, offset + 29))
        if header[:4] != b"PK\x03\x04":
            raise ValueError("Invalid client JAR entry header")
        name_size, extra_size = struct.unpack_from("<HH", header, 26)
        offset += 30 + name_size + extra_size
        compressed = fetch(self.url, (offset, offset + entry.compress_size - 1))
        if entry.compress_type == zipfile.ZIP_DEFLATED:
            data = zlib.decompress(compressed, -15)
        elif entry.compress_type == zipfile.ZIP_STORED:
            data = compressed
        else:
            raise ValueError("Unsupported client JAR compression")
        if len(data) != entry.file_size or zlib.crc32(data) != entry.CRC:
            raise ValueError(f"Panorama CRC mismatch: {entry.filename}")
        return data


def download_faces(version):
    manifest = json.loads(fetch(MANIFEST_URL))
    entry = next((item for item in manifest["versions"] if item["id"] == version), None)
    if entry is None:
        raise ValueError(f"Unknown official Java version: {version}")
    metadata = json.loads(fetch(entry["url"], checksum=entry["sha1"]))
    faces = {}
    if "assetIndex" in metadata and version not in ARCHIVE_OVERRIDES:
        source = metadata["assetIndex"]
        index = json.loads(fetch(source["url"], checksum=source["sha1"]))
        faces = {
            int(match[1]): item
            for name, item in index["objects"].items()
            if (match := FACE_PATTERN.search(name))
        }
    if faces:
        if set(faces) != set(range(6)):
            raise ValueError(f"Incomplete asset-index panorama: {version}")
        result = []
        for index in range(6):
            item = faces[index]
            checksum = item["hash"]
            url = f"https://resources.download.minecraft.net/{checksum[:2]}/{checksum}"
            data = fetch(url, checksum=checksum)
            if len(data) != item["size"]:
                raise ValueError(f"Panorama size mismatch: {url}")
            result.append(data)
        return result
    archive = RemoteArchive(metadata["downloads"]["client"])
    return [archive.read(index) for index in range(6)]


def opaque_rgb(image):
    rgba = image.convert("RGBA")
    if rgba.getchannel("A").getextrema() != (255, 255):
        raise ValueError("Expected an opaque Minecraft panorama")
    return rgba.convert("RGB")


def pack_faces(faces):
    images = []
    for data in faces:
        with Image.open(BytesIO(data)) as image:
            if image.format != "PNG" or image.width != image.height:
                raise ValueError("Expected a square panorama PNG face")
            images.append(opaque_rgb(image))
    if len(images) != 6 or any(image.size != images[0].size for image in images):
        raise ValueError("Panorama requires six equally sized square faces")
    size = images[0].width
    cube = Image.new("RGB", (size, size * 6))
    for layer, face in enumerate(FACE_ORDER):
        cube.paste(images[face].transpose(Image.Transpose.FLIP_TOP_BOTTOM), (0, layer * size))
    return cube


def write_webp(image, destination, quality=95, force=False):
    if image.height != image.width * 6:
        raise ValueError("Expected a vertical strip of six square cubemap faces")
    if destination.exists() and not force:
        raise FileExistsError(f"Output already exists, use --force: {destination}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(dir=destination.parent, suffix=".tmp", delete=False) as temporary:
        pending = Path(temporary.name)
    try:
        image.save(pending, format="WEBP", quality=quality, method=4)
        with Image.open(pending) as decoded:
            decoded.load()
            if decoded.format != "WEBP" or decoded.size != image.size:
                raise ValueError("WebP verification failed")
        pending.replace(destination)
    finally:
        pending.unlink(missing_ok=True)
    print(f"{destination}: {image.width}x{image.height}, Q{quality}, "
          f"{destination.stat().st_size / 1024**2:.2f} MiB", flush=True)
    return destination


def update_index(outputs, replacements=None):
    outputs = [(version, output) for version, output in outputs if output.resolve().parent == DIRECTORY]
    if not outputs and not replacements:
        return
    index = DIRECTORY.parent / "versions.tsv"
    if not index.is_file():
        return
    lines = index.read_text(encoding="utf-8").splitlines()
    mapping = dict(line.split("\t", 1) for line in lines[1:] if line)
    if replacements:
        mapping = {version: replacements.get(path, path) for version, path in mapping.items()}
    for version, output in outputs:
        mapping[version] = f"ui/mc_guess/background/panoramas/{output.name}"
    index.write_text("version\tpanorama\n" + "".join(
        f"{version}\t{path}\n" for version, path in sorted(mapping.items())
    ), encoding="utf-8")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("action", choices=["download", "convert"])
    parser.add_argument("inputs", nargs="*", help="Java version for download, PNG strips for convert")
    parser.add_argument("--name", help="Downloaded output/version-index name, e.g. 1.21 for 1.21.4")
    parser.add_argument("--output-dir", type=Path, default=DIRECTORY)
    parser.add_argument("--quality", type=int, default=95)
    parser.add_argument("--force", action="store_true", help="Replace existing WebP outputs")
    parser.add_argument("--remove-source", action="store_true", help="Remove PNG strips after conversion")
    args = parser.parse_args(argv)
    if not 0 <= args.quality <= 100:
        parser.error("--quality must be between 0 and 100")
    if not features.check("webp"):
        parser.error("Pillow needs WebP support")
    if args.action == "download":
        if len(args.inputs) != 1 or args.remove_source:
            parser.error("download requires one Java version and does not use --remove-source")
        name = args.name or args.inputs[0]
        if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]*", name):
            parser.error("Invalid output version name")
        destination = args.output_dir / f"{name}.webp"
        if destination.exists() and not args.force:
            parser.error(f"Output already exists, use --force: {destination}")
        print(f"Downloading official Java {args.inputs[0]} panorama", flush=True)
        output = write_webp(pack_faces(download_faces(args.inputs[0])), destination, args.quality, args.force)
        update_index([(name, output)])
    else:
        if args.name:
            parser.error("--name is only used with download")
        paths = [Path(value) for value in args.inputs] or sorted(DIRECTORY.glob("*.png"))
        if not paths:
            parser.error("No PNG cubemaps to convert")
        outputs, replacements = [], {}
        for path in paths:
            if path.suffix.lower() != ".png":
                parser.error(f"Expected a PNG input: {path}")
            with Image.open(path) as source:
                image = opaque_rgb(source)
            output = write_webp(image, args.output_dir / f"{path.stem}.webp", args.quality, args.force)
            outputs.append((path.stem, output))
            if path.resolve().parent == DIRECTORY and output.resolve().parent == DIRECTORY:
                old = f"ui/mc_guess/background/panoramas/{path.name}"
                replacements[old] = f"ui/mc_guess/background/panoramas/{output.name}"
        update_index(outputs, replacements)
        if args.remove_source:
            for path in paths:
                path.unlink()


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, zipfile.BadZipFile, zlib.error) as error:
        raise SystemExit(f"Panorama failed: {error}") from error

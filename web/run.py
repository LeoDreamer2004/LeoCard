#!/usr/bin/env python3
"""Build and serve LeoCard, including all runtime assets, for browser testing."""

import argparse
import functools
import http.server
import json
import shutil
import subprocess
import sys
import tomllib
import webbrowser
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TARGET = "wasm32-unknown-unknown"


def run(*arguments):
    subprocess.run(arguments, cwd=ROOT, check=True)


def prepare_tools():
    targets = subprocess.check_output(["rustup", "target", "list", "--installed"], text=True)
    if TARGET not in targets.splitlines():
        run("rustup", "target", "add", TARGET)
    lock = tomllib.loads((ROOT / "Cargo.lock").read_text(encoding="utf-8"))
    version = next(p["version"] for p in lock["package"] if p["name"] == "wasm-bindgen")
    installed = shutil.which("wasm-bindgen")
    if not installed or subprocess.check_output([installed, "--version"], text=True).strip() != f"wasm-bindgen {version}":
        run("cargo", "install", "wasm-bindgen-cli", "--version", version, "--locked")


def runtime_assets():
    root = ROOT / "assets"
    entries = (ROOT / "crates/client/assets/runtime-assets.txt").read_text(encoding="utf-8").splitlines()
    files = set()
    excluded = []
    for raw in entries:
        entry = raw.strip()
        if not entry or entry.startswith("#"):
            continue
        if entry.startswith("!"):
            excluded.append(entry[1:])
            continue
        source = root / entry
        if not source.exists() or not source.resolve().is_relative_to(root.resolve()):
            raise ValueError(f"Invalid runtime asset: {entry}")
        files.update(path for path in source.rglob("*") if path.is_file()) if source.is_dir() else files.add(source)
    return sorted(
        path for path in files
        if not any(path.relative_to(root).as_posix() == rule or (rule.endswith("/") and path.relative_to(root).as_posix().startswith(rule)) for rule in excluded)
    )


def build(output, release):
    prepare_tools()
    profile = "release" if release else "web-dev"
    run("cargo", "build", "-p", "leocard-client", "--bin", "leocard", "--target", TARGET,
        "--no-default-features", "--features", "web", "--profile", profile, "--locked")
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1"], cwd=ROOT))
    binary = Path(metadata["target_directory"]) / TARGET / profile / "leocard.wasm"
    output.mkdir(parents=True, exist_ok=True)
    # Remove only generated subdirectories, leaving unrelated files untouched.
    for name in ("pkg", "assets"):
        if (output / name).exists():
            shutil.rmtree(output / name)
    run("wasm-bindgen", str(binary), "--target", "web", "--out-dir", str(output / "pkg"), "--out-name", "leocard")
    for name in ("index.html", "app.js", "style.css"):
        shutil.copy2(ROOT / "web" / name, output / name)
    shutil.copy2(ROOT / "assets/icons/app-icon.png", output / "icon.png")
    manifest = []
    for source in runtime_assets():
        relative = source.relative_to(ROOT / "assets")
        destination = output / "assets" / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, destination)
        manifest.append({"path": "assets/" + relative.as_posix(), "size": source.stat().st_size})
    (output / "assets-manifest.json").write_text(json.dumps(manifest, ensure_ascii=False), encoding="utf-8")
    print(f"Built {output} ({len(manifest)} assets, {sum(item['size'] for item in manifest) / 1048576:.1f} MiB)", flush=True)


class Handler(http.server.SimpleHTTPRequestHandler):
    extensions_map = {**http.server.SimpleHTTPRequestHandler.extensions_map, ".wasm": "application/wasm", ".js": "text/javascript", ".wgsl": "text/plain"}

    def log_request(self, code="-", size="-"):
        if str(code).startswith(("4", "5")):
            super().log_request(code, size)

    def end_headers(self):
        self.send_header("Cache-Control", "public, max-age=600" if "/assets/" in self.path else "no-cache")
        super().end_headers()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--release", action="store_true", help="build an optimized release")
    parser.add_argument("--build-only", action="store_true", help="only generate the static site")
    parser.add_argument("--skip-build", action="store_true", help="serve the previous build")
    parser.add_argument("--no-open", action="store_true", help="do not open a browser automatically")
    parser.add_argument("--host", default="127.0.0.1", help="use 0.0.0.0 to test from another device")
    parser.add_argument("--port", type=int, default=8080)
    parser.add_argument("--output", type=Path, default=ROOT / "output/web")
    args = parser.parse_args()
    output = args.output.resolve()
    if ROOT.is_relative_to(output) or output.is_relative_to(ROOT / "assets") or output.is_relative_to(ROOT / "web"):
        parser.error("Choose an output directory outside the source assets and web files")
    if not args.skip_build:
        build(output, args.release)
    if args.build_only:
        return
    if not (output / "index.html").is_file():
        parser.error("No previous build found, run without --skip-build first")
    handler = functools.partial(Handler, directory=output)
    server = http.server.ThreadingHTTPServer((args.host, args.port), handler)
    hostname = "127.0.0.1" if args.host == "0.0.0.0" else args.host
    url = f"http://{hostname}:{args.port}/"
    print(f"LeoCard: {url}\nWeb players join with a ws:// or wss:// room address. Ctrl+C stops the server.", flush=True)
    if not args.no_open:
        webbrowser.open(url)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()


if __name__ == "__main__":
    try:
        main()
    except (subprocess.CalledProcessError, OSError, ValueError) as error:
        print(f"Web build failed: {error}", file=sys.stderr)
        sys.exit(1)

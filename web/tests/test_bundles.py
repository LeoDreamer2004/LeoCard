import gzip
import json
import struct
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from bundles import write_asset_packs


class AssetPackTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name) / "assets"
        self.output = Path(self.temporary.name) / "output"

    def asset(self, name, data):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        return path

    def unpack(self, pack):
        data = gzip.decompress((self.output / pack["path"]).read_bytes())
        self.assertEqual(len(data), pack["unpacked_size"])
        length, = struct.unpack_from("<I", data)
        entries = json.loads(data[4:4 + length])
        start = 4 + length
        self.assertEqual(len(entries), pack["asset_count"])
        return {name: data[start + offset:start + offset + size] for name, offset, size in entries}

    def test_packs_preserve_paths_bytes_and_are_reproducible(self):
        files = [self.asset("cards/mahjong/一万.png", b"tile"), self.asset("cards/mahjong/empty.png", b"")]
        packs = write_asset_packs(self.output, self.root, files)
        self.assertEqual(packs, write_asset_packs(self.output, self.root, reversed(files)))
        self.assertEqual(self.unpack(packs[0]), {"cards/mahjong/一万.png": b"tile", "cards/mahjong/empty.png": b""})

    def test_changed_group_does_not_invalidate_other_groups(self):
        common = self.asset("ui/home/button.png", b"common")
        mahjong = self.asset("cards/mahjong/tile.png", b"old")
        before = write_asset_packs(self.output, self.root, [common, mahjong])
        mahjong.write_bytes(b"updated")
        after = write_asset_packs(self.output, self.root, [common, mahjong])
        self.assertNotEqual(before[0]["path"], after[0]["path"])
        self.assertEqual(before[1], after[1])

    def test_splits_large_groups_without_splitting_individual_files(self):
        files = [self.asset(f"audio/mahjong/{index}.ogg", data) for index, data in enumerate([b"aaa", b"bbb", b"oversized"])]
        with patch("bundles.PACK_LIMIT", 5):
            packs = write_asset_packs(self.output, self.root, files)
        self.assertEqual(len(packs), 3)
        unpacked = {name: data for pack in packs for name, data in self.unpack(pack).items()}
        self.assertEqual(unpacked, {path.relative_to(self.root).as_posix(): path.read_bytes() for path in files})

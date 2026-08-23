"""Extract the app-ready Gamjabot pack from the canonical user-provided v2 atlas."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path

from PIL import Image, ImageDraw


ROOT = Path(__file__).resolve().parents[1]
SOURCE = (
    ROOT
    / "images"
    / "characters"
    / "gamjabot"
    / "references"
    / "base-spritesheet-extended.png"
)
PACK = ROOT / "images" / "characters" / "gamjabot" / "pack"
SOURCE_SIZE = (1536, 2288)
CELL_SIZE = (192, 208)
CANVAS_SIZE = (256, 256)
CELL_OFFSET = (32, 24)

# App state -> (canonical hatch-pet row, frame count, frame ms, loop, impact frame)
ANIMATIONS = {
    "idle": (0, 7, 420, True, None),
    "walk": (1, 8, 110, True, None),
    "dragged": (6, 6, 160, True, None),
    "thrown": (4, 5, 100, True, None),
    "kick": (4, 5, 100, False, 2),
    "speak": (3, 4, 180, True, None),
    "chase": (2, 8, 110, True, None),
    "dance": (7, 6, 140, True, None),
    "carry-card": (8, 6, 220, True, None),
    "click-react": (5, 8, 120, False, None),
}


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_cell(atlas: Image.Image, row: int, column: int) -> Image.Image:
    left = column * CELL_SIZE[0]
    top = row * CELL_SIZE[1]
    return atlas.crop((left, top, left + CELL_SIZE[0], top + CELL_SIZE[1]))


def app_frame(cell: Image.Image) -> Image.Image:
    frame = Image.new("RGBA", CANVAS_SIZE, (0, 0, 0, 0))
    frame.alpha_composite(cell, CELL_OFFSET)
    return frame


def write_state(atlas: Image.Image, name: str, row: int, count: int) -> list[str]:
    folder = PACK / name
    folder.mkdir(parents=True, exist_ok=True)
    paths: list[str] = []
    expected: set[Path] = set()

    for column in range(count):
        cell = source_cell(atlas, row, column)
        if cell.getchannel("A").getbbox() is None:
            raise ValueError(f"source row {row} column {column} is empty")
        path = folder / f"{column}.png"
        app_frame(cell).save(path, optimize=True)
        expected.add(path)
        paths.append(f"{name}/{column}.png")

    # New frames exist before stale generated frames are removed.
    for stale in folder.glob("*.png"):
        if stale not in expected:
            stale.unlink()
    return paths


def create_contact_sheet(manifest: dict) -> None:
    states = list(manifest["animations"])
    sheet = Image.new("RGBA", (4 * 256, 3 * 288), (174, 178, 182, 255))
    draw = ImageDraw.Draw(sheet)
    for index, state in enumerate(states):
        x = (index % 4) * 256
        y = (index // 4) * 288
        with Image.open(PACK / manifest["animations"][state]["frames"][0]) as frame:
            sheet.alpha_composite(frame.convert("RGBA"), (x, y))
        draw.text((x + 8, y + 260), state, fill=(0, 0, 0, 255))
    sheet.save(PACK / "contact-sheet.png", optimize=True)


def validate(atlas: Image.Image, manifest: dict) -> dict:
    failures: list[str] = []
    checked = 0
    for state, spec in manifest["animations"].items():
        row = spec["sourceRow"]
        for column, relative in enumerate(spec["frames"]):
            checked += 1
            expected = source_cell(atlas, row, column)
            with Image.open(PACK / relative) as opened:
                frame = opened.convert("RGBA")
            if frame.size != CANVAS_SIZE:
                failures.append(f"{relative}: expected 256x256")
                continue
            actual = frame.crop(
                (
                    CELL_OFFSET[0],
                    CELL_OFFSET[1],
                    CELL_OFFSET[0] + CELL_SIZE[0],
                    CELL_OFFSET[1] + CELL_SIZE[1],
                )
            )
            if actual.tobytes() != expected.tobytes():
                failures.append(f"{relative}: source pixels changed")
            padding = frame.getchannel("A").copy()
            padding.paste(
                0,
                (
                    CELL_OFFSET[0],
                    CELL_OFFSET[1],
                    CELL_OFFSET[0] + CELL_SIZE[0],
                    CELL_OFFSET[1] + CELL_SIZE[1],
                ),
            )
            if padding.getbbox() is not None:
                failures.append(f"{relative}: non-transparent padding")

    return {
        "ok": not failures,
        "sourceAtlas": str(SOURCE.relative_to(ROOT)).replace("\\", "/"),
        "sourceSha256": sha256(SOURCE),
        "sourceCanvas": list(SOURCE_SIZE),
        "sourceCell": list(CELL_SIZE),
        "outputCanvas": list(CANVAS_SIZE),
        "cellOffset": list(CELL_OFFSET),
        "framesChecked": checked,
        "pixelExact": not failures,
        "failures": failures,
    }


def main() -> None:
    with Image.open(SOURCE) as opened:
        atlas = opened.convert("RGBA")
    if atlas.size != SOURCE_SIZE:
        raise ValueError(f"expected canonical v2 atlas {SOURCE_SIZE}, got {atlas.size}")

    PACK.mkdir(parents=True, exist_ok=True)
    animations = {}
    for name, (row, count, frame_ms, loop, impact) in ANIMATIONS.items():
        spec = {
            "frames": write_state(atlas, name, row, count),
            "frameMs": frame_ms,
            "loop": loop,
            "sourceRow": row,
        }
        if impact is not None:
            spec["impactFrame"] = impact
        animations[name] = spec

    manifest = {
        "schemaVersion": 1,
        "id": "gamjabot",
        "displayName": "Gamjabot",
        "canvas": {"width": 256, "height": 256},
        "sourceAtlas": {
            "path": str(SOURCE.relative_to(ROOT)).replace("\\", "/"),
            "sha256": sha256(SOURCE),
            "spriteVersionNumber": 2,
            "cellWidth": 192,
            "cellHeight": 208,
            "extraction": "pixel-exact 1:1; no redraw, resize, rotation, cleanup, or palette conversion",
        },
        "anchors": {
            "feet": {"x": 128, "y": 232},
            "speech": {"x": 128, "y": 18},
            "accessory": {"x": 128, "y": 92},
            "kickImpact": {"x": 232, "y": 196},
        },
        "hitbox": {"x": 32, "y": 24, "width": 192, "height": 208},
        "animations": animations,
        "accessories": [],
        "artRules": {
            "canonicalBase": "user-provided spritesheet-extended.png",
            "pixelFidelity": "Preserve every source RGBA pixel exactly",
            "accessoryStrategy": "Separate transparent 256x256 overlay layers",
        },
    }
    (PACK / "manifest.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    report = validate(atlas, manifest)
    (PACK / "validation.json").write_text(
        json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    create_contact_sheet(manifest)
    if not report["ok"]:
        raise SystemExit("; ".join(report["failures"]))
    print(json.dumps({"states": len(animations), **report}, ensure_ascii=False))


if __name__ == "__main__":
    main()

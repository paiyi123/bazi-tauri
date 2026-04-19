import json
import re
import shutil
from pathlib import Path

from PIL import Image


ROOT = Path(__file__).resolve().parents[1]
SOURCE_ICON = ROOT / "src-tauri" / "icons" / "icon.png"
ANDROID_RES = ROOT / "src-tauri" / "gen" / "android" / "app" / "src" / "main" / "res"
ANDROID_ASSETS = ROOT / "src-tauri" / "gen" / "android" / "app" / "src" / "main" / "assets"
PLAY_ICON = ROOT / "src-tauri" / "icons" / "android-play-512.png"
DIST_DIR = ROOT / "dist"
TAURI_CONF = ROOT / "src-tauri" / "tauri.conf.json"
TAURI_ANDROID_CONF = ROOT / "src-tauri" / "tauri.android.conf.json"
ANDROID_INLINE_HTML = ANDROID_ASSETS / "index.inline.html"

SCRIPT_PATTERN = re.compile(r'<script[^>]+src="(?P<src>/assets/[^"]+\.js)"[^>]*></script>')
STYLE_PATTERN = re.compile(r'<link[^>]+href="(?P<href>/assets/[^"]+\.css)"[^>]*>')
ICON_PATTERN = re.compile(r'\s*<link rel="icon"[^>]*>\s*', re.IGNORECASE)

LEGACY_SIZES = {
    "mipmap-mdpi": 48,
    "mipmap-hdpi": 72,
    "mipmap-xhdpi": 96,
    "mipmap-xxhdpi": 144,
    "mipmap-xxxhdpi": 192,
}

ADAPTIVE_FOREGROUND_SIZES = {
    "mipmap-mdpi": 108,
    "mipmap-hdpi": 162,
    "mipmap-xhdpi": 216,
    "mipmap-xxhdpi": 324,
    "mipmap-xxxhdpi": 432,
}


def resize_square(image: Image.Image, size: int) -> Image.Image:
    return image.resize((size, size), Image.Resampling.LANCZOS)


def build_foreground(image: Image.Image, size: int) -> Image.Image:
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    inset = int(size * 0.12)
    icon_size = size - (inset * 2)
    icon = resize_square(image, icon_size)
    canvas.paste(icon, (inset, inset), icon)
    return canvas


def ensure_parent(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)


def deep_merge(base: dict, override: dict) -> dict:
    result = dict(base)
    for key, value in override.items():
        if isinstance(value, dict) and isinstance(result.get(key), dict):
            result[key] = deep_merge(result[key], value)
        else:
            result[key] = value
    return result


def build_inline_html() -> str:
    index_path = DIST_DIR / "index.html"
    index_html = index_path.read_text(encoding="utf-8")

    script_match = SCRIPT_PATTERN.search(index_html)
    style_match = STYLE_PATTERN.search(index_html)
    if not script_match or not style_match:
        raise SystemExit(f"Unable to locate bundled JS/CSS references in {index_path}")

    js_path = DIST_DIR / script_match.group("src").lstrip("/")
    css_path = DIST_DIR / style_match.group("href").lstrip("/")
    js = js_path.read_text(encoding="utf-8")
    css = css_path.read_text(encoding="utf-8")

    inline_html = ICON_PATTERN.sub("\n", index_html, count=1)
    inline_html = STYLE_PATTERN.sub(lambda _: f"<style>\n{css}\n</style>", inline_html, count=1)
    inline_html = SCRIPT_PATTERN.sub(lambda _: f"<script type=\"module\">\n{js}\n</script>", inline_html, count=1)
    return inline_html


def sync_web_assets() -> None:
    if not DIST_DIR.exists():
        raise SystemExit(f"Build output not found: {DIST_DIR}")

    if ANDROID_ASSETS.exists():
        shutil.rmtree(ANDROID_ASSETS)
    ANDROID_ASSETS.mkdir(parents=True, exist_ok=True)

    for item in DIST_DIR.iterdir():
        target = ANDROID_ASSETS / item.name
        if item.is_dir():
            shutil.copytree(item, target)
        else:
            shutil.copy2(item, target)

    base_config = json.loads(TAURI_CONF.read_text(encoding="utf-8"))
    android_config = json.loads(TAURI_ANDROID_CONF.read_text(encoding="utf-8"))
    merged = deep_merge(base_config, android_config)
    build = merged.setdefault("build", {})
    build["devUrl"] = None
    build["frontendDist"] = "../dist"

    (ANDROID_ASSETS / "tauri.conf.json").write_text(
        json.dumps(merged, ensure_ascii=False, indent=2),
        encoding="utf-8",
    )
    ANDROID_INLINE_HTML.write_text(build_inline_html(), encoding="utf-8")


def main() -> None:
    if not SOURCE_ICON.exists():
        raise SystemExit(f"Source icon not found: {SOURCE_ICON}")

    source = Image.open(SOURCE_ICON).convert("RGBA")

    for folder, size in LEGACY_SIZES.items():
        legacy = resize_square(source, size)
        for filename in ("ic_launcher.png", "ic_launcher_round.png"):
            target = ANDROID_RES / folder / filename
            ensure_parent(target)
            legacy.save(target)

    for folder, size in ADAPTIVE_FOREGROUND_SIZES.items():
        foreground = build_foreground(source, size)
        target = ANDROID_RES / folder / "ic_launcher_foreground.png"
        ensure_parent(target)
        foreground.save(target)

    splash_logo = resize_square(source, 512)
    splash_target = ANDROID_RES / "drawable-nodpi" / "splash_logo.png"
    ensure_parent(splash_target)
    splash_logo.save(splash_target)

    play_icon = resize_square(source, 512)
    play_icon.save(PLAY_ICON)
    sync_web_assets()


if __name__ == "__main__":
    main()

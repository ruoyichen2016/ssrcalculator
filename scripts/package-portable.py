#!/usr/bin/env python3
"""Stage verified Windows x64 binaries, fonts and licenses into a portable ZIP."""
import argparse
import hashlib
import json
import shutil
import struct
import subprocess
import tempfile
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TARGET = "x86_64-pc-windows-msvc"
BINARIES = {"springsharp-gui.exe": ("SpringSharp.exe", 2),
            "springsharp-web.exe": ("SpringSharp-Web.exe", 2),
            "springsharp.exe": ("springsharp-cli.exe", 3)}

def verify_pe(path, subsystem):
    data = path.read_bytes()
    if data[:2] != b"MZ":
        raise ValueError(f"Not a Windows executable: {path}")
    pe = struct.unpack_from("<I", data, 0x3c)[0]
    if data[pe:pe+4] != b"PE\0\0" or struct.unpack_from("<H", data, pe+4)[0] != 0x8664:
        raise ValueError(f"Not Windows x64: {path}")
    if struct.unpack_from("<H", data, pe+24)[0] != 0x20b:
        raise ValueError(f"Not PE32+: {path}")
    if struct.unpack_from("<H", data, pe+24+68)[0] != subsystem:
        raise ValueError(f"Wrong console/window subsystem: {path}")

def license_inventory(destination):
    inventory = {}
    for manifest in [ROOT / "Cargo.toml", ROOT / "gui/Cargo.toml"]:
        raw = subprocess.check_output(["cargo", "metadata", "--locked", "--format-version", "1",
               "--filter-platform", TARGET, "--manifest-path", str(manifest)], cwd=ROOT)
        for package in json.loads(raw)["packages"]:
            key = f"{package['name']}-{package['version']}"
            if key in inventory:
                continue
            inventory[key] = {"license": package["license"], "source": package["source"]}
            source = Path(package["manifest_path"]).parent
            files = [p for p in source.iterdir() if p.is_file() and
                     p.name.upper().startswith(("LICENSE", "LICENCE", "COPYING", "NOTICE"))]
            if package.get("license_file"):
                files.append(source / package["license_file"])
            for file in set(files):
                if file.is_file():
                    folder = destination / "dependencies" / key
                    folder.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(file, folder / file.name)
    (destination / "dependencies.json").write_text(json.dumps(inventory, indent=2) + "\n", encoding="utf-8")

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binaries", type=Path, required=True)
    parser.add_argument("--output", type=Path, default=ROOT / "dist")
    parser.add_argument("--gui-only", action="store_true", help="Package only the standalone SpringSharp.exe (with embedded fonts)")
    args = parser.parse_args()
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1"], cwd=ROOT))
    version = next(p["version"] for p in metadata["packages"] if p["name"] == "springsharp")
    name = f"SpringSharp-{version}-windows-x64"
    active_binaries = {"springsharp-gui.exe": ("SpringSharp.exe", 2)} if args.gui_only else BINARIES
    for source, (_, subsystem) in active_binaries.items():
        verify_pe(args.binaries / source, subsystem)
    fonts = ROOT / "assets/fonts"
    for entry in json.loads((fonts / "sources.json").read_text(encoding="utf-8")):
        actual = hashlib.sha256((fonts / entry["file"]).read_bytes()).hexdigest()
        if actual != entry["sha256"]:
            raise ValueError(f"Font integrity check failed: {entry['file']}")
    args.output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="springsharp-package-") as temp:
        stage = Path(temp) / name
        stage.mkdir()
        for source, (dest, _) in active_binaries.items():
            shutil.copy2(args.binaries / source, stage / dest)
        if not args.gui_only:
            shutil.copytree(ROOT / "assets", stage / "assets")
        shutil.copytree(ROOT / "examples", stage / "examples")
        licenses = stage / "licenses"
        licenses.mkdir()
        for file in ["LICENSE", "NOTICE"]:
            shutil.copy2(ROOT / file, licenses / file)
        license_inventory(licenses)
        shutil.copy2(ROOT / "docs/WINDOWS-PORTABLE.md", stage / "README.txt")
        sums = []
        for file in sorted(stage.rglob("*")):
            if file.is_file():
                sums.append(f"{hashlib.sha256(file.read_bytes()).hexdigest()}  {file.relative_to(stage).as_posix()}")
        (stage / "SHA256SUMS.txt").write_text("\n".join(sums) + "\n", encoding="utf-8")
        archive = args.output / f"{name}.zip"
        with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED, compresslevel=9, strict_timestamps=False) as bundle:
            for file in sorted(stage.rglob("*")):
                if file.is_file():
                    bundle.write(file, file.relative_to(stage.parent))
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    archive.with_suffix(".zip.sha256").write_text(f"{digest}  {archive.name}\n", encoding="ascii")
    if args.gui_only:
        standalone_exe = args.output / "SpringSharp.exe"
        shutil.copy2(args.binaries / "springsharp-gui.exe", standalone_exe)
        exe_digest = hashlib.sha256(standalone_exe.read_bytes()).hexdigest()
        (args.output / "SpringSharp.exe.sha256").write_text(f"{exe_digest}  SpringSharp.exe\n", encoding="ascii")
        print(f"Standalone executable: {standalone_exe.resolve()}")
    print(archive.resolve())

if __name__ == "__main__":
    main()

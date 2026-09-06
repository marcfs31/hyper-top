#!/usr/bin/env python3

import argparse
import os
import platform
import shutil
import subprocess
import sys
import tarfile
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def detect_arch() -> str:
    machine = platform.machine().lower()
    mapping = {
        "x86_64": "x86_64",
        "amd64": "x86_64",
        "aarch64": "aarch64",
        "arm64": "aarch64",
    }
    return mapping.get(machine, "x86_64")


def binary_path(platform_name: str, target: str | None = None) -> Path:
    if target:
        return ROOT / "target" / target / "release" / ("hyper-top.exe" if platform_name == "windows" else "hyper-top")
    return ROOT / "target" / "release" / ("hyper-top.exe" if platform_name == "windows" else "hyper-top")


def copy_launcher(platform_name: str, stage: Path) -> None:
    if platform_name == "linux":
        src = ROOT / "dist" / "linux" / "hyper-top.desktop"
        shutil.copy2(src, stage / "hyper-top.desktop")
    elif platform_name == "macos":
        src = ROOT / "dist" / "macos" / "hyper-top.command"
        shutil.copy2(src, stage / "hyper-top.command")
        (stage / "hyper-top.command").chmod(0o755)
    elif platform_name == "windows":
        for name in ["hyper-top.cmd", "hyper-top.ps1"]:
            src = ROOT / "dist" / "windows" / name
            shutil.copy2(src, stage / name)


def create_archive(platform_name: str, root_name: str, stage: Path, out_dir: Path) -> Path:
    archive_name = f"{root_name}.{ 'zip' if platform_name == 'windows' else 'tar.gz'}"
    archive_path = out_dir / archive_name
    out_dir.mkdir(parents=True, exist_ok=True)

    if platform_name == "windows":
        with zipfile.ZipFile(archive_path, "w", compression=zipfile.ZIP_DEFLATED) as zf:
            for path in sorted(stage.rglob("*")):
                if path.is_file():
                    zf.write(path, path.relative_to(stage.parent))
    else:
        with tarfile.open(archive_path, "w:gz") as tar:
            for path in sorted(stage.rglob("*")):
                if path.is_file():
                    tar.add(path, arcname=path.relative_to(stage.parent))

    return archive_path


def build() -> int:
    parser = argparse.ArgumentParser(description="Package a hyper-top release bundle for one platform.")
    parser.add_argument("--platform", choices=["linux", "macos", "windows"], required=True)
    parser.add_argument("--output-dir", default="dist/releases")
    parser.add_argument("--target", default=None)
    args = parser.parse_args()

    out_dir = ROOT / args.output_dir
    out_dir.mkdir(parents=True, exist_ok=True)

    subprocess.run(["cargo", "build", "--release", *( ["--target", args.target] if args.target else [])], cwd=ROOT, check=True)

    bin_file = binary_path(args.platform, args.target)
    if not bin_file.exists():
        raise FileNotFoundError(f"Built binary not found at {bin_file}")

    stage = ROOT / ".release-stage" / args.platform
    if stage.exists():
        shutil.rmtree(stage)
    stage.mkdir(parents=True, exist_ok=True)

    shutil.copy2(bin_file, stage / ("hyper-top.exe" if args.platform == "windows" else "hyper-top"))
    for name in ["README.md", "LICENSE", "hyper-top.toml.example"]:
        src = ROOT / name
        if src.exists():
            shutil.copy2(src, stage / name)
    copy_launcher(args.platform, stage)

    if args.platform != "windows":
        executable = stage / ("hyper-top.exe" if args.platform == "windows" else "hyper-top")
        executable.chmod(0o755)

    arch = detect_arch()
    archive_root = f"hyper-top-{args.platform}-{arch}"
    archive_stage = ROOT / ".release-stage" / archive_root
    if archive_stage.exists():
        shutil.rmtree(archive_stage)
    shutil.copytree(stage, archive_stage)

    archive_path = create_archive(args.platform, archive_root, archive_stage, out_dir)
    print(f"Created {archive_path}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(build())
    except Exception as exc:  # pragma: no cover
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(1)

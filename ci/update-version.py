#!/usr/bin/env python3

import tomlkit
import glob
import sys
from pathlib import Path

def main() -> None:
    cwd = Path.cwd()
    next_ver = sys.argv[1]

    if next_ver.startswith('v'):
        next_ver = next_ver[1:]

    print(f"Running in {cwd}...")

    for cargo_toml_path in cwd.glob("crates/*/Cargo.toml"):
        cfg = tomlkit.loads(cargo_toml_path.open().read())
        print(f"Updating {cargo_toml_path} to version {next_ver}...")

        cfg["package"]["version"] = next_ver

        for (key, value) in cfg["dependencies"].items():
            if not isinstance(value, dict) or "path" not in value.keys():
                print(f"  Skipping {key}...")
                continue

            print(f"  Updating {key} to version {next_ver}...")
            value["version"] = next_ver

        with cargo_toml_path.open("w") as f:
            f.write(tomlkit.dumps(cfg))
            f.flush()

    for cargo_toml_path in cwd.glob("examples/**/Cargo.toml"):
        cfg = tomlkit.loads(cargo_toml_path.open().read())
        print(f"Updating example {cargo_toml_path} to version {next_ver}...")

        cfg["package"]["version"] = next_ver

        for (key, value) in cfg["dependencies"].items():
            if not isinstance(value, dict) or "path" not in value.keys():
                print(f"  Skipping {key}...")
                continue

            print(f"  Updating {key} to version {next_ver}...")
            value["version"] = next_ver

        with cargo_toml_path.open("w") as f:
            f.write(tomlkit.dumps(cfg))
            f.flush()

if __name__ == '__main__':
    main()

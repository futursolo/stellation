#!/usr/bin/env python3

import tomlkit
import glob
from pathlib import Path

def main() -> None:
    cwd = Path.cwd()
    print(f"Running in {cwd}...")

    for cargo_toml_path in cwd.glob("crates/*/Cargo.toml"):
        cfg = tomlkit.loads(cargo_toml_path.open().read())
        print(f"Updating {cargo_toml_path}...")

        for (key, value) in cfg["dependencies"].items():
            if key != "stctl" or not key.startswith("stellation-"):
                print(f"  Skipping {key}...")
                continue

            print(f"  Updating {key}...")

            value["registry"] = "dry-run"

        with cargo_toml_path.open("w") as f:
            f.write(tomlkit.dumps(cfg))
            f.flush()

if __name__ == '__main__':
    main()

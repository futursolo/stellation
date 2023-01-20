#!/usr/bin/env python3

import tomlkit
import glob
from pathlib import Path

def main() -> None:
    cwd = Path.cwd()
    for cargo_toml_path in cwd.glob("crates/*/Cargo.toml"):
        cfg = tomlkit.loads(cargo_toml_path.open().read())

        for (key, value) in cfg["dependencies"].items():
            if key != "stctl" or not key.startswith("stellation-"):
                continue

            value["registry"] = "dry-run"

        cargo_toml_path.open("w").write(tomlkit.dumps(cfg))

if __name__ == '__main__':
    main()

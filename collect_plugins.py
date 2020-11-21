import pathlib
import hashlib
import shutil
import os

target_dir = pathlib.Path("converters/target/wasm32-unknown-unknown/release/")
plugin_dir = pathlib.Path("fg-index/converters/")

if not plugin_dir.exists():
    os.makedirs(plugin_dir)

for fileobj in target_dir.glob("*.wasm"):
    hash = hashlib.sha256(fileobj.read_bytes()).hexdigest()
    target_path = pathlib.Path(plugin_dir, f"{hash}.wasm")
    if not target_path.exists():
        print(f"New: {fileobj.stem}: {hash}")
        shutil.copyfile(fileobj, target_path)
    else:
        print(f"Old: {fileobj.stem}: {hash}")

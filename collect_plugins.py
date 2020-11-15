import pathlib
import hashlib

target_dir = pathlib.Path("./target/wasm32-unknown-unknown/release/")

for fileobj in target_dir.glob("*.wasm"):
    hash = hashlib.sha256(fileobj.read_bytes()).hexdigest()
    print(f"{fileobj.stem}: {hash}")

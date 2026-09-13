import hashlib
import json
import uuid
import zlib
import base64
import os

# 1. Collect the core logic as the "Content"
files = [
    "src/identifiers/mod.rs",
    "src/registry/mod.rs",
    "src/orchestrator/mod.rs",
    "src/router/mod.rs",
    "src/evolution/mod.rs",
    "src/slm/mod.rs",
    "src/weights/mod.rs"
]

full_content = ""
for f in files:
    if os.path.exists(f):
        with open(f, 'r') as file:
            full_content += f"\n--- {f} ---\n" + file.read()

# 2. Generate DU-UUID (RFC 4122 compliant deterministic hash)
# Pipeline: Content -> SHA-256 -> 16 bytes -> UUID v4 layout
hash_bytes = hashlib.sha256(full_content.encode()).digest()
uuid_bytes = bytearray(hash_bytes[:16])
uuid_bytes[6] = (uuid_bytes[6] & 0x0f) | 0x40 # Version 4
uuid_bytes[8] = (uuid_bytes[8] & 0x3f) | 0x80 # Variant 10xx
du_uuid = uuid.UUID(bytes=bytes(uuid_bytes))

# 3. Compress content
compressed = zlib.compress(full_content.encode())
encoded_data = base64.b64encode(compressed).decode('utf-8')

# 4. Create .ure manifest
manifest = {
    "resource_id": str(du_uuid),
    "resource_type": "identity",
    "complexity_score": 0.5,
    "guidance": "Core implementation of the Predictive Resource Orchestration Mesh",
    "compressed_data": encoded_data,
    "metadata": {
        "os": "linux",
        "arch": "x86_64",
        "impl": "rust"
    }
}

file_name = f"{du_uuid}.ure"
with open(file_name, "w") as f:
    json.dump(manifest, f, indent=2)

print(file_name)

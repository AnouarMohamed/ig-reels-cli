#!/usr/bin/env bash
set -euo pipefail

echo "=== Running Cross-Language MessagePack Protocol Contract Tests ==="

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PYTHONPATH="${REPO_ROOT}/py-ig-gateway:${PYTHONPATH:-}"

echo "1. Running Python protocol contract tests..."
pytest "${REPO_ROOT}/py-ig-gateway/tests/test_contract.py" "${REPO_ROOT}/py-ig-gateway/tests/test_protocol.py"

echo "2. Running Rust protocol contract tests..."
(cd "${REPO_ROOT}/rust-tui" && cargo test --test contract_test -- --test-threads=1)

echo "3. Verifying compact tuple struct encoding fails contract requirement..."
python3 -c "
import sys
sys.path.insert(0, '${REPO_ROOT}/py-ig-gateway')
import msgpack
from protocol import decode_frame

# Array representation (compact tuple struct)
tuple_frame = len(msgpack.packb([1, 'r1', 'ping', {}])).to_bytes(4, 'big') + msgpack.packb([1, 'r1', 'ping', {}])
decoded = decode_frame(tuple_frame)
if isinstance(decoded, dict):
    raise RuntimeError('Compact tuple encoding should fail map requirement!')
print('Verified: compact tuple encoding does not produce a map dict.')
"

echo "=== Protocol Contract Gate: SUCCESS ==="

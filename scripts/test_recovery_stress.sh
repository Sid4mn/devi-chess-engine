#!/bin/bash
# filepath: /Users/funinc/Documents/chess-engine-rust/chess-engine-rust/scripts/test_recovery_stress.sh

echo "=== Thread Recovery Stress Test ==="

for DEPTH in 4 5 6; do
    echo ""
    echo "Testing depth $DEPTH..."
    ./target/release/devi \
        --thread-recovery \
        --depth $DEPTH \
        --checkpoint-interval 500
done

echo ""
echo "=== All tests complete ==="
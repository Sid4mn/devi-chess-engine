# Two-Phase Benchmark Results Summary

## Starting Position

| Config | Median (ms) | Probe (ms) | P1 (ms) | P2 (ms) | Searches/s | Speedup |
|--------|-------------|------------|---------|---------|------------|----------|
| baseline | 79.6 | 0.0 | 79.6 | 0.0 | 12.56 | 1.000x |
| fast_bias | 85.9 | 0.0 | 85.9 | 0.0 | 11.64 | 0.927x |
| two_phase_p1_r6 | 92.7 | 0.1 | 92.4 | 0.0 | 10.79 | 0.859x |
| two_phase_p2_r6 | 91.2 | 1.0 | 90.1 | 0.0 | 10.97 | 0.873x |
| two_phase_p3_r6 | 120.8 | 31.6 | 89.2 | 0.0 | 8.28 | 0.659x |
| two_phase_p1_r5 | 80.8 | 0.1 | 80.5 | 0.0 | 12.38 | 0.986x |
| two_phase_p1_r7 | 1627.3 | 0.2 | 74.6 | 1550.5 | 0.61 | 0.049x |
| two_phase_p1_r8 | 298.2 | 0.2 | 83.9 | 214.3 | 3.35 | 0.267x |

## Kiwipete Position

| Config | Median (ms) | Probe (ms) | P1 (ms) | P2 (ms) | Searches/s | Speedup |
|--------|-------------|------------|---------|---------|------------|----------|
| baseline | 7145.5 | 0.0 | 7145.5 | 0.0 | 0.14 | 1.000x |
| fast_bias | 7136.3 | 0.0 | 7136.3 | 0.0 | 0.14 | 1.001x |
| two_phase_p1_r6 | 8220.4 | 0.3 | 8219.9 | 0.0 | 0.12 | 0.869x |
| two_phase_p2_r6 | 7755.9 | 9.3 | 7746.4 | 0.0 | 0.13 | 0.921x |
| two_phase_p3_r6 | 7814.1 | 612.9 | 7202.7 | 0.0 | 0.13 | 0.914x |
| two_phase_p1_r5 | 6987.6 | 0.3 | 6987.1 | 0.0 | 0.14 | 1.023x |
| two_phase_p1_r7 | 7055.9 | 0.3 | 7055.5 | 0.0 | 0.14 | 1.013x |
| two_phase_p1_r8 | 26636.9 | 1.5 | 4375.1 | 22267.1 | 0.04 | 0.268x |

## Position4 Position

| Config | Median (ms) | Probe (ms) | P1 (ms) | P2 (ms) | Searches/s | Speedup |
|--------|-------------|------------|---------|---------|------------|----------|
| baseline | 137.4 | 0.0 | 137.4 | 0.0 | 7.28 | 1.000x |
| fast_bias | 138.4 | 0.0 | 138.4 | 0.0 | 7.22 | 0.992x |
| two_phase_p1_r6 | 134.4 | 0.0 | 134.2 | 0.0 | 7.44 | 1.022x |
| two_phase_p2_r6 | 136.7 | 1.0 | 135.6 | 0.0 | 7.32 | 1.005x |
| two_phase_p3_r6 | 198.1 | 63.3 | 135.1 | 0.0 | 5.05 | 0.694x |
| two_phase_p1_r5 | 137.7 | 0.0 | 137.6 | 0.0 | 7.26 | 0.997x |
| two_phase_p1_r7 | 134.1 | 0.0 | 133.9 | 0.0 | 7.46 | 1.025x |
| two_phase_p1_r8 | 136.3 | 0.0 | 136.1 | 0.0 | 7.34 | 1.008x |

## Best Configurations

- **starting**: baseline (1.00x speedup)
- **kiwipete**: two_phase_p1_r5 (1.02x speedup)
- **position4**: two_phase_p1_r7 (1.02x speedup)

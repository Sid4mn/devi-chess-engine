#!/usr/bin/env python3
"""
Speedup vs heavy_ratio for the two-phase scheduler (v0.5.1 recheck).

Reads every benchmarks/v0.5.1/two_phase_benchmark_run*.csv sweep and plots
speedup against heavy_ratio per position (probe depth 1), with error bars
spanning the runs. Shows the cliff where two-phase stops aborting to the
P-cores and actually dispatches light moves to the throttled E-cores.

Usage: python3 scripts/plot_speedup_vs_ratio.py [output_dir]
"""

import glob
import sys
from pathlib import Path

import pandas as pd
import matplotlib.pyplot as plt

try:
    plt.style.use('seaborn-v0_8-whitegrid')
except OSError:
    plt.style.use('ggplot')

COLORS = {'starting': '#3498db', 'kiwipete': '#e74c3c', 'position4': '#2ecc71'}


def main():
    out_dir = Path(sys.argv[1]) if len(sys.argv) > 1 else Path('releases/v0.5.1/figures')
    out_dir.mkdir(parents=True, exist_ok=True)

    files = sorted(glob.glob('benchmarks/v0.5.1/two_phase_benchmark_run*.csv'))
    if not files:
        print('no run CSVs found under benchmarks/v0.5.1/')
        return
    df = pd.concat([pd.read_csv(f) for f in files], ignore_index=True)
    print(f"loaded {len(df)} rows from {len(files)} runs")

    # probe-depth-1 two-phase configs only - the clean ratio sweep
    tp = df[(df['probe_depth'] == 1) & (df['heavy_ratio'] > 0)]

    fig, ax = plt.subplots(figsize=(9, 6))
    for pos in ['starting', 'kiwipete', 'position4']:
        sub = tp[tp['position'] == pos]
        if sub.empty:
            continue
        ratios = sorted(sub['heavy_ratio'].unique())
        g = sub.groupby('heavy_ratio')['speedup']
        means = [g.get_group(r).mean() for r in ratios]
        lo = [means[i] - g.get_group(r).min() for i, r in enumerate(ratios)]
        hi = [g.get_group(r).max() - means[i] for i, r in enumerate(ratios)]
        ax.errorbar(ratios, means, yerr=[lo, hi], marker='o', capsize=4,
                    linewidth=2, label=pos, color=COLORS.get(pos))

    ax.axhline(1.0, ls='--', color='gray', lw=1, label='baseline parity')
    ax.set_xlabel('heavy_ratio (fraction of moves kept on P-cores)')
    ax.set_ylabel('speedup vs baseline')
    ax.set_title('Two-phase speedup vs heavy ratio (probe depth 1, depth 6)\n'
                 'markers = mean, bars = range across 3 runs (hot/cooled/cold)')
    ax.legend()
    fig.tight_layout()
    fig.savefig(out_dir / 'speedup_vs_ratio.png', dpi=150, bbox_inches='tight')
    fig.savefig(out_dir / 'speedup_vs_ratio.pdf', bbox_inches='tight')
    print(f"saved: {out_dir / 'speedup_vs_ratio.png'}")


if __name__ == '__main__':
    main()

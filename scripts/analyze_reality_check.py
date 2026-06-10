#!/usr/bin/env python3
"""
v0.6.0 Reality Check figures

Generates the four plots for releases/v0.6.0/figures/ from the CSVs on disk:
- Plot A: probe accuracy - probe node count vs true search cost (the predictor study)
- Plot B: fix journey - kiwipete two-phase time through each fix vs baseline
- Plot C: crossover noise - 8P-only vs 10-core ratio across k
- Plot D: QoS throttle - BACKGROUND vs UTILITY E-pool, same configs

Usage: python3 scripts/analyze_reality_check.py
"""

import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
from pathlib import Path
from scipy import stats

try:
    plt.style.use('seaborn-v0_8-whitegrid')
except OSError:
    plt.style.use('ggplot')

OUT = Path('releases/v0.6.0/figures')
OUT.mkdir(parents=True, exist_ok=True)

COLORS = {
    'baseline': '#2ecc71',
    'bad': '#e74c3c',
    'mid': '#f39c12',
    'good': '#3498db',
    'kiwipete': '#e74c3c',
    'starting': '#3498db',
}


def save(fig, name):
    fig.savefig(OUT / f'{name}.png', dpi=150, bbox_inches='tight')
    fig.savefig(OUT / f'{name}.pdf', bbox_inches='tight')
    print(f"Saved: {OUT / name}.png")
    plt.close(fig)


# --- Plot A: probe accuracy -------------------------------------------------
def plot_probe_accuracy():
    df = pd.read_csv('benchmarks/v0.5.0/classification_accuracy.csv')
    positions = ['starting', 'kiwipete', 'position4']
    depths = [('probe_d1', 'probe depth 1'), ('probe_d2', 'probe depth 2')]

    fig, axes = plt.subplots(2, 3, figsize=(13, 7), sharey='col')
    for row, (col, label) in enumerate(depths):
        for c, pos in enumerate(positions):
            ax = axes[row][c]
            sub = df[df['position'] == pos]
            rho, _ = stats.spearmanr(sub[col], sub['true_time_ms'])
            # constant probe values (startpos @ depth 1: every move probes to 20)
            # make the correlation undefined - which is the whole point
            corr_label = f'rank corr = {rho:.2f}' if not np.isnan(rho) else 'undefined - every move probes the same!'
            ax.scatter(sub[col], sub['true_time_ms'], s=28, alpha=0.7,
                       color=COLORS['good'] if row else COLORS['bad'], edgecolors='black', linewidths=0.4)
            ax.set_title(f'{pos} - {label}\n{corr_label}', fontsize=10)
            if row == 1:
                ax.set_xlabel('probe node count')
            if c == 0:
                ax.set_ylabel('true cost (ms, single thread)')
            ax.set_yscale('log')

    fig.suptitle('Can a shallow probe predict real search cost?\n'
                 'depth 1 (top, red) is noise - depth 2 (bottom, blue) has a weak real signal',
                 fontsize=13, fontweight='bold')
    fig.tight_layout()
    save(fig, 'probe_accuracy')


# --- Plot B: fix journey ----------------------------------------------------
def plot_fix_journey():
    seq = pd.read_csv('benchmarks/fixing_search/before_seq.csv')
    ovl = pd.read_csv('benchmarks/fixing_search/after_overlap.csv')
    uti = pd.read_csv('benchmarks/fixing_search/charitable_utility.csv')

    def med(df, pos, cfg):
        m = df[(df['position'] == pos) & (df['config'] == cfg)]
        return float(m['median_total_ms'].iloc[0]) if len(m) else np.nan

    pos = 'kiwipete'
    baseline = med(uti, pos, 'baseline')
    # aggressive split (ratio 0.8): the config that actually pushes work to E-cores
    stages = [
        ('as shipped\n(sequential, BACKGROUND,\nprobe 1)', med(seq, pos, 'two_phase_p1_r8'), COLORS['bad']),
        ('+ overlap phases\n(BACKGROUND, probe 1)', med(ovl, pos, 'two_phase_p1_r8'), COLORS['mid']),
        ('+ UTILITY QoS\n(probe 2)', med(uti, pos, 'two_phase_p2_r8'), COLORS['good']),
    ]

    fig, ax = plt.subplots(figsize=(9, 6))
    xs = np.arange(len(stages))
    vals = [s[1] / 1000.0 for s in stages]
    bars = ax.bar(xs, vals, color=[s[2] for s in stages], alpha=0.85, width=0.55)
    for bar, v in zip(bars, vals):
        ax.text(bar.get_x() + bar.get_width() / 2, bar.get_height() + 0.4,
                f'{v:.1f}s', ha='center', fontsize=11, fontweight='bold')

    ax.axhline(baseline / 1000.0, color=COLORS['baseline'], linestyle='--', linewidth=2,
               label=f'baseline (10-thread work-stealing): {baseline/1000:.1f}s')
    ax.set_xticks(xs)
    ax.set_xticklabels([s[0] for s in stages], fontsize=9)
    ax.set_ylabel('median search time (s)')
    ax.set_title('Each fix helps a lot - and it still loses to doing nothing special\n'
                 'kiwipete, depth 6, two-phase with E-cores actually used (ratio 0.8)',
                 fontsize=12, fontweight='bold')
    ax.legend()
    fig.tight_layout()
    save(fig, 'fix_journey')


# --- Plot C: crossover noise ------------------------------------------------
def plot_crossover():
    fig, ax = plt.subplots(figsize=(9, 6))
    for fname, pos in [('crossover_48moves.csv', 'kiwipete'), ('crossover_20moves.csv', 'starting')]:
        df = pd.read_csv(f'benchmarks/fixing_search/{fname}')
        ax.plot(df['k'], df['ratio_8P_over_10c'], 'o-', label=f'{pos} ({df["k"].max()} moves)',
                color=COLORS[pos], markersize=7, linewidth=1.5, alpha=0.85)

    ax.axhline(1.0, color='gray', linewidth=1)
    ax.axhspan(0.9, 1.1, color='gray', alpha=0.15, label='+/- 10% band')
    ax.set_xlabel('k = number of root moves searched')
    ax.set_ylabel('8 P-cores only  /  all 10 cores   (below 1.0 = avoiding E-cores wins)')
    ax.set_title('Does avoiding the E-cores ever pay? Mostly noise.\n'
                 'first-k root moves, depth 6, full-window (compare ratios, not times)',
                 fontsize=12, fontweight='bold')
    ax.legend()
    fig.tight_layout()
    save(fig, 'crossover_noise')


# --- Plot D: QoS throttle ---------------------------------------------------
def plot_qos_throttle():
    bg = pd.read_csv('benchmarks/fixing_search/charitable.csv')
    ut = pd.read_csv('benchmarks/fixing_search/charitable_utility.csv')

    fig, axes = plt.subplots(1, 2, figsize=(13, 6))
    for ax, pos in zip(axes, ['starting', 'kiwipete']):
        cfgs = [c for c in bg[bg['position'] == pos]['config'] if c.startswith('two_phase_p2')]
        cfgs.sort()
        labels = [c.replace('two_phase_p2_r', 'ratio 0.') for c in cfgs]

        def meds(df):
            return [float(df[(df['position'] == pos) & (df['config'] == c)]['median_total_ms'].iloc[0])
                    for c in cfgs]

        x = np.arange(len(cfgs))
        w = 0.38
        ax.bar(x - w / 2, meds(bg), w, label='BACKGROUND (throttled ~13x)', color=COLORS['bad'], alpha=0.85)
        ax.bar(x + w / 2, meds(ut), w, label='UTILITY (real ~3x gap)', color=COLORS['good'], alpha=0.85)
        base = float(ut[(ut['position'] == pos) & (ut['config'] == 'baseline')]['median_total_ms'].iloc[0])
        ax.axhline(base, color=COLORS['baseline'], linestyle='--', linewidth=2, label='baseline')
        ax.set_xticks(x)
        ax.set_xticklabels(labels)
        ax.set_ylabel('median time (ms)')
        ax.set_title(f'{pos}', fontsize=11)
        ax.set_yscale('log')
        if pos == 'starting':
            ax.legend(fontsize=9)

    fig.suptitle('Same code, one QoS constant changed: the E-pool was OS-throttled, not just slow\n'
                 'two-phase at probe 2, depth 6, log scale', fontsize=13, fontweight='bold')
    fig.tight_layout()
    save(fig, 'qos_throttle')


# --- Plot E: core pinning, corrected ----------------------------------------
def plot_core_pinning():
    # v0.4 reference numbers (BACKGROUND-era), 8 threads depth 7
    old = {}
    for pol in ['none', 'fast', 'efficient', 'mixed']:
        df = pd.read_csv(f'benchmarks/hetero_8t_{pol}.csv')
        old[pol] = float(df['median_ms'].iloc[0])
    # corrected run (EfficientBias = UTILITY now, plus explicit Background)
    new = {}
    for pol in ['none', 'fast', 'efficient', 'background', 'mixed']:
        df = pd.read_csv(f'benchmarks/v0.6.0/policy_{pol}.csv')
        new[pol] = float(df['median_ms'].iloc[0])
    # 2-thread version: fits on the 2 E-cores exactly, so no spill/oversubscription confounds
    two = {}
    for pol in ['none', 'fast', 'efficient', 'background']:
        df = pd.read_csv(f'benchmarks/v0.6.0/policy2_{pol}.csv')
        two[pol] = float(df['median_ms'].iloc[0])

    fig, axes = plt.subplots(1, 3, figsize=(16, 6), sharey=True)
    for ax, data, title in [
        (axes[0], old, 'v0.4 (efficient = BACKGROUND)\n8 threads'),
        (axes[1], new, 'v0.6 corrected (efficient = UTILITY)\n8 threads'),
        (axes[2], two, 'v0.6, 2 threads\n(fits the 2 E-cores exactly)'),
    ]:
        pols = list(data.keys())
        slow = [data[p] / data['none'] for p in pols]
        cols = [COLORS['bad'] if p == 'background' or (p == 'efficient' and title.startswith('v0.4'))
                else COLORS['good'] for p in pols]
        bars = ax.bar(pols, slow, color=cols, alpha=0.85, width=0.6)
        for b, s in zip(bars, slow):
            ax.text(b.get_x() + b.get_width() / 2, b.get_height() + 0.15, f'{s:.1f}x',
                    ha='center', fontsize=11, fontweight='bold')
        ax.axhline(1.0, color='gray', linewidth=1)
        ax.set_title(title, fontsize=11)
        ax.set_ylabel('slowdown vs policy=none')

    fig.suptitle('Core policy slowdown, before and after the QoS fix\n'
                 'depth 7, 8 threads - the v0.4 "12.8x" was the BACKGROUND throttle, not the E-cores',
                 fontsize=13, fontweight='bold')
    fig.tight_layout()
    save(fig, 'core_pinning_fixed')


# --- Plot F: fault overhead -------------------------------------------------
def plot_fault_overhead():
    old = pd.read_csv('releases/v0.4.0/fault_overhead.csv')
    old_1fault = float(old[old['scenario'] == 'with_panic']['overhead_pct'].iloc[0])
    new = pd.read_csv('benchmarks/fault_overhead.csv')

    labels = [f'v0.4 design\n1 fault']
    vals = [old_1fault]
    cols = [COLORS['bad']]

    one = new[new['scenario'] == 'with_panic']
    labels.append('per-move\n1 fault')
    vals.append(float(one['overhead_pct'].iloc[0]))
    cols.append(COLORS['good'])

    sweep = new[new['scenario'] == 'fault_sweep']
    for _, r in sweep.iterrows():
        labels.append(f'per-move\n{int(r["faults"])} of {int(r["n_moves"])}')
        vals.append(float(r['overhead_pct']))
        cols.append(COLORS['good'])

    fig, ax = plt.subplots(figsize=(10, 6))
    bars = ax.bar(labels, vals, color=cols, alpha=0.85, width=0.65)
    for b, v in zip(bars, vals):
        ax.text(b.get_x() + b.get_width() / 2, b.get_height() + 2, f'{v:.0f}%',
                ha='center', fontsize=10, fontweight='bold')
    ax.set_ylabel('overhead vs no-fault baseline (%)')
    ax.set_title('Fault recovery overhead: whole-search retry vs per-move isolation\n'
                 'depth 7, 8 threads, panic injected after real work',
                 fontsize=12, fontweight='bold')
    fig.tight_layout()
    save(fig, 'fault_overhead')


if __name__ == '__main__':
    plot_probe_accuracy()
    plot_fix_journey()
    plot_crossover()
    plot_qos_throttle()
    plot_core_pinning()
    plot_fault_overhead()
    print("\nDone! Figures in", OUT)

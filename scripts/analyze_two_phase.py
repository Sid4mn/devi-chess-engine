#!/usr/bin/env python3
"""
Two-Phase Scheduler Benchmark Analysis

Generates publication-quality visualizations:
- Plot A: Throughput comparison bar chart
- Plot B: Phase time breakdown (stacked bar)
- Plot C: Probe overhead vs speedup scatter
- Plot D: Classification ratio heatmap

Usage: python3 scripts/analyze_two_phase.py [csv_path]
"""

import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
from pathlib import Path
from scipy import stats
import sys

# Style configuration - use compatible style name
try:
    plt.style.use('seaborn-v0_8-whitegrid')
except OSError:
    try:
        plt.style.use('seaborn-whitegrid')
    except OSError:
        plt.style.use('ggplot')  # Fallback

COLORS = {
    'baseline': '#2ecc71',
    'fast_bias': '#3498db',
    'two_phase': '#e74c3c',
    'probe': '#f39c12',
    'phase1': '#3498db',
    'phase2': '#9b59b6',
}


def load_data(csv_path: str) -> pd.DataFrame:
    """Load benchmark CSV data."""
    df = pd.read_csv(csv_path)
    return df


def plot_throughput_comparison(df: pd.DataFrame, output_dir: Path):
    """Plot A: Bar chart comparing searches/sec across configurations."""
    fig, axes = plt.subplots(1, 3, figsize=(14, 5), sharey=True)
    
    positions = df['position'].unique()
    
    for idx, pos in enumerate(positions):
        ax = axes[idx]
        pos_df = df[df['position'] == pos].copy()
        
        configs = pos_df['config'].tolist()
        sps = pos_df['searches_per_sec'].tolist()
        stddevs = pos_df['stddev_ms'].tolist()
        
        # Convert stddev_ms to stddev in searches/sec
        medians = pos_df['median_total_ms'].tolist()
        sps_errors = [(s / m) * std for s, m, std in zip(sps, medians, stddevs)]
        
        colors = []
        for c in configs:
            if 'baseline' in c:
                colors.append(COLORS['baseline'])
            elif 'fast' in c:
                colors.append(COLORS['fast_bias'])
            else:
                colors.append(COLORS['two_phase'])
        
        x = np.arange(len(configs))
        bars = ax.bar(x, sps, yerr=sps_errors, capsize=3, color=colors, alpha=0.85)
        
        ax.set_xlabel('Configuration')
        ax.set_ylabel('Searches/sec' if idx == 0 else '')
        ax.set_title(f'{pos.capitalize()} Position')
        ax.set_xticks(x)
        ax.set_xticklabels(configs, rotation=45, ha='right', fontsize=8)
        
        # Add value labels
        for bar, val in zip(bars, sps):
            ax.text(bar.get_x() + bar.get_width()/2, bar.get_height() + 0.02,
                   f'{val:.2f}', ha='center', va='bottom', fontsize=7)
    
    plt.suptitle('Two-Phase Scheduler: Throughput Comparison', fontsize=14, fontweight='bold')
    plt.tight_layout()
    plt.savefig(output_dir / 'throughput_comparison.png', dpi=150, bbox_inches='tight')
    plt.savefig(output_dir / 'throughput_comparison.pdf', bbox_inches='tight')
    print(f"Saved: {output_dir / 'throughput_comparison.png'}")
    plt.close()


def plot_phase_breakdown(df: pd.DataFrame, output_dir: Path):
    """Plot B: Stacked bar showing Phase 1/Phase 2/Probe time breakdown."""
    # Filter to two-phase configs only
    tp_df = df[df['config'].str.contains('two_phase')].copy()
    
    if tp_df.empty:
        print("No two-phase data for phase breakdown plot")
        return
    
    fig, ax = plt.subplots(figsize=(10, 6))
    
    configs = tp_df['config'] + '_' + tp_df['position']
    probe = tp_df['median_probe_ms'].values
    phase1 = tp_df['median_phase1_ms'].values
    phase2 = tp_df['median_phase2_ms'].values
    
    x = np.arange(len(configs))
    width = 0.6
    
    bars1 = ax.bar(x, probe, width, label='Probe', color=COLORS['probe'])
    bars2 = ax.bar(x, phase1, width, bottom=probe, label='Phase 1 (P-cores)', color=COLORS['phase1'])
    bars3 = ax.bar(x, phase2, width, bottom=probe+phase1, label='Phase 2 (E-cores)', color=COLORS['phase2'])
    
    ax.set_xlabel('Configuration')
    ax.set_ylabel('Time (ms)')
    ax.set_title('Two-Phase Search: Time Breakdown by Phase')
    ax.set_xticks(x)
    ax.set_xticklabels(configs, rotation=45, ha='right', fontsize=8)
    ax.legend()
    
    plt.tight_layout()
    plt.savefig(output_dir / 'phase_breakdown.png', dpi=150, bbox_inches='tight')
    plt.savefig(output_dir / 'phase_breakdown.pdf', bbox_inches='tight')
    print(f"Saved: {output_dir / 'phase_breakdown.png'}")
    plt.close()


def plot_probe_overhead(df: pd.DataFrame, output_dir: Path):
    """Plot C: Scatter of probe overhead % vs speedup."""
    tp_df = df[df['config'].str.contains('two_phase')].copy()
    
    if tp_df.empty:
        print("No two-phase data for probe overhead plot")
        return
    
    fig, ax = plt.subplots(figsize=(8, 6))
    
    probe_pct = (tp_df['median_probe_ms'] / tp_df['median_total_ms']) * 100
    speedup = tp_df['speedup']
    probe_depths = tp_df['probe_depth']
    
    # Color by probe depth
    scatter = ax.scatter(probe_pct, speedup, c=probe_depths, cmap='viridis', 
                        s=100, alpha=0.7, edgecolors='black')
    
    # Add colorbar
    cbar = plt.colorbar(scatter)
    cbar.set_label('Probe Depth')
    
    # Add labels for each point
    for i, (x, y, config) in enumerate(zip(probe_pct, speedup, tp_df['config'])):
        ax.annotate(config.split('_')[-1], (x, y), textcoords='offset points',
                   xytext=(5, 5), fontsize=7)
    
    ax.set_xlabel('Probe Overhead (%)')
    ax.set_ylabel('Speedup vs Baseline')
    ax.set_title('Probe Depth vs Performance Tradeoff')
    ax.axhline(y=1.0, color='gray', linestyle='--', alpha=0.5, label='Baseline')
    
    plt.tight_layout()
    plt.savefig(output_dir / 'probe_overhead.png', dpi=150, bbox_inches='tight')
    plt.savefig(output_dir / 'probe_overhead.pdf', bbox_inches='tight')
    print(f"Saved: {output_dir / 'probe_overhead.png'}")
    plt.close()


def plot_classification_heatmap(df: pd.DataFrame, output_dir: Path):
    """Plot D: Heatmap of heavy_ratio vs throughput by position."""
    tp_df = df[df['config'].str.contains('two_phase')].copy()
    
    if tp_df.empty:
        print("No two-phase data for heatmap")
        return
    
    # Get unique positions and ratios
    positions = tp_df['position'].unique()
    ratios = sorted(tp_df['heavy_ratio'].unique())
    
    if len(ratios) < 2:
        print("Not enough ratio variations for heatmap")
        return
    
    fig, ax = plt.subplots(figsize=(8, 6))
    
    # Create matrix
    matrix = np.zeros((len(positions), len(ratios)))
    for i, pos in enumerate(positions):
        for j, ratio in enumerate(ratios):
            mask = (tp_df['position'] == pos) & (tp_df['heavy_ratio'] == ratio)
            if mask.any():
                matrix[i, j] = tp_df.loc[mask, 'speedup'].values[0]
    
    im = ax.imshow(matrix, cmap='RdYlGn', aspect='auto', vmin=0.8, vmax=1.2)
    
    ax.set_xticks(np.arange(len(ratios)))
    ax.set_yticks(np.arange(len(positions)))
    ax.set_xticklabels([f'{r:.1f}' for r in ratios])
    ax.set_yticklabels(positions)
    ax.set_xlabel('Heavy Ratio')
    ax.set_ylabel('Position')
    ax.set_title('Classification Ratio Impact on Speedup')
    
    # Add text annotations
    for i in range(len(positions)):
        for j in range(len(ratios)):
            text = ax.text(j, i, f'{matrix[i, j]:.2f}', ha='center', va='center',
                          color='black' if 0.9 < matrix[i, j] < 1.1 else 'white')
    
    plt.colorbar(im, label='Speedup vs Baseline')
    plt.tight_layout()
    plt.savefig(output_dir / 'classification_heatmap.png', dpi=150, bbox_inches='tight')
    plt.savefig(output_dir / 'classification_heatmap.pdf', bbox_inches='tight')
    print(f"Saved: {output_dir / 'classification_heatmap.png'}")
    plt.close()


def compute_statistics(df: pd.DataFrame) -> dict:
    """Compute statistical significance between configs."""
    results = {}
    
    positions = df['position'].unique()
    for pos in positions:
        pos_df = df[df['position'] == pos]
        
        baseline = pos_df[pos_df['config'] == 'baseline']
        if baseline.empty:
            continue
        
        baseline_sps = baseline['searches_per_sec'].values[0]
        
        for _, row in pos_df.iterrows():
            if row['config'] == 'baseline':
                continue
            
            key = f"{pos}_{row['config']}"
            results[key] = {
                'speedup': row['speedup'],
                'baseline_sps': baseline_sps,
                'config_sps': row['searches_per_sec'],
            }
    
    return results


def generate_summary_table(df: pd.DataFrame, output_dir: Path):
    """Generate markdown summary table."""
    md = "# Two-Phase Benchmark Results Summary\n\n"
    
    positions = df['position'].unique()
    
    for pos in positions:
        md += f"## {pos.capitalize()} Position\n\n"
        md += "| Config | Median (ms) | Probe (ms) | P1 (ms) | P2 (ms) | Searches/s | Speedup |\n"
        md += "|--------|-------------|------------|---------|---------|------------|----------|\n"
        
        pos_df = df[df['position'] == pos]
        for _, row in pos_df.iterrows():
            md += f"| {row['config']} | {row['median_total_ms']:.1f} | {row['median_probe_ms']:.1f} | "
            md += f"{row['median_phase1_ms']:.1f} | {row['median_phase2_ms']:.1f} | "
            md += f"{row['searches_per_sec']:.2f} | {row['speedup']:.3f}x |\n"
        
        md += "\n"
    
    # Best configs
    md += "## Best Configurations\n\n"
    for pos in positions:
        pos_df = df[df['position'] == pos]
        best = pos_df.loc[pos_df['speedup'].idxmax()]
        md += f"- **{pos}**: {best['config']} ({best['speedup']:.2f}x speedup)\n"
    
    with open(output_dir / 'summary.md', 'w') as f:
        f.write(md)
    
    print(f"Saved: {output_dir / 'summary.md'}")


def main():
    csv_path = sys.argv[1] if len(sys.argv) > 1 else 'benchmarks/v0.5.0/two_phase_benchmark.csv'
    output_dir = Path('releases/v0.5.0/figures')
    output_dir.mkdir(parents=True, exist_ok=True)
    
    print(f"Loading data from: {csv_path}")
    df = load_data(csv_path)
    
    # Rename columns for consistency if needed
    if 'position_name' in df.columns:
        df = df.rename(columns={'position_name': 'position', 'config_name': 'config'})
    
    print(f"Loaded {len(df)} rows")
    print(f"Positions: {df['position'].unique()}")
    print(f"Configs: {df['config'].unique()}")
    
    print("\nGenerating plots...")
    plot_throughput_comparison(df, output_dir)
    plot_phase_breakdown(df, output_dir)
    plot_probe_overhead(df, output_dir)
    plot_classification_heatmap(df, output_dir)
    
    print("\nGenerating summary...")
    generate_summary_table(df, output_dir)
    
    print("\nComputing statistics...")
    stats_results = compute_statistics(df)
    for key, val in stats_results.items():
        print(f"  {key}: {val['speedup']:.3f}x speedup")
    
    print("\nDone!")


if __name__ == '__main__':
    main()

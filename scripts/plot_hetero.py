#!/usr/bin/env python3
import matplotlib.pyplot as plt
import pandas as pd
import numpy as np
from pathlib import Path

def plot_hetero_results():
    benchmarks_dir = Path(__file__).parent.parent / "benchmarks"
    
    data_8t = {}
    data_10t = {}
    
    for policy in ['none', 'fast', 'efficient', 'mixed']:
        try:
            df_8t = pd.read_csv(benchmarks_dir / f'hetero_8t_{policy}.csv')
            data_8t[policy.capitalize()] = df_8t['searches_per_sec'].iloc[0]
        except:
            data_8t[policy.capitalize()] = 0.0
            
        try:
            df_10t = pd.read_csv(benchmarks_dir / f'hetero_10t_{policy}.csv')
            data_10t[policy.capitalize()] = df_10t['searches_per_sec'].iloc[0]
        except:
            data_10t[policy.capitalize()] = 0.0
    
    policies = list(data_8t.keys())
    values_8t = list(data_8t.values())
    values_10t = list(data_10t.values())
    
    x = np.arange(len(policies))
    width = 0.35
    
    fig, ax = plt.subplots(figsize=(10, 6))
    
    bars1 = ax.bar(x - width/2, values_8t, width, label='8 threads', color='#2E86C1')
    bars2 = ax.bar(x + width/2, values_10t, width, label='10 threads', color='#E74C3C')
    
    for bars in [bars1, bars2]:
        for bar in bars:
            height = bar.get_height()
            if height > 0:
                ax.annotate(f'{height:.2f}',
                           xy=(bar.get_x() + bar.get_width() / 2, height),
                           xytext=(0, 3),
                           textcoords="offset points",
                           ha='center', va='bottom',
                           fontsize=9)
    
    if data_8t['None'] > 0 and data_8t['Efficient'] > 0:
        gap = data_8t['None'] / data_8t['Efficient']
        ax.annotate(f'{gap:.1f}x slower', 
                    xy=(2, data_8t['Efficient']),
                    xytext=(2, data_8t['Efficient'] + 0.5),
                    arrowprops=dict(arrowstyle='->', color='red', lw=2),
                    fontsize=11, fontweight='bold', color='red')
    
    ax.set_xlabel('Core Policy', fontsize=12)
    ax.set_ylabel('Searches per Second', fontsize=12)
    ax.set_title('Heterogeneous Scheduling Performance\nM1 Pro (8P + 2E cores) - Depth 7', fontsize=14)
    ax.set_xticks(x)
    ax.set_xticklabels(policies)
    ax.legend()
    ax.grid(True, alpha=0.3, axis='y')
    
    plt.tight_layout()
    plt.savefig(benchmarks_dir / 'heterogeneous_impact.png', dpi=150)

if __name__ == "__main__":
    plot_hetero_results()
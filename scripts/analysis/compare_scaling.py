#!/usr/bin/env python3
"""
compare_scaling.py
Creates two simple, clear plots comparing Amdahl (depth 4) vs Gustafson (depth 7) scaling
"""

import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
from scipy.optimize import curve_fit
import os

def amdahl_speedup(p, s):
    """Amdahl's Law: S(p) = 1 / (s + (1-s)/p)"""
    return 1.0 / (s + (1.0 - s) / p)

def fit_serial_fraction(threads, speedup):
    """Fit Amdahl's law to find serial fraction"""
    popt, _ = curve_fit(lambda p, s: amdahl_speedup(p, s), threads, speedup, p0=[0.1], bounds=(0, 1))
    return popt[0]

def load_benchmark_data(filepath):
    """Load CSV and return threads, speedup, efficiency arrays"""
    df = pd.read_csv(filepath)
    
    # Handle policy column if it exists
    if 'policy' in df.columns and 'None' in df['policy'].values:
        df = df[df['policy'] == 'None']
    
    return df['threads'].values, df['speedup'].values, df['efficiency'].values

def create_two_plots():
    """Create two simple, clear comparison plots"""
    
    # Set style for clean plots
    plt.style.use('default')
    
    # Load data
    base_dir = '../benchmarks/scaling_analysis'
    threads_d4, speedup_d4, efficiency_d4 = load_benchmark_data(f'{base_dir}/speedup_d4.csv')
    threads_d7, speedup_d7, efficiency_d7 = load_benchmark_data(f'{base_dir}/speedup_d7.csv')
    
    # Fit serial fractions
    serial_d4 = fit_serial_fraction(threads_d4, speedup_d4)
    serial_d7 = fit_serial_fraction(threads_d7, speedup_d7)
    
    # Generate theoretical curves
    threads_theory = np.linspace(1, 10, 100)
    amdahl_d4_theory = [amdahl_speedup(t, serial_d4) for t in threads_theory]
    amdahl_d7_theory = [amdahl_speedup(t, serial_d7) for t in threads_theory]
    
    # =================
    # Plot 1: Speedup Comparison
    # =================
    fig1, ax1 = plt.subplots(figsize=(10, 7))
    
    # Empirical data
    ax1.plot(threads_d4, speedup_d4, 'o-', color='#2E86AB', markersize=10, linewidth=2.5, label=f'Depth 4 (measured)', zorder=5)
    ax1.plot(threads_d7, speedup_d7, 's-', color='#A23B72', markersize=10, linewidth=2.5, label=f'Depth 7 (measured)', zorder=5)
    
    # Theoretical fits
    ax1.plot(threads_theory, amdahl_d4_theory, '--', color='#2E86AB', linewidth=1.5, alpha=0.7, label=f'Depth 4 theory (s={serial_d4:.1%})')
    ax1.plot(threads_theory, amdahl_d7_theory, '--', color='#A23B72', linewidth=1.5, alpha=0.7, label=f'Depth 7 theory (s={serial_d7:.1%})')
    
    # Ideal linear speedup
    ax1.plot(threads_theory, threads_theory, ':', color='gray', linewidth=1, alpha=0.5, label='Ideal linear')
    
    # Annotations for key points
    # Find 8-thread index
    idx_8_d4 = list(threads_d4).index(8) if 8 in threads_d4 else -1
    idx_8_d7 = list(threads_d7).index(8) if 8 in threads_d7 else -1
    
    if idx_8_d4 >= 0:
        ax1.annotate(f'{speedup_d4[idx_8_d4]:.1f}x', xy=(8, speedup_d4[idx_8_d4]),  xytext=(8.5, speedup_d4[idx_8_d4] - 0.3), fontsize=11, color='#2E86AB', fontweight='bold')
    
    if idx_8_d7 >= 0:
        ax1.annotate(f'{speedup_d7[idx_8_d7]:.1f}x', xy=(8, speedup_d7[idx_8_d7]),  xytext=(8.5, speedup_d7[idx_8_d7] + 0.3), fontsize=11, color='#A23B72', fontweight='bold')
    
    # Styling
    ax1.set_xlabel('Number of Threads', fontsize=13, fontweight='bold')
    ax1.set_ylabel('Speedup Factor', fontsize=13, fontweight='bold')
    ax1.set_title('Parallel Speedup: Small Problem (Depth 4) vs Large Problem (Depth 7)', fontsize=14, fontweight='bold', pad=20)
    
    ax1.grid(True, alpha=0.3, linestyle='-', linewidth=0.5)
    ax1.set_xlim(0.5, 10.5)
    ax1.set_ylim(0, max(max(speedup_d4), max(speedup_d7)) + 1)
    
    # Custom legend
    ax1.legend(loc='upper left', fontsize=11, framealpha=0.95)
    
    # Add text box with key finding
    textstr = f'Key Finding:\nDepth 7 has {(1-serial_d7/serial_d4)*100:.0f}% less serial overhead\n' + f'-> Better scaling at high thread counts'
    props = dict(boxstyle='round', facecolor='wheat', alpha=0.8)
    ax1.text(0.98, 0.50, textstr, transform=ax1.transAxes, fontsize=11, verticalalignment='center', horizontalalignment='right', bbox=props)
    
    # Save Plot 1
    output_dir = '../benchmarks/scaling_analysis'
    os.makedirs(output_dir, exist_ok=True)
    fig1.tight_layout()
    fig1.savefig(f'{output_dir}/speedup_comparison.png', dpi=150, bbox_inches='tight')
    print(f"Saved: {output_dir}/speedup_comparison.png")
    
    # =================
    # Plot 2: Efficiency Comparison
    # =================
    fig2, ax2 = plt.subplots(figsize=(10, 7))
    
    # Efficiency data
    ax2.plot(threads_d4, efficiency_d4, 'o-', color='#2E86AB', markersize=10, linewidth=2.5, label=f'Depth 4 (s={serial_d4:.1%})', zorder=5)
    ax2.plot(threads_d7, efficiency_d7, 's-', color='#A23B72', markersize=10, linewidth=2.5, label=f'Depth 7 (s={serial_d7:.1%})', zorder=5)
    
    # Reference lines
    ax2.axhline(y=75, color='green', linestyle=':', alpha=0.5, linewidth=1.5)
    ax2.text(10.2, 75, 'Good', fontsize=10, color='green', va='center')
    
    ax2.axhline(y=50, color='orange', linestyle=':', alpha=0.5, linewidth=1.5)
    ax2.text(10.2, 50, 'Fair', fontsize=10, color='orange', va='center')
    
    # Highlight efficiency gap at 8 threads
    if idx_8_d4 >= 0 and idx_8_d7 >= 0:
        # Draw vertical span to show gap
        ax2.vlines(x=8, ymin=efficiency_d4[idx_8_d4], ymax=efficiency_d7[idx_8_d7],
                  colors='red', linestyles='solid', linewidth=2, alpha=0.7)
        
        # Add arrow and annotation
        gap = efficiency_d7[idx_8_d7] - efficiency_d4[idx_8_d4]
        mid_y = (efficiency_d4[idx_8_d4] + efficiency_d7[idx_8_d7]) / 2
        ax2.annotate(f'{gap:.1f}% gap\nat 8 threads', xy=(8, mid_y), xytext=(6.5, mid_y), arrowprops=dict(arrowstyle='->', color='red', alpha=0.7), fontsize=11, color='red', fontweight='bold', ha='center')
    
    # Styling
    ax2.set_xlabel('Number of Threads', fontsize=13, fontweight='bold')
    ax2.set_ylabel('Parallel Efficiency (%)', fontsize=13, fontweight='bold')
    ax2.set_title('Efficiency Degradation: Impact of Problem Size on Scaling', fontsize=14, fontweight='bold', pad=20)
    
    ax2.grid(True, alpha=0.3, linestyle='-', linewidth=0.5)
    ax2.set_xlim(0.5, 10.5)
    ax2.set_ylim(30, 105)
    
    # Legend
    ax2.legend(loc='upper right', fontsize=11, framealpha=0.95)
    
    # Add insight text
    textstr = 'Larger problems maintain\nhigher efficiency because\nparallel work dominates\nover serial overhead'
    props = dict(boxstyle='round', facecolor='lightblue', alpha=0.8)
    ax2.text(0.02, 0.25, textstr, transform=ax2.transAxes, fontsize=11, verticalalignment='center', bbox=props)
    
    # Save Plot 2
    fig2.tight_layout()
    fig2.savefig(f'{output_dir}/efficiency_comparison.png', dpi=150, bbox_inches='tight')
    print(f"Saved: {output_dir}/efficiency_comparison.png")
    
    return serial_d4, serial_d7

if __name__ == "__main__":
    s4, s7 = create_two_plots()
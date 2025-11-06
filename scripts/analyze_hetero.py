#!/usr/bin/env python3
import pandas as pd
import sys
from pathlib import Path

def analyze_hetero_results():
    """Analyze heterogeneous scheduling results for v0.4.0"""
    
    # Look in benchmarks directory (not current directory)
    benchmarks_dir = Path(__file__).parent.parent / "benchmarks"
    
    # Read all CSV files
    csv_files = {
        '8t': {
            'none': benchmarks_dir / 'hetero_8t_none.csv',
            'fast': benchmarks_dir / 'hetero_8t_fast.csv',
            'efficient': benchmarks_dir / 'hetero_8t_efficient.csv',
            'mixed': benchmarks_dir / 'hetero_8t_mixed.csv'
        },
        '10t': {
            'none': benchmarks_dir / 'hetero_10t_none.csv',
            'fast': benchmarks_dir / 'hetero_10t_fast.csv',
            'efficient': benchmarks_dir / 'hetero_10t_efficient.csv',
            'mixed': benchmarks_dir / 'hetero_10t_mixed.csv'
        }
    }
    
    results = {}
    
    for thread_config, policies in csv_files.items():
        results[thread_config] = {}
        for policy, filepath in policies.items():
            try:
                df = pd.read_csv(filepath)
                # Expecting columns from export_benchmark_csv: 
                # timestamp,threads,policy,median_ms,searches_per_sec,speedup,efficiency
                if 'searches_per_sec' in df.columns:
                    median_sps = df['searches_per_sec'].iloc[0]  # Single row per file
                elif 'median_ms' in df.columns:
                    median_ms = df['median_ms'].iloc[0]
                    median_sps = 1000.0 / median_ms if median_ms > 0 else 0
                else:
                    print(f"Warning: Unknown CSV format in {filepath}")
                    median_sps = 0.0
                
                results[thread_config][policy] = median_sps
            except Exception as e:
                print(f"Error reading {filepath}: {e}")
                results[thread_config][policy] = 0.0
    
    # Rest of the analysis code remains the same...
    print("# Heterogeneous Scheduling Analysis (v0.4.0)")
    print("\n## Results\n")
    
    # 8-thread table
    print("### 8 Threads (P-cores only)")
    print("| Policy | Searches/sec | Relative | Notes |")
    print("|--------|--------------|----------|-------|")
    
    baseline_8t = results['8t']['none']
    for policy in ['none', 'fast', 'efficient', 'mixed']:
        sps = results['8t'][policy]
        relative = (sps / baseline_8t * 100) if baseline_8t > 0 else 0
        
        notes = ""
        if policy == 'efficient':
            gap = baseline_8t / sps if sps > 0 else 0
            notes = f"**{gap:.1f}x slower**"
        elif policy == 'mixed':
            notes = "Critical-path bottleneck"
        elif policy == 'fast':
            notes = "QoS confirms P-bias"
        else:
            notes = "OS default"
        
        print(f"| {policy.capitalize():10} | {sps:.2f} | {relative:.0f}% | {notes} |")
    
    # 10-thread table
    print("\n### 10 Threads (8P + 2E)")
    print("| Policy | Searches/sec | Relative | Notes |")
    print("|--------|--------------|----------|-------|")
    
    baseline_10t = results['10t']['none']
    for policy in ['none', 'fast', 'efficient', 'mixed']:
        sps = results['10t'][policy]
        relative = (sps / baseline_10t * 100) if baseline_10t > 0 else 0
        
        notes = ""
        if policy == 'efficient':
            gap = baseline_10t / sps if sps > 0 else 0
            notes = f"**{gap:.1f}x slower** (oversubscribed)"
        elif policy == 'mixed':
            notes = "8P + 2E explicit"
        elif policy == 'fast':
            notes = "All 10 threads P-biased"
        else:
            notes = "OS uses all cores"
        
        print(f"| {policy.capitalize():10} | {sps:.2f} | {relative:.0f}% | {notes} |")
    
    # Key findings
    print("\n## Key Findings\n")
    
    gap_8t = results['8t']['none'] / results['8t']['efficient'] if results['8t']['efficient'] > 0 else 0
    gap_10t = results['10t']['none'] / results['10t']['efficient'] if results['10t']['efficient'] > 0 else 0
    
    print(f"1. **E-cores are {gap_8t:.1f}x slower** at 8 threads (clean comparison)")
    print(f"2. **E-cores are {gap_10t:.1f}x slower** at 10 threads (oversubscribed)")
    
    mixed_8t_expected = 0.8 * results['8t']['fast'] + 0.2 * results['8t']['efficient']
    mixed_8t_actual = results['8t']['mixed']
    mixed_8t_ratio = (mixed_8t_actual / mixed_8t_expected * 100) if mixed_8t_expected > 0 else 0
    
    print(f"3. **Mixed policy achieves {mixed_8t_ratio:.0f}% of expected** (critical-path limited)")
    
    improvement_none = (results['10t']['none'] / results['8t']['none'] - 1) * 100 if results['8t']['none'] > 0 else 0
    print(f"4. **10 threads vs 8 threads: {improvement_none:+.1f}%** for baseline policy")

if __name__ == "__main__":
    analyze_hetero_results()
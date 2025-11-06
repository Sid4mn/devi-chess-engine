#!/usr/bin/env python3
import matplotlib.pyplot as plt
import sys
import re
import os

def extract_metric(filename):
    try:
        with open(filename, 'r') as f:
            content = f.read()
            match = re.search(r'Searches/second:\s+([\d.]+)', content)
            if match:
                return float(match.group(1))
    except:
        pass
    return None

def main():
    os.makedirs('../benchmarks', exist_ok=True)
    
    policies = ['none', 'fast', 'efficient', 'mixed']
    results = {}
    
    for policy in policies:
        if policy == 'mixed':
            filename = f'../benchmarks/policy_{policy}_75.txt'
        else:
            filename = f'../benchmarks/policy_{policy}.txt'
        
        results[policy] = extract_metric(filename)
    
    if not all(results.values()):
        print("Missing data:", [k for k,v in results.items() if v is None])
        sys.exit(1)
    
    baseline = results['none']
    relative_perf = {p: (v/baseline * 100) for p, v in results.items()}
    
    # make the plot
    fig, ax = plt.subplots(figsize=(11, 7))
    
    values = [results[p] for p in policies]
    colors = ['#6B7280', '#10B981', '#EF4444', '#3B82F6']
    
    bars = ax.bar(range(len(values)), values, color=colors, edgecolor='black', linewidth=2)
    
    # labels on bars
    for i, (policy, val) in enumerate(zip(policies, values)):
        height = bars[i].get_height()
        if policy == 'efficient':
            label = f'{val:.3f}\n({relative_perf[policy]:.1f}%)'
        else:
            label = f'{val:.2f}\n({relative_perf[policy]:.0f}%)'
        ax.text(i, height + max(values)*0.02, label, ha='center', va='bottom', fontweight='bold')
    
    # x-axis
    labels_text = ['None\n(baseline)', 'FastBias\n(P-cores)', 'EfficientBias\n(E-cores)', 'Mixed\n(75% P)']
    ax.set_xticks(range(len(labels_text)))
    ax.set_xticklabels(labels_text)
    
    # arrow showing the difference
    if results['fast'] and results['efficient']:
        ratio = results['fast'] / results['efficient']
        ax.annotate(f'{ratio:.0f}x slower!', 
                    xy=(2, results['efficient']), 
                    xytext=(1.5, max(values) * 0.4),
                    arrowprops=dict(arrowstyle='->', color='red', lw=3),
                    fontsize=12, fontweight='bold', color='red')
    
    ax.axhline(y=baseline, color='gray', linestyle='--', alpha=0.4)
    ax.set_ylabel('Searches per Second', fontsize=14, fontweight='bold')
    ax.set_title('Core Scheduling Impact on Chess Search Performance\nM1 Pro (8P+2E), Depth 7, 8 threads', fontsize=15, fontweight='bold')
    ax.grid(axis='y', alpha=0.3)
    ax.set_ylim(0, max(values) * 1.15)
    
    plt.tight_layout()
    plt.savefig('../benchmarks/heterogeneous_impact.png', dpi=150, bbox_inches='tight')
    
    # minimal output
    print(f"P-cores {ratio:.0f}x faster than E-cores")
    print("Graph: ../benchmarks/heterogeneous_impact.png")

if __name__ == '__main__':
    main()
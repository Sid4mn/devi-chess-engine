import matplotlib.pyplot as plt
import os
import sys
import pandas as pd


def read_speedup_data():
    """Read speedup data from CSV file"""
    
    csv_path = "../benchmarks/speedup.csv"

    if not os.path.exists(csv_path):
        print(f"Error: {csv_path} not found. Run benchmark first:")
        print(f" /target/release/devi --benchmark")
        sys.exit(1)
    
    try:
        df = pd.read_csv(csv_path)
        if 'policy' in df.columns:
            unique_policies = df['policy'].unique()
            
            if len(unique_policies) > 1:
                if 'None' in unique_policies:
                    df = df[df['policy'] == 'None']
                    print("Using policy: None (baseline)")
                elif 'FastBias' in unique_policies:
                    df = df[df['policy'] == 'FastBias']
                    print("Using policy: FastBias (P-cores)")
                else:
                    df = df[df['policy'] == unique_policies[0]]
                    print(f"Using policy: {unique_policies[0]}")
            else:
                print(f"Single policy found: {unique_policies[0]}")
        
        # Verify required columns exist
        if 'threads' not in df.columns or 'speedup' not in df.columns:
            print(f"Error: Missing required columns")
            print(f"Found columns: {df.columns.tolist()}")
            sys.exit(1)
        
        threads = df['threads'].tolist()
        speedup = df['speedup'].tolist()
        return threads, speedup

    except Exception as e:
        print(f"Error reading {csv_path}: {e}")
        sys.exit(1)


def perf_vis():

    threads, actual_speedup = read_speedup_data()
    linear_theoretical = threads


    plt.figure(figsize=(10, 6))
    
    plt.plot(threads, actual_speedup, 'o-', linewidth=3, markersize=10, color='#2E86C1', label='devi Engine (M1 Pro)')
    
    plt.plot(threads, linear_theoretical, '--', linewidth=2, color='#E74C3C', alpha=0.7, label='Perfect Linear Scaling')
    
    plt.xlabel('Thread Count', fontsize=12)
    plt.ylabel('Speedup Factor', fontsize=12)
    plt.title('devi Chess Engine - Parallel Speedup\nApple M1 Pro (10-core: 8P+2E)', fontsize=14)
    
    plt.grid(True, alpha=0.3)
    plt.legend(fontsize=11)

    plt.xlim(0.5, 10.5)
    plt.ylim(0, 11)

    max_speedup = max(actual_speedup)
    max_threads = threads[actual_speedup.index(max_speedup)]
    max_efficiency = (max_speedup / max_threads) * 100


    plt.annotate(
        f'Peak: {max_speedup:.2f}x\n{max_efficiency:.1f}% efficiency', 
        xy=(10, max_speedup), 
        xytext=(8, max_speedup + 1.5),  # Higher
        arrowprops=dict(arrowstyle='->', color='green'),
        fontsize=9, fontweight='bold',
        bbox=dict(boxstyle="round,pad=0.2", facecolor="lightgreen", alpha=0.7)
)
    
    if 10 in threads:
        idx_10 = threads.index(10)
        speedup_10 = actual_speedup[idx_10]
        efficiency_10 = (speedup_10 / 10) * 100
        
        status = "Good" if efficiency_10 > 50 else "Limited"
        
        plt.annotate(f'Full utilization\n{efficiency_10:.1f}% efficiency ({status})', 
                    xy=(10, max_speedup), 
                    xytext=(8, max_speedup - 1.5),  # Lower
                    arrowprops=dict(arrowstyle='->', color='blue'),
                    fontsize=9, style='italic',
                    bbox=dict(boxstyle="round,pad=0.2", facecolor="lightblue", alpha=0.6)
                    )
        

    os.makedirs('../benchmarks', exist_ok=True)
    plt.savefig('../benchmarks/speedup.png', dpi=150, bbox_inches='tight')
    plt.savefig('../benchmarks/speedup_hires.png', dpi=300, bbox_inches='tight')
    
    print("Speedup graphs saved to benchmarks/")
    
    print("\n=== Performance Analysis ===")
    for t, s in zip(threads, actual_speedup):
        efficiency = (s / t) * 100
        status = "Excellent" if efficiency > 80 else "Good" if efficiency > 50 else "Fair"
        print(f"{t} threads: {s:.2f}x speedup, {efficiency:.1f}% efficiency ({status})")
    
    print(f"\nPeak speedup: {max_speedup:.2f}x on {max_threads} threads")

    if 4 in threads:
        idx_4 = threads.index(4)
        speedup_4 = actual_speedup[idx_4]
        efficiency_4 = (speedup_4 / 4) * 100
        print(f"4-thread performance: {speedup_4:.2f}x speedup ({efficiency_4:.1f}% efficiency)")
    
    if 10 in threads:
        idx_10 = threads.index(10)
        speedup_10 = actual_speedup[idx_10]
        efficiency_10 = (speedup_10 / 10) * 100
        print(f"10-thread (full): {speedup_10:.2f}x speedup ({efficiency_10:.1f}% efficiency)")

if __name__ == "__main__":
    perf_vis()
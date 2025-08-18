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
    plt.title('devi Chess Engine - Parallel Speedup\nApple M1 Pro (8-core)', fontsize=14)
    
    plt.grid(True, alpha=0.3)
    plt.legend(fontsize=11)

    plt.xlim(0.5, 8.5)
    plt.ylim(0, 9)

    max_speedup = max(actual_speedup)
    max_threads = threads[actual_speedup.index(max_speedup)]
    efficiency = (max_speedup / max_threads) * 100

    plt.annotate(f'Strong scaling!\n{max_speedup:.2f}x on {max_threads} threads', 
                xy=(max_threads, max_speedup), xytext=(max_threads-1.5, max_speedup+0.5),
                arrowprops=dict(arrowstyle='->', color='green'),
                fontsize=10, fontweight='bold',
                bbox=dict(boxstyle="round,pad=0.3", facecolor="lightgreen", alpha=0.7))
    
    if 4 in threads:
        idx_4 = threads.index(4)
        speedup_4 = actual_speedup[idx_4]
        efficiency_4 = (speedup_4 / 4) * 100


        plt.annotate(f'Good scaling\n{efficiency_4:.1f}% efficiency', 
                    xy=(4, speedup_4), xytext=(2.5, speedup_4+0.5),
                    arrowprops=dict(arrowstyle='->', color='blue'),
                    fontsize=9, style='italic',
                    bbox=dict(boxstyle="round,pad=0.2", facecolor="lightblue", alpha=0.6))
        

    os.makedirs('../benchmarks', exist_ok=True)
    plt.savefig('../benchmarks/speedup.png', dpi=150, bbox_inches='tight')
    plt.savefig('../benchmarks/speedup_hires.png', dpi=300, bbox_inches='tight')
    
    print("Speedup graphs saved to benchmarks/")
    
    print("\n=== Performance Analysis ===")
    for t, s in zip(threads, actual_speedup):
        efficiency = (s / t) * 100
        status = "Excellent" if efficiency > 80 else "Good" if efficiency > 50 else "Fair"
        print(f"{t} threads: {s:.2f}x speedup, {efficiency:.1f}% efficiency ({status})")
    
    print(f"\nPeak speedup: {max(actual_speedup):.2f}x on {threads[actual_speedup.index(max(actual_speedup))]} threads")

    if len(threads) > 1:
        print(f"Best efficiency: 2 threads ({(actual_speedup[1]/2)*100:.1f}% - near-linear scaling)")
    if 4 in threads:
        print(f"4-thread performance: {actual_speedup[idx_4]:.2f}x speedup ({(actual_speedup[idx_4]/4)*100:.1f}% efficiency)")
    
    print(f"Strong {max_threads}-thread performance: {max_speedup:.2f}x speedup demonstrates effective parallelization")
    
    plt.show()

if __name__ == "__main__":
    perf_vis()
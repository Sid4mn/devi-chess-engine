import matplotlib.pyplot as plt
import os

def perf_vis():
    threads = [1, 2, 4, 8]
    actual_speedup = [1.00, 1.81, 3.30, 4.69]
    linear_theoretical = [1, 2, 4, 8]

    plt.figure(figsize=(10, 6))
    
    plt.plot(threads, actual_speedup, 'o-', linewidth=3, markersize=10, 
             color='#2E86C1', label='devi Engine (M1 Pro)')
    
    plt.plot(threads, linear_theoretical, '--', linewidth=2, 
             color='#E74C3C', alpha=0.7, label='Perfect Linear Scaling')
    
    plt.xlabel('Thread Count', fontsize=12)
    plt.ylabel('Speedup Factor', fontsize=12)
    plt.title('devi Chess Engine - Parallel Speedup\nApple M1 Pro (8-core)', fontsize=14)
    
    plt.grid(True, alpha=0.3)
    plt.legend(fontsize=11)

    plt.xlim(0.5, 8.5)
    plt.ylim(0, 9)
    
    plt.annotate('Strong scaling!\n4.69x on 8 threads', 
                xy=(8, 4.69), xytext=(6.5, 5.5),
                arrowprops=dict(arrowstyle='->', color='green'),
                fontsize=10, fontweight='bold',
                bbox=dict(boxstyle="round,pad=0.3", facecolor="lightgreen", alpha=0.7))
    
    plt.annotate('Good scaling\n82.6% efficiency', 
                xy=(4, 3.30), xytext=(2.5, 3.8),
                arrowprops=dict(arrowstyle='->', color='blue'),
                fontsize=9, style='italic',
                bbox=dict(boxstyle="round,pad=0.2", facecolor="lightblue", alpha=0.6))
    
    os.makedirs('../benchmarks', exist_ok=True)
    plt.savefig('../benchmarks/speedup.png', dpi=150, bbox_inches='tight')
    plt.savefig('../benchmarks/speedup_hires.png', dpi=300, bbox_inches='tight')
    
    print("✅ Speedup graphs saved to benchmarks/")
    
    print("\n=== Performance Analysis ===")
    for t, s in zip(threads, actual_speedup):
        efficiency = (s / t) * 100
        status = "Excellent" if efficiency > 80 else "Good" if efficiency > 50 else "Fair"
        print(f"{t} threads: {s:.2f}x speedup, {efficiency:.1f}% efficiency ({status})")
    
        print(f"\nPeak speedup: {max(actual_speedup):.2f}x on {threads[actual_speedup.index(max(actual_speedup))]} threads")
    print(f"Best efficiency: 2 threads ({(actual_speedup[1]/2)*100:.1f}% - near-linear scaling)")
    print(f"4-thread performance: {actual_speedup[2]:.2f}x speedup ({(actual_speedup[2]/4)*100:.1f}% efficiency)")
    print(f"Strong 8-thread performance: {actual_speedup[3]:.2f}x speedup demonstrates effective parallelization")
    
    plt.show()

if __name__ == "__main__":
    perf_vis()
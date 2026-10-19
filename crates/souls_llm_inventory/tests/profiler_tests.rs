use souls_llm_inventory::profiler::{
    estimate_throughput_tps, evaluate_model_fit, HardwareDiagnostic, HardwareFit,
};

#[test]
fn test_agnostic_profiling_across_gpu_scales() {
    // Case 1: Entry GPU (4GB VRAM, e.g. GTX 1650)
    let gpu_4gb = HardwareDiagnostic {
        has_dedicated_gpu: true,
        gpu_name: "GTX 1650".to_string(),
        vram_total_mb: 4096,
        vram_free_mb: 3200,
        vram_safe_usable_mb: 3246, // 4096 - 850
        host_ram_total_mb: 16384,
        host_ram_available_mb: 12000,
        host_ram_safe_usable_mb: 8928,
        cpu_cores_physical: 4,
        cpu_cores_logical: 8,
        has_avx2: true,
        has_avx512: false,
    };

    // 1.5B Q5_K_M (~1.15 GB) -> fits in 4GB GPU
    let fit_1_5b = evaluate_model_fit(&gpu_4gb, 1150 * 1024 * 1024, 8192, 1_500_000_000);
    assert_eq!(fit_1_5b, HardwareFit::OptimalVram);

    // 7B Q4_K_M (~4.35 GB) -> exceeds 3.2GB VRAM, but fits in 8.9GB Host RAM
    let fit_7b = evaluate_model_fit(&gpu_4gb, 4350 * 1024 * 1024, 8192, 7_000_000_000);
    assert_eq!(fit_7b, HardwareFit::PartialCpuOffload);

    // Case 2: Mid-tier GPU (12GB VRAM, e.g. RTX 4070)
    let gpu_12gb = HardwareDiagnostic {
        has_dedicated_gpu: true,
        gpu_name: "RTX 4070".to_string(),
        vram_total_mb: 12288,
        vram_free_mb: 11000,
        vram_safe_usable_mb: 10813, // 12288 - 1475 (12%)
        host_ram_total_mb: 32768,
        host_ram_available_mb: 26000,
        host_ram_safe_usable_mb: 22928,
        cpu_cores_physical: 8,
        cpu_cores_logical: 16,
        has_avx2: true,
        has_avx512: false,
    };

    // 7B Q8_0 (~7.7 GB) -> fits in 12GB GPU OptimalVram!
    let fit_7b_q8 = evaluate_model_fit(&gpu_12gb, 7700 * 1024 * 1024, 8192, 7_000_000_000);
    assert_eq!(fit_7b_q8, HardwareFit::OptimalVram);

    // Case 3: CPU-only Host (No dedicated GPU, 32GB RAM)
    let cpu_only = HardwareDiagnostic {
        has_dedicated_gpu: false,
        gpu_name: "None (CPU Execution)".to_string(),
        vram_total_mb: 0,
        vram_free_mb: 0,
        vram_safe_usable_mb: 0,
        host_ram_total_mb: 32768,
        host_ram_available_mb: 24000,
        host_ram_safe_usable_mb: 20928,
        cpu_cores_physical: 8,
        cpu_cores_logical: 16,
        has_avx2: true,
        has_avx512: false,
    };

    // Model must classify as PartialCpuOffload on CPU-only host
    let fit_cpu = evaluate_model_fit(&cpu_only, 4350 * 1024 * 1024, 8192, 7_000_000_000);
    assert_eq!(fit_cpu, HardwareFit::PartialCpuOffload);

    // Extreme model (40GB) exceeds even safe host RAM
    let fit_extreme = evaluate_model_fit(&cpu_only, 40000 * 1024 * 1024, 8192, 70_000_000_000);
    assert_eq!(fit_extreme, HardwareFit::IncompatibleSpillover);
}

#[test]
fn test_throughput_projections() {
    let tps_vram = estimate_throughput_tps(HardwareFit::OptimalVram, 7_000_000_000, true);
    assert!(tps_vram >= 25.0);

    let tps_cpu = estimate_throughput_tps(HardwareFit::PartialCpuOffload, 7_000_000_000, true);
    assert!(tps_cpu >= 8.0 && tps_cpu < 20.0);

    let tps_spillover = estimate_throughput_tps(HardwareFit::IncompatibleSpillover, 7_000_000_000, true);
    assert!(tps_spillover < 2.0);
}

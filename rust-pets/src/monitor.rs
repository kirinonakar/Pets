use nvml_wrapper::Nvml;
use sysinfo::System;

pub struct SystemStats {
    pub cpu_usage: f32,
    pub ram_usage_pct: f32,
    pub gpu_usage: Option<f32>,
    pub gpu_mem_pct: Option<f32>,
}

pub struct Monitor {
    sys: System,
    nvml: Option<Nvml>,
}

impl Monitor {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        let nvml = Nvml::init().ok();

        Self { sys, nvml }
    }

    pub fn update(&mut self) -> SystemStats {
        self.sys.refresh_cpu();
        self.sys.refresh_memory();

        let cpu_usage = self.sys.global_cpu_info().cpu_usage();
        let total_mem = self.sys.total_memory() as f32;
        let used_mem = self.sys.used_memory() as f32;
        let ram_usage_pct = (used_mem / total_mem) * 100.0;

        let mut gpu_usage = None;
        let mut gpu_mem_pct = None;

        if let Some(nvml) = &self.nvml {
            if let Ok(device) = nvml.device_by_index(0) {
                if let Ok(util) = device.utilization_rates() {
                    gpu_usage = Some(util.gpu as f32);
                }
                if let Ok(mem) = device.memory_info() {
                    gpu_mem_pct = Some((mem.used as f32 / mem.total as f32) * 100.0);
                }
            }
        }

        SystemStats {
            cpu_usage,
            ram_usage_pct,
            gpu_usage,
            gpu_mem_pct,
        }
    }
}

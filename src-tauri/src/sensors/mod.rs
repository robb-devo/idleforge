use serde::Serialize;
use sysinfo::System;

#[derive(Debug, Clone, Serialize)]
pub struct HardwareInfo {
    pub cpu_name: String,
    pub gpu_name: String,
    pub cpu_cores: u32,
    pub ram_gb: Option<f64>,
    pub on_battery: Option<bool>,
    pub sensors_available: bool,
    pub note: String,
}

#[derive(Debug, Clone, Default)]
pub struct SensorReading {
    pub system_cpu_percent: Option<f64>,
    pub cpu_temp_c: Option<f64>,
    pub gpu_temp_c: Option<f64>,
    pub cpu_power_w: Option<f64>,
    pub gpu_power_w: Option<f64>,
    pub on_battery: Option<bool>,
}

pub struct SensorHub {
    sys: System,
}

impl SensorHub {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self { sys }
    }

    pub fn hardware(&mut self) -> HardwareInfo {
        self.sys.refresh_cpu_all();
        self.sys.refresh_memory();

        let cpu_name = self
            .sys
            .cpus()
            .first()
            .map(|c| c.brand().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "CPU (unbekannt)".into());

        let gpu_name = detect_gpu_name().unwrap_or_else(|| {
            "GPU (nicht erkannt — Treiber/Tools fehlen, graceful degradation)".into()
        });

        let ram_gb = Some(self.sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0));
        let on_battery = detect_on_battery();

        HardwareInfo {
            cpu_name,
            gpu_name,
            cpu_cores: self.sys.cpus().len() as u32,
            ram_gb,
            on_battery,
            sensors_available: cfg!(windows) || true,
            note: if cfg!(windows) {
                "Windows-Sensoren: CPU über sysinfo; GPU-Temp/Leistung best-effort (NVAPI/nvidia-smi falls verfügbar)."
                    .into()
            } else {
                "Nicht-Windows-Host: Sensoren degradiert. IdleForge zielt auf Windows-Desktops."
                    .into()
            },
        }
    }

    pub fn read(&mut self) -> SensorReading {
        self.sys.refresh_cpu_all();
        let cpu = Some(self.sys.global_cpu_usage() as f64);
        let (gpu_temp, gpu_power) = read_nvidia_smi();

        SensorReading {
            system_cpu_percent: cpu,
            cpu_temp_c: None, // Windows WMI/LibreHardwareMonitor can be wired later
            gpu_temp_c: gpu_temp,
            cpu_power_w: None,
            gpu_power_w: gpu_power,
            on_battery: detect_on_battery(),
        }
    }
}

fn detect_gpu_name() -> Option<String> {
    #[cfg(windows)]
    {
        if let Some(name) = read_nvidia_smi_name() {
            return Some(name);
        }
    }
    read_nvidia_smi_name()
}

fn read_nvidia_smi_name() -> Option<String> {
    let output = std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=name", "--format=csv,noheader"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s.lines().next().unwrap_or(&s).trim().to_string())
    }
}

fn read_nvidia_smi() -> (Option<f64>, Option<f64>) {
    let output = std::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=temperature.gpu,power.draw",
            "--format=csv,noheader,nounits",
        ])
        .output();
    let Ok(output) = output else {
        return (None, None);
    };
    if !output.status.success() {
        return (None, None);
    }
    let line = String::from_utf8_lossy(&output.stdout);
    let mut parts = line.split(',').map(|p| p.trim());
    let temp = parts.next().and_then(|v| v.parse::<f64>().ok());
    let power = parts.next().and_then(|v| v.parse::<f64>().ok());
    (temp, power)
}

fn detect_on_battery() -> Option<bool> {
    #[cfg(windows)]
    {
        // Best-effort via PowerShell; None on failure.
        let output = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "(Get-CimInstance -ClassName BatteryStatus -Namespace root\\wmi -ErrorAction SilentlyContinue).PowerOnline",
            ])
            .output()
            .ok()?;
        let s = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
        if s.contains("true") {
            return Some(false);
        }
        if s.contains("false") {
            return Some(true);
        }
        return None;
    }
    #[cfg(not(windows))]
    {
        None
    }
}

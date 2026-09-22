//! Hardware sensors.
//!
//! CPU load comes from `sysinfo` (no child process).
//! AC vs battery uses `GetSystemPowerStatus` on Windows — never PowerShell.
//! `nvidia-smi` is optional, hidden, and cached for 20s. If it is missing,
//! it is not retried.

use crate::process_util::new_hidden;
use serde::Serialize;
use std::time::{Duration, Instant};
use sysinfo::System;

const CACHE_TTL: Duration = Duration::from_secs(20);

#[derive(Debug, Clone, Serialize)]
pub struct HardwareInfo {
    pub cpu_name: String,
    pub gpu_name: String,
    pub cpu_cores: u32,
    pub ram_gb: Option<f64>,
    pub on_battery: Option<bool>,
    pub sensors_available: bool,
    pub system_cpu_percent: Option<f64>,
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
    gpu_name: Option<String>,
    gpu_name_resolved: bool,
    battery: Option<bool>,
    battery_at: Option<Instant>,
    nvidia: (Option<f64>, Option<f64>),
    nvidia_at: Option<Instant>,
    nvidia_missing: bool,
    cpu_sample: Option<f64>,
    cpu_sample_at: Option<Instant>,
}

impl SensorHub {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self {
            sys,
            gpu_name: None,
            gpu_name_resolved: false,
            battery: None,
            battery_at: None,
            nvidia: (None, None),
            nvidia_at: None,
            nvidia_missing: false,
            cpu_sample: None,
            cpu_sample_at: None,
        }
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

        let gpu_name = self.gpu_name();
        let ram_gb = Some(self.sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0));
        let on_battery = self.battery_cached();
        let cpu_usage = self.sys.global_cpu_usage();
        let system_cpu_percent = if cpu_usage.is_finite() {
            Some(cpu_usage as f64)
        } else {
            None
        };

        HardwareInfo {
            cpu_name,
            gpu_name,
            cpu_cores: self.sys.cpus().len() as u32,
            ram_gb,
            on_battery,
            sensors_available: true,
            system_cpu_percent,
            note: if cfg!(windows) {
                "Echte Hardware: CPU über sysinfo, Netzteil über GetSystemPowerStatus, GPU über nvidia-smi (versteckt, gecacht) oder EnumDisplayDevices. Keine PowerShell-Konsole."
                    .into()
            } else {
                "Tauri-Sensoren: CPU-Name ist echt. GPU-Name braucht nvidia-smi. Diese Umgebung ist nicht das Windows-Ziel."
                    .into()
            },
        }
    }

    pub fn read(&mut self) -> SensorReading {
        let cpu = self.cpu_cached();
        let (gpu_temp, gpu_power) = self.nvidia_cached();

        SensorReading {
            system_cpu_percent: cpu,
            cpu_temp_c: None,
            gpu_temp_c: gpu_temp,
            cpu_power_w: None,
            gpu_power_w: gpu_power,
            on_battery: self.battery_cached(),
        }
    }

    fn gpu_name(&mut self) -> String {
        if !self.gpu_name_resolved {
            self.gpu_name = detect_gpu_name();
            self.gpu_name_resolved = true;
        }
        self.gpu_name.clone().unwrap_or_else(|| {
            "GPU (nicht erkannt — Treiber/Tools fehlen, graceful degradation)".into()
        })
    }

    fn cpu_cached(&mut self) -> Option<f64> {
        let fresh = self
            .cpu_sample_at
            .map(|t| t.elapsed() < Duration::from_secs(2))
            .unwrap_or(false);
        if !fresh {
            self.sys.refresh_cpu_all();
            let usage = self.sys.global_cpu_usage() as f64;
            self.cpu_sample = if usage.is_finite() { Some(usage) } else { None };
            self.cpu_sample_at = Some(Instant::now());
        }
        self.cpu_sample
    }

    fn battery_cached(&mut self) -> Option<bool> {
        let fresh = self
            .battery_at
            .map(|t| t.elapsed() < CACHE_TTL)
            .unwrap_or(false);
        if !fresh {
            self.battery = query_on_battery();
            self.battery_at = Some(Instant::now());
        }
        self.battery
    }

    fn nvidia_cached(&mut self) -> (Option<f64>, Option<f64>) {
        if self.nvidia_missing {
            return (None, None);
        }
        let fresh = self
            .nvidia_at
            .map(|t| t.elapsed() < CACHE_TTL)
            .unwrap_or(false);
        if fresh {
            return self.nvidia;
        }
        match read_nvidia_smi() {
            NvidiaProbe::Missing => {
                self.nvidia_missing = true;
                self.nvidia = (None, None);
                self.nvidia_at = Some(Instant::now());
            }
            NvidiaProbe::Reading(temp, power) => {
                self.nvidia = (temp, power);
                self.nvidia_at = Some(Instant::now());
            }
        }
        self.nvidia
    }
}

enum NvidiaProbe {
    Missing,
    Reading(Option<f64>, Option<f64>),
}

fn detect_gpu_name() -> Option<String> {
    if let Some(name) = read_nvidia_smi_name() {
        return Some(name);
    }
    read_gpu_display_devices()
}

fn read_nvidia_smi_name() -> Option<String> {
    let output = new_hidden("nvidia-smi")
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

fn read_nvidia_smi() -> NvidiaProbe {
    let output = new_hidden("nvidia-smi")
        .args([
            "--query-gpu=temperature.gpu,power.draw",
            "--format=csv,noheader,nounits",
        ])
        .output();
    match output {
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => NvidiaProbe::Missing,
        Err(_) => NvidiaProbe::Reading(None, None),
        Ok(output) if !output.status.success() => NvidiaProbe::Reading(None, None),
        Ok(output) => {
            let line = String::from_utf8_lossy(&output.stdout);
            let mut parts = line.split(',').map(|p| p.trim());
            let temp = parts.next().and_then(|v| v.parse::<f64>().ok());
            let power = parts.next().and_then(|v| v.parse::<f64>().ok());
            NvidiaProbe::Reading(temp, power)
        }
    }
}

/// `Some(true)` = on battery, `Some(false)` = AC mains, `None` = unknown.
fn query_on_battery() -> Option<bool> {
    #[cfg(windows)]
    {
        query_on_battery_win()
    }
    #[cfg(not(windows))]
    {
        None
    }
}

#[cfg(windows)]
fn query_on_battery_win() -> Option<bool> {
    #[repr(C)]
    struct SystemPowerStatus {
        ac_line_status: u8,
        _battery_flag: u8,
        _battery_life_percent: u8,
        _system_status_flag: u8,
        _battery_life_time: u32,
        _battery_full_life_time: u32,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetSystemPowerStatus(status: *mut SystemPowerStatus) -> i32;
    }

    unsafe {
        let mut status = std::mem::zeroed::<SystemPowerStatus>();
        if GetSystemPowerStatus(&mut status) == 0 {
            return None;
        }
        match status.ac_line_status {
            0 => Some(true),
            1 => Some(false),
            _ => None,
        }
    }
}

fn read_gpu_display_devices() -> Option<String> {
    #[cfg(windows)]
    {
        read_gpu_display_devices_win()
    }
    #[cfg(not(windows))]
    {
        None
    }
}

#[cfg(windows)]
fn read_gpu_display_devices_win() -> Option<String> {
    #[repr(C)]
    #[allow(dead_code)]
    struct DisplayDeviceW {
        cb: u32,
        device_name: [u16; 32],
        device_string: [u16; 128],
        state_flags: u32,
        device_id: [u16; 128],
        device_key: [u16; 128],
    }

    #[link(name = "user32")]
    extern "system" {
        fn EnumDisplayDevicesW(
            device: *const u16,
            dev_num: u32,
            display_device: *mut DisplayDeviceW,
            flags: u32,
        ) -> i32;
    }

    let mut names = Vec::new();
    for i in 0..8 {
        let mut device = unsafe { std::mem::zeroed::<DisplayDeviceW>() };
        device.cb = std::mem::size_of::<DisplayDeviceW>() as u32;
        let ok = unsafe { EnumDisplayDevicesW(std::ptr::null(), i, &mut device, 0) };
        if ok == 0 {
            break;
        }
        let raw = String::from_utf16_lossy(&device.device_string);
        let name = raw.trim_end_matches('\0').trim().to_string();
        if !name.is_empty() && !names.iter().any(|n: &String| n == &name) {
            names.push(name);
        }
    }
    if names.is_empty() {
        None
    } else {
        Some(names.join(" · "))
    }
}

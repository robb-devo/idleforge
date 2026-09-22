use super::adapter::{MinerAdapter, MinerKind, MinerStats, StartRequest};
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// XMRig adapter — spawns external binary, prefers HTTP API, falls back to stdout hints.
pub struct XmrigAdapter {
    child: Option<Child>,
    api_port: u16,
    started_at: Option<Instant>,
    last_error: Option<String>,
    accepted: u64,
    rejected: u64,
    last_hashrate: f64,
    log_tail: Arc<Mutex<Vec<String>>>,
}

impl XmrigAdapter {
    pub fn new() -> Self {
        Self {
            child: None,
            api_port: 18088,
            started_at: None,
            last_error: None,
            accepted: 0,
            rejected: 0,
            last_hashrate: 0.0,
            log_tail: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn reap(&mut self) {
        if let Some(child) = self.child.as_mut() {
            if let Ok(Some(status)) = child.try_wait() {
                if !status.success() {
                    self.last_error = Some(format!("XMRig beendet: {status}"));
                }
                self.child = None;
            }
        }
    }

    fn fetch_api(&mut self) -> Option<MinerStats> {
        let url = format!("http://127.0.0.1:{}/2/summary", self.api_port);
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_millis(800))
            .build()
            .ok()?;
        let resp = client.get(&url).send().ok()?;
        let json: serde_json::Value = resp.json().ok()?;
        let hashrate = json
            .pointer("/hashrate/total/0")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let accepted = json
            .pointer("/results/shares_good")
            .and_then(|v| v.as_u64())
            .unwrap_or(self.accepted);
        let total = json
            .pointer("/results/shares_total")
            .and_then(|v| v.as_u64())
            .unwrap_or(accepted);
        self.accepted = accepted;
        self.rejected = total.saturating_sub(accepted);
        self.last_hashrate = hashrate;
        Some(MinerStats {
            hashrate_hs: hashrate,
            utilization_percent: None,
            temperature_c: None,
            power_w: None,
            accepted_shares: self.accepted,
            rejected_shares: self.rejected,
        })
    }
}

impl MinerAdapter for XmrigAdapter {
    fn id(&self) -> &'static str {
        "xmrig"
    }

    fn kind(&self) -> MinerKind {
        MinerKind::Cpu
    }

    fn start(&mut self, req: &StartRequest) -> Result<(), String> {
        self.stop()?;
        self.api_port = req.api_port;
        self.last_error = None;

        let intensity = crate::safety::clamp_ratio(req.intensity);
        if req.mock_mode {
            self.started_at = Some(Instant::now());
            self.last_hashrate = 5500.0 * intensity;
            return Ok(());
        }

        if !std::path::Path::new(&req.binary_path).exists() {
            return Err(format!(
                "XMRig nicht gefunden: {}. Pfad in der Config setzen oder Mock-Modus nutzen.",
                req.binary_path
            ));
        }

        let threads = crate::safety::capped_thread_count(num_cpus_approx(), req.threads_ratio);

        let mut cmd = Command::new(&req.binary_path);
        cmd.arg("-o")
            .arg(&req.pool_url)
            .arg("-u")
            .arg(&req.user)
            .arg("-p")
            .arg(&req.pass)
            .arg("--algo")
            .arg("rx/0")
            .arg("--threads")
            .arg(threads.to_string())
            // Below normal so the desktop stays usable even at the 95% cap.
            .arg("--cpu-priority")
            .arg("1")
            .arg("--http-host")
            .arg("127.0.0.1")
            .arg("--http-port")
            .arg(req.api_port.to_string())
            .arg("--donate-level")
            .arg("1")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        for a in &req.extra_args {
            cmd.arg(a);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("XMRig start fehlgeschlagen: {e}"))?;

        if let Some(stdout) = child.stdout.take() {
            let tail = Arc::clone(&self.log_tail);
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().flatten() {
                    if let Ok(mut g) = tail.lock() {
                        g.push(line);
                        if g.len() > 200 {
                            g.remove(0);
                        }
                    }
                }
            });
        }

        self.child = Some(child);
        self.started_at = Some(Instant::now());
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.started_at = None;
        self.last_hashrate = 0.0;
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.started_at.is_some() && (self.child.is_some() || self.last_hashrate > 0.0)
    }

    fn poll_stats(&mut self) -> Result<MinerStats, String> {
        self.reap();
        if self.started_at.is_none() {
            return Ok(MinerStats {
                hashrate_hs: 0.0,
                utilization_percent: None,
                temperature_c: None,
                power_w: None,
                accepted_shares: self.accepted,
                rejected_shares: self.rejected,
            });
        }

        if let Some(stats) = self.fetch_api() {
            return Ok(stats);
        }

        // Mock / API-unavailable fallback keeps last known rate with mild jitter.
        let jitter = 0.97 + ((Instant::now().elapsed().as_millis() % 100) as f64) * 0.0006;
        Ok(MinerStats {
            hashrate_hs: self.last_hashrate * jitter,
            utilization_percent: None,
            temperature_c: None,
            power_w: None,
            accepted_shares: self.accepted,
            rejected_shares: self.rejected,
        })
    }

    fn last_error(&self) -> Option<String> {
        self.last_error.clone()
    }
}

fn num_cpus_approx() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

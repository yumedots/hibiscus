use std::time::Duration;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EnergySample {
    pub(crate) power_milliwatts: i64,
    pub(crate) charge_percent: Option<i64>,
    pub(crate) minutes_remaining: Option<i64>,
    pub(crate) charging: bool,
}

impl EnergySample {
    pub(crate) fn power_watts(&self) -> f64 {
        self.power_milliwatts.unsigned_abs() as f64 / 1_000.0
    }

    pub(crate) fn consumed_milliwatt_hours(&self, interval: Duration) -> f64 {
        self.power_milliwatts.unsigned_abs() as f64 * interval.as_secs_f64() / 3_600_000.0
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn sample() -> Option<EnergySample> {
    let output = std::process::Command::new("/usr/sbin/ioreg")
        .args(["-r", "-c", "AppleSmartBattery", "-l"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    parse(std::str::from_utf8(&output.stdout).ok()?)
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn sample() -> Option<EnergySample> {
    None
}

#[cfg(target_os = "macos")]
fn parse(output: &str) -> Option<EnergySample> {
    let voltage = integer(output, "AppleRawBatteryVoltage")?;
    let amperage = integer(output, "InstantAmperage")?;
    Some(EnergySample {
        power_milliwatts: voltage.saturating_mul(amperage) / 1_000,
        charge_percent: integer(output, "CurrentCapacity"),
        minutes_remaining: integer(output, "TimeRemaining").filter(|minutes| *minutes > 0),
        charging: output.contains("\"IsCharging\" = Yes"),
    })
}

#[cfg(target_os = "macos")]
fn integer(output: &str, key: &str) -> Option<i64> {
    let needle = format!("\"{key}\" = ");
    output
        .lines()
        .filter_map(|line| {
            line.trim_start()
                .trim_start_matches('|')
                .trim_start()
                .strip_prefix(&needle)
        })
        .find_map(|value| value.trim().parse::<u64>().ok())
        .map(|value| value as i64)
}

#[cfg(test)]
#[path = "energy_tests.rs"]
mod tests;

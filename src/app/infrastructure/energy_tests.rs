use std::time::Duration;

#[cfg(target_os = "macos")]
#[test]
fn discharging_reads_negative_wattage_from_the_wrapped_amperage() {
    let output = concat!(
        "  | {\n",
        "  |   \"TimeRemaining\" = 261\n",
        "  |   \"CurrentCapacity\" = 93\n",
        "  |   \"AppleRawBatteryVoltage\" = 12419\n",
        "  |   \"IsCharging\" = No\n",
        "  |   \"InstantAmperage\" = 18446744073709550330\n",
        "  |   \"BatteryData\" = {\"CurrentCapacity\"=93,\"Amperage\"=18446744073709550330,\"TimeRemaining\"=0}\n",
        "  | }\n",
    );
    let energy = super::parse(output).expect("battery fixture");
    assert_eq!(energy.power_milliwatts, -15_970);
    assert_eq!(energy.charge_percent, Some(93));
    assert_eq!(energy.minutes_remaining, Some(261));
    assert!(!energy.charging);
    assert_eq!(energy.power_watts(), 15.97);
}

#[cfg(target_os = "macos")]
#[test]
fn nested_battery_fields_never_shadow_the_top_level_reading() {
    let output = concat!(
        "  |   \"AppleRawBatteryVoltage\" = 12000\n",
        "  |   \"InstantAmperage\" = 1000\n",
        "  |   \"BatteryData\" = {\"AppleRawBatteryVoltage\"=99999,\"InstantAmperage\"=99999}\n",
    );
    let energy = super::parse(output).expect("battery fixture");
    assert_eq!(energy.power_milliwatts, 12_000);
}

#[cfg(target_os = "macos")]
#[test]
fn a_missing_voltage_or_current_yields_no_sample() {
    assert!(super::parse("  |   \"CurrentCapacity\" = 93\n").is_none());
}

#[test]
fn consumed_energy_scales_with_the_sample_interval() {
    let energy = super::EnergySample {
        power_milliwatts: -3_600_000,
        charge_percent: None,
        minutes_remaining: None,
        charging: false,
    };
    assert!((energy.consumed_milliwatt_hours(Duration::from_secs(1)) - 1.0).abs() < f64::EPSILON);
    assert!((energy.consumed_milliwatt_hours(Duration::from_secs(60)) - 60.0).abs() < f64::EPSILON);
}

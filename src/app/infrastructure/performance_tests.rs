use super::*;

#[test]
fn operation_slots_follow_enum_discriminants() {
    for (index, kind) in OperationKind::ALL.into_iter().enumerate() {
        assert_eq!(kind as usize, index);
    }
}

#[test]
fn percentile_uses_the_nearest_rank() {
    let values = (1..=100).map(Duration::from_millis).collect::<Vec<_>>();
    assert_eq!(percentile(&values, 95), Duration::from_millis(95));
    assert_eq!(percentile(&[], 95), Duration::default());
}

#[test]
fn individual_timing_logs_only_slow_operations() {
    assert!(!should_log_duration(Duration::from_millis(1)));
    assert!(should_log_duration(Duration::from_millis(2)));
}

#[test]
fn the_busiest_operation_is_the_one_with_the_most_accumulated_time() {
    let summary = PerformanceSummary {
        operations: vec![
            operation("Quiet", 4, 3),
            operation("Hot", 2, 40),
            operation("Idle", 0, 900),
        ],
        ..PerformanceSummary::default()
    };
    assert_eq!(
        busiest_operation(&summary),
        Some(("Hot", Duration::from_millis(40)))
    );
    assert_eq!(busiest_operation(&PerformanceSummary::default()), None);
}

#[test]
fn the_work_share_is_the_measured_work_over_the_sample_window() {
    let summary = PerformanceSummary {
        sample_interval: Duration::from_secs(2),
        operations: vec![operation("A", 1, 20), operation("B", 1, 30)],
        ..PerformanceSummary::default()
    };
    assert_eq!(in_app_work(&summary), Duration::from_millis(50));
    assert!((work_share(&summary) - 2.5).abs() < f64::EPSILON);
    assert_eq!(work_share(&PerformanceSummary::default()), 0.0);
}

fn operation(label: &'static str, calls: u64, total_ms: u64) -> OperationSummary {
    OperationSummary {
        label,
        calls,
        total: Duration::from_millis(total_ms),
        max: Duration::from_millis(total_ms),
        work: calls,
        work_label: "items",
    }
}

#[test]
fn tracing_every_operation_logs_phases_under_the_slow_operation_floor() {
    let instant = Duration::from_micros(1);
    assert!(!should_log_operation_with(instant, false));
    assert!(should_log_operation_with(instant, true));
    assert!(should_log_operation_with(SLOW_OPERATION, false));
}

#[test]
fn only_truthy_trace_values_force_every_operation_to_log() {
    for value in ["1", "true", "yes"] {
        assert!(trace_from_env(Some(value)));
    }
    for value in [None, Some(""), Some("0"), Some("false"), Some("off")] {
        assert!(!trace_from_env(value));
    }
}

#[test]
fn high_latency_requires_a_dropped_frame_or_long_render_queue() {
    let mut summary = PerformanceSummary {
        draw_max: HIGH_LATENCY_DRAW - Duration::from_millis(1),
        dirty_to_draw_p95: HIGH_LATENCY_DIRTY_TO_DRAW - Duration::from_millis(1),
        ..PerformanceSummary::default()
    };
    assert!(!is_high_latency(&summary));

    summary.draw_max = HIGH_LATENCY_DRAW;
    assert!(is_high_latency(&summary));

    summary.draw_max = Duration::default();
    summary.dirty_to_draw_p95 = HIGH_LATENCY_DIRTY_TO_DRAW;
    assert!(is_high_latency(&summary));
}

#[test]
fn high_latency_reports_are_rate_limited() {
    let summary = PerformanceSummary {
        draw_max: HIGH_LATENCY_DRAW,
        ..PerformanceSummary::default()
    };
    let now = Instant::now();
    assert!(should_report_high_latency(&summary, None, now));
    assert!(!should_report_high_latency(&summary, Some(now), now));
    assert!(should_report_high_latency(
        &summary,
        Some(now - HIGH_LATENCY_REPORT_COOLDOWN),
        now,
    ));
}

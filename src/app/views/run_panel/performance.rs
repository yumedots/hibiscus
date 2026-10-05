use gpui::{FontWeight, IntoElement, ParentElement as _, Styled as _, div};

use crate::app::ui::{primitives::section_heading, theme::theme};

pub(in crate::app) fn render_performance(
    summary: &crate::app::infrastructure::performance::PerformanceSummary,
) -> impl IntoElement {
    div()
        .p(theme().space.sm)
        .border(theme().border)
        .border_color(theme().colors.border)
        .bg(theme().colors.canvas)
        .flex()
        .flex_col()
        .gap(theme().space.xs)
        .child(section_heading(format!(
            "GPUI profiler · {:.2} s sample",
            summary.sample_interval.as_secs_f64()
        )))
        .child(metric_row("Draw", draw_label(summary)))
        .child(metric_row("Battery", charge_label(summary)))
        .child(metric_row("Energy this sample", energy_label(summary)))
        .child(metric_row("Top consumer", top_consumer_label(summary)))
        .child(metric_row("In-app work", in_app_work_label(summary)))
        .child(metric_row(
            "Frames",
            format!("{} sampled", summary.frame_count),
        ))
        .child(metric_row(
            "Draw p95 / max",
            format!(
                "{} / {}",
                crate::app::infrastructure::performance::duration_label(summary.draw_p95),
                crate::app::infrastructure::performance::duration_label(summary.draw_max)
            ),
        ))
        .child(metric_row(
            "Dirty to draw p95",
            crate::app::infrastructure::performance::duration_label(summary.dirty_to_draw_p95),
        ))
        .child(metric_row(
            "Dirty requests avg / max",
            format!(
                "{:.1} / {}",
                summary.dirty_requests_average, summary.dirty_requests_max
            ),
        ))
        .child(metric_row(
            "Snapshots / stream events / coalesced",
            format!(
                "{} / {} / {}",
                summary.snapshots_published,
                summary.stream_events_observed,
                summary.stream_events_coalesced
            ),
        ))
        .child(metric_row(
            "Transcript compared / projected / remeasured",
            format!(
                "{} / {} / {}",
                summary.transcript_items_compared,
                summary.transcript_items_projected,
                summary.transcript_rows_remeasured
            ),
        ))
        .child(metric_row(
            "Catalog scans / parses / cache hits",
            format!(
                "{} / {} / {}",
                summary.catalog_scans, summary.catalog_files_parsed, summary.catalog_cache_hits
            ),
        ))
        .child(metric_row(
            "Markdown cache hits / misses",
            format!(
                "{} / {}",
                summary.markdown_cache_hits,
                summary
                    .operations
                    .iter()
                    .find(|operation| operation.label == "Markdown cache miss")
                    .map_or(0, |operation| operation.calls)
            ),
        ))
        .child(metric_row(
            "Scroll events · start / move / end",
            format!(
                "{} · {} / {} / {}",
                summary.scroll_events,
                summary.scroll_started,
                summary.scroll_moved,
                summary.scroll_ended,
            ),
        ))
        .child(metric_row(
            "Scroll after end · events / max",
            format!(
                "{} / {}",
                summary.scroll_events_after_end,
                crate::app::infrastructure::performance::duration_label(
                    summary.scroll_after_end_max
                ),
            ),
        ))
        .child(metric_row(
            "Scroll handler max gap",
            crate::app::infrastructure::performance::duration_label(summary.scroll_event_gap_max),
        ))
        .child(metric_row(
            "Scroll defers · count / max wait",
            format!(
                "{} / {}",
                summary.scroll_deferred_updates,
                crate::app::infrastructure::performance::duration_label(summary.scroll_defer_max),
            ),
        ))
        .children(summary.operations.iter().map(operation_metric_row))
        .child(metric_row(
            "Slowest task poll",
            summary.slowest_task.clone().unwrap_or_else(|| "—".into()),
        ))
        .child(metric_row(
            "Slowest action",
            summary.slowest_action.clone().unwrap_or_else(|| "—".into()),
        ))
}

fn operation_metric_row(
    operation: &crate::app::infrastructure::performance::OperationSummary,
) -> impl IntoElement + use<> {
    metric_row(
        operation.label,
        format!(
            "{} calls · {} total · {} max · {} {}",
            operation.calls,
            crate::app::infrastructure::performance::duration_label(operation.total),
            crate::app::infrastructure::performance::duration_label(operation.max),
            operation.work,
            operation.work_label,
        ),
    )
}

fn draw_label(summary: &crate::app::infrastructure::performance::PerformanceSummary) -> String {
    let Some(energy) = summary.energy.as_ref() else {
        return "unavailable".into();
    };
    let direction = if energy.charging {
        "charging"
    } else {
        "discharging"
    };
    format!("{:.2} W {direction}", energy.power_watts())
}

fn charge_label(summary: &crate::app::infrastructure::performance::PerformanceSummary) -> String {
    let Some(energy) = summary.energy.as_ref() else {
        return "unavailable".into();
    };
    let charge = energy
        .charge_percent
        .map_or_else(|| "—".into(), |percent| format!("{percent}%"));
    match energy.minutes_remaining {
        Some(minutes) => format!("{} · {}h {:02}m left", charge, minutes / 60, minutes % 60),
        None => charge,
    }
}

fn energy_label(summary: &crate::app::infrastructure::performance::PerformanceSummary) -> String {
    let Some(energy) = summary.energy.as_ref() else {
        return "unavailable".into();
    };
    format!(
        "{:.3} mWh over {:.2} s",
        energy.consumed_milliwatt_hours(summary.sample_interval),
        summary.sample_interval.as_secs_f64()
    )
}

fn top_consumer_label(
    summary: &crate::app::infrastructure::performance::PerformanceSummary,
) -> String {
    crate::app::infrastructure::performance::busiest_operation(summary).map_or_else(
        || "—".into(),
        |(label, total)| {
            format!(
                "{} · {}",
                label,
                crate::app::infrastructure::performance::duration_label(total)
            )
        },
    )
}

fn in_app_work_label(
    summary: &crate::app::infrastructure::performance::PerformanceSummary,
) -> String {
    format!(
        "{} · {:.1}% of the sample",
        crate::app::infrastructure::performance::duration_label(
            crate::app::infrastructure::performance::in_app_work(summary)
        ),
        crate::app::infrastructure::performance::work_share(summary)
    )
}

fn metric_row(label: &'static str, value: String) -> impl IntoElement {
    div()
        .min_h(theme().layout.status_row_height)
        .flex()
        .items_center()
        .justify_between()
        .gap(theme().space.sm)
        .text_size(theme().type_scale.caption)
        .child(div().text_color(theme().colors.subtle).child(label))
        .child(
            div()
                .min_w_0()
                .text_align(gpui::TextAlign::Right)
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme().colors.muted)
                .child(value),
        )
}

use super::*;

pub(super) struct BootstrapTasks {
    pub(super) runtime_events: Task<()>,
}

pub(super) struct PerformanceState {
    pub(super) monitor: Option<crate::app::infrastructure::performance::PerformanceMonitor>,
    pub(super) task: Option<Task<()>>,
}

pub(super) fn spawn(runtime: &RuntimeHandle, cx: &mut Context<FarcasterApp>) -> BootstrapTasks {
    let runtime_wake = runtime.wake_receiver();
    let runtime_events = cx.spawn(async move |weak, cx| {
        while runtime_wake.recv().await.is_ok() {
            if weak.update(cx, |this, cx| this.drain_runtime(cx)).is_err() {
                break;
            }
        }
    });
    BootstrapTasks { runtime_events }
}

pub(super) fn start_performance_monitor(
    window: &Window,
    cx: &mut Context<FarcasterApp>,
) -> PerformanceState {
    let debug = std::env::var("DEBUG").ok().as_deref() == Some("true");
    let monitor = Some(
        crate::app::infrastructure::performance::PerformanceMonitor::new(
            window.window_handle().window_id(),
            debug,
        ),
    );
    let task = Some(cx.spawn(async move |weak, cx| {
        loop {
            cx.background_executor()
                .timer(crate::app::infrastructure::performance::sample_interval())
                .await;
            let detailed = weak
                .update(cx, |this, _| {
                    this.lifecycle.performance_monitor.as_ref().is_some_and(
                        crate::app::infrastructure::performance::PerformanceMonitor::is_detailed,
                    )
                })
                .unwrap_or(false);
            let energy = if detailed {
                cx.background_spawn(async { crate::app::infrastructure::energy::sample() })
                    .await
            } else {
                None
            };
            if weak
                .update(cx, |this, cx| {
                    let sampled =
                        this.lifecycle
                            .performance_monitor
                            .as_mut()
                            .is_some_and(|monitor| {
                                monitor.set_energy(energy);
                                monitor.sample_if_due()
                            });
                    if sampled {
                        this.notify_performance_widget(cx);
                    }
                })
                .is_err()
            {
                break;
            }
        }
    }));

    PerformanceState { monitor, task }
}

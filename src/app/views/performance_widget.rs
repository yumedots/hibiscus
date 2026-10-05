use gpui::{
    App, AppContext as _, Context, CursorStyle, EmptyView, Entity, FontWeight,
    InteractiveElement as _, IntoElement, MouseButton, ParentElement as _, Pixels, Point, Render,
    Size, StatefulInteractiveElement as _, Styled as _, WeakEntity, Window, div, px,
};

use super::FarcasterApp;
use crate::app::ui::{
    assets::AppIcon,
    primitives::{AppIconSize, app_icon, icon_control},
    theme::theme,
};

const WIDGET_WIDTH: f32 = 340.0;
const WIDGET_MAX_HEIGHT: f32 = 460.0;
const WIDGET_MARGIN: f32 = 24.0;
const WIDGET_TOP: f32 = 76.0;
const WIDGET_ID: &str = "performance-widget";

struct PerformanceWidgetDrag;

#[derive(Default)]
pub(in crate::app) struct PerformanceWidget {
    position: Option<Point<Pixels>>,
    grab: Option<Point<Pixels>>,
    viewport: Size<Pixels>,
}

impl PerformanceWidget {
    pub(in crate::app) fn observe_viewport(&mut self, viewport: Size<Pixels>) {
        self.viewport = viewport;
    }

    fn position(&self) -> Point<Pixels> {
        clamped_position(
            self.position.unwrap_or_else(|| self.default_position()),
            self.viewport,
        )
    }

    fn default_position(&self) -> Point<Pixels> {
        Point {
            x: self.viewport.width - px(WIDGET_WIDTH + WIDGET_MARGIN),
            y: px(WIDGET_TOP),
        }
    }

    fn begin_drag(&mut self, pointer: Point<Pixels>) {
        let position = self.position();
        self.grab = Some(Point {
            x: pointer.x - position.x,
            y: pointer.y - position.y,
        });
    }

    fn update_drag(&mut self, pointer: Point<Pixels>) -> bool {
        let Some(grab) = self.grab else {
            return false;
        };
        let next = clamped_position(
            Point {
                x: pointer.x - grab.x,
                y: pointer.y - grab.y,
            },
            self.viewport,
        );
        if self.position == Some(next) {
            return false;
        }
        self.position = Some(next);
        true
    }

    fn finish_drag(&mut self) -> bool {
        self.grab.take().is_some()
    }
}

fn clamped_position(position: Point<Pixels>, viewport: Size<Pixels>) -> Point<Pixels> {
    let max_x = (viewport.width - px(WIDGET_WIDTH)).max(px(0.0));
    let max_y = (viewport.height - px(WIDGET_MAX_HEIGHT)).max(px(0.0));
    Point {
        x: position.x.clamp(px(0.0), max_x),
        y: position.y.clamp(px(0.0), max_y),
    }
}

pub(in crate::app) struct PerformanceWidgetView {
    app: WeakEntity<FarcasterApp>,
    state: PerformanceWidget,
}

fn follow_pointer(widget: &Entity<PerformanceWidgetView>, pointer: Point<Pixels>, cx: &mut App) {
    widget.update(cx, |this, cx| {
        if this.update_drag(pointer) {
            cx.notify();
        }
    });
}

fn end_drag(widget: &Entity<PerformanceWidgetView>, cx: &mut App) {
    widget.update(cx, |this, cx| {
        if this.finish_drag() {
            cx.notify();
        }
    });
}

impl PerformanceWidgetView {
    pub(in crate::app) fn new(app: WeakEntity<FarcasterApp>) -> Self {
        Self {
            app,
            state: PerformanceWidget::default(),
        }
    }

    pub(in crate::app) fn update_drag(&mut self, pointer: Point<Pixels>) -> bool {
        self.state.update_drag(pointer)
    }

    pub(in crate::app) fn finish_drag(&mut self) -> bool {
        self.state.finish_drag()
    }
}

impl Render for PerformanceWidgetView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let _timing =
            crate::app::infrastructure::performance::Timing::new("render.performance_widget");
        self.state.observe_viewport(window.viewport_size());
        let Some(app) = self.app.upgrade() else {
            return div().into_any_element();
        };
        let summary = app
            .read(cx)
            .lifecycle
            .performance_monitor
            .as_ref()
            .filter(|monitor| monitor.is_detailed())
            .map(|monitor| monitor.summary.clone());
        let Some(summary) = summary else {
            return div().into_any_element();
        };
        let position = self.state.position();
        let widget = cx.entity();
        let close = self.app.clone();
        let mover = widget.clone();
        let stopper = widget.clone();
        div()
            .id(WIDGET_ID)
            .on_drag_move::<PerformanceWidgetDrag>(move |event, _, cx| {
                follow_pointer(&mover, event.event.position, cx);
            })
            .on_mouse_up(MouseButton::Left, move |_, _, cx| end_drag(&stopper, cx))
            .absolute()
            .left(position.x)
            .top(position.y)
            .w(px(WIDGET_WIDTH))
            .max_h(px(WIDGET_MAX_HEIGHT))
            .flex()
            .flex_col()
            .overflow_hidden()
            .rounded(theme().radius)
            .border(theme().border)
            .border_color(theme().colors.border)
            .bg(theme().colors.surface)
            .shadow_md()
            .occlude()
            .child(
                div()
                    .id("performance-widget-header")
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(theme().space.sm)
                    .pl(theme().space.sm)
                    .pr(theme().space.xs)
                    .py(theme().space.xs)
                    .border_b(theme().border)
                    .border_color(theme().colors.border)
                    .cursor(CursorStyle::OpenHand)
                    .on_mouse_down(MouseButton::Left, move |event, _, cx| {
                        cx.stop_propagation();
                        widget.update(cx, |this, cx| {
                            this.state.begin_drag(event.position);
                            cx.notify();
                        });
                    })
                    .on_drag(PerformanceWidgetDrag, |_, _, _, cx| cx.new(|_| EmptyView))
                    .child(
                        div()
                            .min_w_0()
                            .text_size(theme().type_scale.caption)
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme().colors.text)
                            .child("Performance · battery"),
                    )
                    .child(
                        icon_control(
                            "performance-widget-close",
                            "Hide the performance monitor (F9)",
                        )
                        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .child(app_icon(AppIcon::X, AppIconSize::Inline))
                        .on_click(move |_, _, cx| {
                            cx.stop_propagation();
                            let _ =
                                close.update(cx, |this, cx| this.toggle_performance_monitor(cx));
                        }),
                    ),
            )
            .child(
                div()
                    .id("performance-widget-body")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(super::run_panel::performance::render_performance(&summary)),
            )
            .into_any_element()
    }
}

impl FarcasterApp {
    pub(in crate::app) fn toggle_performance_monitor(&mut self, cx: &mut Context<Self>) {
        if let Some(monitor) = self.lifecycle.performance_monitor.as_mut() {
            let detailed = !monitor.is_detailed();
            monitor.set_detailed(detailed);
        }
        cx.notify();
    }

    pub(in crate::app) fn notify_performance_widget(&self, cx: &mut Context<Self>) {
        self.views
            .performance_widget
            .update(cx, |_, cx| cx.notify());
    }

    pub(in crate::app) fn finish_performance_widget_drag(&mut self, cx: &mut Context<Self>) {
        self.views.performance_widget.update(cx, |widget, cx| {
            if widget.finish_drag() {
                cx.notify();
            }
        });
    }
}

#[cfg(test)]
#[path = "performance_widget_tests.rs"]
mod tests;

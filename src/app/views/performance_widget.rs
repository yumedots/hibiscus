use gpui::{
    App, AppContext as _, Context, CursorStyle, EmptyView, Entity, FontWeight,
    InteractiveElement as _, IntoElement, MouseButton, ParentElement as _, Pixels, Point, Render,
    Size, StatefulInteractiveElement as _, Styled as _, WeakEntity, Window, div,
    prelude::FluentBuilder as _, px,
};

use super::FarcasterApp;
use crate::app::ui::{
    assets::AppIcon,
    primitives::{AppIconSize, app_icon, icon_control},
    theme::theme,
};

const WIDGET_ID: &str = "performance-widget";
const WIDGET_DEFAULT_WIDTH: f32 = 340.0;
const WIDGET_DEFAULT_HEIGHT: f32 = 360.0;
const WIDGET_MIN_WIDTH: f32 = 260.0;
const WIDGET_MIN_HEIGHT: f32 = 140.0;
const WIDGET_HEADER_HEIGHT: f32 = 34.0;
const WIDGET_GRIP: f32 = 14.0;
const WIDGET_MARGIN: f32 = 24.0;
const WIDGET_TOP: f32 = 76.0;

#[derive(Clone, Copy)]
enum PerformanceWidgetDrag {
    Move,
    Resize,
}

#[derive(Default)]
pub(in crate::app) struct PerformanceWidget {
    position: Option<Point<Pixels>>,
    size: Option<Size<Pixels>>,
    collapsed: bool,
    grab: Option<Point<Pixels>>,
    resize: Option<(Point<Pixels>, Size<Pixels>)>,
    viewport: Size<Pixels>,
}

impl PerformanceWidget {
    pub(in crate::app) fn observe_viewport(&mut self, viewport: Size<Pixels>) {
        self.viewport = viewport;
    }

    fn size(&self) -> Size<Pixels> {
        clamped_size(
            self.size.unwrap_or(Size {
                width: px(WIDGET_DEFAULT_WIDTH),
                height: px(WIDGET_DEFAULT_HEIGHT),
            }),
            self.viewport,
        )
    }

    fn card_size(&self) -> Size<Pixels> {
        if self.collapsed {
            Size {
                width: self.size().width,
                height: px(WIDGET_HEADER_HEIGHT),
            }
        } else {
            self.size()
        }
    }

    fn position(&self) -> Point<Pixels> {
        clamped_position(
            self.position.unwrap_or_else(|| self.default_position()),
            self.card_size(),
            self.viewport,
        )
    }

    fn default_position(&self) -> Point<Pixels> {
        Point {
            x: self.viewport.width - self.card_size().width - px(WIDGET_MARGIN),
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
            self.card_size(),
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

    fn begin_resize(&mut self, pointer: Point<Pixels>) {
        self.resize = Some((pointer, self.size()));
    }

    fn update_resize(&mut self, pointer: Point<Pixels>) -> bool {
        let Some((start, start_size)) = self.resize else {
            return false;
        };
        let next = clamped_size(
            Size {
                width: start_size.width + pointer.x - start.x,
                height: start_size.height + pointer.y - start.y,
            },
            self.viewport,
        );
        if self.size == Some(next) {
            return false;
        }
        self.size = Some(next);
        true
    }

    fn finish_resize(&mut self) -> bool {
        self.resize.take().is_some()
    }

    fn finish_pointer(&mut self) -> bool {
        let dragged = self.finish_drag();
        let resized = self.finish_resize();
        dragged || resized
    }

    fn toggle_collapsed(&mut self) {
        self.collapsed = !self.collapsed;
    }
}

fn clamped_size(size: Size<Pixels>, viewport: Size<Pixels>) -> Size<Pixels> {
    let max_width = (viewport.width - px(WIDGET_MARGIN)).max(px(WIDGET_MIN_WIDTH));
    let max_height = (viewport.height - px(WIDGET_MARGIN)).max(px(WIDGET_MIN_HEIGHT));
    Size {
        width: size.width.clamp(px(WIDGET_MIN_WIDTH), max_width),
        height: size.height.clamp(px(WIDGET_MIN_HEIGHT), max_height),
    }
}

fn clamped_position(
    position: Point<Pixels>,
    size: Size<Pixels>,
    viewport: Size<Pixels>,
) -> Point<Pixels> {
    let max_x = (viewport.width - size.width).max(px(0.0));
    let max_y = (viewport.height - size.height).max(px(0.0));
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
        if this.state.update_drag(pointer) {
            cx.notify();
        }
    });
}

fn follow_resize(widget: &Entity<PerformanceWidgetView>, pointer: Point<Pixels>, cx: &mut App) {
    widget.update(cx, |this, cx| {
        if this.state.update_resize(pointer) {
            cx.notify();
        }
    });
}

fn end_pointer(widget: &Entity<PerformanceWidgetView>, cx: &mut App) {
    widget.update(cx, |this, cx| {
        if this.state.finish_pointer() {
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

    pub(in crate::app) fn finish_drag(&mut self) -> bool {
        self.state.finish_pointer()
    }

    fn header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let widget = cx.entity();
        let into_header = widget.clone();
        let close = self.app.clone();
        let collapsed = self.state.collapsed;
        div()
            .id("performance-widget-header")
            .debug_selector(|| "performance-widget-header".into())
            .flex_none()
            .h(px(WIDGET_HEADER_HEIGHT))
            .flex()
            .items_center()
            .justify_between()
            .gap(theme().space.sm)
            .pl(theme().space.sm)
            .pr(theme().space.xs)
            .when(!collapsed, |header| {
                header
                    .border_b(theme().border)
                    .border_color(theme().colors.border)
            })
            .when(collapsed, |header| header.pb(theme().border))
            .cursor(CursorStyle::OpenHand)
            .on_mouse_down(MouseButton::Left, move |event, _, cx| {
                cx.stop_propagation();
                widget.update(cx, |this, cx| {
                    this.state.begin_drag(event.position);
                    cx.notify();
                });
            })
            .on_drag(PerformanceWidgetDrag::Move, |_, _, _, cx| {
                cx.new(|_| EmptyView)
            })
            .child(
                div()
                    .debug_selector(|| "performance-widget-title".into())
                    .min_w_0()
                    .text_size(theme().type_scale.caption)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme().colors.text)
                    .child("Performance"),
            )
            .child(
                div()
                    .flex_none()
                    .flex()
                    .items_center()
                    .gap(theme().space.sm)
                    .child(
                        icon_control(
                            "performance-widget-minimize",
                            "Collapse the performance monitor to its title bar",
                        )
                        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .child(app_icon(AppIcon::Minus, AppIconSize::Inline))
                        .on_click(move |_, _, cx| {
                            cx.stop_propagation();
                            into_header.update(cx, |this, cx| {
                                this.state.toggle_collapsed();
                                cx.notify();
                            });
                        }),
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
        let card = self.state.card_size();
        let collapsed = self.state.collapsed;
        let position = self.state.position();
        let widget = cx.entity();
        let mover = widget.clone();
        let resizer = widget.clone();
        let stopper = widget.clone();
        let grip = widget.clone();
        let header = self.header(cx);
        div()
            .id(WIDGET_ID)
            .on_drag_move::<PerformanceWidgetDrag>(move |event, _, cx| {
                let pointer = event.event.position;
                match event.drag(cx) {
                    PerformanceWidgetDrag::Move => follow_pointer(&mover, pointer, cx),
                    PerformanceWidgetDrag::Resize => follow_resize(&resizer, pointer, cx),
                }
            })
            .on_mouse_up(MouseButton::Left, move |_, _, cx| end_pointer(&stopper, cx))
            .absolute()
            .left(position.x)
            .top(position.y)
            .w(card.width)
            .h(card.height)
            .flex()
            .flex_col()
            .overflow_hidden()
            .rounded(theme().radius)
            .border(theme().border)
            .border_color(theme().colors.border)
            .bg(theme().colors.surface)
            .shadow_md()
            .occlude()
            .child(header)
            .when(!collapsed, |card| {
                card.child(
                    div()
                        .id("performance-widget-body")
                        .flex_1()
                        .min_h_0()
                        .overflow_y_scroll()
                        .child(super::run_panel::performance::render_performance(&summary)),
                )
                .child(
                    div()
                        .id("performance-widget-resize")
                        .absolute()
                        .right(px(0.0))
                        .bottom(px(0.0))
                        .w(px(WIDGET_GRIP))
                        .h(px(WIDGET_GRIP))
                        .cursor(CursorStyle::ResizeUpLeftDownRight)
                        .on_mouse_down(MouseButton::Left, move |event, _, cx| {
                            cx.stop_propagation();
                            grip.update(cx, |this, cx| {
                                this.state.begin_resize(event.position);
                                cx.notify();
                            });
                        })
                        .on_drag(PerformanceWidgetDrag::Resize, |_, _, _, cx| {
                            cx.new(|_| EmptyView)
                        }),
                )
            })
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

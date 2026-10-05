use gpui::{Entity, MouseButton, Pixels, Point, VisualTestContext, point, px, size};

use super::{
    PerformanceWidget, WIDGET_MARGIN, WIDGET_MAX_HEIGHT, WIDGET_TOP, WIDGET_WIDTH, clamped_position,
};
use crate::app::FarcasterApp;

fn widget_position(cx: &mut VisualTestContext, app: &Entity<FarcasterApp>) -> Point<Pixels> {
    let widget = cx.update(|_, cx| app.read(cx).views.performance_widget.clone());
    cx.update(|_, cx| widget.read(cx).state.position())
}

fn widget(width: f32, height: f32) -> PerformanceWidget {
    let mut widget = PerformanceWidget::default();
    widget.observe_viewport(size(px(width), px(height)));
    widget
}

#[gpui::test]
fn a_drag_follows_the_pointer_away_from_the_card(cx: &mut gpui::TestAppContext) {
    let test_name = concat!(
        module_path!(),
        "::a_drag_follows_the_pointer_away_from_the_card"
    );
    let viewport = size(px(1400.0), px(900.0));
    crate::app::test_support::with_prepared_offline_app(
        test_name,
        cx,
        |_| {},
        |cx, app, _, _| {
            cx.simulate_resize(viewport);
            cx.update(|_, cx| app.update(cx, |app, cx| app.toggle_performance_monitor(cx)));
            cx.update(|window, cx| window.draw(cx).clear(cx));
            let opened_at = widget_position(cx, app);
            let header = opened_at + point(px(60.0), px(8.0));

            cx.simulate_mouse_down(header, MouseButton::Left, Default::default());
            cx.simulate_mouse_move(
                header + point(px(-6.0), px(4.0)),
                Some(MouseButton::Left),
                Default::default(),
            );
            for offset in [
                point(px(-60.0), px(40.0)),
                point(px(-140.0), px(90.0)),
                point(px(-500.0), px(200.0)),
                point(px(200.0), px(700.0)),
            ] {
                cx.simulate_mouse_move(
                    header + offset,
                    Some(MouseButton::Left),
                    Default::default(),
                );
                assert_eq!(
                    widget_position(cx, app),
                    clamped_position(opened_at + offset, viewport),
                    "the card should follow the pointer at {offset:?}"
                );
            }

            let released = header + point(px(200.0), px(700.0));
            cx.simulate_mouse_up(released, MouseButton::Left, Default::default());
            cx.simulate_mouse_move(
                released + point(px(-20.0), px(-20.0)),
                Some(MouseButton::Left),
                Default::default(),
            );
            assert_eq!(
                widget_position(cx, app),
                clamped_position(opened_at + point(px(200.0), px(700.0)), viewport)
            );
        },
    );
}

#[test]
fn the_widget_opens_in_the_top_right_corner() {
    let widget = widget(1200.0, 900.0);
    assert_eq!(
        widget.position(),
        point(px(1200.0 - WIDGET_WIDTH - WIDGET_MARGIN), px(WIDGET_TOP))
    );
}

#[test]
fn dragging_keeps_the_grab_offset_and_never_leaves_the_window() {
    let mut widget = widget(1200.0, 900.0);
    let opened_at = widget.position();

    widget.begin_drag(point(px(1000.0), px(100.0)));
    assert!(widget.update_drag(point(px(700.0), px(300.0))));
    assert_eq!(
        widget.position(),
        point(opened_at.x - px(300.0), opened_at.y + px(200.0))
    );

    widget.update_drag(point(px(9000.0), px(9000.0)));
    assert_eq!(
        widget.position(),
        point(px(1200.0 - WIDGET_WIDTH), px(900.0 - WIDGET_MAX_HEIGHT))
    );

    widget.update_drag(point(px(-9000.0), px(-9000.0)));
    assert_eq!(widget.position(), point(px(0.0), px(0.0)));

    assert!(!widget.update_drag(point(px(-9000.0), px(-9000.0))));
    assert!(widget.finish_drag());
    assert!(!widget.finish_drag());
}

#[test]
fn a_drag_without_a_press_does_nothing() {
    let mut widget = widget(1200.0, 900.0);
    let opened_at = widget.position();
    assert!(!widget.update_drag(point(px(10.0), px(10.0))));
    assert_eq!(widget.position(), opened_at);
}

#[test]
fn a_window_smaller_than_the_widget_pins_it_to_the_origin() {
    let viewport = size(px(200.0), px(200.0));
    for pointer in [point(px(-500.0), px(-500.0)), point(px(900.0), px(900.0))] {
        assert_eq!(clamped_position(pointer, viewport), point(px(0.0), px(0.0)));
    }
}

#[test]
fn a_viewport_that_never_rendered_still_yields_a_usable_position() {
    let mut widget = PerformanceWidget::default();
    widget.observe_viewport(size(px(0.0), px(0.0)));
    assert_eq!(widget.position(), point(px(0.0), px(0.0)));
}

use gpui::{Entity, MouseButton, Pixels, Point, Size, VisualTestContext, point, px, size};

use super::{
    PerformanceWidget, WIDGET_DEFAULT_HEIGHT, WIDGET_DEFAULT_WIDTH, WIDGET_HEADER_HEIGHT,
    WIDGET_MARGIN, WIDGET_MIN_HEIGHT, WIDGET_MIN_WIDTH, WIDGET_TOP, clamped_position,
};
use crate::app::FarcasterApp;

fn widget_position(cx: &mut VisualTestContext, app: &Entity<FarcasterApp>) -> Point<Pixels> {
    cx.update(|_, cx| {
        let widget = app.read(cx).views.performance_widget.clone();
        widget.read(cx).state.position()
    })
}

fn widget_size(cx: &mut VisualTestContext, app: &Entity<FarcasterApp>) -> Size<Pixels> {
    cx.update(|_, cx| {
        let widget = app.read(cx).views.performance_widget.clone();
        widget.read(cx).state.size()
    })
}

fn card_height(cx: &mut VisualTestContext, app: &Entity<FarcasterApp>) -> Pixels {
    cx.update(|_, cx| {
        let widget = app.read(cx).views.performance_widget.clone();
        widget.read(cx).state.card_size().height
    })
}

fn collapsed(cx: &mut VisualTestContext, app: &Entity<FarcasterApp>) -> bool {
    cx.update(|_, cx| {
        let widget = app.read(cx).views.performance_widget.clone();
        widget.read(cx).state.collapsed
    })
}

fn widget_state(width: f32, height: f32) -> PerformanceWidget {
    let mut widget = PerformanceWidget::default();
    widget.observe_viewport(size(px(width), px(height)));
    widget
}

fn default_size() -> Size<Pixels> {
    size(px(WIDGET_DEFAULT_WIDTH), px(WIDGET_DEFAULT_HEIGHT))
}

fn minimize_button(cx: &mut VisualTestContext, app: &Entity<FarcasterApp>) -> Point<Pixels> {
    let position = widget_position(cx, app);
    let card = widget_size(cx, app);
    point(
        position.x + card.width - px(54.0),
        position.y + px(WIDGET_HEADER_HEIGHT / 2.0),
    )
}

#[test]
fn the_widget_opens_in_the_top_right_corner() {
    let widget = widget_state(1200.0, 900.0);
    assert_eq!(widget.size(), default_size());
    assert_eq!(
        widget.position(),
        point(
            px(1200.0 - WIDGET_DEFAULT_WIDTH - WIDGET_MARGIN),
            px(WIDGET_TOP)
        )
    );
}

#[test]
fn dragging_keeps_the_grab_offset_and_never_leaves_the_window() {
    let viewport = size(px(1200.0), px(900.0));
    let mut widget = widget_state(1200.0, 900.0);
    let opened_at = widget.position();

    widget.begin_drag(point(px(1000.0), px(100.0)));
    assert!(widget.update_drag(point(px(700.0), px(300.0))));
    assert_eq!(
        widget.position(),
        clamped_position(
            point(opened_at.x - px(300.0), opened_at.y + px(200.0)),
            default_size(),
            viewport
        )
    );

    widget.update_drag(point(px(9000.0), px(9000.0)));
    assert_eq!(
        widget.position(),
        clamped_position(point(px(9000.0), px(9000.0)), default_size(), viewport)
    );

    widget.update_drag(point(px(-9000.0), px(-9000.0)));
    assert_eq!(widget.position(), point(px(0.0), px(0.0)));

    assert!(!widget.update_drag(point(px(-9000.0), px(-9000.0))));
    assert!(widget.finish_drag());
    assert!(!widget.finish_drag());
}

#[test]
fn a_drag_without_a_press_does_nothing() {
    let mut widget = widget_state(1200.0, 900.0);
    let opened_at = widget.position();
    assert!(!widget.update_drag(point(px(10.0), px(10.0))));
    assert_eq!(widget.position(), opened_at);
}

#[test]
fn a_resize_without_a_press_does_nothing() {
    let mut widget = widget_state(1200.0, 900.0);
    assert!(!widget.update_resize(point(px(10.0), px(10.0))));
    assert_eq!(widget.size(), default_size());
}

#[test]
fn resizing_grows_and_shrinks_within_the_widget_limits() {
    let viewport = size(px(1200.0), px(900.0));
    let mut widget = widget_state(1200.0, 900.0);
    let grip = point(px(1100.0), px(400.0));

    widget.begin_resize(grip);
    assert!(widget.update_resize(grip + point(px(60.0), px(40.0))));
    assert_eq!(
        widget.size(),
        size(
            px(WIDGET_DEFAULT_WIDTH + 60.0),
            px(WIDGET_DEFAULT_HEIGHT + 40.0)
        )
    );

    assert!(!widget.update_resize(grip + point(px(60.0), px(40.0))));

    widget.update_resize(grip - point(px(9000.0), px(9000.0)));
    assert_eq!(
        widget.size(),
        size(px(WIDGET_MIN_WIDTH), px(WIDGET_MIN_HEIGHT))
    );

    widget.update_resize(grip + point(px(9000.0), px(9000.0)));
    assert_eq!(
        widget.size(),
        size(
            viewport.width - px(WIDGET_MARGIN),
            viewport.height - px(WIDGET_MARGIN)
        )
    );

    assert!(widget.finish_resize());
    assert!(!widget.finish_resize());
}

#[test]
fn resizing_keeps_a_dragged_widget_inside_the_window() {
    let viewport = size(px(1200.0), px(900.0));
    let mut widget = widget_state(1200.0, 900.0);
    let grip = point(px(1100.0), px(400.0));

    widget.update_drag(point(px(1190.0), px(890.0)));
    widget.begin_drag(point(px(1190.0), px(890.0)));
    widget.update_drag(point(px(1190.0), px(890.0)));
    widget.begin_resize(grip);
    widget.update_resize(grip + point(px(600.0), px(600.0)));

    let position = widget.position();
    let size = widget.size();
    assert!(position.x + size.width <= viewport.width);
    assert!(position.y + size.height <= viewport.height);
}

#[test]
fn collapsing_pins_the_card_to_its_header_and_restores_the_size() {
    let viewport = size(px(1200.0), px(900.0));
    let mut widget = widget_state(1200.0, 900.0);
    let open = widget.position();

    widget.toggle_collapsed();
    assert_eq!(
        widget.card_size(),
        size(px(WIDGET_DEFAULT_WIDTH), px(WIDGET_HEADER_HEIGHT))
    );
    assert_eq!(widget.size(), default_size());
    assert_eq!(widget.position(), open);

    widget.begin_drag(point(px(1000.0), px(400.0)));
    widget.update_drag(point(px(1000.0), px(890.0)));
    assert_eq!(
        widget.position(),
        clamped_position(
            point(open.x, open.y + px(490.0)),
            size(px(WIDGET_DEFAULT_WIDTH), px(WIDGET_HEADER_HEIGHT)),
            viewport
        )
    );

    widget.toggle_collapsed();
    assert_eq!(widget.card_size(), default_size());
    assert!(widget.position().y + default_size().height <= viewport.height);
}

#[test]
fn a_window_smaller_than_the_widget_clamps_it_to_the_top_left() {
    let viewport = size(px(200.0), px(200.0));
    let mut widget = widget_state(200.0, 200.0);
    let card = widget.card_size();
    assert!(
        card.width > viewport.width,
        "the card keeps its minimum width"
    );
    let floor = viewport.height - card.height;

    for pointer in [point(px(-500.0), px(-500.0)), point(px(900.0), px(900.0))] {
        let clamped = clamped_position(pointer, card, viewport);
        assert_eq!(clamped.x, px(0.0));
        assert_eq!(clamped.y, clamped.y.clamp(px(0.0), floor));
    }
    assert_eq!(
        clamped_position(point(px(900.0), px(900.0)), card, viewport),
        point(px(0.0), floor)
    );

    widget.begin_drag(point(px(10.0), px(10.0)));
    widget.update_drag(point(px(-900.0), px(-900.0)));
    assert_eq!(widget.position(), point(px(0.0), px(0.0)));

    widget.begin_drag(point(px(10.0), px(10.0)));
    widget.update_drag(point(px(900.0), px(900.0)));
    assert_eq!(widget.position(), point(px(0.0), floor));
}

#[test]
fn a_viewport_that_never_rendered_still_yields_a_usable_position() {
    let mut widget = PerformanceWidget::default();
    widget.observe_viewport(size(px(0.0), px(0.0)));
    assert_eq!(widget.position(), point(px(0.0), px(0.0)));
    widget.toggle_collapsed();
    assert_eq!(widget.position(), point(px(0.0), px(0.0)));
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
                    clamped_position(opened_at + offset, widget_size(cx, app), viewport),
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
                clamped_position(
                    opened_at + point(px(200.0), px(700.0)),
                    widget_size(cx, app),
                    viewport
                )
            );
        },
    );
}

#[gpui::test]
fn resizing_from_the_grip_changes_the_card(cx: &mut gpui::TestAppContext) {
    let test_name = concat!(module_path!(), "::resizing_from_the_grip_changes_the_card");
    let viewport = size(px(1400.0), px(900.0));
    crate::app::test_support::with_prepared_offline_app(
        test_name,
        cx,
        |_| {},
        |cx, app, _, _| {
            cx.simulate_resize(viewport);
            cx.update(|_, cx| app.update(cx, |app, cx| app.toggle_performance_monitor(cx)));
            cx.update(|window, cx| window.draw(cx).clear(cx));
            let card = widget_size(cx, app);
            let grip =
                widget_position(cx, app) + point(card.width - px(4.0), card.height - px(4.0));

            cx.simulate_mouse_down(grip, MouseButton::Left, Default::default());
            cx.simulate_mouse_move(
                grip + point(px(4.0), px(4.0)),
                Some(MouseButton::Left),
                Default::default(),
            );
            cx.simulate_mouse_move(
                grip + point(px(60.0), px(40.0)),
                Some(MouseButton::Left),
                Default::default(),
            );
            assert_eq!(
                widget_size(cx, app),
                size(card.width + px(60.0), card.height + px(40.0)),
                "the card should grow with the pointer"
            );

            cx.simulate_mouse_up(
                grip + point(px(60.0), px(40.0)),
                MouseButton::Left,
                Default::default(),
            );
            cx.simulate_mouse_move(
                grip + point(px(160.0), px(140.0)),
                Some(MouseButton::Left),
                Default::default(),
            );
            assert_eq!(
                widget_size(cx, app),
                size(card.width + px(60.0), card.height + px(40.0))
            );
        },
    );
}

#[gpui::test]
fn collapsing_keeps_the_title_bar_in_place(cx: &mut gpui::TestAppContext) {
    let test_name = concat!(module_path!(), "::collapsing_keeps_the_title_bar_in_place");
    let viewport = size(px(1400.0), px(900.0));
    crate::app::test_support::with_prepared_offline_app(
        test_name,
        cx,
        |_| {},
        |cx, app, _, _| {
            cx.simulate_resize(viewport);
            cx.update(|_, cx| app.update(cx, |app, cx| app.toggle_performance_monitor(cx)));
            cx.update(|window, cx| window.draw(cx).clear(cx));
            let expanded_header = cx.debug_bounds("performance-widget-header");
            let expanded_title = cx.debug_bounds("performance-widget-title");
            assert!(expanded_header.is_some() && expanded_title.is_some());
            let minimize = minimize_button(cx, app);
            cx.simulate_click(minimize, Default::default());
            cx.update(|window, cx| window.draw(cx).clear(cx));

            assert!(collapsed(cx, app));
            assert_eq!(
                cx.debug_bounds("performance-widget-header"),
                expanded_header
            );
            assert_eq!(
                cx.debug_bounds("performance-widget-title"),
                expanded_title,
                "collapsing must not shift the title"
            );
        },
    );
}

#[gpui::test]
fn the_header_buttons_collapse_and_hide_the_card(cx: &mut gpui::TestAppContext) {
    let test_name = concat!(
        module_path!(),
        "::the_header_buttons_collapse_and_hide_the_card"
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
            let position = widget_position(cx, app);
            let card = widget_size(cx, app);
            let right = position.x + card.width;
            let header_y = position.y + px(WIDGET_HEADER_HEIGHT / 2.0);
            let minimize = point(right - px(54.0), header_y);

            assert!(!collapsed(cx, app));
            cx.simulate_click(minimize, Default::default());
            assert!(
                collapsed(cx, app),
                "the minimize button should collapse the card"
            );
            assert_eq!(card_height(cx, app), px(WIDGET_HEADER_HEIGHT));

            cx.simulate_click(minimize, Default::default());
            assert!(!collapsed(cx, app), "the button should restore the card");

            let close = point(right - px(18.0), header_y);
            cx.simulate_click(close, Default::default());
            let detailed = cx.update(|_, cx| {
                app.read(cx)
                    .lifecycle
                    .performance_monitor
                    .as_ref()
                    .is_some_and(|monitor| monitor.is_detailed())
            });
            assert!(!detailed, "the close button should hide the monitor");
        },
    );
}

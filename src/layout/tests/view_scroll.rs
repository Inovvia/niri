use super::*;

fn setup() -> Layout<TestWindow> {
    setup_scaled(1.)
}

fn setup_scaled(scale: f64) -> Layout<TestWindow> {
    fn wide_window(id: usize) -> TestWindowParams {
        let mut params = TestWindowParams::new(id);
        params.min_max_size.0 = Size::new(700, 0);
        params
    }
    check_ops([
        Op::AddScaledOutput {
            id: 1,
            scale,
            layout_config: None,
        },
        Op::AddWindow {
            params: wide_window(1),
        },
        Op::AddWindow {
            params: wide_window(2),
        },
        Op::AddWindow {
            params: wide_window(3),
        },
        Op::Communicate(1),
        Op::Communicate(2),
        Op::Communicate(3),
        Op::FocusColumnFirst,
        Op::CompleteAnimations,
    ])
}

fn position(layout: &Layout<TestWindow>) -> f64 {
    layout.active_workspace().unwrap().scrolling().view_pos()
}

#[test]
fn steps_accumulate_and_preserve_focus() {
    let mut layout = setup();
    let start = position(&layout);
    check_ops_on_layout(&mut layout, [Op::ScrollViewRight, Op::ScrollViewRight]);
    assert_eq!(position(&layout), start + 80.);
    assert_eq!(
        layout
            .active_workspace()
            .unwrap()
            .scrolling()
            .active_column_idx(),
        0
    );
    check_ops_on_layout(&mut layout, [Op::ScrollViewLeft]);
    assert_eq!(position(&layout), start + 40.);
}

#[test]
fn redraws_client_updates_and_config_reload_preserve_pan() {
    let mut layout = setup();
    layout.scroll_view(true);
    let panned = position(&layout);
    check_ops_on_layout(
        &mut layout,
        [
            Op::Refresh { is_active: true },
            Op::Communicate(1),
            Op::AdvanceAnimations { msec_delta: 2000 },
        ],
    );
    assert_eq!(position(&layout), panned);
    layout.update_options(Options {
        view_scroll_step: niri_config::ViewScrollStep(25.5),
        ..Default::default()
    });
    check_ops_on_layout(&mut layout, [Op::CompleteAnimations]);
    assert_eq!(position(&layout), panned);
    layout.scroll_view(true);
    assert_eq!(position(&layout), panned + 25.5);
}

#[test]
fn pan_interrupts_animation_at_displayed_position() {
    let mut layout = setup();
    check_ops_on_layout(
        &mut layout,
        [
            Op::FocusColumnLast,
            Op::AdvanceAnimations { msec_delta: 30 },
        ],
    );
    let displayed = position(&layout);
    assert_ne!(
        displayed,
        layout
            .active_workspace()
            .unwrap()
            .scrolling()
            .target_view_pos()
    );
    layout.scroll_view(true);
    assert_eq!(position(&layout), displayed + 40.);
    check_ops_on_layout(&mut layout, [Op::CompleteAnimations]);
    assert_eq!(position(&layout), displayed + 40.);
}

#[test]
fn bounds_stop_repeated_pan_and_focus_navigation_still_works() {
    let mut layout = setup();
    let left = position(&layout);
    for _ in 0..100 {
        layout.scroll_view(false);
    }
    assert_eq!(position(&layout), left);
    for _ in 0..100 {
        layout.scroll_view(true);
    }
    let right = position(&layout);
    assert!(right > left);
    layout.scroll_view(true);
    assert_eq!(position(&layout), right);
    check_ops_on_layout(&mut layout, [Op::FocusColumnFirst, Op::CompleteAnimations]);
    assert_eq!(position(&layout), left);
    layout.verify_invariants();
}

#[test]
fn empty_single_column_fullscreen_and_floating_are_noops() {
    let mut layout = check_ops([Op::AddOutput(1)]);
    layout.scroll_view(true);
    layout.verify_invariants();
    check_ops_on_layout(
        &mut layout,
        [
            Op::AddWindow {
                params: TestWindowParams::new(1),
            },
            Op::CompleteAnimations,
        ],
    );
    let single = position(&layout);
    layout.scroll_view(true);
    layout.scroll_view(false);
    assert_eq!(position(&layout), single);
    check_ops_on_layout(
        &mut layout,
        [
            Op::FullscreenWindow(1),
            Op::Communicate(1),
            Op::CompleteAnimations,
        ],
    );
    let fullscreen = position(&layout);
    layout.scroll_view(true);
    assert_eq!(position(&layout), fullscreen);
    let mut params = TestWindowParams::new(2);
    params.is_floating = true;
    check_ops_on_layout(
        &mut layout,
        [Op::AddWindow { params }, Op::CompleteAnimations],
    );
    let floating = position(&layout);
    layout.scroll_view(true);
    assert_eq!(position(&layout), floating);
}

#[test]
fn closing_and_moving_columns_after_panning_leave_valid_state() {
    let mut layout = setup();
    check_ops_on_layout(
        &mut layout,
        [
            Op::ScrollViewRight,
            Op::MoveColumnRight,
            Op::ScrollViewRight,
            Op::CloseWindow(1),
            Op::CloseWindow(2),
            Op::CloseWindow(3),
            Op::ScrollViewLeft,
            Op::CompleteAnimations,
        ],
    );
}

#[test]
fn logical_step_is_independent_of_output_scale() {
    for scale in [1., 1.5, 2.] {
        let mut layout = setup_scaled(scale);
        let start = position(&layout);
        layout.scroll_view(true);
        assert_eq!(position(&layout), start + 40.);
        layout.verify_invariants();
    }
}

#[test]
fn gesture_end_cannot_override_wheel_pan() {
    let mut layout = setup();
    check_ops_on_layout(
        &mut layout,
        [
            Op::ViewOffsetGestureBegin {
                output_idx: 1,
                workspace_idx: None,
                is_touchpad: true,
            },
            Op::ViewOffsetGestureUpdate {
                delta: 20.,
                timestamp: Duration::from_millis(10),
                is_touchpad: true,
            },
        ],
    );
    let displayed = position(&layout);
    layout.scroll_view(true);
    assert_eq!(position(&layout), displayed + 40.);
    check_ops_on_layout(
        &mut layout,
        [
            Op::ViewOffsetGestureEnd {
                is_touchpad: Some(true),
            },
            Op::CompleteAnimations,
        ],
    );
    assert_eq!(position(&layout), displayed + 40.);
}

#[test]
fn pan_targets_active_monitor_and_survives_workspace_switch() {
    let mut layout = setup();
    layout.scroll_view(true);
    let panned = position(&layout);
    check_ops_on_layout(
        &mut layout,
        [Op::AddOutput(2), Op::FocusOutput(2), Op::ScrollViewRight],
    );
    check_ops_on_layout(&mut layout, [Op::FocusOutput(1), Op::CompleteAnimations]);
    assert_eq!(position(&layout), panned);
    check_ops_on_layout(
        &mut layout,
        [
            Op::FocusWorkspaceDown,
            Op::FocusWorkspaceUp,
            Op::CompleteAnimations,
        ],
    );
    assert_eq!(position(&layout), panned);
}

#[test]
fn closing_background_column_preserves_pan_until_bounds_shrink() {
    let mut layout = setup();
    layout.scroll_view(true);
    let panned = position(&layout);
    check_ops_on_layout(&mut layout, [Op::CloseWindow(3), Op::CompleteAnimations]);
    assert_eq!(position(&layout), panned);
    check_ops_on_layout(&mut layout, [Op::CloseWindow(2), Op::CompleteAnimations]);
    assert!(position(&layout) < panned);
    assert_eq!(
        layout
            .active_workspace()
            .unwrap()
            .scrolling()
            .active_column_idx(),
        0
    );
}

#[test]
fn oversized_single_column_can_pan_between_edges() {
    let mut params = TestWindowParams::new(1);
    params.min_max_size.0 = Size::new(2000, 0);
    let mut layout = check_ops([
        Op::AddOutput(1),
        Op::AddWindow { params },
        Op::Communicate(1),
        Op::CompleteAnimations,
    ]);
    let left = position(&layout);
    layout.scroll_view(true);
    assert_eq!(position(&layout), left + 40.);
    for _ in 0..100 {
        layout.scroll_view(true);
    }
    let right = position(&layout);
    assert!(right > left);
    layout.scroll_view(true);
    assert_eq!(position(&layout), right);
    layout.verify_invariants();
}

#[test]
fn click_focus_preserves_view_and_keyboard_focus_reveals_window() {
    let mut layout = setup();
    let start = position(&layout);
    layout.activate_window_on_click(&2);
    check_ops_on_layout(&mut layout, [Op::CompleteAnimations, Op::Communicate(2)]);
    assert_eq!(position(&layout), start);
    assert_eq!(
        layout
            .active_workspace()
            .unwrap()
            .scrolling()
            .active_column_idx(),
        1
    );
    layout.activate_window(&2);
    check_ops_on_layout(&mut layout, [Op::CompleteAnimations]);
    assert_ne!(position(&layout), start);
    layout.verify_invariants();
}

#[test]
fn click_focus_reveals_window_when_almost_entirely_hidden() {
    let mut layout = setup();
    let start = position(&layout);
    layout.activate_window_on_click(&3);
    check_ops_on_layout(&mut layout, [Op::CompleteAnimations]);
    assert_ne!(position(&layout), start);
    layout.verify_invariants();
}

#[test]
fn click_scroll_threshold_is_one_percent_of_width() {
    for visible_fraction in [0.005, 0.015] {
        let mut layout = setup();
        let (right_edge, width) = layout
            .active_workspace()
            .unwrap()
            .scrolling()
            .tiles_with_render_positions()
            .find_map(|(tile, pos, _)| {
                (*tile.window().id() == 1).then(|| {
                    let width = tile.tile_size().w;
                    (pos.x + width, width)
                })
            })
            .unwrap();
        layout.update_options(Options {
            view_scroll_step: niri_config::ViewScrollStep(right_edge - width * visible_fraction),
            ..Default::default()
        });
        layout.scroll_view(true);
        let panned = position(&layout);
        layout.activate_window_on_click(&1);
        check_ops_on_layout(&mut layout, [Op::CompleteAnimations]);
        if visible_fraction > 0.01 {
            assert_eq!(position(&layout), panned);
        } else {
            assert!(position(&layout) < panned);
        }
        layout.verify_invariants();
    }
}

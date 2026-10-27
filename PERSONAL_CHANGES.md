# Personal niri changes

This file tracks the changes on the `personal` branch relative to upstream niri.
Update it when adding or changing personal behavior. `plan.md` contains the original
viewport-scrolling implementation plan; this file describes the implemented features.

## Viewport wheel scrolling

- `scroll-view-left` and `scroll-view-right` pan the viewport without changing focus.
- `view-scroll-step` sets the distance in logical pixels (default: 40).
- Panning stops at the content bounds and stays where it was left, without snapping
  to a column. Explicit keyboard focus navigation still reveals the focused column.
- Mod + wheel up pans left; Mod + wheel down pans right.

Example configuration:

```kdl
view-scroll-step 80
binds {
    Mod+WheelScrollUp { scroll-view-left; }
    Mod+WheelScrollDown { scroll-view-right; }
}
```

## Click focus without viewport movement

Ordinary mouse clicks focus a tiled window without moving the viewport when more
than 1% of its column width is visible. At 1% or less, normal focus scrolling reveals
the window. Visibility is measured horizontally against the output, using the
currently displayed viewport position.

This threshold is built into the fork. Clicking stops an existing viewport animation
at its displayed position. Keyboard focus navigation, overview selection, and
modifier-assisted move/resize keep their existing behavior. Floating windows retain
their usual activation and raising behavior.

## Screen-covering floating overlays

Additional window rules support overlays such as screenshot tools:

- `ignore-struts true`: resolve initial floating size, position, and bounds against
  the whole output, including areas reserved by panels or layout struts. Position
  offsets account for the working-area origin.
- `render-above-top-layer true`: render the window above the top layer-shell layer
  and allow it to receive clicks there.
- `open-animation false`: skip the window-opening animation for the matched window.

## Build, install, and launch

```sh
cargo build --release
./niri-local.sh --validate
```

`niri-local.sh --nested` runs the repository binary in a test window;
`niri-local.sh --session` starts a personal session from a TTY after logging out.
The full session uses `~/.config/niri-personal/config.kdl`.

On this machine, the login session launches `~/.local/bin/niri-personal`, so building
alone does not update the installed compositor. Install the new binary atomically:

```sh
install -m 755 target/release/niri ~/.local/bin/.niri-personal.new
mv -f ~/.local/bin/.niri-personal.new ~/.local/bin/niri-personal
~/.local/bin/niri-personal validate --config ~/.config/niri-personal/config.kdl
```

Log out and back in to load the installed binary. Config reloads cannot load new
compositor code. The personal config and systemd overrides live outside this repo.

## Verification

The viewport and click-focus regression tests run with:

```sh
cargo test --lib layout::tests::view_scroll
```

They cover preserved viewport positions, keyboard navigation, hidden-window reveal,
and click behavior on either side of the 1% visibility threshold.

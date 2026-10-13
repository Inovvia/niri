# View scrolling (personal fork)

Pan the tiled viewport with the vertical mouse wheel:

```kdl
view-scroll-step 40

binds {
    Mod+WheelScrollDown { scroll-view-right; }
    Mod+WheelScrollUp { scroll-view-left; }
}
```

`view-scroll-step` defaults to **40 logical pixels per binding activation**.
It accepts positive, finite integers or decimal numbers, including fractional
steps. Changes apply on config reload. Both actions also work through IPC:

```sh
niri msg action scroll-view-right
niri msg action scroll-view-left
```

The actions target the active workspace on the active monitor, following normal
column-action targeting rather than the monitor under the pointer. Focus stays
on the same window, even when it moves outside the viewport. Movement is immediate:
an existing viewport animation or gesture is replaced at its displayed position.
Stopping, releasing Mod, redraws, ordinary client updates, and config reloads do
not snap the viewport back to the focused column. Explicit focus or layout actions
can reposition the view normally.

Movement is bounded by the first and last column positions using the existing
gap, working-area, and centering rules. A single column that fits in the viewport
does not pan; an oversized column can pan between its edges. Empty workspaces,
fullscreen tiled focus, floating focus, and active interactive resizing are no-ops.
Layout/config changes can clamp the position to the new bounds.

Wheel bindings use niri's existing accumulated wheel ticks, including high-resolution
wheel events. Natural scrolling changes binding direction. There is no cooldown
in the example, so every activation moves by the configured step, except at bounds.
Without Mod, app scrolling follows the existing input path.

The bundled defaults include these bindings. Existing configurations need the
setting and bindings added separately. For a setup with managed shell bindings,
put overrides in `cfg/keybinds.kdl`, after the managed include. Use a separate fork
configuration while retaining stock niri as a fallback: stock niri does not
recognize these settings/actions.

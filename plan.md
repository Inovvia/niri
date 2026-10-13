# Personal niri fork: Mod + vertical wheel pans the view

## Goal

Your fork should behave like this:

- **Mod + wheel down:** move the viewport right.
- **Mod + wheel up:** move the viewport left.
- **Configurable scroll distance**, measured in logical pixels.
- **No column snapping after you stop.**
- Panning preserves the currently focused window.
- Plain wheel scrolling continues to work normally in apps.

Start with direct, non-animated movement. Smooth animation can be a separate improvement later.

**Important:** The configuration and action names below are the interface you will implement. They are not assumed to exist in stock niri. Exact source locations and APIs must be checked against your checkout.

---

## Phase 0 — Set up your fork

Fork `niri-wm/niri` on GitHub, then:

```bash
git clone git@github.com:YOURUSER/niri.git
cd niri

git remote add upstream https://github.com/niri-wm/niri.git
git fetch upstream

git switch -c personal upstream/main
git rev-parse HEAD
```

Record that last commit hash in your notes. It gives you a reproducible starting point.

Install the toolchain and build dependencies described by **that checkout’s** development documentation. Follow `rust-toolchain.toml` if present.

### Leave the touchpad patch out

Do **not** cherry-pick `free-view-scroll` initially.

Your requirement is mouse-wheel panning. Implementing that independently keeps the patch smaller and makes failures easier to diagnose. You can evaluate the touchpad patch separately later.

---

## Phase 1 — Establish a working baseline

Before editing anything:

```bash
cargo build
cargo test
./target/debug/niri --version
```

Resolve build failures first.

Use debug builds during development; build release binaries for final session testing.

Also inspect the available CLI:

```bash
./target/debug/niri --help
```

Check the current documentation for nested/windowed testing. Use it if supported, but still test the feature in a real session before adopting it.

---

## Phase 2 — Define the configuration interface

Implement this proposed configuration:

```kdl
view-scroll-step 40

binds {
    Mod+WheelScrollDown { scroll-view-right; }
    Mod+WheelScrollUp { scroll-view-left; }
}
```

### Meaning

`view-scroll-step` is the distance moved **per wheel binding activation**, in logical pixels.

| Value | Approximate feel |
|---|---|
| `20` | Fine positioning |
| `40` | Moderate nudges |
| `80` | Faster navigation |

Use `40` as the default.

### Validation rules

Accept a positive, finite numeric value.

Reject:

- Negative values.
- Zero.
- Non-finite values, if the parser can represent them.
- Incorrect argument types or extra arguments.

Rejecting zero avoids a confusing configuration that accepts the bindings but makes them do nothing.

### No cooldown

Do not add `cooldown-ms` initially. It can suppress activations and make the distance traveled depend on wheel timing.

### Modifier choice

Keep `Mod` for your requested behavior. If you later want literal **Alt + wheel**, change the modifier using the syntax supported by your build.

---

## Phase 3 — Trace the existing code

Before implementing, find these paths:

1. **Configuration parsing**
   - Where top-level settings are declared.
   - How defaults and validation are implemented.
   - How updated settings reach the layout.

2. **Action plumbing**
   - The configuration action enum.
   - The IPC action enum.
   - Conversions between them.
   - The central action dispatcher.

3. **Viewport behavior**
   - Current displayed viewport position.
   - Animation target position.
   - Animation cancellation.
   - Bounds or clamping rules.
   - Focus-driven viewport repositioning.

Useful starting points:

```bash
git grep -n "ViewOffset" -- src
git grep -n "WheelScroll" -- src niri-config
git grep -n "enum Action" -- src niri-config niri-ipc
git log --oneline -- src/input/mod.rs
```

Use a recently added, comparable action as your implementation template.

**Do not assume the complete dispatcher lives in `src/input/mod.rs`, or that assigning `ViewOffset::Static` alone guarantees persistent free panning.**

### Confirm wheel semantics

Inspect how vertical axis input becomes binding activations:

- Are events accumulated into discrete activations?
- How are high-resolution wheel events handled?
- Does natural scrolling affect binding direction?
- Which events are consumed when a modifier binding matches?

Your initial feature operates on **binding activations**, not raw wheel deltas. That keeps the scope small.

---

## Phase 4 — Add the setting and actions

### A. Configuration setting

Add `view-scroll-step` to the appropriate top-level configuration structure.

Implement:

- Default: `40`.
- Parsing and validation.
- Propagation to the code that handles viewport movement.
- Config reload behavior.
- Parsing tests.

Keep it out of a touchpad-only `gestures` structure.

### B. Actions

Add these parameterless actions:

```text
scroll-view-left
scroll-view-right
```

Wire them through every required layer:

- Configuration action enum.
- IPC action enum.
- Conversion mappings.
- Action dispatch.
- Any help, documentation, or exhaustive matches required by the repository.

If you expose them through IPC, they should use the same configured step as wheel bindings.

### C. Dispatch

Conceptually:

```text
scroll-view-left  → move view left by configured step
scroll-view-right → move view right by configured step
```

Determine the internal coordinate sign from the existing implementation. Do not assume positive means right.

Reuse niri’s normal action-targeting convention for choosing the monitor/workspace, and document it.

---

## Phase 5 — Implement non-snapping viewport movement

This is the main logic.

A conceptual layout method is:

```rust
pub fn scroll_view_by(&mut self, delta: f64) {
    // Resolve the currently displayed viewport position.
    // Cancel any conflicting viewport animation.
    // Apply delta using the layout's coordinate conventions.
    // Apply the appropriate viewport bounds.
    // Preserve focus and store a non-snapping position.
}
```

This is a sketch, not a drop-in implementation.

### Required behavior

#### Start from the displayed position

If the viewport is already animating, start from where it **currently appears**, not the old animation’s destination.

Otherwise, the first wheel activation could cause a jump.

#### Cancel conflicting movement

An old focus animation or gesture must not resume afterward and override the wheel movement.

Use existing cancellation/state-transition machinery where available.

#### Preserve focus

Moving the viewport must not automatically select another column or window.

#### Preserve the stopped position

Releasing Mod, waiting, or ordinary redraws must not cause a snap.

Audit layout updates carefully: an arbitrary static offset may still be overwritten by existing “keep the focused window visible” logic.

#### Handle bounds consistently

Reuse applicable layout bounds, accounting for existing gaps and centering behavior.

Define sensible behavior for:

- Empty workspaces: no-op.
- Single-column workspaces.
- Fullscreen windows: preferably no-op initially.
- Floating focus: follow existing workspace-action conventions.

**If persistent free panning conflicts with core layout assumptions, resolve that explicitly. Do not patch around it by disabling unrelated focus behavior globally.**

### Scope boundary

An explicit focus change may legitimately reposition the view. For example, clicking a partly hidden window can bring it into view.

That is different from the viewport snapping merely because you stopped scrolling.

---

## Phase 6 — Add bindings and documentation

### Your actual user configuration

Add the setting to a loaded top-level config file:

```kdl
view-scroll-step 40
```

Add the bindings to your setup’s **`cfg/keybinds.kdl` override**, following its existing structure:

```kdl
binds {
    Mod+WheelScrollDown { scroll-view-right; }
    Mod+WheelScrollUp { scroll-view-left; }
}
```

Do not edit the managed `shell/keybinds.kdl`.

### Fork defaults

Optionally add the same bindings to `resources/default-config.kdl`.

**Changing the bundled default does not update an existing user configuration.** Do both if you want your current setup and fresh configs to behave this way.

### Documentation

Document:

- Setting name, default, unit, and valid values.
- Both actions.
- Vertical-wheel example.
- Focus preservation.
- No snapping on stopping.
- The monitor/workspace targeting convention.

---

## Phase 7 — Test

### Automated checks

Follow any additional checks required by the repository:

```bash
cargo fmt --check
cargo test
cargo clippy
cargo build --release
```

Inspect the validation command:

```bash
./target/release/niri validate --help
```

Then validate the **actual config you will use**, with the supported config-path options.

### Automated test cases

Add focused tests for:

- Default step is `40`.
- Invalid values are rejected.
- Left and right move in opposite directions.
- Repeated actions accumulate movement correctly.
- Bounds are respected.
- Panning preserves focus.
- Animation interruption starts from the displayed position.
- Empty workspaces do not crash.
- Ordinary layout updates do not unexpectedly undo panning.

### Session checklist

- [ ] Mod + vertical wheel down pans right.
- [ ] Mod + vertical wheel up pans left.
- [ ] Changing `view-scroll-step` changes the movement distance.
- [ ] Config reload applies the new step.
- [ ] Stopping and releasing Mod produces no delayed snap.
- [ ] Rapid wheel input works without a cooldown.
- [ ] Plain wheel scrolling still works in apps.
- [ ] Releasing Mod immediately restores ordinary app scrolling.
- [ ] Panning during an existing animation does not jump.
- [ ] Closing or moving windows leaves valid viewport state.
- [ ] Empty, single-column, fullscreen, and floating cases behave as defined.
- [ ] Multi-monitor targeting follows the documented convention.
- [ ] Mixed-scale monitors use logical-pixel distances consistently.
- [ ] Workspace switching and ordinary focus navigation still work.

If directions are inverted, inspect the natural-scroll/binding handling before deciding whether to swap your personal bindings.

---

## Phase 8 — Commit the feature cleanly

Prefer two commits:

1. **Feature:** setting, actions, layout logic, tests, documentation.
2. **Personal defaults:** bundled wheel bindings.

Stage the relevant paths explicitly, then commit:

```bash
git commit -m \
    "Add configurable non-snapping viewport scroll actions"
```

For the default-config change:

```bash
git add resources/default-config.kdl
git commit -m "FORK: bind Mod+vertical wheel to viewport scrolling"
```

Check the resulting patch:

```bash
git log --oneline upstream/main..personal
git diff --stat upstream/main...personal
```

---

## Phase 9 — Install safely

**Do not overwrite `/usr/bin/niri`.**

Install your executable alongside the packaged version:

```bash
sudo install -Dm755 target/release/niri \
    /usr/local/bin/niri-personal
```

Create a separate test-session entry or launcher using your distro’s documented niri session setup.

Ensure it launches **`/usr/local/bin/niri-personal` explicitly**, including any required session wrapper/environment setup. Merely installing a differently named binary will not make your existing session use it.

Keep:

- The packaged niri executable.
- Its normal session entry.
- A known-good config.
- A way to log into a TTY for recovery.

For long-term use, a distro package built from your fork is another good option.

---

## Phase 10 — Maintain the fork

Before updating, keep a reference to the working version:

```bash
git switch personal
git branch backup/personal-working

git fetch upstream
git rebase upstream/main

cargo fmt --check
cargo test
cargo clippy
cargo build --release
```

Use a unique backup branch name for later updates.

Review upstream changes around input dispatch and viewport state, then repeat the important session tests **before replacing your installed personal binary**.

If upstream eventually adds equivalent actions, compare behavior and configuration before dropping your implementation.

---

## Expected effort

- **Initial prototype:** potentially a focused afternoon.
- **Reliable personal feature:** budget one to several days.
- **Longer if:** niri’s focus/layout invariants actively prevent persistent arbitrary viewport positions.

**Your minimal patch is now: one configurable step, two actions, vertical-wheel bindings, and a viewport movement path that genuinely stays unsnapped. No touchpad patch required.**

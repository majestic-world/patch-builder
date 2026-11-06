# Dark Theme

Status: ready-for-agent

## Problem Statement

Patch Builder only offers a fixed light appearance. A publisher working in a dark or low-light environment cannot choose a darker interface, even though the application already owns its window chrome and centralizes most visual colors. The operating system's color preference is intentionally ignored, and there is no explicit in-app alternative.

The missing option affects the entire application surface: window chrome, Source and Update tree fields, hash controls, comparison table, Build Summary, progress states, status indicators, and scrollbars. Adding an isolated dark background would leave these elements inconsistent and could make Source file status or control state harder to understand.

## Solution

Provide two explicit appearance choices: Light and Dark. Light remains the default for first runs and for existing settings that predate this feature. A compact icon control in the right side of the title bar switches between them immediately. The control is centered below the combined minimize and maximize area, leaving the close-button column unobstructed.

The chosen appearance applies to the whole interface, including standard Slint widgets, and is remembered across application restarts. Changing appearance is presentation-only: it does not start a Scan, clear the current Plan, interrupt a Build, change Source or Update tree settings, or alter the Manifest.

The light appearance remains visually unchanged. The dark appearance uses a cool charcoal neutral palette, preserves the existing monochrome identity, and provides dark-surface variants of New, Changed, and Unchanged status colors.

## User Stories

1. As a publisher, I want Patch Builder to open in the familiar light theme on first use, so that the feature does not unexpectedly change the current experience.
2. As a publisher, I want to switch to a dark theme, so that I can work comfortably in a dark or low-light environment.
3. As a publisher, I want to switch back to the light theme, so that I can choose the appearance appropriate to my environment.
4. As a publisher, I want the theme change to happen immediately, so that I do not need to restart Patch Builder.
5. As a publisher, I want the theme control in the title bar at the requested right-side position, so that it is always available without competing with the application workflow.
6. As a publisher, I want the theme control below the combined minimize and maximize area, so that the close-button column remains visually and functionally distinct.
7. As a publisher, I want to see a moon icon while using Light, so that the control communicates that it will activate Dark.
8. As a publisher, I want to see a sun icon while using Dark, so that the control communicates that it will activate Light.
9. As a keyboard or assistive-technology user, I want the theme control to expose the action it will perform, so that the icon-only control is understandable without relying on its shape.
10. As a publisher, I want the whole window surface to adopt the selected theme, so that no large bright or dark area remains inconsistent.
11. As a publisher, I want the Source and Update tree fields to adopt the selected theme, so that the primary folder controls remain readable.
12. As a publisher, I want the hash checkboxes to adopt the selected theme, so that selected, unselected, enabled, and disabled states remain clear.
13. As a publisher, I want the Rescan and Build controls to adopt the selected theme, so that primary and secondary actions preserve their visual hierarchy.
14. As a publisher, I want active Scan and Build progress states to remain readable in both themes, so that I can monitor work without changing appearance.
15. As a publisher, I want the comparison table and its headers to adopt the selected theme, so that Source file information remains coherent.
16. As a publisher, I want hover and selected table rows to remain distinguishable in Dark, so that navigation feedback is not lost.
17. As a publisher, I want the custom scrollbar to adopt the selected theme, so that its track, thumb, hover, and pressed states remain visible.
18. As a publisher, I want the Build Summary and its cards to adopt the selected theme, so that Manifest, Archive, and reuse information remains legible.
19. As a publisher, I want New, Changed, and Unchanged indicators to retain their meanings in Dark, so that theme selection never changes the interpretation of a Plan.
20. As a publisher, I want text, hashes, icons, borders, guides, and dividers to have sufficient contrast in both themes, so that detailed Source file information remains readable.
21. As a publisher, I want window controls to retain clear hover and pressed feedback in both themes, so that minimizing, maximizing, restoring, and closing remain predictable.
22. As a publisher, I want title-bar dragging and double-click maximization to keep working around the new control, so that custom window chrome behaves as before.
23. As a publisher, I want the theme control to consume its own click, so that switching themes never starts a window drag or toggles maximization.
24. As a publisher, I want the control to remain correctly positioned at the minimum window size, so that it does not overlap the product title or window controls.
25. As a publisher, I want the control to remain correctly positioned when the window is maximized or restored, so that the title bar stays stable.
26. As a publisher, I want my selected theme remembered after closing Patch Builder, so that I do not need to choose it every session.
27. As an existing publisher, I want settings created by older versions to continue loading as Light, so that the settings upgrade is backward compatible.
28. As a publisher, I want a theme change to leave the current Source, Update tree, and hash selection unchanged, so that appearance cannot alter my publishing configuration.
29. As a publisher, I want a theme change to leave the current comparison and Plan intact, so that appearance cannot invalidate the next Build.
30. As a publisher, I want switching themes during a Scan or Build to leave that work running, so that a presentation choice cannot interrupt publishing work.
31. As a publisher, I want a theme change not to start another Scan, so that choosing an appearance does not rehash the Source.
32. As a publisher, I want Patch Builder to report when my theme preference cannot be saved, so that I understand why it may not be restored next time.
33. As a publisher, I want the selected theme to remain active for the current session even if persistence fails, so that a storage error does not undo my immediate choice.
34. As a publisher, I want the explicit in-app selection to take precedence over the operating system theme, so that Patch Builder behaves consistently with my choice.
35. As a publisher, I want the existing logo, typography, spacing, and information hierarchy preserved, so that Dark feels like the same Patch Builder rather than a redesign.
36. As a publisher, I want Light to retain its existing appearance and states, so that adding Dark does not regress the default interface.

## Implementation Decisions

- Appearance has exactly two states, represented by a boolean dark-theme setting. `false` means Light and is the default. A system-following state is not introduced.
- The root window exposes the current appearance to the UI and emits a presentation-only theme-change action for persistence.
- The global theme remains the single source of semantic visual tokens. Components continue consuming semantic surface, text, border, action, status, scrollbar, shadow, and window-control values rather than branching on appearance individually.
- Existing Light token values remain unchanged.
- The Dark base palette is cool charcoal: main surface `#18191D`, elevated surface `#1E2025`, control surface `#24262C`, border `#373A43`, primary text `#F3F4F6`, and secondary text/icon `#A9ADB7`.
- The primary Build action gains semantic foreground and progress-overlay tokens. In Dark, it uses a light neutral action surface with dark content, preserving strong hierarchy without relying on hard-coded white content.
- New, Changed, and Unchanged receive explicit dark-surface dot, text, and fill variants. Their green, amber, and slate meanings remain unchanged.
- The Slint widget palette follows the explicit application appearance. This keeps standard widgets, especially hash checkboxes, visually aligned with custom controls. It does not read or follow the operating system preference.
- The title bar remains 78 logical pixels high. The theme control is 30 by 30 logical pixels, vertically positioned near 40 pixels and centered on the combined minimize/maximize region, whose horizontal center is 92 pixels from the right edge. This leaves 8 pixels before the title bar's lower divider and does not occupy the close-button column.
- The control shows the destination appearance: moon in Light and sun in Dark. It uses the same restrained hover and pressed language as the non-destructive window controls.
- The icon assets follow the existing Lucide-derived visual style and repository licensing convention.
- The icon-only control exposes a dynamic accessible label and hint equivalent to “Use dark theme” and “Use light theme.”
- The theme control intercepts pointer interaction before the title bar's drag and double-click handlers.
- The saved-settings schema gains a defaulted dark-theme field. Missing data from older settings resolves to Light without migration code or rejection of the file.
- Saved appearance is applied before the window is shown, preventing an avoidable Light-to-Dark startup flash.
- Settings persistence is separated from the operation that starts a Scan. Folder or hash changes still save and then Scan; appearance changes save without scanning.
- A persistence failure keeps the selected appearance for the current process and uses the existing settings-error reporting behavior.
- Appearance changes do not mutate Source, Update tree, hash selection, comparison rows, Plan, Manifest data, Scan progress, or Build progress.
- No ADR governing Scan, Plan, or Build behavior changes. In particular, a Build continues applying the last successful Scan.

## Testing Decisions

- Tests assert consumer-visible behavior, compatibility boundaries, and workflow invariants. They do not assert token names, source text, callback forwarding, exact internal bindings, or the presence of implementation-specific properties.
- The highest UI seam is the running application with its populated preview data. One smoke scenario exercises the actual title bar, standard widgets, comparison table, status variants, Build Summary, disabled actions, and scrollbar in both appearances.
- The preview smoke verifies immediate Light-to-Dark and Dark-to-Light switching, icon changes, accessible action labels, and the absence of visual gaps or unreadable states.
- The smoke scenario exercises hover, pressed, selected, enabled, disabled, progress-capable, and scrollbar states where they are observable on the real surface.
- The smoke scenario verifies the minimum window size, initial size, maximized state, restored state, title-bar dragging, double-click maximization, and all three existing window controls.
- The smoke scenario changes appearance while comparison data is visible and confirms that rows, summary data, and the current Plan presentation remain intact and no Scan progress begins.
- Persistence is tested at the existing saved-settings serialization seam. This is the narrowest permanent automated test needed because the repository already treats settings JSON compatibility there.
- A legacy settings document without the appearance field must deserialize to Light.
- Light and Dark settings must each serialize and deserialize while preserving Source, Update tree, and hash-algorithm values.
- A real restart smoke uses isolated application-data storage: select Dark, close, reopen, and observe Dark before interaction; then select Light and repeat.
- A persistence-failure smoke confirms that the active appearance changes immediately, an error is reported, and no Scan begins.
- The Slint compilation step is relied upon to validate component properties, global bindings, icon resources, and callback signatures.
- The Rust test suite remains the regression check for settings compatibility and for unrelated Scan, Plan, Build, Archive, and Manifest behavior.
- Visual acceptance checks normal text contrast of at least 4.5:1. The proposed primary text, secondary text, primary action, and status-text combinations exceed this threshold.
- Automated screenshot or token-value tests are not added. They would bind the suite to incidental rendering details while providing less confidence than the real application smoke.
- Existing saved-settings unit tests are the prior art for schema-default and round-trip coverage; the existing preview mode is the prior art for a populated visual verification surface.

## Out of Scope

- Following the Windows or operating-system appearance automatically.
- Adding a third “System” appearance option.
- Scheduling appearance changes by time of day.
- Redesigning the title bar, window controls, application logo, typography, content layout, or spacing system.
- Adding a settings screen or theme picker beyond the title-bar control.
- Introducing user-defined colors, additional themes, or accent-color customization.
- Changing Source, Update tree, hash selection, Scan, Plan, Build, Archive, or Manifest semantics.
- Adding UI screenshot regression infrastructure or a new automated UI-test harness solely for this feature.

## Further Notes

- The feature is deliberately a clean two-state cutover: no deprecated theme path, compatibility alias, or system-theme shim is needed.
- Light is both the product default and the compatibility default. A user-selected Dark value is nevertheless restored on subsequent normal launches.
- The dark palette keeps the current neutral visual identity. Status colors are the only purposeful chromatic elements, matching the existing Light design principle.
- Theme switching remains available during Scan and Build because it changes presentation only. It must not affect work ownership, progress reporting, or the in-memory Plan.
- The implementation should be visually reviewed on the actual Windows borderless window because the header combines custom Slint interaction with native window movement and resizing.

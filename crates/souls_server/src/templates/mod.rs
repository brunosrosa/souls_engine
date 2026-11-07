//! Modular Template Assembly Engine for Souls Engine Headless Dashboard.
//!
//! Stitched at compile-time via `concat!` and `include_str!` for zero runtime disk I/O,
//! zero parsing overhead, and single-binary portability.

/// Static compile-time stitched Single Page Application (SPA) HTML.
pub const DASHBOARD_HTML: &str = concat!(
    include_str!("base_head.html"),
    "  <style>\n",
    include_str!("styles.css"),
    include_str!("base_body_start.html"),
    include_str!("components/topbar.html"),
    include_str!("components/navigation.html"),
    include_str!("tabs/overview.html"),
    include_str!("tabs/subagents.html"),
    include_str!("tabs/local_models.html"),
    include_str!("tabs/cloud_arena.html"),
    include_str!("tabs/skills_hub.html"),
    include_str!("settings/layout_start.html"),
    include_str!("settings/sidebar.html"),
    include_str!("settings/content_start.html"),
    include_str!("settings/subtab_brain.html"),
    include_str!("settings/subtab_tools.html"),
    include_str!("settings/subtab_reasoning.html"),
    include_str!("settings/subtab_finops.html"),
    include_str!("settings/subtab_router.html"),
    include_str!("settings/subtab_memory.html"),
    include_str!("settings/subtab_sandbox.html"),
    include_str!("settings/subtab_obsidian.html"),
    include_str!("settings/subtab_raw.html"),
    include_str!("settings/layout_end.html"),
    "  <script>\n",
    include_str!("scripts/dashboard.js"),
    "\n",
    include_str!("scripts/skills_hub.js"),
    "  </script>\n",
    include_str!("components/modals.html"),
    include_str!("base_body_end.html")
);

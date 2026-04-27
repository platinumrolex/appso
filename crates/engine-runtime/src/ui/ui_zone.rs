use crate::ui::header::{EngineHeader, EngineHeaderAction, RuntimeZone, ScaledMetrics};
use wgpu_ui::{HoverEffect, primitives::UiAction};

// top level ui zone
#[derive(PartialEq, Eq, Debug, Clone, Copy, Default)]
pub enum UiZone {
    Runtime(RuntimeZone),
    #[default]
    App,
}

// The top-level action that InteractionState will track
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum RootAction<T: UiAction> {
    Runtime(RuntimeAction), // Internal engine UI (Header, Sidebar)
    App(T),                 // Injected logic (Website/App specific)
    #[default]
    None,
}

// Internal Engine UI Actions
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RuntimeAction {
    Header(EngineHeaderAction),
   // Sidebar(SidebarAction),
    // Dropdown is usually handled via Header state or its own variant
}

// Implementation of the Trait for the Root
impl<T: UiAction> UiAction for RootAction<T> {
    fn is_interactive(&self) -> bool {
        match self {
            RootAction::Runtime(r) => match r {
                RuntimeAction::Header(h) => h.is_interactive(),
              //  RuntimeAction::Sidebar(s) => s.is_interactive(),
            },
            RootAction::App(a) => a.is_interactive(),
            RootAction::None => false,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct InteractionState<A: UiAction> {
    pub zone: UiZone,
    pub hovered: Option<A>,
    pub active: Vec<A>,
    pub is_hover_visual: bool,
}

impl<A: UiAction> InteractionState<A> {
    /// 1. Updates the high-level zone and returns true if it changed
    pub fn update_zone(&mut self, mouse_pos: (f32, f32), screen_width: f32, metrics: &ScaledMetrics, header: &mut EngineHeader) {
        self.zone = match header.zone_at(mouse_pos, screen_width, metrics) {
            Some(zone) => UiZone::Runtime(zone),
            None => UiZone::App,
        };
    }

    pub fn check_hovered(&mut self, next_action: A, hover_data: Option<HoverEffect>) -> bool {
        let action_changed = self.hovered != Some(next_action);
        
        // Does the current region have a visual hover state?
        let wants_visual = match hover_data {
            Some(h) => !matches!(h, HoverEffect::None),
            None => false,
        };

        // We only trigger a redraw if:
        // 1. We just entered a visual region (it needs to light up)
        // 2. We just left a visual region (it needs to return to idle)
        let needs_redraw = action_changed && (wants_visual || self.is_hover_visual);

        // ALWAYS update the logical state (for tooltips/cursors/clicks)
        self.hovered = Some(next_action);
        
        // Update the visual memory for the next frame
        self.is_hover_visual = wants_visual;

        needs_redraw
    }

    pub fn clear_hover(&mut self) {
        self.hovered = None;
        self.is_hover_visual = false;
    }
}

/// Determines which logical UI zone the mouse currently occupies.
/// Delegates all geometry checks to `EngineHeader::zone_at()`.
pub fn determine_active_zone(
    mouse_pos: (f32, f32),
    screen_width: f32,
    metrics: &ScaledMetrics,
    header: &mut EngineHeader,
) -> UiZone {
    match header.zone_at(mouse_pos, screen_width, metrics) {
        Some(zone) => UiZone::Runtime(zone),
        None => UiZone::App,
    }
}
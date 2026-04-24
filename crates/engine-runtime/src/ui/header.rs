use wgpu_text::glyph_brush::{Layout, Section, Text};
use winit::window::Window;
use wgpu_ui::{
    ui, ButtonStyle, Primitive, HitRegion, HoverEffect, Rect,
    Button, CustomTitle, Selector, SelectorOption, Container,
};
use wgpu_ui::primitives::UiAction;

use crate::RuntimeZone;
//use crate::ui::dropdown::{DropdownMenu, DropdownEntry, DropdownOption};


// ... rest unchanged, but use `ui!` instead of `ui_tree!`


pub const BASE_HEADER_H: f32 = 32.0;
pub const BASE_BTN_W: f32 = 46.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingsAttention { None, Needed, Required }

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct FpsLimit { pub value: u32, pub is_auto: bool }

impl FpsLimit {
    pub fn label(&self) -> &'static str {
        if self.is_auto { "Auto (Sync)" }
        else {
            match self.value {
                30 => "30 FPS", 60 => "60 FPS", 144 => "144 FPS", 240 => "240 FPS", _ => "?? FPS",
            }
        }
    }
}

pub struct ScaledMetrics {
    pub header_h: f32,
    pub btn_w: f32,
    pub dropdown_w: f32,
    pub row_h: f32,
    pub scale: f32,
}

impl ScaledMetrics {
    pub fn new(scale: f32) -> Self {
        Self {
            header_h: BASE_HEADER_H * scale,
            btn_w: BASE_BTN_W * scale,
            dropdown_w: 280.0 * scale,
            row_h: 28.0 * scale,
            scale,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HeaderAction {
    None, Drag, SettingsSelector, FpsSelector,
    SetFpsAuto, SetFps30, SetFps60, SetFps144, SetFps240,
    Minimize, Maximize, Close,
    CloseSubmenu,
}
impl UiAction for HeaderAction {
    fn is_interactive(&self) -> bool {
        match self {
            // Drag and None are "background" states
            HeaderAction::None | HeaderAction::Drag => false,
            // Everything else is a button/selector
            _ => true,
        }
    }
}

pub struct EngineHeader {
    pub title: String,
    pub settings_dropdown_open: bool,
    pub fps_selector_open: bool,
    pub settings_attention: SettingsAttention,
    pub current_fps: FpsLimit,
    
    // UI cache
    cached_primitives: Vec<Primitive>,
    pub cached_hits: Vec<HitRegion<HeaderAction>>,
    cache_valid: bool,
    cached_window_width: f32,
    cached_is_maximized: bool,
}

impl EngineHeader {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            settings_dropdown_open: false,
            fps_selector_open: false,
            settings_attention: SettingsAttention::None,
            current_fps: FpsLimit { value: 144, is_auto: true },
            cached_primitives: Vec::new(),
            cached_hits: Vec::new(),
            cache_valid: false,
            cached_window_width: 0.0,
            cached_is_maximized: false,
        }
    }

    /// Build UI primitives and hit regions for current state (no caching).
    fn build_ui(&self, window_width: f32, metrics: &ScaledMetrics, is_maximized: bool) -> (Vec<Primitive>, Vec<HitRegion<HeaderAction>>) {
    let scale = metrics.scale;
        let btn_w = metrics.btn_w;
        let header_h = metrics.header_h;
        let settings_x = window_width - (btn_w * 4.0);
        let min_x = window_width - (btn_w * 3.0);
        let max_x = window_width - (btn_w * 2.0);
        let close_x = window_width - btn_w;
        let dropdown_x = settings_x - (160.0 * scale);
        let row_h = 28.0 * scale;

        let mut settings_style = ButtonStyle::icon();
        if self.settings_dropdown_open {
            settings_style.bg_idle = settings_style.bg_hover;
            settings_style.text_idle = settings_style.text_hover;
        }

        // Main buttons
        let (mut primitives, mut hits) = ui! {
            root {
                title: CustomTitle {
                text: self.title.clone(),
                size: 13.0,
                color: [0.9, 0.9, 0.9, 1.0],
            } at (16.0 * scale, 9.0 * scale, 0.0, 0.0)

            settings_btn: Button {
                label: "0".into(),
                action: HeaderAction::SettingsSelector,
                style: settings_style,
            } at (settings_x, 0.0, btn_w, header_h)

            minimize_btn: Button {
                label: "─".into(),
                action: HeaderAction::Minimize,
                style: ButtonStyle::icon(),
            } at (min_x, 0.0, btn_w, header_h)

            maximize_btn: Button {
                label: if is_maximized { "1".into() } else { "2".into() },
                action: HeaderAction::Maximize,
                style: ButtonStyle::icon(),
            } at (max_x, 0.0, btn_w, header_h)

            close_btn: Button {
                label: "X".into(),
                action: HeaderAction::Close,
                style: ButtonStyle::danger(),
            } at (close_x, 0.0, btn_w, header_h)
        }
    };


        // Dropdown (conditionally)
        if self.settings_dropdown_open {
            let options = vec![
                SelectorOption {
                    label: "Auto (Sync)".into(),
                    selected: self.current_fps.is_auto,
                    action: HeaderAction::SetFpsAuto,
                },
                SelectorOption {
                    label: "30 FPS".into(),
                    selected: !self.current_fps.is_auto && self.current_fps.value == 30,
                    action: HeaderAction::SetFps30,
                },
                SelectorOption {
                    label: "60 FPS".into(),
                    selected: !self.current_fps.is_auto && self.current_fps.value == 60,
                    action: HeaderAction::SetFps60,
                },
                SelectorOption {
                    label: "144 FPS".into(),
                    selected: !self.current_fps.is_auto && self.current_fps.value == 144,
                    action: HeaderAction::SetFps144,
                },
                SelectorOption {
                    label: "240 FPS".into(),
                    selected: !self.current_fps.is_auto && self.current_fps.value == 240,
                    action: HeaderAction::SetFps240,
                },
            ];

            let (mut sel_prims, mut sel_hits) = ui! {
                root {
                    fps_selector: Selector {
                        label: "Frame limit".into(),
                        current: self.current_fps.label().to_string(),
                        toggle_action: HeaderAction::FpsSelector,
                        expanded: self.fps_selector_open,
                        style: ButtonStyle::primary(),
                        options: options,
                    } at (dropdown_x, header_h, 240.0 * scale, row_h)
                }
            };
            primitives.append(&mut sel_prims);
            hits.append(&mut sel_hits);
        }

        // Attention dot (unchanged)
        if self.settings_attention != SettingsAttention::None {
            let color = if self.settings_attention == SettingsAttention::Required {
                [0.91, 0.07, 0.07, 1.0]
            } else {
                [1.0, 0.8, 0.0, 1.0]
            };
            primitives.push(Primitive::Rect {
                x: settings_x + btn_w/2.0 + 4.0*scale - 5.0,
                y: 6.0*scale - 5.0,
                w: 10.0,
                h: 10.0,
                color,
                corner_radius: 5.0,
            });
        }

        (primitives, hits)
    }

    /// Rebuild the UI cache if dimensions or state changed.
    fn ensure_cache(&mut self, window_width: f32, metrics: &ScaledMetrics, is_maximized: bool) {
        let state_changed = !self.cache_valid 
            || self.cached_window_width != window_width 
            || self.cached_is_maximized != is_maximized;
        
        if state_changed {
            let (primitives, hits) = self.build_ui(window_width, metrics, is_maximized);
            self.cached_primitives = primitives;
            self.cached_hits = hits;
            self.cache_valid = true;
            self.cached_window_width = window_width;
            self.cached_is_maximized = is_maximized;
        }
    }

    /// Invalidate the cache (call after any state-changing action).
    fn invalidate_cache(&mut self) {
        self.cache_valid = false;
    }

    pub fn zone_at(&mut self, mouse: (f32, f32), window_width: f32, metrics: &ScaledMetrics) -> Option<RuntimeZone> {
        self.ensure_cache(window_width, metrics, false);
        if self.settings_dropdown_open {
            for hit in self.cached_hits.iter().rev() {
                if hit.bounds.contains(mouse) && hit.bounds.y >= metrics.header_h {
                    return Some(RuntimeZone::Dropdown);
                }
            }
        }
        if mouse.1 <= metrics.header_h { Some(RuntimeZone::Header) } else { None }
    }

    pub fn action_at(&self, mouse: (f32, f32), width: f32, metrics: &ScaledMetrics) -> HeaderAction {
        // Convert screen mouse to header-local mouse if necessary
        // (Assuming header is always at top 0,0)
        
        for region in &self.cached_hits {
            if region.bounds.contains(mouse) {
                return region.action;
            }
        }
        
        // If mouse is in header area but not on a button, it's a Drag zone
        if mouse.1 <= metrics.header_h {
            return HeaderAction::Drag;
        }

        HeaderAction::None
    }

   // pub fn action_at(&mut self, mouse: (f32, f32), window_width: f32, metrics: &ScaledMetrics) -> HeaderAction {
   //     self.ensure_cache(window_width, metrics, false);
   //     for hit in self.cached_hits.iter().rev() {
   //         if hit.bounds.contains(mouse) {
   //             return hit.action;
   //         }
   //     }
   //     if mouse.1 <= metrics.header_h { HeaderAction::Drag } else { HeaderAction::None }
   // }

    pub fn handle_action(&mut self, window: &Window, action: HeaderAction) -> bool {
        let changed = match action {
            HeaderAction::SettingsSelector => {
                self.settings_dropdown_open = !self.settings_dropdown_open;
                if !self.settings_dropdown_open { self.fps_selector_open = false; }
                true
            }
            HeaderAction::CloseSubmenu => { self.fps_selector_open = false; true }
            HeaderAction::FpsSelector => { self.fps_selector_open = !self.fps_selector_open; true }
            HeaderAction::SetFpsAuto => { self.current_fps.is_auto = true; self.fps_selector_open = false; true }
            HeaderAction::SetFps30 => { self.current_fps.value = 30; self.current_fps.is_auto = false; self.fps_selector_open = false; true }
            HeaderAction::SetFps60 => { self.current_fps.value = 60; self.current_fps.is_auto = false; self.fps_selector_open = false; true }
            HeaderAction::SetFps144 => { self.current_fps.value = 144; self.current_fps.is_auto = false; self.fps_selector_open = false; true }
            HeaderAction::SetFps240 => { self.current_fps.value = 240; self.current_fps.is_auto = false; self.fps_selector_open = false; true }
            HeaderAction::Drag => { let _ = window.drag_window(); true }
            HeaderAction::Minimize => { window.set_minimized(true); true }
            HeaderAction::Maximize => { window.set_maximized(!window.is_maximized()); true }
            _ => false,
        };
        if changed {
            self.invalidate_cache();
        }
        changed
    }

    /// Returns background rects with hover effects applied.
    pub fn get_background_rects(&mut self, window_width: f32, metrics: &ScaledMetrics, mouse_pos: (f32, f32), is_pressed: bool) -> Vec<(f32, f32, f32, f32, [f32; 4], f32)> {
        self.ensure_cache(window_width, metrics, false);
        let mut rects = Vec::new();
        let mut hit_idx = 0;
        for prim in &self.cached_primitives {
            if let Primitive::Rect { x, y, w, h, color, corner_radius } = prim {
                let hover_effect = self.cached_hits.get(hit_idx).map(|h| &h.hover);
                let final_color = if let Some(effect) = hover_effect {
                    let hovered = self.cached_hits[hit_idx].bounds.contains(mouse_pos);
                    effect.resolve_bg(hovered, is_pressed).unwrap_or(*color)
                } else {
                    *color
                };
                let radius = hover_effect.map(|e| e.corner_radius()).unwrap_or(*corner_radius);
                if final_color[3] > 0.0 {
                    rects.push((*x, *y, *w, *h, final_color, radius));
                }
                hit_idx += 1;
            }
        }
        rects
    }

    /// Generates wgpu_text Sections with hover-aware text colors.
    pub fn sections<'a>(&'a mut self, window_width: f32, is_maximized: bool, mouse_pos: (f32, f32), active_zone: crate::ui::ui_zone::UiZone, metrics: &ScaledMetrics) -> Vec<Section<'a>> {
        self.ensure_cache(window_width, metrics, is_maximized);
        let scale = metrics.scale;
        let is_header_active = matches!(active_zone, crate::ui::ui_zone::UiZone::Runtime(_));
        let effective_mouse = if is_header_active { mouse_pos } else { (-1.0, -1.0) };

        let mut sections = Vec::new();
        let mut hit_idx = 0;
        for prim in &self.cached_primitives {
            match prim {
                Primitive::Text { content, x, y, color, size, h_align, v_align } => {
                    let hovered = self.cached_hits.get(hit_idx).map_or(false, |h| h.bounds.contains(effective_mouse));
                    let text_color = if let Some(HitRegion { hover: HoverEffect::Button { text_idle, text_hover, .. }, .. }) = self.cached_hits.get(hit_idx) {
                        if hovered { text_hover } else { text_idle }
                    } else {
                        color
                    };
                    sections.push(
                        Section::default()
                            .add_text(
                                Text::new(content)
                                    .with_color(*text_color)
                                    .with_scale(*size * scale)
                            )
                            .with_screen_position((*x, *y))
                            .with_layout(
                                Layout::default()
                                    .h_align(*h_align)
                                    .v_align(*v_align)
                            )
                    );
                    hit_idx += 1;
                }
                Primitive::Rect { .. } => {
                    hit_idx += 1;
                }
            }
        }
        sections
    }
}


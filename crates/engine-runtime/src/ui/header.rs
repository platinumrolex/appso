// header.rs
use std::borrow::Cow;
use wgpu_text::glyph_brush::{Layout, Section, Text};
use winit::window::Window;
use wgpu_ui::{
    ui, ButtonStyle, Primitive, HoverEffect, Rect,
    Button, CustomTitle, Selector, SelectorOption, Container, Interaction
};
use wgpu_ui::primitives::UiAction;

use crate::ui::ui_zone::{RuntimeZone, UiZone};
use nested_enum_macros::ui_blueprint;

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

pub struct EngineHeader {
    pub title: String,
    pub settings_dropdown_open: bool,
    pub fps_selector_open: bool,
    pub settings_attention: SettingsAttention,
    pub current_fps: FpsLimit,
    
    cached_primitives: Vec<Primitive<EngineHeaderAction>>,
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
            cache_valid: false,
            cached_window_width: 0.0,
            cached_is_maximized: false,
        }
    }
    
    pub fn invalidate_cache(&mut self) {
        self.cache_valid = false;
    }
}

// -----------------------------------------------------------------------------
// THE UNIFIED BLUEPRINT
// Generates EngineHeaderAction and injects all UI logic.
// -----------------------------------------------------------------------------
ui_blueprint! {
    EngineHeader {
        
        // 1. HIGH-PERFORMANCE RENDER
        pub fn render_ui(&self, primitives: &mut Vec<Primitive<EngineHeaderAction>>, window_width: f32, metrics: &ScaledMetrics, is_maximized: bool) {
            primitives.clear();
            
            let scale = metrics.scale;
            let btn_w = metrics.btn_w;
            let header_h = metrics.header_h;
            let settings_x = window_width - (btn_w * 4.0);
            let min_x = window_width - (btn_w * 3.0);
            let max_x = window_width - (btn_w * 2.0);
            let close_x = window_width - btn_w;
            let dropdown_x = settings_x - (160.0 * scale);

            ui!(@to primitives, {
                root {
                    title: CustomTitle {
                        text: self.title.clone(),
                        size: 13.0,
                        color: [0.9, 0.9, 0.9, 1.0],
                    } at (16.0 * scale, 9.0 * scale, 0.0, 0.0)

                    settings_btn: Button {
                        label: "0".into(),
                        action: EngineHeaderAction::ToggleSettings, 
                        style: ButtonStyle::icon(),
                    } at (settings_x, 0.0, btn_w, header_h)

                    minimize_btn: Button {
                        label: "─".into(),
                        action: EngineHeaderAction::MinimizeWindow,
                        style: ButtonStyle::icon(),
                    } at (min_x, 0.0, btn_w, header_h)

                    maximize_btn: Button {
                        label: if is_maximized { "1".into() } else { "2".into() },
                        action: EngineHeaderAction::MaximizeWindow,
                        style: ButtonStyle::icon(),
                    } at (max_x, 0.0, btn_w, header_h)

                    close_btn: Button {
                        label: "X".into(),
                        action: EngineHeaderAction::CloseWindow,
                        style: ButtonStyle::danger(),
                    } at (close_x, 0.0, btn_w, header_h)
                }
            });

            if self.settings_dropdown_open {
                let options = vec![
                    SelectorOption { label: "Auto (Sync)".into(), selected: self.current_fps.is_auto, action: EngineHeaderAction::SetFpsAuto },
                    SelectorOption { label: "30 FPS".into(), selected: !self.current_fps.is_auto && self.current_fps.value == 30, action: EngineHeaderAction::SetFps30 },
                    SelectorOption { label: "60 FPS".into(), selected: !self.current_fps.is_auto && self.current_fps.value == 60, action: EngineHeaderAction::SetFps60 },
                    SelectorOption { label: "144 FPS".into(), selected: !self.current_fps.is_auto && self.current_fps.value == 144, action: EngineHeaderAction::SetFps144 },
                    SelectorOption { label: "240 FPS".into(), selected: !self.current_fps.is_auto && self.current_fps.value == 240, action: EngineHeaderAction::SetFps240 },
                ];

                ui!(@to primitives, {
                    dropdown_zone {
                        fps_selector: Selector {
                            label: "Frame limit".into(),
                            current: self.current_fps.label().into(), 
                            toggle_action: EngineHeaderAction::ToggleFpsMenu,
                            expanded: self.fps_selector_open,
                            style: ButtonStyle::primary(),
                            options: options,
                        } at (dropdown_x, header_h, 240.0 * scale, 28.0 * scale)
                    }
                });
            }

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
                    interaction: None,
                });
            }
        }

        // 2. THE LOGIC HANDLER
        pub fn handle_action(&mut self, window: &Window, action: EngineHeaderAction) -> bool {
            let mut changed = true;
            match action {
                EngineHeaderAction::ToggleSettings => {
                    self.settings_dropdown_open = !self.settings_dropdown_open;
                    if !self.settings_dropdown_open { self.fps_selector_open = false; }
                }
                EngineHeaderAction::ToggleFpsMenu => { self.fps_selector_open = !self.fps_selector_open; }
                EngineHeaderAction::SetFpsAuto => { self.current_fps.is_auto = true; self.fps_selector_open = false; }
                EngineHeaderAction::SetFps30   => { self.current_fps = FpsLimit { value: 30, is_auto: false }; self.fps_selector_open = false; }
                EngineHeaderAction::SetFps60   => { self.current_fps = FpsLimit { value: 60, is_auto: false }; self.fps_selector_open = false; }
                EngineHeaderAction::SetFps144  => { self.current_fps = FpsLimit { value: 144, is_auto: false }; self.fps_selector_open = false; }
                EngineHeaderAction::SetFps240  => { self.current_fps = FpsLimit { value: 240, is_auto: false }; self.fps_selector_open = false; }
                EngineHeaderAction::MinimizeWindow => { window.set_minimized(true); }
                EngineHeaderAction::MaximizeWindow => { window.set_maximized(!window.is_maximized()); }
                EngineHeaderAction::CloseWindow => { /* close logic */ }
                EngineHeaderAction::Drag => { let _ = window.drag_window(); }
                EngineHeaderAction::None => { changed = false; }
            }
            if changed { self.invalidate_cache(); }
            changed
        }

        // 3. CACHE & HIT TESTING
        fn ensure_cache(&mut self, window_width: f32, metrics: &ScaledMetrics, is_maximized: bool) {
            let state_changed = !self.cache_valid 
                || self.cached_window_width != window_width 
                || self.cached_is_maximized != is_maximized;
                
            if state_changed {
                let mut tmp_prims = std::mem::take(&mut self.cached_primitives);
                self.render_ui(&mut tmp_prims, window_width, metrics, is_maximized);
                self.cached_primitives = tmp_prims;
                
                self.cache_valid = true;
                self.cached_window_width = window_width;
                self.cached_is_maximized = is_maximized;
            }
        }

        pub fn zone_at(&mut self, mouse: (f32, f32), window_width: f32, metrics: &ScaledMetrics) -> Option<RuntimeZone> {
            self.ensure_cache(window_width, metrics, false);
            let (x, y) = mouse;
            
            if self.settings_dropdown_open {
                let scale = metrics.scale;
                let btn_w = metrics.btn_w;
                let settings_x = window_width - (btn_w * 4.0);
                let dropdown_x = settings_x - (160.0 * scale);
                let dropdown_w = 240.0 * scale; 
                let base_menu_h = 28.0 * scale;
                let expanded_list_h = if self.fps_selector_open { 5.0 * 28.0 * scale } else { 0.0 };
                let total_h = base_menu_h + expanded_list_h;
                let y_start = metrics.header_h;
                
                if x >= dropdown_x && x <= dropdown_x + dropdown_w
                   && y >= y_start && y <= y_start + total_h {
                    return Some(RuntimeZone::Dropdown);
                }
            }
            if mouse.1 <= metrics.header_h { Some(RuntimeZone::Header) } else { None }
        }

        pub fn action_and_hover_at(&mut self, mouse: (f32, f32), width: f32, metrics: &ScaledMetrics) -> (EngineHeaderAction, Option<HoverEffect>) {
            self.ensure_cache(width, metrics, false);
            for prim in self.cached_primitives.iter().rev() {
                if let Some(interaction) = get_interaction(prim) {
                    if interaction.bounds.contains(mouse) {
                        return (interaction.action, Some(interaction.hover_effect));
                    }
                }
            }
            if mouse.1 <= metrics.header_h {
                (EngineHeaderAction::Drag, None)
            } else {
                (EngineHeaderAction::None, None)
            }
        }

        pub fn action_at(&mut self, mouse: (f32, f32), width: f32, metrics: &ScaledMetrics) -> EngineHeaderAction {
            self.ensure_cache(width, metrics, false);
            for prim in self.cached_primitives.iter().rev() {
                if let Some(interaction) = get_interaction(prim) {
                    if interaction.bounds.contains(mouse) {
                        return interaction.action;
                    }
                }
            }
            if mouse.1 <= metrics.header_h { EngineHeaderAction::Drag } else { EngineHeaderAction::None }
        }

        pub fn get_background_rects(&mut self, window_width: f32, metrics: &ScaledMetrics, mouse_pos: (f32, f32), is_pressed: bool) -> Vec<(f32, f32, f32, f32, [f32; 4], f32)> {
            self.ensure_cache(window_width, metrics, false);
            let active_action = if self.settings_dropdown_open { Some(EngineHeaderAction::ToggleSettings) } else { None };
            let mut rects = Vec::new();
            for prim in &self.cached_primitives {
                if let Primitive::Rect { x, y, w, h, color, corner_radius, interaction } = prim {
                    if let Some(inter) = interaction {
                        let hovered = inter.bounds.contains(mouse_pos);
                        let is_active = active_action == Some(inter.action);
                        let final_color = inter.hover_effect.resolve_bg(hovered, is_pressed, is_active).unwrap_or(*color);
                        let radius = inter.hover_effect.corner_radius();
                        if final_color[3] > 0.0 { rects.push((*x, *y, *w, *h, final_color, radius)); }
                    } else {
                        if color[3] > 0.0 { rects.push((*x, *y, *w, *h, *color, *corner_radius)); }
                    }
                }
            }
            rects
        }

        pub fn sections<'a>(&'a mut self, window_width: f32, is_maximized: bool, mouse_pos: (f32, f32), active_zone: UiZone, metrics: &ScaledMetrics) -> Vec<Section<'a>> {
            self.ensure_cache(window_width, metrics, is_maximized);
            let scale = metrics.scale;
            let is_header_active = matches!(active_zone, UiZone::Runtime(_));
            let effective_mouse = if is_header_active { mouse_pos } else { (-1.0, -1.0) };
            let active_action = if self.settings_dropdown_open { Some(EngineHeaderAction::ToggleSettings) } else { None };

            let mut sections = Vec::new();
            for prim in &self.cached_primitives {
                if let Primitive::Text { content, x, y, color, size, h_align, v_align, interaction } = prim {
                    let text_color = if let Some(inter) = interaction {
                        let hovered = inter.bounds.contains(effective_mouse);
                        let is_active = active_action == Some(inter.action);
                        inter.hover_effect.resolve_text(hovered, is_active).unwrap_or(*color)
                    } else { *color };
                    
                    sections.push(
                        Section::default()
                            .add_text(Text::new(content).with_color(text_color).with_scale(*size * scale))
                            .with_screen_position((*x, *y))
                            .with_layout(Layout::default().h_align(*h_align).v_align(*v_align))
                    );
                }
            }
            sections
        }
    }
}

// Global helper
fn get_interaction<A>(prim: &Primitive<A>) -> Option<&Interaction<A>> {
    match prim {
        Primitive::Rect { interaction, .. } | Primitive::Text { interaction, .. } => interaction.as_ref(),
    }
}
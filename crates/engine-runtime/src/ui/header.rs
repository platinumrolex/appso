use wgpu_text::glyph_brush::{Layout, Section, Text};
use winit::window::Window;
use wgpu_ui::{
    ui, ButtonStyle, Primitive, HoverEffect, Rect,
    Button, CustomTitle, Selector, SelectorOption, Container,
};
use wgpu_ui::primitives::UiAction;

use crate::RuntimeZone;
use wgpu_ui::Interaction;
use crate::UiZone;

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
    
    // UI cache – now only primitives, each carrying its interaction
    cached_primitives: Vec<Primitive<HeaderAction>>,
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

    fn build_ui(&self, window_width: f32, metrics: &ScaledMetrics, is_maximized: bool) -> Vec<Primitive<HeaderAction>> {
        let scale = metrics.scale;
        let btn_w = metrics.btn_w;
        let header_h = metrics.header_h;
        let settings_x = window_width - (btn_w * 4.0);
        let min_x = window_width - (btn_w * 3.0);
        let max_x = window_width - (btn_w * 2.0);
        let close_x = window_width - btn_w;
        let dropdown_x = settings_x - (160.0 * scale);
        // (row_h removed, unused)

        let mut settings_style = ButtonStyle::icon();
       // if self.settings_dropdown_open {
       //     settings_style.bg_idle = settings_style.bg_hover;
       //     settings_style.text_idle = settings_style.text_hover;
       // }

        let mut primitives = ui! {
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

        // Dropdown
        if self.settings_dropdown_open {
            let options = vec![
                SelectorOption { label: "Auto (Sync)".into(), selected: self.current_fps.is_auto, action: HeaderAction::SetFpsAuto },
                SelectorOption { label: "30 FPS".into(), selected: !self.current_fps.is_auto && self.current_fps.value == 30, action: HeaderAction::SetFps30 },
                SelectorOption { label: "60 FPS".into(), selected: !self.current_fps.is_auto && self.current_fps.value == 60, action: HeaderAction::SetFps60 },
                SelectorOption { label: "144 FPS".into(), selected: !self.current_fps.is_auto && self.current_fps.value == 144, action: HeaderAction::SetFps144 },
                SelectorOption { label: "240 FPS".into(), selected: !self.current_fps.is_auto && self.current_fps.value == 240, action: HeaderAction::SetFps240 },
            ];

            let mut sel_prims = ui! {
                root {
                    fps_selector: Selector {
                        label: "Frame limit".into(),
                        current: self.current_fps.label().to_string(),
                        toggle_action: HeaderAction::FpsSelector,
                        expanded: self.fps_selector_open,
                        style: ButtonStyle::primary(),
                        options: options,
                    } at (dropdown_x, header_h, 240.0 * scale, 28.0 * scale)
                }
            };
            primitives.append(&mut sel_prims);
        }

        // Attention dot (non-interactive, placed manually)
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

        primitives
    }

    fn ensure_cache(&mut self, window_width: f32, metrics: &ScaledMetrics, is_maximized: bool) {
        let state_changed = !self.cache_valid 
            || self.cached_window_width != window_width 
            || self.cached_is_maximized != is_maximized;
        if state_changed {
            self.cached_primitives = self.build_ui(window_width, metrics, is_maximized);
            self.cache_valid = true;
            self.cached_window_width = window_width;
            self.cached_is_maximized = is_maximized;
        }
    }

    fn invalidate_cache(&mut self) {
        self.cache_valid = false;
    }

    pub fn zone_at(&mut self, mouse: (f32, f32), window_width: f32, metrics: &ScaledMetrics) -> Option<RuntimeZone> {
        self.ensure_cache(window_width, metrics, false);
        // Check dropdown area manually (as before) – these bounds were known at build time
        if self.settings_dropdown_open {
            let scale = metrics.scale;
            let btn_w = metrics.btn_w;
            let settings_x = window_width - (btn_w * 4.0);
            let dropdown_x = settings_x - (160.0 * scale);
            let dropdown_w = 240.0 * scale; // width of the selector
            // Height: at least one row, more if expanded.
            let base_menu_h = 28.0 * scale;
            let expanded_list_h = if self.fps_selector_open { 5.0 * 28.0 * scale } else { 0.0 };
            let total_h = base_menu_h + expanded_list_h;
            let y_start = metrics.header_h;
            if mouse.0 >= dropdown_x && mouse.0 <= dropdown_x + dropdown_w
               && mouse.1 >= y_start && mouse.1 <= y_start + total_h {
                return Some(RuntimeZone::Dropdown);
            }
        }
        if mouse.1 <= metrics.header_h { Some(RuntimeZone::Header) } else { None }
    }

    /// Returns (action, hover_effect) for the first interactive primitive that contains the mouse.
    pub fn action_and_hover_at(&mut self, mouse: (f32, f32), width: f32, metrics: &ScaledMetrics) -> (HeaderAction, Option<HoverEffect>) {
        self.ensure_cache(width, metrics, false);
        for prim in &self.cached_primitives {
            if let Some(interaction) = get_interaction(prim) {
                if interaction.bounds.contains(mouse) {
                    return (interaction.action, Some(interaction.hover_effect));
                }
            }
        }
        // Default: drag zone if in header, else none.
        if mouse.1 <= metrics.header_h {
            (HeaderAction::Drag, None)
        } else {
            (HeaderAction::None, None)
        }
    }

    /// Returns only the action (used for clicks).
    pub fn action_at(&mut self, mouse: (f32, f32), width: f32, metrics: &ScaledMetrics) -> HeaderAction {
        self.ensure_cache(width, metrics, false);
        for prim in &self.cached_primitives {
            if let Some(interaction) = get_interaction(prim) {
                if interaction.bounds.contains(mouse) {
                    return interaction.action;
                }
            }
        }
        if mouse.1 <= metrics.header_h { HeaderAction::Drag } else { HeaderAction::None }
    }

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

    pub fn get_background_rects(&mut self, window_width: f32, metrics: &ScaledMetrics,
                            mouse_pos: (f32, f32), is_pressed: bool)
                            -> Vec<(f32, f32, f32, f32, [f32; 4], f32)>
{
    self.ensure_cache(window_width, metrics, false);
    let active_action = if self.settings_dropdown_open {
        Some(HeaderAction::SettingsSelector)
    } else {
        None
    };

    let mut rects = Vec::new();
    for prim in &self.cached_primitives {
        if let Primitive::Rect { x, y, w, h, color, corner_radius, interaction } = prim {
            if let Some(inter) = interaction {
                let hovered = inter.bounds.contains(mouse_pos);
                let is_active = active_action == Some(inter.action);
                let final_color = inter.hover_effect.resolve_bg(hovered, is_pressed, is_active)
                    .unwrap_or(*color);
                let radius = inter.hover_effect.corner_radius();
                if final_color[3] > 0.0 {
                    rects.push((*x, *y, *w, *h, final_color, radius));
                }
            } else {
                // no interaction – use static color
                if color[3] > 0.0 {
                    rects.push((*x, *y, *w, *h, *color, *corner_radius));
                }
            }
        }
    }
    rects
}

   pub fn sections<'a>(&'a mut self, window_width: f32, is_maximized: bool,
                    mouse_pos: (f32, f32), active_zone: UiZone, metrics: &ScaledMetrics)
                    -> Vec<Section<'a>>
{
    self.ensure_cache(window_width, metrics, is_maximized);
    let scale = metrics.scale;
    let is_header_active = matches!(active_zone, UiZone::Runtime(_));
    let effective_mouse = if is_header_active { mouse_pos } else { (-1.0, -1.0) };

    let active_action = if self.settings_dropdown_open {
        Some(HeaderAction::SettingsSelector)
    } else {
        None
    };

    let mut sections = Vec::new();
    for prim in &self.cached_primitives {
        if let Primitive::Text { content, x, y, color, size, h_align, v_align, interaction } = prim {
            let text_color = if let Some(inter) = interaction {
                let hovered = inter.bounds.contains(effective_mouse);
                let is_active = active_action == Some(inter.action);
                inter.hover_effect.resolve_text(hovered, is_active).unwrap_or(*color)
            } else {
                *color
            };
            sections.push(
                Section::default()
                    .add_text(
                        Text::new(content)
                            .with_color(text_color)
                            .with_scale(*size * scale)
                    )
                    .with_screen_position((*x, *y))
                    .with_layout(Layout::default().h_align(*h_align).v_align(*v_align))
            );
        }
    }
    sections
}
}
// Helper to extract interaction (works for both Rect and Text)
fn get_interaction<A>(prim: &Primitive<A>) -> Option<&Interaction<A>> {
    match prim {
        Primitive::Rect { interaction, .. } | Primitive::Text { interaction, .. } => interaction.as_ref(),
    }
}
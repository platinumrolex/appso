use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Instant, Duration};
use winit::application::ApplicationHandler;
use winit::event::{WindowEvent, ElementState, MouseScrollDelta, MouseButton};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::Window;
use wgpu::Instance;
use winit::keyboard::Key;
use winit::keyboard::NamedKey;
use engine_sdk::{Camera, INITIAL_WORLD_POS};
//use engine_runtime::ui::ui;
use engine_runtime::{
    core::{init_graphics, get_window_refresh_rate, WgpuState},
    ui::{ui,
        header::{EngineHeader, ScaledMetrics, HeaderAction, FpsLimit},
        ui_zone::{determine_active_zone, UiZone, RuntimeZone},
        window,
    },
    EngineState,
    calculate_frame_delay
};
use diagram_app::{DiagramApp};

const GPU_USAGE_RECOMMENDED: bool = true;
const RUN_IN_BACKGROUND_ON_CLOSE: bool = false;
const AFK_DECAY_SECS: f32 = 5.0;
const AFK_MIN_FPS: u32 = 1;
const WARM_TO_PREFLIGHT_SECS: f32 = 10.0;
const PREFLIGHT_TO_COLD_SECS: f32 = 15.0;

//---Start :App specific ----//
// 1. Define what the "App" part of the action is for THIS specific application
// Since your app is the DiagramApp, let's assume it has actions (or use a placeholder for now)
use engine_runtime::ui::ui_zone::RuntimeAction;
use engine_runtime::ui::ui_zone::RootAction;
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DiagramAction {
    #[default]
    None,
    SelectNode(usize),
    DragCanvas,
}

// You must implement UiAction for it
impl wgpu_ui::primitives::UiAction for DiagramAction {
    fn is_interactive(&self) -> bool {
        match self {
            DiagramAction::None => false,
            _ => true,
        }
    }
}

//---End:App specific ----//


#[derive(Debug)]
pub enum AppEvent {
    WorkerMessage(String),
    UnstoppableTaskStarted(u64),
    UnstoppableTaskFinished(u64),
    IoHandoffComplete(u64),
    NetworkRestored,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum HibernationState {
    Active,
    Warm,
    PreFlight,
    Cold,
}

pub struct IoJob {
    pub id: u64,
    pub payload_url: String,
}

use engine_runtime::ui::ui_zone::InteractionState;
struct CoreState {
    window: Option<Arc<Window>>,
    camera: Camera,
    metrics: ScaledMetrics,
    mouse_pos: (f32, f32),
    frame_mouse_pos: (f32, f32),
    mouse_moved: bool,
    cursor_in_window: bool,
    is_mouse_down: bool,
    ui_state: InteractionState<RootAction<DiagramAction>>,
    needs_state_update: bool,
    needs_redraw: bool,
    last_interaction: Instant,
    last_frame_time: Instant,
    target_fps: FpsLimit,
    is_hidden: bool,
    events_count: u32,
    logic_count: u32,
    frame_count: u32,
    last_log_time: Instant,
    hibernation_state: HibernationState,
    hidden_since: Option<Instant>,
    active_unstoppable_tasks: u32,
    audio_is_playing: bool,
    audio_is_muted: bool,
    runtime_io_tx: mpsc::Sender<IoJob>,
}

pub struct App {
    instance: Instance,
    wgpu_state: Option<WgpuState>,
    core: CoreState,
    diagram: DiagramApp,
    engine: EngineState,
}

impl App {
    pub fn new(event_loop: &winit::event_loop::EventLoop<AppEvent>) -> Self {
        let proxy = event_loop.create_proxy();
        let proxy_io = event_loop.create_proxy();

        let (runtime_io_tx, runtime_io_rx) = mpsc::channel::<IoJob>();
        thread::spawn(move || {
            println!("[Global Runtime] Handoff Manager Started.");
            for job in runtime_io_rx {
                println!("[Global Runtime] Took ownership of I/O Job: {}. Processing independently...", job.id);
                thread::sleep(Duration::from_secs(15));
                proxy_io.send_event(AppEvent::IoHandoffComplete(job.id)).ok();
            }
        });

        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(10));
                proxy.send_event(AppEvent::WorkerMessage("Heavy task complete".to_string())).ok();
            }
        });

        let instance = Instance::default();

        let core = CoreState {
            window: None,
            camera: Camera {
                pan_x: -INITIAL_WORLD_POS,
                pan_y: -INITIAL_WORLD_POS,
                zoom: 1.0,
                screen_width: 1.0,
                screen_height: 1.0,
                scale_factor: 1.0,
            },
            metrics: ScaledMetrics::new(1.0),
            mouse_pos: (0.0, 0.0),
            frame_mouse_pos: (0.0, 0.0),
            mouse_moved: false,
            cursor_in_window: false,
            is_mouse_down: false,
            ui_state: InteractionState::default(),
            needs_state_update: true,
            needs_redraw: true,
            last_interaction: Instant::now(),
            last_frame_time: Instant::now(),
            target_fps: FpsLimit { value: 30, is_auto: false },
            is_hidden: false,
            events_count: 0,
            logic_count: 0,
            frame_count: 0,
            last_log_time: Instant::now(),
            hibernation_state: HibernationState::Active,
            hidden_since: None,
            active_unstoppable_tasks: 0,
            audio_is_playing: false,
            audio_is_muted: false,
            runtime_io_tx,
        };

        let diagram = DiagramApp::new();
        let engine = EngineState::new();

        Self {
            instance,
            wgpu_state: None,
            core,
            diagram,
            engine,
        }
    }

    fn update_network_governance(&self) {
        let wants_video = !self.core.is_hidden;
        let wants_audio = self.core.audio_is_playing && !self.core.audio_is_muted;

        println!("\n[Governance] TRACE: Calculating Network Policy...");
        println!("  - Visibility: {} => Wants Video: {}", self.core.is_hidden, wants_video);
        println!("  - Audio Playing: {}, Muted: {} => Wants Audio: {}", self.core.audio_is_playing, self.core.audio_is_muted, wants_audio);

        if !wants_video && !wants_audio {
            println!("[Governance] ACTION: Signaling server to HALT all UDP streams.");
        } else if !wants_video && wants_audio {
            println!("[Governance] ACTION: Transitioning to AUDIO-ONLY profile (Bandwidth -90%).");
        } else if wants_video && !wants_audio {
            println!("[Governance] ACTION: Transitioning to Video-Only profile stream to save bandwidth.");
        } else {
            println!("[Governance] ACTION: Full 4K/60fps Video Stream enabled. (incl. Audio)");
        }
    }

    fn engine_event(&mut self) {
        self.core.needs_state_update = false;
        if let Some(_hidden_time) = self.core.hidden_since {
            if self.core.is_hidden {
                if self.core.audio_is_playing && !self.core.audio_is_muted {
                    if self.core.hibernation_state != HibernationState::Warm {
                        println!("[Hibernation] TRACE: Audio is active. Locking state to WARM (Keep-Alive).");
                        self.core.hibernation_state = HibernationState::Warm;
                    }
                } else if self.core.hibernation_state == HibernationState::PreFlight && self.core.active_unstoppable_tasks == 0 {
                    println!("[Hibernation] TRACE: No tasks. Entering COLD. Purging WGPU/VRAM.");
                    self.wgpu_state = None;
                    self.core.hibernation_state = HibernationState::Cold;
                }
            }
        } else if self.core.hibernation_state != HibernationState::Active {
            println!("\n[Hibernation] TRACE: User returned. Waking to ACTIVE.");
            self.core.hibernation_state = HibernationState::Active;
            self.core.hidden_since = None;

            if self.wgpu_state.is_none() {
                println!("[Hibernation] ACTION: Cold Start detected. Rebuilding GPU Context...");
                if let Some(win) = self.core.window.clone() {
                    self.wgpu_state = Some(init_graphics(&self.instance, win, &mut self.core.camera, &self.diagram));
                }
            }
            self.core.needs_redraw = true;
            self.update_network_governance();
        }
    }
}



impl ApplicationHandler<AppEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.core.window.is_none() {
            let window = window::create_window(event_loop, &self.engine.header.title);
            window::apply_platform_style(&window);

            self.core.camera.scale_factor = window.scale_factor() as f32;
            self.core.metrics = ScaledMetrics::new(window.scale_factor() as f32);
            let hz = get_window_refresh_rate(&window);
            self.core.target_fps = FpsLimit { value: hz, is_auto: true };
            println!("[Startup] Detected Monitor Hz: {}, setting manual FPS target.", hz);

            self.core.window = Some(window.clone());
            self.wgpu_state = Some(init_graphics(&self.instance, window, &mut self.core.camera, &self.diagram));
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        // --- Hibernation Governance ---
        if self.core.is_hidden {
            if let Some(hidden_time) = self.core.hidden_since {
                let elapsed = now.duration_since(hidden_time).as_secs_f32();
                if (elapsed as u32) % 5 == 0 && elapsed > 1.0 {
                    println!("[Hibernation] Monitoring... Elapsed: {:.1}s | Tasks: {} | Audio: {}",
                        elapsed, self.core.active_unstoppable_tasks, self.core.audio_is_playing);
                }

                if self.core.audio_is_playing && !self.core.audio_is_muted {
                    if self.core.hibernation_state != HibernationState::Warm {
                        println!("[Hibernation] TRACE: Audio is active. Locking state to WARM (Keep-Alive).");
                        self.core.hibernation_state = HibernationState::Warm;
                    }
                } else if elapsed >= WARM_TO_PREFLIGHT_SECS && self.core.hibernation_state == HibernationState::Warm {
                    println!("[Hibernation] TRACE: >{}s hidden. Entering PRE-FLIGHT. Saving state...", WARM_TO_PREFLIGHT_SECS);
                    self.core.hibernation_state = HibernationState::PreFlight;
                } else if self.core.hibernation_state == HibernationState::PreFlight && self.core.active_unstoppable_tasks == 0 {
                    println!("[Hibernation] TRACE: No tasks. Entering COLD. Purging WGPU/VRAM.");
                    self.wgpu_state = None;
                    self.core.hibernation_state = HibernationState::Cold;
                }
            }
        } else if self.core.hibernation_state != HibernationState::Active {
            println!("\n[Hibernation] TRACE: User returned. Waking to ACTIVE.");
            self.core.hibernation_state = HibernationState::Active;
            self.core.hidden_since = None;

            if self.wgpu_state.is_none() {
                println!("[Hibernation] ACTION: Cold Start detected. Rebuilding GPU Context...");
                if let Some(win) = self.core.window.clone() {
                    self.wgpu_state = Some(init_graphics(&self.instance, win, &mut self.core.camera, &self.diagram));
                }
            }
            self.core.needs_redraw = true;
            self.update_network_governance();
        }

        let frame_delay = calculate_frame_delay(
            self.core.is_hidden,
            self.core.target_fps,
            self.core.last_interaction,
        );

        if now.duration_since(self.core.last_frame_time) > frame_delay * 2 {
            self.core.last_frame_time = now;
        }

        let next_frame_time = self.core.last_frame_time + frame_delay;

        if self.core.hibernation_state == HibernationState::Cold {
            event_loop.set_control_flow(ControlFlow::Wait);
            return;
        }

        if self.core.needs_state_update {
            self.engine_event();
        }

        if (self.core.mouse_moved || self.core.needs_redraw) && now >= next_frame_time {
            if self.core.mouse_moved {
                let pos = self.core.mouse_pos;
                let delta = (pos.0 - self.core.frame_mouse_pos.0, pos.1 - self.core.frame_mouse_pos.1);
                self.core.frame_mouse_pos = pos;
                self.core.mouse_moved = false;

                if !self.core.is_mouse_down {
                    if let Some(win) = self.core.window.as_ref() {
                        self.core.ui_state.update_zone(
                            self.core.mouse_pos,
                            win.inner_size().width as f32,
                            &self.core.metrics,
                            &mut self.engine.header,
                        );
                        // println!("[Zone]: {:?}", self.core.ui_state.zone);
                    
                    let pos = self.core.mouse_pos;
                    let width = win.inner_size().width as f32;

                    let (action, hover) = match self.core.ui_state.zone {
                        UiZone::Runtime(RuntimeZone::Header) => {
                            // Look for a hit in the header's cache
                            let hit = self.engine.header.cached_hits.iter()
                                .find(|h| h.bounds.contains(pos));
                                
                            match hit {
                                Some(h) => (
                                    RootAction::Runtime(RuntimeAction::Header(h.action)), 
                                    Some(h.hover) // Extract ONLY the hover effect
                                ),
                                None => (RootAction::Runtime(RuntimeAction::Header(HeaderAction::Drag)), None),
                            }
                        }
                        UiZone::App => {
                            if let Some(idx) = self.diagram.hit_test(&self.core.camera, pos) {
                                (RootAction::App(DiagramAction::SelectNode(idx)), None)
                            } else {
                                (RootAction::App(DiagramAction::None), None)
                            }
                        }
                        _ => (RootAction::None, None),
                    };

                    // Now this call is type-safe because 'hover' is just a HoverEffect enum
                    if self.core.ui_state.check_hovered(action, hover) {
                        self.core.needs_redraw = true;
                        println!("[Zone]: {:?} hover: {:?} - redraw request upon hovers/mousemove", action, hover);
                    }

                    }
                }

                match self.core.ui_state.zone {
                    UiZone::Runtime(sub_zone) => {
                        // Runtime zones (Header/Dropdown) – no app interaction needed
                        match sub_zone {
                            RuntimeZone::Header => {}
                            RuntimeZone::Dropdown => {}
                        }
                    }
                    UiZone::App => {
                        self.diagram.handle_mousemove(
                            &mut self.core.camera,
                            pos,
                            delta,
                        );
                        if self.diagram.current_drag.is_some() {
                            self.core.needs_redraw = true;
                        }
                    }
                }
            }

            if self.core.needs_redraw {
                self.core.last_frame_time = next_frame_time;
                self.core.logic_count += 1;

                if !self.core.is_hidden {
                    if let Some(win) = self.core.window.as_ref() {
                        win.request_redraw();
                    }
                }
                event_loop.set_control_flow(ControlFlow::WaitUntil(next_frame_time));
            } else {
                event_loop.set_control_flow(ControlFlow::Wait);
            }
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }

        self.core.events_count += 1;

        if now.duration_since(self.core.last_log_time) >= Duration::from_secs(1) {
            println!(
                "[Stats] State: {:?} | CPU Ticks: {} | CPU Exec: {} | GPU Redraws: {} | FPS Target: {:?} | VRAM: {} | Hibernation State: {:?}",
                self.core.hibernation_state,
                self.core.events_count,
                self.core.logic_count,
                self.core.frame_count,
                self.core.target_fps,
                if self.wgpu_state.is_some() { "ALLOCATED" } else { "EMPTY" },
                self.core.hibernation_state
            );
            self.core.events_count = 0;
            self.core.logic_count = 0;
            self.core.frame_count = 0;
            self.core.last_log_time = now;
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::WorkerMessage(msg) => {
                println!("[Worker Result] Received: {}", msg);
                self.core.needs_redraw = true;
            }
            AppEvent::UnstoppableTaskStarted(id) => {
                self.core.active_unstoppable_tasks += 1;
                println!("[Governance] Unstoppable task {} started. Total: {}", id, self.core.active_unstoppable_tasks);
            }
            AppEvent::UnstoppableTaskFinished(id) => {
                self.core.active_unstoppable_tasks = self.core.active_unstoppable_tasks.saturating_sub(1);
                println!("[Governance] Unstoppable task {} finished. Total: {}", id, self.core.active_unstoppable_tasks);
            }
            AppEvent::IoHandoffComplete(id) => {
                println!("[Global Runtime] Handoff Job {} completed successfully.", id);
            }
            AppEvent::NetworkRestored => {
                println!("[Governance] Network Restored event received. Resuming paused I/O tasks...");
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: winit::window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed && !event.repeat {
                    match event.logical_key {
                        Key::Character(ref c) if c == "1" => {
                            println!("[DEBUG] Forcing ACTIVE state");
                            self.core.hibernation_state = HibernationState::Active;
                            self.core.is_hidden = false;
                            self.core.hidden_since = None;
                            self.update_network_governance();
                        }
                        Key::Character(ref c) if c == "2" => {
                            println!("[DEBUG] Forcing WARM state (Simulated Hide)");
                            self.core.hibernation_state = HibernationState::Warm;
                            self.core.is_hidden = true;
                            self.core.hidden_since = Some(Instant::now());
                        }
                        Key::Character(ref c) if c == "3" => {
                            println!("[DEBUG] Forcing PRE-FLIGHT (Time Travel +25s)");
                            self.core.hibernation_state = HibernationState::PreFlight;
                            self.core.hidden_since = Some(Instant::now() - Duration::from_secs(25));
                        }
                        Key::Character(ref c) if c == "4" => {
                            println!("[DEBUG] Forcing COLD state (VRAM PURGE)");
                            self.core.hibernation_state = HibernationState::Cold;
                            self.wgpu_state = None;
                        }
                        Key::Character(ref c) if c.to_lowercase() == "u" => {
                            self.core.active_unstoppable_tasks += 1;
                            println!("[DEBUG] Task Started. Total: {}", self.core.active_unstoppable_tasks);
                        }
                        Key::Character(ref c) if c.to_lowercase() == "i" => {
                            self.core.active_unstoppable_tasks = self.core.active_unstoppable_tasks.saturating_sub(1);
                            println!("[DEBUG] Task Finished. Total: {}", self.core.active_unstoppable_tasks);
                        }
                        Key::Character(ref c) if c.to_lowercase() == "a" => {
                            self.core.audio_is_playing = !self.core.audio_is_playing;
                            println!("[DEBUG] Audio Stream Playing: {}", self.core.audio_is_playing);
                            self.update_network_governance();
                        }
                        Key::Named(NamedKey::Escape) => event_loop.exit(),
                        _ => {}
                    }
                    self.core.needs_redraw = true;
                }
            }
            WindowEvent::CursorEntered { .. } => {
                println!("cursor entered window");
                self.core.cursor_in_window = true;
            }
            WindowEvent::CursorLeft { .. } => {
                println!("cursor left window");
                self.core.cursor_in_window = false;
            }
            WindowEvent::CloseRequested => {
                if RUN_IN_BACKGROUND_ON_CLOSE {
                    if let Some(win) = self.core.window.as_ref() {
                        win.set_visible(false);
                    }
                    self.core.is_hidden = true;
                    self.core.hidden_since = Some(Instant::now());
                    self.core.hibernation_state = HibernationState::Warm;
                    self.update_network_governance();
                } else {
                    event_loop.exit();
                }
            }
            WindowEvent::Focused(focused) => {
                if focused {
                    println!("[Event] Window gained focus");
                    self.core.last_interaction = Instant::now();
                } else {
                    self.core.is_mouse_down = false;
                }
            }
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    if let Some(wgpu) = self.wgpu_state.as_mut() {
                        let old_center_x = self.core.camera.screen_width / 2.0;
                        let old_center_y = self.core.camera.screen_height / 2.0;
                        let (world_cx, world_cy) = self.core.camera.screen_to_world(old_center_x, old_center_y);

                        wgpu.config.width = size.width;
                        wgpu.config.height = size.height;
                        wgpu.surface.configure(&wgpu.device, &wgpu.config);
                        wgpu.brush.resize_view(size.width as f32, size.height as f32, &wgpu.queue);

                        self.core.camera.screen_width = size.width as f32;
                        self.core.camera.screen_height = size.height as f32;

                        let new_center_x = self.core.camera.screen_width / 2.0;
                        let new_center_y = self.core.camera.screen_height / 2.0;
                        let ez = self.core.camera.effective_zoom();
                        self.core.camera.pan_x = new_center_x - (world_cx * ez);
                        self.core.camera.pan_y = new_center_y - (world_cy * ez);

                        self.core.needs_redraw = true;
                    }
                    self.core.is_hidden = false;
                    self.core.needs_state_update = true;
                } else {
                    println!("[Event] Window minimized (Zero Size)");
                    self.core.is_hidden = true;
                    self.core.hidden_since = Some(Instant::now());
                    self.core.needs_state_update = true;
                }
            }
            WindowEvent::Moved(_) => {
                if let Some(win) = self.core.window.as_ref() {
                    let new_hz = get_window_refresh_rate(&win);
                    let fps = &mut self.engine.header.current_fps;

                    if fps.is_auto {
                        fps.value = new_hz;
                        println!("[Monitor Change] Auto-FPS: Syncing to {}Hz monitor.", new_hz);
                    } else {
                        if new_hz < fps.value {
                            println!("[Monitor Change] Display {}Hz is lower than manual {}Hz - capping to monitor.", new_hz, fps.value);
                            fps.value = new_hz;
                        } else {
                            println!("[Monitor Change] Display capable of {}Hz (Respecting manual {}Hz limit)", new_hz, fps.value);
                        }
                    }

                    self.core.target_fps = *fps;
                    self.core.needs_redraw = true;
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.core.camera.scale_factor = scale_factor as f32;
                self.core.metrics = ScaledMetrics::new(scale_factor as f32);
                self.core.needs_redraw = true;
                println!("[DPI] Scale factor changed to: {}", scale_factor);
            }
            WindowEvent::CursorMoved { position, .. } => {
                if !self.core.cursor_in_window { return; }
                self.core.mouse_pos = (position.x as f32, position.y as f32);
                self.core.mouse_moved = true;
                self.core.last_interaction = Instant::now();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                println!("mouse input");
                self.core.is_mouse_down = state == ElementState::Pressed;
                self.core.last_interaction = Instant::now();

                if state == ElementState::Pressed {
                    let mut handled_by_ui = false;
                    if let Some(win) = self.core.window.as_ref() {
                        let size = win.inner_size();
                        let action = self.engine.header.action_at(
                            self.core.mouse_pos,
                            size.width as f32,
                            &self.core.metrics,
                        );
                        // close menues
                        if !matches!(self.core.ui_state.zone, UiZone::Runtime(RuntimeZone::Dropdown)) 
                            && action != HeaderAction::SettingsSelector {
                            self.engine.header.settings_dropdown_open = false;
                            self.engine.header.fps_selector_open = false;
                          //  self.engine.header.invalidate_menu();
                        }

                       // let action = self.engine.header.hit_test(size.width as f32, self.core.mouse_pos, self.core.camera.scale_factor);

                        if button == MouseButton::Left {
                            match action {
                                HeaderAction::Close => {
                                    event_loop.exit();
                                    return;
                                }
                                _ => {
                                    if self.engine.header.handle_action(&win, action) {
                                        handled_by_ui = true;
                                    }
                                }
                            }
                        }
                    }

                    if !handled_by_ui {
                        self.diagram.handle_click(&self.core.camera, self.core.mouse_pos, button);
                    }
                } else {
                    self.diagram.handle_release();
                }
                self.core.needs_redraw = true;
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.core.last_interaction = Instant::now();

                let zoom_delta = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y * 0.1,
                    MouseScrollDelta::PixelDelta(pos) => (pos.y as f32 / self.core.camera.scale_factor) * 0.001,
                };

                let old_zoom = self.core.camera.zoom;
                let new_zoom = (old_zoom + zoom_delta).clamp(0.1, 5.0);
                let is_over_node = self.diagram.is_mouse_over_node(&self.core.camera, self.core.mouse_pos);

                if zoom_delta > 0.0 || is_over_node {
                    let anchor = self.core.mouse_pos;
                    let (wx, wy) = self.core.camera.screen_to_world(anchor.0, anchor.1);
                    self.core.camera.zoom = new_zoom;
                    let ez = self.core.camera.effective_zoom();
                    self.core.camera.pan_x = anchor.0 - (wx * ez);
                    self.core.camera.pan_y = anchor.1 - (wy * ez);
                } else {
                    let screen_cx = self.core.camera.screen_width / 2.0;
                    let screen_cy = self.core.camera.screen_height / 2.0;
                    let anchor = (screen_cx, screen_cy);
                    let (wx, wy) = self.core.camera.screen_to_world(anchor.0, anchor.1);
                    self.core.camera.zoom = new_zoom;
                    let ez = self.core.camera.effective_zoom();
                    self.core.camera.pan_x = anchor.0 - (wx * ez);
                    self.core.camera.pan_y = anchor.1 - (wy * ez);

                    let cached_center = self.diagram.get_world_center_of_nodes();
                    let (nodes_sx, nodes_sy) = self.core.camera.world_to_screen(cached_center.0, cached_center.1);
                    let pull_strength = 0.2;
                    self.core.camera.pan_x += (screen_cx - nodes_sx) * pull_strength;
                    self.core.camera.pan_y += (screen_cy - nodes_sy) * pull_strength;
                }

                self.core.needs_redraw = true;
            }
            WindowEvent::RedrawRequested => {
                self.core.needs_redraw = false;
                if self.core.hibernation_state == HibernationState::Cold || self.core.is_hidden {
                    return;
                }

                if let Some(win) = self.core.window.as_ref() {
                    if let Some(wgpu) = self.wgpu_state.as_mut() {
                        let size = win.inner_size();
                        if size.width == 0 || size.height == 0 {
                            return;
                        }

                        let frame = match wgpu.surface.get_current_texture() {
                            wgpu::CurrentSurfaceTexture::Success(texture)
                            | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
                            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                                wgpu.surface.configure(&wgpu.device, &wgpu.config);
                                return;
                            }
                            _ => return,
                        };
                        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                        let mut encoder = wgpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                        let scale = self.core.camera.scale_factor;
                        
                       // let bg_rects = self.engine.header.get_background_rects(size.width as f32, &self.core.metrics); // was scale instead metrics
                      //  let bg_rects = self.engine.header.get_background_rects(size.width as f32, &self.core.metrics, self.core.mouse_pos, false);
                        let bg_rects = self.engine.header.get_background_rects(
                            size.width as f32,
                            &self.core.metrics,
                            self.core.mouse_pos,
                            self.core.is_mouse_down,
                        );

                        let mut quad_buffer = None;
                        if !bg_rects.is_empty() {
                             let mut instance_data = Vec::with_capacity(bg_rects.len() * 11);
                            for (x, y, w, h, color, radius) in &bg_rects {
                                instance_data.extend_from_slice(&[*x, *y, *w, *h]);
                                instance_data.extend_from_slice(color);
                                instance_data.push(*radius);
                                instance_data.extend_from_slice(&[size.width as f32, size.height as f32]);
                            }
                        //   let mut instance_data = Vec::with_capacity(bg_rects.len() * 10);
                        //   for (x, y, w, h, color) in &bg_rects {
                        //       instance_data.extend_from_slice(&[*x, *y, *w, *h]);
                        //       instance_data.extend_from_slice(color);
                        //       instance_data.extend_from_slice(&[size.width as f32, size.height as f32]);
                        //   }
                            
                            let instance_bytes: &[u8] = unsafe { 
                                std::slice::from_raw_parts(instance_data.as_ptr() as *const u8, instance_data.len() * 4) 
                            };
                            
                            let buffer = wgpu.device.create_buffer(&wgpu::BufferDescriptor {
                                label: Some("UI Background Buffer"),
                                size: instance_bytes.len() as wgpu::BufferAddress,
                                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                                mapped_at_creation: false,
                            });
                            wgpu.queue.write_buffer(&buffer, 0, instance_bytes);
                            quad_buffer = Some(buffer);
                        }

                        let mut sections = Vec::new();
                        sections.extend(self.engine.header.sections(
                            size.width as f32,
                            win.is_maximized(),
                            self.core.mouse_pos,
                            self.core.ui_state.zone,
                            &self.core.metrics,
                        ));
                        // sections.extend(self.engine.header.sections(size.width as f32, win.is_maximized(), self.core.mouse_pos, scale));
                        self.diagram.queue_text(&self.core.camera, &mut sections);
                        wgpu.brush.queue(&wgpu.device, &wgpu.queue, sections).unwrap();

                        {
                            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.05, g: 0.05, b: 0.05, a: 1.0 }),
                                        store: wgpu::StoreOp::Store,
                                    },
                                    depth_slice: None,
                                })],
                                ..Default::default()
                            });

                            if let Some(buffer) = &quad_buffer {
                                rpass.set_pipeline(&wgpu.quad_pipeline);
                                rpass.set_vertex_buffer(0, buffer.slice(..));
                                rpass.draw(0..6, 0..bg_rects.len() as u32);
                            }

                            wgpu.brush.draw(&mut rpass);
                        }

                        wgpu.queue.submit(std::iter::once(encoder.finish()));
                        frame.present();

                        self.core.frame_count += 1;
                    }
                }
            }
            _ => (),
        }
    }
}
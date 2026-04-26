use engine_sdk::{NodeData, Camera, INITIAL_WORLD_POS};
use winit::event::MouseButton;
use wgpu_text::glyph_brush::{Section, Text};

#[derive(Clone, Copy, PartialEq)]
pub enum DragMode {
    Node(usize),
    Canvas,
}

pub struct DiagramApp {
    pub tables: Vec<NodeData>,
    pub current_drag: Option<DragMode>,
    pub grab_offset: (f32, f32),
}

impl DiagramApp {
    pub fn new() -> Self {
        Self {
            tables: vec![NodeData::new(
                0,
                "Users",
                INITIAL_WORLD_POS,
                INITIAL_WORLD_POS,
                vec!["id".into(), "email".into()],
            )],
            current_drag: None,
            grab_offset: (0.0, 0.0),
        }
    }

    pub fn get_world_center_of_nodes(&self) -> (f32, f32) {
        if self.tables.is_empty() {
            return (INITIAL_WORLD_POS, INITIAL_WORLD_POS);
        }

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for node in &self.tables {
            min_x = min_x.min(node.x);
            min_y = min_y.min(node.y);
            max_x = max_x.max(node.x + node.width);
            max_y = max_y.max(node.y + node.height);
        }

        ((min_x + max_x) / 2.0, (min_y + max_y) / 2.0)
    }

    pub fn is_mouse_over_node(&self, camera: &Camera, mouse_pos: (f32, f32)) -> bool {
        let (wx, wy) = camera.screen_to_world(mouse_pos.0, mouse_pos.1);
        for table in &self.tables {
            if wx >= table.x && wx <= table.x + table.width && wy >= table.y && wy <= table.y + table.height {
                return true;
            }
        }
        false
    }

    pub fn handle_click(&mut self, camera: &Camera, mouse_pos: (f32, f32), button: MouseButton) {
        match button {
            MouseButton::Left => {
                let (wx, wy) = camera.screen_to_world(mouse_pos.0, mouse_pos.1);
                let mut hit_idx = None;
                for (i, table) in self.tables.iter().enumerate().rev() {
                    if wx >= table.x && wx <= table.x + table.width && wy >= table.y && wy <= table.y + table.height {
                        hit_idx = Some(i);
                        break;
                    }
                }

                if let Some(idx) = hit_idx {
                    self.grab_offset = (wx - self.tables[idx].x, wy - self.tables[idx].y);
                    self.current_drag = Some(DragMode::Node(idx));
                }
            }
            MouseButton::Right => {
                self.current_drag = Some(DragMode::Canvas);
            }
            _ => {}
        }
    }

    pub fn handle_release(&mut self) {
        self.current_drag = None;
    }

    pub fn handle_mousemove(
        &mut self,
        camera: &mut Camera,
        mouse_pos: (f32, f32),
        delta: (f32, f32),
    ) {
        let Some(mode) = self.current_drag else { return };

        match mode {
            DragMode::Node(idx) => {
                let (wx, wy) = camera.screen_to_world(mouse_pos.0, mouse_pos.1);
                self.tables[idx].x = wx - self.grab_offset.0;
                self.tables[idx].y = wy - self.grab_offset.1;
            }
            DragMode::Canvas => {
                camera.pan_x += delta.0;
                camera.pan_y += delta.1;
            }
        }
    }

    pub fn queue_text<'a>(&'a self, camera: &Camera, sections: &mut Vec<Section<'a>>) {
        for node in &self.tables {
            if camera.is_visible(node.x, node.y, node.width, node.height) {
                let (sx, sy) = camera.world_to_screen(node.x, node.y);
                let ez = camera.effective_zoom();
                sections.push(
                    Section::default()
                        .add_text(
                            Text::new(&node.name)
                                .with_color([1.0, 1.0, 1.0, 1.0])
                                .with_scale(14.0 * ez),
                        )
                        .with_screen_position((sx + (10.0 * ez), sy + (8.0 * ez))),
                );
            }
        }
    }

    pub fn hit_test(&self, camera: &Camera, mouse_pos: (f32, f32)) -> Option<usize> {
        let (wx, wy) = camera.screen_to_world(mouse_pos.0, mouse_pos.1);
        for (i, table) in self.tables.iter().enumerate().rev() {
            if wx >= table.x && wx <= table.x + table.width
                && wy >= table.y && wy <= table.y + table.height
            {
                return Some(i);
            }
        }
        None
    }
}
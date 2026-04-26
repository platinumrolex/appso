use std::time::{Duration, Instant};
use crate::ui::header::FpsLimit;

const AFK_DECAY_SECS: f32 = 5.0;
const AFK_MIN_FPS: u32 = 1;

pub fn calculate_frame_delay(
    is_hidden: bool,
    target_fps: FpsLimit,
    last_interaction: Instant,
) -> Duration {
    if is_hidden {
        return Duration::from_secs_f32(1.0 / AFK_MIN_FPS.max(1) as f32);
    }

    let base_delay = 1.0 / target_fps.value as f32;
    let elapsed = last_interaction.elapsed().as_secs_f32();

    if elapsed >= AFK_DECAY_SECS {
        Duration::from_secs_f32(1.0 / AFK_MIN_FPS.max(1) as f32)
    } else {
        let afk_delay = 1.0 / AFK_MIN_FPS.max(1) as f32;
        Duration::from_secs_f32(
            base_delay + (afk_delay - base_delay) * (elapsed / AFK_DECAY_SECS).powi(2),
        )
    }
}


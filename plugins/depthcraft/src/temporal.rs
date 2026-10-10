//! Temporal smoothing filter for video frame sequences.
//!
//! Eliminates frame-to-frame high-frequency flicker in monocular depth estimation
//! while preserving motion fidelity via adaptive edge-gating.

use std::cell::RefCell;

pub struct TemporalState {
    pub prev_depth: Vec<f32>,
    pub prev_lum: Vec<f32>,
    pub width: usize,
    pub height: usize,
    pub last_time: f64,
}

thread_local! {
    static TEMPORAL_CACHE: RefCell<TemporalState> = const {
        RefCell::new(TemporalState {
            prev_depth: Vec::new(),
            prev_lum: Vec::new(),
            width: 0,
            height: 0,
            last_time: -1.0,
        })
    };
}

/// Applies temporal smoothing across consecutive video frames.
/// `strength`: 0.0 to 100.0 (0 = no smoothing, 100 = maximum flicker reduction).
pub fn apply_temporal_smoothing(current_depth: &mut [f32], current_lum: &[f32], width: usize, height: usize, time: f64, strength: f32) {
    if strength <= 0.0 || width == 0 || height == 0 || current_depth.len() < width * height || current_lum.len() < width * height {
        return;
    }

    let smooth_factor = (strength / 100.0).clamp(0.0, 1.0);
    let base_blend = smooth_factor * 0.85; // Max 85% previous frame retention

    TEMPORAL_CACHE.with(|state_cell| {
        let mut state = state_cell.borrow_mut();

        // Check if previous frame has matching dimensions and reasonable time delta
        let has_valid_history = state.width == width
            && state.height == height
            && state.prev_depth.len() == width * height
            && state.prev_lum.len() == width * height
            && state.last_time >= 0.0
            && (time - state.last_time).abs() < 1.0; // Within 1 second

        if has_valid_history {
            for i in 0..width * height {
                let curr_d = match current_depth.get(i) {
                    Some(d) => *d,
                    None => continue,
                };
                let prev_d = match state.prev_depth.get(i) {
                    Some(d) => *d,
                    None => continue,
                };
                let curr_l = match current_lum.get(i) {
                    Some(l) => *l,
                    None => continue,
                };
                let prev_l = match state.prev_lum.get(i) {
                    Some(l) => *l,
                    None => continue,
                };

                // Motion & edge gate: large luminance or depth jump indicates fast motion
                let lum_delta = (curr_l - prev_l).abs();
                let depth_delta = (curr_d - prev_d).abs();
                let motion_score = (lum_delta * 3.0 + depth_delta * 2.0).clamp(0.0, 1.0);

                // Lower retention for fast-moving areas to prevent ghosting
                let effective_blend = base_blend * (1.0 - motion_score);
                let smoothed = curr_d * (1.0 - effective_blend) + prev_d * effective_blend;

                if let Some(target) = current_depth.get_mut(i) {
                    *target = smoothed;
                }
            }
        }

        // Update state for next frame
        if state.prev_depth.len() != width * height {
            state.prev_depth.resize(width * height, 0.0);
        }
        if state.prev_lum.len() != width * height {
            state.prev_lum.resize(width * height, 0.0);
        }

        state.prev_depth.copy_from_slice(&current_depth[..width * height]);
        state.prev_lum.copy_from_slice(&current_lum[..width * height]);
        state.width = width;
        state.height = height;
        state.last_time = time;
    });
}

/// Reset temporal history (e.g. on seek or cut).
pub fn reset_temporal_state() {
    TEMPORAL_CACHE.with(|state_cell| {
        let mut state = state_cell.borrow_mut();
        state.prev_depth.clear();
        state.prev_lum.clear();
        state.width = 0;
        state.height = 0;
        state.last_time = -1.0;
    });
}

//! Presets definition for LumaSweep.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PresetId {
    Custom = 0,
    Chrome = 1,
    SoftStudio = 2,
    Prism = 3,
    BrushedMetal = 4,
    GoldLustre = 5,
    LaserBeam = 6,
}

impl PresetId {
    pub fn from_index(idx: usize) -> Self {
        match idx {
            1 => PresetId::Chrome,
            2 => PresetId::SoftStudio,
            3 => PresetId::Prism,
            4 => PresetId::BrushedMetal,
            5 => PresetId::GoldLustre,
            6 => PresetId::LaserBeam,
            _ => PresetId::Custom,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PresetConfig {
    pub shape: usize,
    pub softness: f32,
    pub width: f32,
    pub sweep_intensity: f32,
    pub highlight_color: [f32; 4],
    pub shadow_color: [f32; 4],
    pub shadow_intensity: f32,
    pub bevel_intensity: f32,
    pub bevel_depth: f32,
    pub bevel_profile: usize,
    pub reception_mode: usize,
    pub chromatic_fringe: f32,
    pub texture_intensity: f32,
    pub texture_scale: f32,
    pub texture_mode: usize,
    pub glow_intensity: f32,
    pub glow_radius: f32,
}

impl PresetConfig {
    pub fn for_preset(preset: PresetId) -> Option<Self> {
        match preset {
            PresetId::Custom => None,
            PresetId::Chrome => Some(Self {
                shape: 2, // Sharp Peak
                softness: 25.0,
                width: 70.0,
                sweep_intensity: 150.0,
                highlight_color: [0.95, 0.98, 1.0, 1.0],
                shadow_color: [0.03, 0.04, 0.08, 1.0],
                shadow_intensity: 60.0,
                bevel_intensity: 180.0,
                bevel_depth: 3.0,
                bevel_profile: 1, // Chisel
                reception_mode: 0,
                chromatic_fringe: 15.0,
                texture_intensity: 8.0,
                texture_scale: 30.0,
                texture_mode: 0,
                glow_intensity: 15.0,
                glow_radius: 25.0,
            }),
            PresetId::SoftStudio => Some(Self {
                shape: 0, // Smooth Hermite
                softness: 80.0,
                width: 120.0,
                sweep_intensity: 90.0,
                highlight_color: [1.0, 0.98, 0.92, 1.0],
                shadow_color: [0.08, 0.08, 0.08, 1.0],
                shadow_intensity: 20.0,
                bevel_intensity: 80.0,
                bevel_depth: 6.0,
                bevel_profile: 0, // Curved
                reception_mode: 1, // Composite
                chromatic_fringe: 0.0,
                texture_intensity: 0.0,
                texture_scale: 25.0,
                texture_mode: 0,
                glow_intensity: 45.0,
                glow_radius: 50.0,
            }),
            PresetId::Prism => Some(Self {
                shape: 0, // Smooth Hermite
                softness: 40.0,
                width: 65.0,
                sweep_intensity: 170.0,
                highlight_color: [1.0, 1.0, 1.0, 1.0],
                shadow_color: [0.05, 0.05, 0.1, 1.0],
                shadow_intensity: 30.0,
                bevel_intensity: 160.0,
                bevel_depth: 4.0,
                bevel_profile: 3, // Rim
                reception_mode: 0,
                chromatic_fringe: 85.0,
                texture_intensity: 0.0,
                texture_scale: 25.0,
                texture_mode: 0,
                glow_intensity: 40.0,
                glow_radius: 35.0,
            }),
            PresetId::BrushedMetal => Some(Self {
                shape: 1, // Linear
                softness: 50.0,
                width: 90.0,
                sweep_intensity: 110.0,
                highlight_color: [0.92, 0.94, 0.98, 1.0],
                shadow_color: [0.04, 0.04, 0.06, 1.0],
                shadow_intensity: 45.0,
                bevel_intensity: 120.0,
                bevel_depth: 3.5,
                bevel_profile: 1, // Chisel
                reception_mode: 0,
                chromatic_fringe: 10.0,
                texture_intensity: 70.0,
                texture_scale: 20.0,
                texture_mode: 0, // Brushed Anisotropic
                glow_intensity: 10.0,
                glow_radius: 20.0,
            }),
            PresetId::GoldLustre => Some(Self {
                shape: 0, // Smooth Hermite
                softness: 55.0,
                width: 85.0,
                sweep_intensity: 130.0,
                highlight_color: [1.0, 0.88, 0.45, 1.0],
                shadow_color: [0.15, 0.08, 0.02, 1.0],
                shadow_intensity: 50.0,
                bevel_intensity: 150.0,
                bevel_depth: 4.5,
                bevel_profile: 0, // Curved
                reception_mode: 0,
                chromatic_fringe: 20.0,
                texture_intensity: 15.0,
                texture_scale: 30.0,
                texture_mode: 1, // Micro Grain
                glow_intensity: 30.0,
                glow_radius: 35.0,
            }),
            PresetId::LaserBeam => Some(Self {
                shape: 2, // Sharp Peak
                softness: 15.0,
                width: 30.0,
                sweep_intensity: 240.0,
                highlight_color: [0.25, 0.95, 1.0, 1.0],
                shadow_color: [0.0, 0.04, 0.12, 1.0],
                shadow_intensity: 70.0,
                bevel_intensity: 220.0,
                bevel_depth: 2.0,
                bevel_profile: 3, // Rim
                reception_mode: 2, // Cutout
                chromatic_fringe: 45.0,
                texture_intensity: 0.0,
                texture_scale: 25.0,
                texture_mode: 0,
                glow_intensity: 120.0,
                glow_radius: 60.0,
            }),
        }
    }
}

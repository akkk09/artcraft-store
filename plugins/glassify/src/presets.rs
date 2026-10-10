//! Presets definition for Glassify.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PresetId {
    Custom = 0,
    Lucid = 1,
    FrostedSatin = 2,
    LiquidGlass = 3,
    OnyxSmoke = 4,
    PrismaticCrystal = 5,
    RibbedFluted = 6,
}

impl PresetId {
    pub fn from_index(idx: usize) -> Self {
        match idx {
            1 => PresetId::Lucid,
            2 => PresetId::FrostedSatin,
            3 => PresetId::LiquidGlass,
            4 => PresetId::OnyxSmoke,
            5 => PresetId::PrismaticCrystal,
            6 => PresetId::RibbedFluted,
            _ => PresetId::Custom,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PresetConfig {
    pub refraction: f32,
    pub ior: f32,
    pub thickness: f32,
    pub shape_profile: usize,
    pub roughness: f32,
    pub edge_softness: f32,
    pub highlight_intensity: f32,
    pub fresnel_reflection: f32,
    pub tint_color: [f32; 4],
    pub tint_intensity: f32,
    pub edge_tint_intensity: f32,
    pub chromatic_dispersion: f32,
    pub distortion_type: usize,
    pub distortion_scale: f32,
    pub distortion_amount: f32,
    pub distortion_speed: f32,
}

impl PresetConfig {
    pub fn for_preset(preset: PresetId) -> Option<Self> {
        match preset {
            PresetId::Custom => None,
            PresetId::Lucid => Some(Self {
                refraction: 40.0,
                ior: 1.52,
                thickness: 18.0,
                shape_profile: 0, // Smooth Convex
                roughness: 0.0,
                edge_softness: 5.0,
                highlight_intensity: 120.0,
                fresnel_reflection: 45.0,
                tint_color: [0.96, 0.98, 1.0, 1.0],
                tint_intensity: 10.0,
                edge_tint_intensity: 30.0,
                chromatic_dispersion: 15.0,
                distortion_type: 0,
                distortion_scale: 40.0,
                distortion_amount: 0.0,
                distortion_speed: 0.0,
            }),
            PresetId::FrostedSatin => Some(Self {
                refraction: 25.0,
                ior: 1.48,
                thickness: 20.0,
                shape_profile: 2, // Flat Pane / UI
                roughness: 65.0,
                edge_softness: 6.0,
                highlight_intensity: 80.0,
                fresnel_reflection: 30.0,
                tint_color: [1.0, 1.0, 1.0, 1.0],
                tint_intensity: 15.0,
                edge_tint_intensity: 40.0,
                chromatic_dispersion: 0.0,
                distortion_type: 2, // Frosted Grain
                distortion_scale: 20.0,
                distortion_amount: 15.0,
                distortion_speed: 0.0,
            }),
            PresetId::LiquidGlass => Some(Self {
                refraction: 60.0,
                ior: 1.34,
                thickness: 25.0,
                shape_profile: 0, // Smooth Convex
                roughness: 5.0,
                edge_softness: 8.0,
                highlight_intensity: 160.0,
                fresnel_reflection: 55.0,
                tint_color: [0.88, 0.96, 1.0, 1.0],
                tint_intensity: 25.0,
                edge_tint_intensity: 60.0,
                chromatic_dispersion: 30.0,
                distortion_type: 1, // Liquid Waves
                distortion_scale: 50.0,
                distortion_amount: 35.0,
                distortion_speed: 1.0,
            }),
            PresetId::OnyxSmoke => Some(Self {
                refraction: 35.0,
                ior: 1.58,
                thickness: 22.0,
                shape_profile: 2, // Flat Pane / UI
                roughness: 10.0,
                edge_softness: 4.0,
                highlight_intensity: 140.0,
                fresnel_reflection: 60.0,
                tint_color: [0.12, 0.12, 0.14, 1.0],
                tint_intensity: 75.0,
                edge_tint_intensity: 90.0,
                chromatic_dispersion: 10.0,
                distortion_type: 0,
                distortion_scale: 40.0,
                distortion_amount: 0.0,
                distortion_speed: 0.0,
            }),
            PresetId::PrismaticCrystal => Some(Self {
                refraction: 80.0,
                ior: 2.2,
                thickness: 20.0,
                shape_profile: 1, // Chisel Bevel
                roughness: 0.0,
                edge_softness: 2.5,
                highlight_intensity: 200.0,
                fresnel_reflection: 80.0,
                tint_color: [1.0, 1.0, 1.0, 1.0],
                tint_intensity: 0.0,
                edge_tint_intensity: 0.0,
                chromatic_dispersion: 85.0,
                distortion_type: 0,
                distortion_scale: 40.0,
                distortion_amount: 0.0,
                distortion_speed: 0.0,
            }),
            PresetId::RibbedFluted => Some(Self {
                refraction: 50.0,
                ior: 1.52,
                thickness: 25.0,
                shape_profile: 2, // Flat Pane / UI
                roughness: 12.0,
                edge_softness: 4.0,
                highlight_intensity: 130.0,
                fresnel_reflection: 50.0,
                tint_color: [0.94, 0.98, 0.96, 1.0],
                tint_intensity: 15.0,
                edge_tint_intensity: 45.0,
                chromatic_dispersion: 20.0,
                distortion_type: 4, // Ribbed / Fluted
                distortion_scale: 24.0,
                distortion_amount: 40.0,
                distortion_speed: 0.0,
            }),
        }
    }
}

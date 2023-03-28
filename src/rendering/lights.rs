use nalgebra_glm as glm;

#[repr(C)]
pub struct PointLight {
    pub color: glm::Vec3,
    // intensity between 0 and 1
    pub intensity: f32,
    // Maximum range of the light,  0 is no limit
    pub range: f32,
    // amount the light dims along the distance of the light
    pub decay: f32,
}

impl PointLight {
    pub fn new(color: glm::Vec3, intensity: f32, range: f32, decay: f32) -> Self {
        Self {
            color,
            intensity: {
                if intensity > 1.0 {
                    1.0
                } else {
                    intensity
                }
            },
            range,
            decay,
        }
    }
}

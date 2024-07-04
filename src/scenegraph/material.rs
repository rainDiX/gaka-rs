use nalgebra_glm as glm;

pub struct Material {
    pub name: String,
    pub ambient: glm::Vec3,
    pub diffuse: glm::Vec3,
    pub specular: glm::Vec3,
    pub shininess: f32,
}

impl Material {
    pub fn new(
        name: String,
        ambient: glm::Vec3,
        diffuse: glm::Vec3,
        specular: glm::Vec3,
        shininess: f32,
    ) -> Self {
        Self {
            name,
            ambient,
            diffuse,
            specular,
            shininess,
        }
    }
}

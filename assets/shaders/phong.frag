#version 460 core
out vec4 color;

in vec3 position_world;
in vec3 position_view;
in vec3 normal;
in vec2 tex_coords;

struct Material {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    float shininess;
};

uniform Material material;

struct PointLight {
    vec3 color;
    float intensity;
    float range;
    float decay;
    vec3 position;
};  

#define MAX_POINTS_LIGHTS 10

uniform int nb_point_lights;
uniform PointLight pointLights[MAX_POINTS_LIGHTS];
uniform mat4 model;


vec3 calculatePointLight(PointLight light) {
    vec3 light_dir = normalize(light.position - position_world);
    vec3 reflect_dir = reflect(-light_dir, normal);
    float distance = length(light.position - position_world);

    float attenuation = 1.0;

    vec3 view_dir = normalize(position_view - position_world);

    if (light.range > 0.0 && distance > light.range) {
        attenuation = 0.0;
    } else if (light.decay > 0.0) {
        attenuation = 1.0 / (1.0 + light.decay * pow(distance / light.range, 2.0));
    }

    vec3 ambient = material.ambient * light.color * light.intensity;
    vec3 diffuse = material.diffuse * light.color * light.intensity * max(dot(normal, light_dir), 0.0);
    vec3 specular = material.specular * light.color * light.intensity * pow(max(dot(view_dir, reflect_dir), 0.0), material.shininess);

    return (ambient + diffuse /* + specular */ ) * attenuation;
}


void main() {
    vec3 result = vec3(0.0);

    for(int i = 0; i < nb_point_lights; i++)
        result += calculatePointLight(pointLights[i]);

    // TODO : support textures to avoid this quirk
    color = vec4(result.x * tex_coords.x, result.y * tex_coords.y, result.z, 1.0);
}
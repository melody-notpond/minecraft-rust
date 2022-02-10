#version 150

in vec3 normal_out;
in vec3 tex_coords_out;
in vec4 light_out;

uniform sampler3D textures;

out vec4 color;

void main() {
    color = texture(textures, tex_coords_out) * light_out;
}

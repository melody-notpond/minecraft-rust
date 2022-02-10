#version 150

in vec3 normal_out;
in vec3 tex_coords_out;

uniform vec3 light;
uniform sampler3D textures;

out vec4 color;

void main() {
    float brightness = dot(normalize(normal_out), normalize(light));
    vec4 colour = texture(textures, tex_coords_out);
    vec3 dark_color = colour.xyz / 2.0;
    color = vec4(mix(dark_color, colour.xyz, brightness), colour.w);
}

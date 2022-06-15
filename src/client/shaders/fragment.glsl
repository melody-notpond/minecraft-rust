#version 330 core

in vec3 v_normal;

out vec4 colour;

uniform vec3 light;

void main() {
    float brightness = dot(normalize(v_normal), normalize(light));
    vec3 dark_colour = vec3(0.6, 0.0, 0.0);
    vec3 bright_colour = vec3(1.0, 0.0, 0.0);
    colour = vec4(mix(dark_colour, bright_colour, brightness), 1.0);
}

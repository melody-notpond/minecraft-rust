#version 140

in vec3 normal_out;

uniform vec3 light;

out vec4 color;

void main() {
    float brightness = dot(normalize(normal_out), normalize(light));
    vec3 dark_color = vec3(0.6, 0.0, 0.0);
    vec3 regular_color = vec3(1.0, 0.0, 0.0);
    color = vec4(mix(dark_color, regular_color, brightness), 1.0);
}

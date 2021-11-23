#version 140

in vec3 normal_out;

uniform vec3 light;
uniform vec3 colour;

out vec4 color;

void main() {
    float brightness = dot(normalize(normal_out), normalize(light));
    vec3 dark_color = colour / 2.0;
    color = vec4(mix(dark_color, colour, brightness), 1.0);
}

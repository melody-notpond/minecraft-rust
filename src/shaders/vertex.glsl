#version 150

in vec3 position;
in vec3 normal;

uniform mat4 model;
uniform mat4 view;
uniform mat4 perspective;

out vec3 normal_out;

void main() {
    mat4 model_view = view * model;
    normal_out = transpose(inverse(mat3(model_view))) * normal;
    gl_Position = perspective * model_view * vec4(position, 1.0);
}

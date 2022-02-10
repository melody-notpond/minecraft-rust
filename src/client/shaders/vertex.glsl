#version 150

in vec3 position;
in vec2 tex_coords;
in vec3 normal;
in uvec2 data;
in uint light;

uniform mat4 model;
uniform mat4 view;
uniform mat4 perspective;
uniform uint texture_count;

out vec3 tex_coords_out;
out vec3 normal_out;
out vec4 light_out;

void main() {
    float x = ((data.x & 0x00f0u) >>  4u) * 0.5;
    float y = ((data.x & 0x0f00u) >>  8u) * 0.5;
    float z = ((data.x & 0xf000u) >> 12u) * 0.5;

    mat4 new_model = model;
    new_model[3].x += x;
    new_model[3].y += y;
    new_model[3].z += z;

    mat4 face_rotation;

    switch (data.x & 0x000fu) {
        // up (+y)
        case 0u:
            face_rotation = mat4(
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0
            );
            break;

        // down (-y)
        case 1u:
            face_rotation = mat4(
                1.0, 0.0, 0.0, 0.0,
                0.0, -1.0, 0.0, 0.0,
                0.0, 0.0, -1.0, 0.0,
                0.0, 0.0, 0.0, 1.0
            ) * mat4(
                -1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, -1.0, 0.0,
                0.0, 0.0, 0.0, 1.0
            );
            break;

        // front (+x)
        case 2u:
            face_rotation = mat4(
                0.0, -1.0, 0.0, 0.0,
                1.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0
            ) * mat4(
                0.0, 0.0, 1.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                -1.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 1.0
            );
            break;

        // back (-x)
        case 3u:
            face_rotation = mat4(
                0.0, 1.0, 0.0, 0.0,
                -1.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0
            ) * mat4(
                0.0, 0.0, -1.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                1.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 1.0
            );
            break;

        // left (+z)
        case 4u:
            face_rotation = mat4(
                1.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, -1.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 1.0
            ) * mat4(
                -1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, -1.0, 0.0,
                0.0, 0.0, 0.0, 1.0
            );
            break;

        // right (-z)
        case 5u:
            face_rotation = mat4(
                1.0, 0.0, 0.0, 0.0,
                0.0, 0.0, -1.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 1.0
            );
            break;

        // identity just in case
        default:
            face_rotation = mat4(
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0
            );
            break;
    }

    vec3 light_colour = vec3(((light & 0xf000u) >> 12) / 15.0, ((light & 0x0f00u) >> 8) / 15.0, ((light & 0x00f0u) >> 4) / 15.0) * (light & 0x000fu) / 15.0;
    const float min_light = 0.05;
    light_colour *= vec3(1.0 - min_light);
    light_colour += vec3(min_light);
    light_out = vec4(light_colour, 1.0);

    mat4 model_view = view * new_model * face_rotation;
    normal_out = transpose(inverse(mat3(model_view))) * normal;
    tex_coords_out = vec3(tex_coords, (data.y + 1u) * 1.0 / texture_count);
    gl_Position = perspective * model_view * vec4(position, 1.0);
}

#version 150

in vec3 position;
in vec2 tex_coords;
in vec3 normal;
in uvec2 data;

uniform mat4 model;
uniform mat4 view;
uniform mat4 perspective;
uniform uint texture_count;

out vec3 tex_coords_out;
out vec3 normal_out;

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

    mat4 model_view = view * new_model * face_rotation;
    normal_out = transpose(inverse(mat3(model_view))) * normal;
    tex_coords_out = vec3(tex_coords, (data.y + 1u) * 1.0 / texture_count);
    gl_Position = perspective * model_view * vec4(position, 1.0);
}

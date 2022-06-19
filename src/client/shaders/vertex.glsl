#version 330 core

in vec3 position;
in uvec2 data;

out vec3 v_normal;

uniform mat4 perspective;
uniform mat4 view;
uniform mat4 model;

void main() {
    float x = float((data.x & 0x00f0u) >>  4u) * 0.25;
    float y = float((data.x & 0x0f00u) >>  8u) * 0.25;
    float z = float((data.x & 0xf000u) >> 12u) * 0.25;

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

        // front (-z)
        case 2u:
            face_rotation = mat4(
                1.0, 0.0, 0.0, 0.0,
                0.0, 0.0, -1.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 1.0
            );
            break;

        // back (+z)
        case 3u:
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

        // left (+x)
        case 4u:
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

        // right (-x)
        case 5u:
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
    v_normal = transpose(inverse(mat3(model_view))) * (face_rotation * vec4(0.0, 1.0, 0.0, 1.0)).xyz;
    gl_Position = perspective * model_view * vec4(position, 1.0);
}

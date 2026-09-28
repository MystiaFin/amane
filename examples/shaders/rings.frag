// amane adds the #version line and gives uv, size and time, write the result to color
void main() {
    vec2 pixel = uv * size;
    vec2 center = size / 2.0;

    float rings = sin(distance(pixel, center) * 0.15 - time * 3.0) * 0.5 + 0.5;

    color = vec4(rings, 0.3, 1.0 - rings, 1.0);
}

/** CRT post-processing fragment shader (GLSL 300 es) */
export const crtFragmentShader = /* glsl */ `
in vec2 vTextureCoord;

uniform sampler2D uTexture;
uniform float uTime;
uniform vec2 uResolution;
uniform float uScanlineIntensity;

out vec4 finalColor;

// Pseudo-random noise
float rand(vec2 co) {
    return fract(sin(dot(co, vec2(12.9898, 78.233))) * 43758.5453);
}

void main() {
    vec2 uv = vTextureCoord;

    // Very subtle barrel distortion
    vec2 centered = uv - 0.5;
    float dist = dot(centered, centered);
    uv = uv + centered * dist * 0.008;

    // Clamp to avoid sampling outside texture
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        finalColor = vec4(0.0, 0.0, 0.0, 1.0);
        return;
    }

    // Minimal chromatic aberration — only visible at extreme edges
    float aberration = dist * 0.0008;
    float r = texture(uTexture, uv + vec2(aberration, 0.0)).r;
    float g = texture(uTexture, uv).g;
    float b = texture(uTexture, uv - vec2(aberration, 0.0)).b;
    vec3 color = vec3(r, g, b);

    // Faint scanlines
    float scanline = sin((uv.y * uResolution.y + uTime * 1.0) * 3.14159) * 0.5 + 0.5;
    color *= 1.0 - uScanlineIntensity * 0.4 * (1.0 - scanline);

    // Gentle vignette
    float vignette = 1.0 - dist * 1.0;
    vignette = clamp(vignette, 0.0, 1.0);
    color *= vignette;

    // Light film grain
    float grain = rand(uv + fract(uTime * 0.1)) * 0.03;
    color += grain;

    // Cool phosphor tint — slight blue boost for neon aesthetic
    color.b *= 1.03;

    finalColor = vec4(color, 1.0);
}
`;

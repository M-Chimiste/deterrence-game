import { Filter, GlProgram } from "pixi.js";
import { crtFragmentShader } from "./crt.frag";

const defaultVertex = /* glsl */ `
in vec2 aPosition;
out vec2 vTextureCoord;

uniform vec4 uInputSize;
uniform vec4 uOutputFrame;
uniform vec4 uOutputTexture;

vec4 filterVertexPosition(void) {
    vec2 position = aPosition * uOutputFrame.zw + uOutputFrame.xy;
    position.x = position.x * (2.0 / uOutputTexture.x) - 1.0;
    position.y = position.y * (2.0*uOutputTexture.z / uOutputTexture.y) - uOutputTexture.z;
    return vec4(position, 0.0, 1.0);
}

vec2 filterTextureCoord(void) {
    return aPosition * (uOutputFrame.zw * uInputSize.zw);
}

void main(void) {
    gl_Position = filterVertexPosition();
    vTextureCoord = filterTextureCoord();
}
`;

export class CRTFilter extends Filter {
  private _time: number = 0;
  private _resolution: Float32Array;

  constructor(width: number, height: number) {
    const resolution = new Float32Array([width, height]);
    const glProgram = GlProgram.from({
      vertex: defaultVertex,
      fragment: crtFragmentShader,
    });

    super({
      glProgram,
      resources: {
        crtUniforms: {
          uTime: { value: 0, type: "f32" },
          uResolution: { value: resolution, type: "vec2<f32>" },
          uScanlineIntensity: { value: 0.15, type: "f32" },
        },
      },
    });

    this._resolution = resolution;
  }

  /** Advance time uniform for animated effects */
  update(deltaTime: number) {
    this._time += deltaTime * 0.016; // normalize to ~seconds
    this.resources.crtUniforms.uniforms.uTime = this._time;
  }

  setResolution(width: number, height: number) {
    this._resolution[0] = width;
    this._resolution[1] = height;
    this.resources.crtUniforms.uniforms.uResolution = this._resolution;
  }

  set scanlineIntensity(value: number) {
    this.resources.crtUniforms.uniforms.uScanlineIntensity = value;
  }

  get scanlineIntensity(): number {
    return this.resources.crtUniforms.uniforms.uScanlineIntensity;
  }
}

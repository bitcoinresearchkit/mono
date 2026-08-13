declare module 'lean-qr' {
  interface ImageDataLike {
    readonly data: Uint8ClampedArray;
  }

  interface Context2DLike<DataT extends ImageDataLike> {
    createImageData(width: number, height: number): DataT;
    putImageData(data: DataT, x: number, y: number): void;
  }

  interface CanvasLike<DataT extends ImageDataLike> {
    width: number;
    height: number;
    getContext(type: '2d'): Context2DLike<DataT> | null;
  }

  /**
   * A colour in `[red, green, blue, alpha]` format (all values from 0 to 255).
   * If alpha is omitted, it is assumed to be 255 (opaque).
   */
  export type RGBA = readonly [number, number, number, number?];

  export interface Bitmap1D {
    /**
     * Appends a sequence of bits.
     *
     * @param value an integer containing the bits to append (big endian).
     * @param bits the number of bits to read from `value`. Must be between 1 and 24.
     */
    push(value: number, bits: number): void;
  }

  export interface StringOptions {
    /** the text to use for modules which are 'on' (typically black) */
    on?: string;

    /** the text to use for modules which are 'off' (typically white) */
    off?: string;

    /** the text to use for linefeeds between rows */
    lf?: string;

    /** the padding to apply around the output (populated with 'off' modules) */
    pad?: number;

    /**
     * the padding to apply on the left and right of the output (populated with 'off' modules)
     * @deprecated use `pad` instead
     */
    padX?: number;

    /**
     * the padding to apply on the top and bottom of the output (populated with 'off' modules)
     * @deprecated use `pad` instead
     */
    padY?: number;
  }

  export interface ImageDataOptions {
    /** the colour to use for modules which are 'on' (typically black) */
    on?: RGBA;

    /** the colour to use for modules which are 'off' (typically white) */
    off?: RGBA;

    /** the padding to apply around the output (filled with 'off') */
    pad?: number;

    /**
     * the padding to apply on the left and right of the output (filled with 'off')
     * @deprecated use `pad` instead
     */
    padX?: number;

    /**
     * the padding to apply on the top and bottom of the output (filled with 'off')
     * @deprecated use `pad` instead
     */
    padY?: number;
  }

  export interface Bitmap2D {
    /** the width / height of the QR code in modules (excluding any padding) */
    readonly size: number;

    /**
     * Read the state of a module from the QR code.
     *
     * @param x the x coordinate to read. Can be negative / out of bounds.
     * @param y the y coordinate to read. Can be negative / out of bounds.
     * @returns true if the requested module is set (i.e. typically black)
     */
    get(x: number, y: number): boolean;

    /**
     * Generate a string containing the QR code, suitable for displaying in a
     * terminal environment. Generally, you should customise on and off to use
     * the ANSI escapes of your target terminal for better rendering.
     *
     * @param options optional configuration for the display.
     */
    toString(options?: Readonly<StringOptions>): string;

    /**
     * Generate image data containing the QR code, at a scale of 1 pixel per
     * module. Use this if you need more control than toCanvas allows.
     *
     * @param context a context to use for creating the image data.
     * @param options optional configuration for the display.
     */
    toImageData<DataT extends ImageDataLike>(
      context: Context2DLike<DataT>,
      options?: Readonly<ImageDataOptions>,
    ): DataT;

    /**
     * Generate a `data:image/*` URL for the QR code.
     *
     * @param options optional configuration for the output.
     * @returns a string suitable for use as the `src` of an `img` tag.
     */
    toDataURL(
      options?: Readonly<
        ImageDataOptions & {
          type?: `image/${string}`;
          scale?: number;
        }
      >,
    ): string;

    /**
     * Populate a given canvas with the QR code, at a scale of 1 pixel per
     * module. Set image-rendering: pixelated and scale the canvas using CSS
     * for a large image. Automatically resizes the canvas to fit the QR code
     * if necessary.
     *
     * @param canvas the canvas to populate.
     * @param options optional configuration for the display.
     */
    toCanvas(
      canvas: CanvasLike<ImageDataLike>,
      options?: Readonly<ImageDataOptions>,
    ): void;
  }

  export type Mask = 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7;
  export type Mode = (data: Bitmap1D, version: number) => void;
  export interface ModeFactory {
    (value: string): Mode;
    /** a function which returns true when given a character which the current mode can represent */
    test(string: string): boolean;
    /** a function which returns an estimate of the number of bits required to encode a given value */
    est(value: string, version: number): number;
    /** an optional ECI which must be active for this mode to be interpreted correctly by a reader */
    eci?: number;
  }

  interface ModeAutoOptions {
    /** a list of modes which can be considered when encoding a message */
    modes?: ReadonlyArray<ModeFactory>;
  }

  export const mode: Readonly<{
    /** automatically picks the most optimal combination of modes for the requested message */
    auto(value: string, options?: Readonly<ModeAutoOptions>): Mode;
    /** concatenates multiple modes together */
    multi(...modes: ReadonlyArray<Mode>): Mode;
    /** sets the Extended Channel Interpretation for the message from this point onwards */
    eci(id: number): Mode;
    /** supports `0-9` and stores 3 characters per 10 bits */
    numeric: ModeFactory;
    /** supports `0-9A-Z $%*+-./:` and stores 2 characters per 11 bits */
    alphaNumeric: ModeFactory;
    /** arbitrary byte data, typically combined with `eci` */
    bytes(data: Uint8Array | ReadonlyArray<number>): Mode;
    /** supports 7-bit ASCII and stores 1 character per 8 bits with no ECI */
    ascii: ModeFactory;
    /** supports 8-bit ISO-8859-1 and stores 1 character per 8 bits with ECI 3 */
    iso8859_1: ModeFactory;
    /** supports double-byte Shift-JIS characters stores 1 character per 13 bits */
    shift_jis: ModeFactory;
    /** supports variable length UTF-8 with ECI 26 */
    utf8: ModeFactory;
  }>;

  export type Correction = number & { readonly _: unique symbol };
  export const correction: Readonly<{
    /**
     * minimum possible correction level (same as L)
     * @deprecated use correction.L
     */
    min: Correction;
    /** ~7.5% error tolerance, ~25% data overhead */
    L: Correction;
    /** ~15% error tolerance, ~60% data overhead */
    M: Correction;
    /** ~22.5% error tolerance, ~120% data overhead */
    Q: Correction;
    /** ~30% error tolerance, ~190% data overhead */
    H: Correction;
    /**
     * maximum possible correction level (same as H)
     * @deprecated use correction.H
     */
    max: Correction;
  }>;

  export interface GenerateOptions extends ModeAutoOptions {
    /** the minimum correction level to use (higher levels may still be used if the chosen version has space) */
    minCorrectionLevel?: Correction;
    /** the maximum correction level to use */
    maxCorrectionLevel?: Correction;
    /** the minimum version (size) of code to generate (must be between 1 and 40) */
    minVersion?: number;
    /** the maximum version (size) of code to generate (must be between 1 and 40) */
    maxVersion?: number;
    /** a mask to use on the QR code (should be left as `null` for ISO compliance but may be changed for artistic effect) */
    mask?: null | Mask;
    /** padding bits to use for extra space in the QR code (should be left as the default for ISO compliance but may be changed for artistic effect) */
    trailer?: number;
  }

  /**
   * Generate a QR code.
   *
   * @param data either a string, or a pre-encoded mode.
   * @param options optional configuration for the QR code.
   * @returns the requested QR code.
   */
  export type GenerateFn = (
    data: Mode | string,
    options?: Readonly<GenerateOptions>,
  ) => Bitmap2D;
  interface Generate extends GenerateFn {
    /**
     * Creates a scoped `generate` function which considers additional modes
     * when using auto encoding.
     *
     * @param modes the modes to add.
     * @returns a `generate` function which will additionally consider the
     * given modes when using auto encoding.
     *
     * @deprecated this will be removed in version 3. Prefer passing an explicit list of modes when calling `generate`.
     */
    with(...modes: ReadonlyArray<ModeFactory>): GenerateFn;
  }
  export const generate: Generate;
}

declare module 'lean-qr/nano' {
  import type {
    Correction,
    Bitmap2D as FullBitmap2D,
    GenerateOptions as FullGenerateOptions,
  } from 'lean-qr';
  import { correction as fullCorrection } from 'lean-qr';

  export type { Correction };

  export const correction: Pick<typeof fullCorrection, 'L' | 'M' | 'Q' | 'H'>;

  export type Bitmap2D = Pick<FullBitmap2D, 'size' | 'get' | 'toCanvas'>;

  export type GenerateOptions = Pick<
    FullGenerateOptions,
    'minCorrectionLevel' | 'minVersion'
  >;

  /**
   * Generate a QR code.
   *
   * @param data either a string, or a pre-encoded mode.
   * @param options optional configuration for the QR code.
   * @returns the requested QR code.
   */
  export function generate(
    data: string,
    options?: Readonly<GenerateOptions>,
  ): Bitmap2D;
}

declare module 'lean-qr/extras/svg' {
  import type { Bitmap2D as FullBitmap2D } from 'lean-qr';

  type Bitmap2D = Pick<FullBitmap2D, 'size' | 'get'>;

  export interface SVGOptions {
    /** the colour to use for modules which are 'on' (typically black) */
    on?: string;
    /** the colour to use for modules which are 'off' (typically white) */
    off?: string;
    /** the padding to apply around the output (filled with 'off') */
    pad?: number;
    /**
     * the padding to apply on the left and right of the output (filled with 'off')
     * @deprecated use `pad` instead
     */
    padX?: number;
    /**
     * the padding to apply on the top and bottom of the output (filled with 'off')
     * @deprecated use `pad` instead
     */
    padY?: number;
    /** a width to apply to the resulting image (overrides `scale`) */
    width?: number | null;
    /** a height to apply to the resulting image (overrides `scale`) */
    height?: number | null;
    /** a scale to apply to the resulting image (`scale` pixels = 1 module) */
    scale?: number;
  }

  /**
   * Generate the raw outline of the QR code for use in an existing SVG.
   *
   * @param code the QR code to convert.
   * @returns a string suitable for passing to the `d` attribute of a `path`.
   */
  export function toSvgPath(code: Bitmap2D): string;

  /**
   * Generate an SVG element which can be added to the DOM.
   *
   * @param code the QR code to convert.
   * @param options optional configuration for the display.
   * @returns an SVG element.
   */
  export function toSvg(
    code: Bitmap2D,
    target: Document | SVGElement,
    options?: Readonly<SVGOptions>,
  ): SVGElement;

  /**
   * Generate an SVG document which can be exported to a file or served from a
   * web server.
   *
   * @param code the QR code to convert.
   * @param options optional configuration for the display.
   * @returns an SVG document.
   */
  export function toSvgSource(
    code: Bitmap2D,
    options?: Readonly<
      SVGOptions & {
        /** `true` to include an XML declaration at the start of the source (for standalone documents which will not be embedded inside another document) */
        xmlDeclaration?: boolean;
      }
    >,
  ): string;

  /**
   * Generate a `data:image/svg+xml` URL.
   *
   * @param code the QR code to convert.
   * @param options optional configuration for the display.
   * @returns a string suitable for use as the `src` of an `img` tag.
   */
  export function toSvgDataURL(
    code: Bitmap2D,
    options?: Readonly<SVGOptions>,
  ): string;
}

declare module 'lean-qr/extras/node_export' {
  import type { RGBA, Bitmap2D as FullBitmap2D } from 'lean-qr';

  type Bitmap2D = Pick<FullBitmap2D, 'size' | 'get'>;

  export interface PNGOptions {
    /** the colour to use for modules which are 'on' (typically black) */
    on?: RGBA;
    /** the colour to use for modules which are 'off' (typically white) */
    off?: RGBA;
    /** the padding to apply around the output (filled with 'off') */
    pad?: number;
    /**
     * the padding to apply on the left and right of the output (filled with 'off')
     * @deprecated use `pad` instead
     */
    padX?: number;
    /**
     * the padding to apply on the top and bottom of the output (filled with 'off')
     * @deprecated use `pad` instead
     */
    padY?: number;
    /** a scale to apply to the resulting image (`scale` pixels = 1 module) */
    scale?: number;
  }

  /**
   * Generate a PNG document which can be exported to a file or served from a
   * web server.
   *
   * @param code the QR code to convert.
   * @param options optional configuration for the display.
   * @returns a PNG document.
   */
  export function toPngBuffer(
    code: Bitmap2D,
    options?: Readonly<PNGOptions>,
  ): Uint8Array;

  /**
   * Generate a `data:image/png` URL.
   *
   * @param code the QR code to convert.
   * @param options optional configuration for the display.
   * @returns a string suitable for use as the `src` of an `img` tag.
   */
  export function toPngDataURL(
    code: Bitmap2D,
    options?: Readonly<PNGOptions>,
  ): string;
}

declare module 'lean-qr/extras/react' {
  import type {
    Bitmap2D as FullBitmap2D,
    GenerateOptions,
    ImageDataOptions,
  } from 'lean-qr';
  import type {
    SVGOptions,
    toSvgDataURL as toSvgDataURLFn,
  } from 'lean-qr/extras/svg';

  export interface AsyncFramework<T> {
    createElement: (
      type: 'canvas',
      props: {
        ref: any;
        style: { imageRendering: 'pixelated' };
        className: string;
      },
    ) => T;
    useRef<T>(initialValue: T | null): { readonly current: T | null };
    useEffect(fn: () => void | (() => void), deps: unknown[]): void;
  }

  interface QRComponentProps {
    content: string;
    className?: string;
  }

  export interface AsyncQRComponentProps
    extends ImageDataOptions,
      GenerateOptions,
      QRComponentProps {}

  export type AsyncQRComponent<T> = (
    props: Readonly<AsyncQRComponentProps>,
  ) => T;

  /**
   * Generate an asynchronous QR component (rendering to a `canvas`).
   * You should call this just once, in the global scope.
   *
   * ```js
   * import * as React from 'react';
   * import { generate } from 'lean-qr';
   * import { makeAsyncComponent } from 'lean-qr/extras/react';
   * const QR = makeAsyncComponent(React, generate);
   * ```
   *
   * This is not suitable for server-side rendering (use `makeSyncComponent`
   * instead).
   *
   * @param framework the framework to use (e.g. `React`).
   * @param generate the `generate` function to use
   * (from `lean-qr` or `lean-qr/nano`).
   * @param defaultProps optional default properties to apply when the
   * component is used (overridden by properties set on use).
   * @returns a component which can be rendered elsewhere.
   */
  export function makeAsyncComponent<T>(
    framework: Readonly<AsyncFramework<T>>,
    generate: (
      data: string,
      options?: Readonly<GenerateOptions>,
    ) => Pick<FullBitmap2D, 'toCanvas'>,
    defaultProps?: Readonly<Partial<AsyncQRComponentProps>>,
  ): AsyncQRComponent<T>;

  export interface SyncFramework<T> {
    createElement: (
      type: 'img',
      props: {
        src: string;
        style: { imageRendering: 'pixelated' };
        className: string;
      },
    ) => T;
    useMemo<T>(fn: () => T, deps: unknown[]): T;
  }

  export interface SyncQRComponentProps
    extends SVGOptions,
      GenerateOptions,
      QRComponentProps {}

  export type SyncQRComponent<T> = (props: Readonly<SyncQRComponentProps>) => T;

  /**
   * Generate a synchronous QR component (rendering to an SVG).
   * You should call this just once, in the global scope.
   *
   * ```js
   * import * as React from 'react';
   * import { generate } from 'lean-qr';
   * import { toSvgDataURL } from 'lean-qr/extras/svg';
   * import { makeSyncComponent } from 'lean-qr/extras/react';
   * const QR = makeSyncComponent(React, generate, toSvgDataURL);
   * ```
   *
   * This is best suited for server-side rendering (prefer
   * `makeAsyncComponent` if you only need client-side rendering).
   *
   * @param framework the framework to use (e.g. `React`).
   * @param generate the `generate` function to use
   * (from `lean-qr` or `lean-qr/nano`).
   * @param toSvgDataURL the `toSvgDataURL` function to use
   * (from `lean-qr/extras/svg`).
   * @param defaultProps optional default properties to apply when the
   * component is used (overridden by properties set on use).
   * @returns a component which can be rendered elsewhere.
   */
  export function makeSyncComponent<T>(
    framework: Readonly<SyncFramework<T>>,
    generate: (
      data: string,
      options?: Readonly<GenerateOptions>,
    ) => Pick<FullBitmap2D, 'size' | 'get'>,
    toSvgDataURL: typeof toSvgDataURLFn,
    defaultProps?: Readonly<Partial<SyncQRComponentProps>>,
  ): SyncQRComponent<T>;
}

declare module 'lean-qr/extras/vue' {
  import type {
    Bitmap2D as FullBitmap2D,
    GenerateOptions,
    ImageDataOptions,
  } from 'lean-qr';
  import type {
    SVGOptions,
    toSvgDataURL as toSvgDataURLFn,
  } from 'lean-qr/extras/svg';

  export interface Framework {
    h:
      | ((type: 'canvas', props: { ref: string; style: string }) => unknown)
      | ((type: 'img', props: { src: string; style: string }) => unknown);
  }

  interface QRComponentProps {
    content: string;
  }

  export interface VueCanvasComponentProps
    extends ImageDataOptions,
      GenerateOptions,
      QRComponentProps {}

  type VueComponentDefinition<Props> = {
    props: {
      [k in keyof Props]-?: {
        type: { (): Required<Props>[k] };
        required: undefined extends Props[k] ? false : true;
      };
    };
  } & ThisType<unknown>;

  /**
   * Generate a QR component which renders to a `canvas`.
   * You should call this just once, in the global scope.
   *
   * ```js
   * import { h, defineComponent } from 'vue';
   * import { generate } from 'lean-qr';
   * import { makeVueCanvasComponent } from 'lean-qr/extras/vue';
   * export const QR = defineComponent(makeVueCanvasComponent({ h }, generate));
   * ```
   *
   * This is not suitable for server-side rendering (use `makeSyncComponent`
   * instead).
   *
   * @param framework the framework to use (e.g. `{ h }`).
   * @param generate the `generate` function to use
   * (from `lean-qr` or `lean-qr/nano`).
   * @param defaultProps optional default properties to apply when the
   * component is used (overridden by properties set on use).
   * @returns a component which can be rendered elsewhere.
   */
  export function makeVueCanvasComponent(
    framework: Readonly<Framework>,
    generate: (
      data: string,
      options?: Readonly<GenerateOptions>,
    ) => Pick<FullBitmap2D, 'toCanvas'>,
    defaultProps?: Readonly<Partial<VueCanvasComponentProps>>,
  ): VueComponentDefinition<VueCanvasComponentProps>;

  export interface VueSVGComponentProps
    extends SVGOptions,
      GenerateOptions,
      QRComponentProps {}

  /**
   * Generate a QR component which renders to an SVG.
   * You should call this just once, in the global scope:
   *
   * ```js
   * import { h, defineComponent } from 'vue';
   * import { generate } from 'lean-qr';
   * import { toSvgDataURL } from 'lean-qr/extras/svg';
   * import { makeVueSvgComponent } from 'lean-qr/extras/vue';
   * export const QR = defineComponent(makeVueSvgComponent({ h }, generate, toSvgDataURL));
   * ```
   *
   * This is best suited for server-side rendering (prefer
   * `makeAsyncComponent` if you only need client-side rendering).
   *
   * @param framework the framework to use (e.g. `{ h }`).
   * @param generate the `generate` function to use
   * (from `lean-qr` or `lean-qr/nano`).
   * @param toSvgDataURL the `toSvgDataURL` function to use
   * (from `lean-qr/extras/svg`).
   * @param defaultProps optional default properties to apply when the
   * component is used (overridden by properties set on use).
   * @returns a component which can be rendered elsewhere.
   */
  export function makeVueSvgComponent(
    framework: Readonly<Framework>,
    generate: (
      data: string,
      options?: Readonly<GenerateOptions>,
    ) => Pick<FullBitmap2D, 'size' | 'get'>,
    toSvgDataURL: typeof toSvgDataURLFn,
    defaultProps?: Readonly<Partial<VueSVGComponentProps>>,
  ): VueComponentDefinition<VueSVGComponentProps>;
}

declare module 'lean-qr/extras/errors' {
  /**
   * Convert an error into a human-readable message. This is intended for use
   * with Lean QR errors, but will return somewhat meaningful messages for
   * other errors too.
   *
   * @param error the error to convert.
   * @returns a human-readable message explaining the error.
   */
  export function readError(error: unknown): string;
}

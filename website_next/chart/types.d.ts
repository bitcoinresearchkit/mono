import { bitview } from "../../utils/client.js";
import { orders } from "./order.js";
import { scales } from "./scale.js";
import { timeframes, timeframeOptions } from "./timeframes.js";
import { views } from "./views.js";

declare global {
  type ChartX = Date | number;
  type ChartSample = {
    x: ChartX;
    y: number;
  };
  type ChartMetric = (client: typeof bitview) => TimeframeMetric;
  type ChartOrder = (typeof orders)[number]["value"];
  type ChartPoint = ChartSample & {
    plotX: number;
    plotY: number;
  };
  type ChartResult = {
    dateEntries(): Iterable<[Date, number | null | undefined]>;
  };
  type ChartScale = (typeof scales)[number]["value"];
  type ChartSeries = {
    label: string;
    color: () => string;
    role?: "line";
    metric: ChartMetric;
  };
  type ChartUnit = {
    id: string;
    name: string;
    format(value: number): string;
  };
  type ChartView = (typeof views)[number]["value"];
  type Chart = {
    title: string;
    unit: ChartUnit;
    defaultType?: ChartView;
    defaultScale?: ChartScale;
    series: ChartSeries[];
  };
  type ChartFrame = {
    width: number;
    height: number;
    left: number;
    right: number;
    top: number;
    bottom: number;
    plotWidth: number;
    plotHeight: number;
  };
  type ChartFrameOptions = {
    leftPadding?: number;
    rightPadding?: number;
    topPadding?: number;
    bottomPadding?: number;
  };

  type LegendReadout = {
    rows: ({ value: HTMLOutputElement } | null)[];
  };
  type LoadedSeries = {
    series: ChartSeries;
    color: string;
    samples: ChartSample[];
  };
  type PlotContext = {
    group: SVGGElement;
    loadedSeries: LoadedSeries[];
    frame: ChartFrame;
    highlight: SeriesHighlight;
    scale: ChartScale;
    order: ChartOrder;
  };
  type PlottedSeries = {
    series: ChartSeries;
    color: string;
    points: ChartPoint[];
    hitTest?: (
      point: ChartPoint | StackedPoint,
      pointerX: number,
      pointerY: number,
    ) => number;
  };
  type ScaleBounds = {
    min: number;
    max: number;
    minPositive: number;
  };
  type SeriesHighlight = {
    addNode(
      node: SVGPathElement | SVGCircleElement,
      index: number,
    ): void;
    clearPreview(): void;
    clearNodes(): void;
    preview(index: number): void;
  };
  type StackedPoint = ChartPoint & {
    plotY0: number;
    plotY1: number;
  };
  type StackedPlottedSeries = Omit<PlottedSeries, "points" | "hitTest"> & {
    points: StackedPoint[];
    hitTest?: PlottedSeries["hitTest"];
  };
  type XyPlottedSeries = {
    points: ChartPoint[];
    readout?: number | string;
  };
  type XySeries = {
    label: string;
    color: () => string;
    kind: "line" | "point";
    hidden?: boolean;
  };

  type TimeframeEndpoint = {
    fetch(): Promise<ChartResult>;
    last(count: number): { fetch(): Promise<ChartResult> };
  };
  type TimeframeIndex = (typeof timeframes)[TimeframeValue]["index"];
  type TimeframeMetric = {
    by: Record<TimeframeIndex, TimeframeEndpoint>;
  };
  type TimeframeValue = (typeof timeframeOptions)[number]["value"];
}

export {};

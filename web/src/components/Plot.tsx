import { useCallback, useEffect, useRef, useState } from "react";
import { niceStep, type Compiled } from "../lib/engine";

export type View = { xMin: number; xMax: number; yMin: number; yMax: number };

export type Track = {
  id: string;
  label: string;
  color: string;
  compiled: Compiled;
  showDerivative: boolean;
  showMarkers: boolean;
};

type Props = {
  tracks: Track[];
  view: View;
  onView: (view: View) => void;
  time: number;
};

type Readout = { x: number; values: { label: string; color: string; y: number }[] } | null;

const GRID = "#181c25";
const GRID_MINOR = "#101319";
const AXIS = "#39414f";
const LABEL = "#6b7385";
const CURSOR = "#4b5566";

function decimalsFor(step: number): number {
  if (step <= 0) return 2;
  return Math.max(0, Math.min(8, -Math.floor(Math.log10(step))));
}

function formatTick(value: number, decimals: number): string {
  if (value === 0) return "0";
  const magnitude = Math.abs(value);
  if (magnitude >= 1e5 || magnitude < 1e-4) return value.toExponential(1).replace("e+", "e");
  return value.toFixed(decimals);
}

export default function Plot({ tracks, view, onView, time }: Props) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const wrapRef = useRef<HTMLDivElement | null>(null);
  const sizeRef = useRef({ width: 0, height: 0 });
  const dragRef = useRef<{ x: number; y: number; view: View } | null>(null);
  const pointerRef = useRef<{ x: number; y: number } | null>(null);
  const [readout, setReadout] = useState<Readout>(null);

  const draw = useCallback(() => {
    const canvas = canvasRef.current;
    const wrap = wrapRef.current;
    if (!canvas || !wrap) return;

    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    const width = wrap.clientWidth;
    const height = wrap.clientHeight;
    if (width === 0 || height === 0) return;

    if (sizeRef.current.width !== width || sizeRef.current.height !== height) {
      canvas.width = Math.round(width * dpr);
      canvas.height = Math.round(height * dpr);
      canvas.style.width = `${width}px`;
      canvas.style.height = `${height}px`;
      sizeRef.current = { width, height };
    }

    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, width, height);

    const { xMin, xMax, yMin, yMax } = view;
    const spanX = xMax - xMin;
    const spanY = yMax - yMin;
    const toX = (x: number) => ((x - xMin) / spanX) * width;
    const toY = (y: number) => ((yMax - y) / spanY) * height;

    const stepX = niceStep(spanX, width / 90);
    const stepY = niceStep(spanY, height / 70);
    const decX = decimalsFor(stepX);
    const decY = decimalsFor(stepY);

    ctx.lineWidth = 1;
    ctx.strokeStyle = GRID_MINOR;
    ctx.beginPath();
    for (let x = Math.ceil(xMin / (stepX / 5)) * (stepX / 5); x <= xMax; x += stepX / 5) {
      const px = Math.round(toX(x)) + 0.5;
      ctx.moveTo(px, 0);
      ctx.lineTo(px, height);
    }
    for (let y = Math.ceil(yMin / (stepY / 5)) * (stepY / 5); y <= yMax; y += stepY / 5) {
      const py = Math.round(toY(y)) + 0.5;
      ctx.moveTo(0, py);
      ctx.lineTo(width, py);
    }
    ctx.stroke();

    ctx.strokeStyle = GRID;
    ctx.beginPath();
    for (let x = Math.ceil(xMin / stepX) * stepX; x <= xMax; x += stepX) {
      const px = Math.round(toX(x)) + 0.5;
      ctx.moveTo(px, 0);
      ctx.lineTo(px, height);
    }
    for (let y = Math.ceil(yMin / stepY) * stepY; y <= yMax; y += stepY) {
      const py = Math.round(toY(y)) + 0.5;
      ctx.moveTo(0, py);
      ctx.lineTo(width, py);
    }
    ctx.stroke();

    const axisY = Math.min(Math.max(toY(0), 0), height);
    const axisX = Math.min(Math.max(toX(0), 0), width);

    ctx.strokeStyle = AXIS;
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    ctx.moveTo(0, Math.round(axisY) + 0.5);
    ctx.lineTo(width, Math.round(axisY) + 0.5);
    ctx.moveTo(Math.round(axisX) + 0.5, 0);
    ctx.lineTo(Math.round(axisX) + 0.5, height);
    ctx.stroke();

    ctx.fillStyle = LABEL;
    ctx.font = "11px ui-monospace, SFMono-Regular, Menlo, monospace";
    ctx.textAlign = "center";
    ctx.textBaseline = "top";
    for (let x = Math.ceil(xMin / stepX) * stepX; x <= xMax; x += stepX) {
      if (Math.abs(x) < stepX * 1e-6) continue;
      const px = toX(x);
      const py = Math.min(Math.max(axisY + 6, 4), height - 16);
      ctx.fillText(formatTick(x, decX), px, py);
    }

    ctx.textAlign = "right";
    ctx.textBaseline = "middle";
    for (let y = Math.ceil(yMin / stepY) * stepY; y <= yMax; y += stepY) {
      if (Math.abs(y) < stepY * 1e-6) continue;
      const py = toY(y);
      const px = Math.min(Math.max(axisX - 8, 34), width - 4);
      ctx.fillText(formatTick(y, decY), px, py);
    }

    const drawPoints = (points: Float32Array, color: string, dashed: boolean) => {
      ctx.strokeStyle = color;
      ctx.lineWidth = dashed ? 1.4 : 2;
      ctx.setLineDash(dashed ? [5, 4] : []);
      ctx.lineJoin = "round";
      ctx.lineCap = "round";
      ctx.beginPath();
      let pen = false;
      for (let i = 0; i < points.length; i += 2) {
        const x = points[i];
        const y = points[i + 1];
        if (Number.isNaN(x) || Number.isNaN(y)) {
          pen = false;
          continue;
        }
        const px = toX(x);
        const py = toY(y);
        if (py < -1e5 || py > 1e5) {
          pen = false;
          continue;
        }
        if (pen) ctx.lineTo(px, py);
        else ctx.moveTo(px, py);
        pen = true;
      }
      ctx.stroke();
      ctx.setLineDash([]);
    };

    for (const track of tracks) {
      const fn = track.compiled.handle;
      fn.setTime(time);

      if (track.showDerivative) {
        const points = fn.sampleDerivative(xMin, xMax, yMin, yMax, width, height);
        drawPoints(points, track.color, true);
      }

      const points = fn.sample(xMin, xMax, yMin, yMax, width, height);
      ctx.save();
      ctx.shadowColor = track.color;
      ctx.shadowBlur = 8;
      drawPoints(points, track.color, false);
      ctx.restore();

      if (track.showMarkers) {
        const roots = fn.roots(xMin, xMax);
        ctx.fillStyle = "#0a0b0f";
        ctx.strokeStyle = track.color;
        ctx.lineWidth = 1.8;
        for (const root of roots) {
          ctx.beginPath();
          ctx.arc(toX(root), toY(0), 4, 0, Math.PI * 2);
          ctx.fill();
          ctx.stroke();
        }

        const extrema = fn.extrema(xMin, xMax);
        for (let i = 0; i < extrema.length; i += 3) {
          const ex = extrema[i];
          const ey = extrema[i + 1];
          if (ey < yMin || ey > yMax) continue;
          ctx.fillStyle = track.color;
          ctx.beginPath();
          ctx.arc(toX(ex), toY(ey), 3.5, 0, Math.PI * 2);
          ctx.fill();
        }
      }
    }

    const pointer = pointerRef.current;
    if (pointer && !dragRef.current) {
      const x = xMin + (pointer.x / width) * spanX;
      ctx.strokeStyle = CURSOR;
      ctx.lineWidth = 1;
      ctx.setLineDash([3, 3]);
      ctx.beginPath();
      ctx.moveTo(Math.round(pointer.x) + 0.5, 0);
      ctx.lineTo(Math.round(pointer.x) + 0.5, height);
      ctx.stroke();
      ctx.setLineDash([]);

      for (const track of tracks) {
        const y = track.compiled.handle.eval(x);
        if (!Number.isFinite(y) || y < yMin || y > yMax) continue;
        ctx.fillStyle = track.color;
        ctx.beginPath();
        ctx.arc(toX(x), toY(y), 4.5, 0, Math.PI * 2);
        ctx.fill();
        ctx.strokeStyle = "#0a0b0f";
        ctx.lineWidth = 1.5;
        ctx.stroke();
      }
    }
  }, [tracks, view, time]);

  useEffect(() => {
    draw();
  }, [draw]);

  const viewRef = useRef(view);
  viewRef.current = view;
  const fittedRef = useRef(false);

  useEffect(() => {
    const wrap = wrapRef.current;
    if (!wrap) return;

    const observer = new ResizeObserver(() => {
      const width = wrap.clientWidth;
      const height = wrap.clientHeight;
      if (width === 0 || height === 0) return;

      const current = viewRef.current;
      const previous = sizeRef.current;
      const centreX = (current.xMin + current.xMax) / 2;
      const centreY = (current.yMin + current.yMax) / 2;

      if (!fittedRef.current) {
        fittedRef.current = true;
        const spanX = current.xMax - current.xMin;
        const spanY = (spanX * height) / width;
        onView({
          xMin: current.xMin,
          xMax: current.xMax,
          yMin: centreY - spanY / 2,
          yMax: centreY + spanY / 2,
        });
        return;
      }

      if (previous.width !== width || previous.height !== height) {
        const perPixelX = (current.xMax - current.xMin) / previous.width;
        const perPixelY = (current.yMax - current.yMin) / previous.height;
        const spanX = perPixelX * width;
        const spanY = perPixelY * height;
        onView({
          xMin: centreX - spanX / 2,
          xMax: centreX + spanX / 2,
          yMin: centreY - spanY / 2,
          yMax: centreY + spanY / 2,
        });
        return;
      }

      draw();
    });

    observer.observe(wrap);
    return () => observer.disconnect();
  }, [draw, onView]);

  const updateReadout = useCallback(
    (px: number) => {
      const wrap = wrapRef.current;
      if (!wrap) return;
      const width = wrap.clientWidth;
      const x = view.xMin + (px / width) * (view.xMax - view.xMin);
      setReadout({
        x,
        values: tracks.map((track) => ({
          label: track.label,
          color: track.color,
          y: track.compiled.handle.eval(x),
        })),
      });
    },
    [tracks, view],
  );

  const onWheel = (event: React.WheelEvent<HTMLCanvasElement>) => {
    const wrap = wrapRef.current;
    if (!wrap) return;
    const rect = wrap.getBoundingClientRect();
    const px = event.clientX - rect.left;
    const py = event.clientY - rect.top;
    const factor = Math.exp(event.deltaY * 0.0016);

    const x = view.xMin + (px / rect.width) * (view.xMax - view.xMin);
    const y = view.yMax - (py / rect.height) * (view.yMax - view.yMin);

    const lockX = event.shiftKey;
    const lockY = event.altKey;

    onView({
      xMin: lockY ? view.xMin : x + (view.xMin - x) * factor,
      xMax: lockY ? view.xMax : x + (view.xMax - x) * factor,
      yMin: lockX ? view.yMin : y + (view.yMin - y) * factor,
      yMax: lockX ? view.yMax : y + (view.yMax - y) * factor,
    });
  };

  const onPointerDown = (event: React.PointerEvent<HTMLCanvasElement>) => {
    (event.target as HTMLCanvasElement).setPointerCapture(event.pointerId);
    dragRef.current = { x: event.clientX, y: event.clientY, view };
  };

  const onPointerMove = (event: React.PointerEvent<HTMLCanvasElement>) => {
    const wrap = wrapRef.current;
    if (!wrap) return;
    const rect = wrap.getBoundingClientRect();
    pointerRef.current = { x: event.clientX - rect.left, y: event.clientY - rect.top };

    const drag = dragRef.current;
    if (drag) {
      const dx = ((event.clientX - drag.x) / rect.width) * (drag.view.xMax - drag.view.xMin);
      const dy = ((event.clientY - drag.y) / rect.height) * (drag.view.yMax - drag.view.yMin);
      onView({
        xMin: drag.view.xMin - dx,
        xMax: drag.view.xMax - dx,
        yMin: drag.view.yMin + dy,
        yMax: drag.view.yMax + dy,
      });
      return;
    }

    updateReadout(event.clientX - rect.left);
    draw();
  };

  const endDrag = (event: React.PointerEvent<HTMLCanvasElement>) => {
    (event.target as HTMLCanvasElement).releasePointerCapture(event.pointerId);
    dragRef.current = null;
  };

  const onLeave = () => {
    pointerRef.current = null;
    setReadout(null);
    draw();
  };

  return (
    <div className="plot" ref={wrapRef}>
      <canvas
        ref={canvasRef}
        onWheel={onWheel}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={endDrag}
        onPointerCancel={endDrag}
        onPointerLeave={onLeave}
        onDoubleClick={() => {
          const wrap = wrapRef.current;
          const spanY = wrap ? (20 * wrap.clientHeight) / wrap.clientWidth : 13;
          onView({ xMin: -10, xMax: 10, yMin: -spanY / 2, yMax: spanY / 2 });
        }}
      />
      {readout && (
        <div className="readout">
          <div className="readout-x">x = {readout.x.toFixed(4)}</div>
          {readout.values.map((value) => (
            <div className="readout-row" key={value.label}>
              <span className="dot" style={{ background: value.color }} />
              <span className="readout-label">{value.label}</span>
              <span className="readout-value">
                {Number.isFinite(value.y) ? value.y.toFixed(4) : "undefined"}
              </span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

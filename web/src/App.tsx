import { useCallback, useEffect, useState } from "react";
import Chaos, { PRESETS } from "./components/Chaos";
import Functions from "./components/Functions";
import { boot } from "./lib/engine";

type Module = "functions" | "chaos";

export type Route = { module: Module; preset: string; twin: boolean; warm: number };

const FALLBACK: Route = { module: "functions", preset: "lorenz", twin: false, warm: 0 };

function parseHash(): Route {
  const raw = decodeURIComponent(window.location.hash.replace(/^#\/?/, "")).trim();
  if (!raw) return FALLBACK;

  const [head, ...rest] = raw.split("/");
  if (head === "functions") {
    return { module: "functions", preset: rest.join("/"), twin: false, warm: 0 };
  }
  if (head !== "chaos") return FALLBACK;

  let spec = rest.join("/");
  let warm = 0;

  const at = spec.lastIndexOf("@");
  if (at >= 0) {
    warm = Math.min(Math.max(Number(spec.slice(at + 1)) || 0, 0), 600);
    spec = spec.slice(0, at);
  }

  const twin = spec.endsWith("+twin");
  const preset = twin ? spec.slice(0, -"+twin".length) : spec;
  const known = PRESETS.some((entry) => entry.id === preset);

  return { module: "chaos", preset: known ? preset : "lorenz", twin, warm };
}

function hashFor(route: Route): string {
  if (route.module !== "chaos") {
    return route.preset === "algebra" ? "#functions/algebra" : "#functions";
  }
  const warm = route.warm > 0 ? `@${route.warm}` : "";
  return `#chaos/${route.preset}${route.twin ? "+twin" : ""}${warm}`;
}

const HINTS: Record<Module, string> = {
  functions: "Drag to pan · Scroll to zoom · Double-click to reset",
  chaos: "Drag to rotate · every trajectory is integrated live",
};

export default function App() {
  const [ready, setReady] = useState(false);
  const [route, setRoute] = useState<Route>(() => parseHash());

  useEffect(() => {
    boot().then(() => setReady(true));
  }, []);

  useEffect(() => {
    const target = hashFor(route);
    if (window.location.hash !== target) {
      window.history.replaceState(null, "", target);
    }
  }, [route]);

  useEffect(() => {
    const onHashChange = () => setRoute(parseHash());
    window.addEventListener("hashchange", onHashChange);
    return () => window.removeEventListener("hashchange", onHashChange);
  }, []);

  const setPreset = useCallback((preset: string) => {
    setRoute((current) => ({ ...current, preset }));
  }, []);

  const setTwin = useCallback((twin: boolean) => {
    setRoute((current) => ({ ...current, twin }));
  }, []);

  return (
    <div className="shell">
      <header className="topbar">
        <div className="brand">
          <span className="mark" />
          <span className="name">kosmos</span>
        </div>
        <nav className="modules">
          <button
            className={route.module === "functions" ? "module active" : "module"}
            onClick={() => setRoute({ ...route, module: "functions" })}
          >
            Functions
          </button>
          <button
            className={route.module === "chaos" ? "module active" : "module"}
            onClick={() => setRoute({ ...route, module: "chaos" })}
          >
            Chaos
          </button>
          <button className="module" disabled title="coming next">
            Fields
          </button>
          <button className="module" disabled title="coming next">
            Life
          </button>
        </nav>
        <div className="hint">{HINTS[route.module]}</div>
      </header>

      {ready ? (
        route.module === "functions" ? (
          <Functions showcase={route.preset === "algebra"} />
        ) : (
          <Chaos
            presetId={route.preset}
            onPreset={setPreset}
            twin={route.twin}
            onTwin={setTwin}
            warm={route.warm}
          />
        )
      ) : (
        <div className="loading">Loading the engine …</div>
      )}
    </div>
  );
}

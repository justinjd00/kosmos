import { useEffect, useState } from "react";
import Chaos from "./components/Chaos";
import Functions from "./components/Functions";
import { boot } from "./lib/engine";

type Module = "functions" | "chaos";

const HINTS: Record<Module, string> = {
  functions: "Drag to pan · Scroll to zoom · Double-click to reset",
  chaos: "Drag to rotate · every trajectory is integrated live",
};

export default function App() {
  const [ready, setReady] = useState(false);
  const [module, setModule] = useState<Module>("functions");
  const [preset, setPreset] = useState("lorenz");

  useEffect(() => {
    boot().then(() => setReady(true));
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
            className={module === "functions" ? "module active" : "module"}
            onClick={() => setModule("functions")}
          >
            Functions
          </button>
          <button
            className={module === "chaos" ? "module active" : "module"}
            onClick={() => setModule("chaos")}
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
        <div className="hint">{HINTS[module]}</div>
      </header>

      {ready ? (
        module === "functions" ? (
          <Functions />
        ) : (
          <Chaos presetId={preset} onPreset={setPreset} />
        )
      ) : (
        <div className="loading">Loading the engine …</div>
      )}
    </div>
  );
}

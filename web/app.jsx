const { useEffect, useMemo, useRef, useState } = React;

const FLOW_FREE_COLORS = [
  '#e74c3c',
  '#3498db',
  '#2ecc71',
  '#f1c40f',
  '#e67e22',
  '#9b59b6',
  '#1abc9c',
  '#e91e63',
  '#00bcd4',
  '#cddc39',
  '#3f51b5',
  '#ffc107',
  '#795548',
  '#9e9e9e',
  '#ff5722',
  '#607d8b',
];

function colorFor(n) {
  if (!n || n <= 0) return '';
  return FLOW_FREE_COLORS[(n - 1) % FLOW_FREE_COLORS.length];
}

function contrast(color) {
  if (!color) return '#000';
  const c = color.substring(1);
  const rgb = parseInt(c, 16);
  const r = (rgb >> 16) & 0xff;
  const g = (rgb >> 8) & 0xff;
  const b = rgb & 0xff;
  const yiq = (r * 299 + g * 587 + b * 114) / 1000;
  return yiq >= 128 ? '#000' : '#fff';
}

function createBoard(h, w) {
  return Array.from({ length: h }, () => Array(w).fill(0));
}

function App() {
  // Visual constants
  const CELL = 44; // px
  const GAP = 4; // px

  // Grid + board state
  const [H, setH] = useState(10);
  const [W, setW] = useState(10);
  const [board, setBoard] = useState(() => createBoard(10, 10));
  const [currentNumber, setCurrentNumber] = useState(1);

  // Solver settings
  const [allowZigzag, setAllowZigzag] = useState(false);
  const [useVcut, setUseVcut] = useState(false);
  const [useTable, setUseTable] = useState(false);
  const [useDiagonals, setUseDiagonals] = useState(true);

  // Status + solution
  const [status, setStatus] = useState('');
  const [isError, setIsError] = useState(false);
  const [solution, setSolution] = useState(null); // { edges, colors, nodes, elapsed_ms }

  // Recompute SVG size
  const svgSize = useMemo(() => ({
    width: W * (CELL + GAP) + GAP,
    height: H * (CELL + GAP) + GAP,
  }), [W, H, CELL, GAP]);

  // Keyboard: left/right changes currentNumber
  useEffect(() => {
    function onKey(e) {
      if (e.key === 'ArrowLeft' || e.key === 'a') {
        e.preventDefault();
        setCurrentNumber((n) => Math.max(1, n - 1));
      }
      if (e.key === 'ArrowRight' || e.key === 'd') {
        e.preventDefault();
        setCurrentNumber((n) => Math.max(1, n + 1));
      }
    }
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  function resetBoard(newH, newW) {
    setSolution(null);
    setBoard(createBoard(newH, newW));
  }

  function toggleCell(r, c) {
    setIsError(false);
    setStatus('');
    setBoard((prev) => {
      const next = prev.map((row) => row.slice());
      const existing = next[r][c];
      if (existing !== 0) {
        next[r][c] = 0;
        return next;
      }
      let count = 0;
      for (let rr = 0; rr < H; rr++) {
        for (let cc = 0; cc < W; cc++) {
          if (next[rr][cc] === currentNumber) count++;
        }
      }
      if (count >= 2) {
        alert(`There are already two ${currentNumber}`);
        return prev;
      }
      next[r][c] = currentNumber;
      return next;
    });
  }

  async function handleSolve() {
    setSolution(null);
    setIsError(false);
    setStatus('Solving...');
    try {
      const payload = {
        board,
        allow_zigzag: !!allowZigzag,
        use_vcut: !!useVcut,
        use_table: !!useTable,
        use_diagonals: !!useDiagonals,
      };
      const res = await fetch('/solve', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      if (data.solved) {
        setSolution(data);
        setStatus(`Solved. Nodes: ${data.nodes}. Time: ${data.elapsed_ms} ms.`);
      } else {
        setStatus('No solution found');
      }
    } catch (e) {
      console.error(e);
      setIsError(true);
      setStatus('Error solving; see console');
      alert('Error invoking solver');
    }
  }

  function NumberChip() {
    const bg = colorFor(currentNumber) || '#ffffff';
    const circleSize = Math.round(CELL * 0.75);
    return (
      <div className="inline-flex items-center gap-2 shrink-0">
        <button
          className="px-2 py-1 rounded border border-neutral-300 bg-neutral-100 hover:bg-white text-neutral-900"
          onClick={() => setCurrentNumber((n) => Math.max(1, n - 1))}
          aria-label="Decrement"
          title="Decrease number (Left arrow)"
        >
          ◀
        </button>
        <div
          className="flex items-center justify-center rounded-full border border-neutral-300"
          style={{
            width: circleSize,
            height: circleSize,
            background: bg,
            color: '#ffffff',
            userSelect: 'none',
          }}
          title="Use Left/Right arrow keys to change"
        >
          <span className="font-bold">{currentNumber}</span>
        </div>
        <button
          className="px-2 py-1 rounded border border-neutral-300 bg-neutral-100 hover:bg-white text-neutral-900"
          onClick={() => setCurrentNumber((n) => Math.max(1, n + 1))}
          aria-label="Increment"
          title="Increase number (Right arrow)"
        >
          ▶
        </button>
      </div>
    );
  }

  return (
    <div className="min-h-screen">
      <header className="border-b border-neutral-200 bg-white">
        <div className="mx-auto max-w-5xl p-4">
          <h1 className="text-xl font-semibold text-neutral-900">Flow Solver</h1>
          <p className="text-sm text-neutral-600">Generator and backtracking solver for Flow puzzles.</p>
        </div>
      </header>
      <main className="mx-auto max-w-5xl grid md:grid-cols-[320px_1fr] gap-4 p-4">
        {/* Left column: Grid size + Number picker + Solver options */}
        <section className="space-y-4">
          <div className="border border-neutral-200 bg-white p-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="block text-sm text-neutral-400">Height</label>
                <input
                  type="number"
                  min={1}
                  value={H}
                  onChange={(e) => {
                    const v = Math.max(1, Number(e.target.value) || 1);
                    setH(v);
                    resetBoard(v, W);
                  }}
                  className="w-28 rounded border border-neutral-300 bg-white px-2 py-1 text-sm outline-none focus:ring-2 focus:ring-neutral-300"
                />
              </div>
              <div className="space-y-2">
                <label className="block text-sm text-neutral-400">Width</label>
                <input
                  type="number"
                  min={1}
                  value={W}
                  onChange={(e) => {
                    const v = Math.max(1, Number(e.target.value) || 1);
                    setW(v);
                    resetBoard(H, v);
                  }}
                  className="w-28 rounded border border-neutral-300 bg-white px-2 py-1 text-sm outline-none focus:ring-2 focus:ring-neutral-300"
                />
              </div>
            </div>
          </div>

          <div className="border border-neutral-200 bg-white p-4">
            <div className="flex items-center justify-center">
              <NumberChip />
            </div>
          </div>

          {/* Solver Options (under grid/number) */}
          <div className="border border-neutral-200 bg-white p-4 space-y-3">
            <div className="text-sm font-medium text-neutral-800">Solver Options</div>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
              <label className="inline-flex items-center gap-2 text-sm text-neutral-800"><input type="checkbox" className="accent-neutral-600" checked={allowZigzag} onChange={(e) => setAllowZigzag(e.target.checked)} /> Allow Zigzags </label>
              <label className="inline-flex items-center gap-2 text-sm text-neutral-800"><input type="checkbox" className="accent-neutral-600" checked={useVcut} onChange={(e) => setUseVcut(e.target.checked)} /> VCut Pruning </label>
              <label className="inline-flex items-center gap-2 text-sm text-neutral-800"><input type="checkbox" className="accent-neutral-600" checked={useTable} onChange={(e) => setUseTable(e.target.checked)} /> Use Cache (beta) </label>
              <label className="inline-flex items-center gap-2 text-sm text-neutral-800"><input type="checkbox" className="accent-neutral-600" checked={useDiagonals} onChange={(e) => setUseDiagonals(e.target.checked)} /> Diagonal Pruning </label>
            </div>
            <div className="flex items-center gap-3 pt-2">
              <button onClick={handleSolve} className="inline-flex items-center justify-center rounded border border-neutral-300 bg-neutral-100 px-3 py-2 text-sm text-black hover:bg-white">Solve</button>
              <span className={`text-sm ${isError ? 'text-red-600' : 'text-neutral-600'}`}>{status}</span>
            </div>
          </div>
        </section>

        {/* Right column: Board only */}
        <section className="space-y-4 relative">
          <div className="relative inline-block border border-neutral-200 bg-white p-1">
            <div
              className="grid relative"
              style={{
                gridTemplateColumns: `repeat(${W}, ${CELL}px)`,
                gap: `${GAP}px`,
                background: '#e5e7eb',
              }}
            >
              {Array.from({ length: H }).map((_, r) => (
                Array.from({ length: W }).map((__, c) => {
                  const val = board[r][c];
                  const isEndpoint = val > 0;
                  const color = colorFor(val);
                  const circleSize = Math.round(CELL * 0.75);
                  return (
                    <div
                      key={`${r}-${c}`}
                      className="relative flex items-center justify-center cursor-pointer select-none"
                      style={{ 
                        width: CELL, 
                        height: CELL, 
                        background: '#ffffff',
                        zIndex: 1,
                      }}
                      onClick={() => toggleCell(r, c)}
                    >
                      {isEndpoint && (
                        <>
                          <div
                            className="rounded-full"
                            style={{
                              width: circleSize,
                              height: circleSize,
                              background: color,
                              zIndex: 3,
                            }}
                          />
                          <div
                            className="absolute inset-0 flex items-center justify-center text-sm font-bold pointer-events-none text-white"
                            style={{ zIndex: 10 }}
                          >
                            {val}
                          </div>
                        </>
                      )}
                    </div>
                  );
                })
              ))}
            </div>
            {/* Overlay (lines) above board cells but under endpoints */}
            <svg
              className="pointer-events-none absolute top-1 left-1"
              width={svgSize.width}
              height={svgSize.height}
              style={{ zIndex: 2 }}
            >
              {solution && solution.edges && solution.colors && (
                <SolutionLines H={H} W={W} CELL={CELL} GAP={GAP} edges={solution.edges} colors={solution.colors} />
              )}
            </svg>
          </div>
        </section>
      </main>
    </div>
  );
}

function SolutionLines({ H, W, CELL, GAP, edges, colors }) {
  const toXY = (r, c) => {
    const x = GAP + c * (CELL + GAP) + CELL / 2;
    const y = GAP + r * (CELL + GAP) + CELL / 2;
    return [x, y];
  };
  const lines = [];
  const { h, w, down, right } = edges;
  for (let r = 0; r < h; r++) {
    for (let c = 0; c < w; c++) {
      const u = r * w + c;
      if (right[u] && c + 1 < w) {
        const [x1, y1] = toXY(r, c);
        const [x2, y2] = toXY(r, c + 1);
        const col = colorFor(colors[u]);
        lines.push(
          <line key={`r-${u}`} x1={x1} y1={y1} x2={x2} y2={y2} stroke={col} strokeWidth={8} strokeLinecap="round" />
        );
      }
      if (down[u] && r + 1 < h) {
        const [x1, y1] = toXY(r, c);
        const [x2, y2] = toXY(r + 1, c);
        const col = colorFor(colors[u]);
        lines.push(
          <line key={`d-${u}`} x1={x1} y1={y1} x2={x2} y2={y2} stroke={col} strokeWidth={8} strokeLinecap="round" />
        );
      }
    }
  }
  return <>{lines}</>;
}

const root = ReactDOM.createRoot(document.getElementById('root'));
root.render(<App />);

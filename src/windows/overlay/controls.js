// noinspection JSUnresolvedReference

// Auto-injected caption-control runtime (vanilla, framework-agnostic).
// Builds the top-right minimize/maximize/close buttons, fetches pixel-perfect
//
// glyph paths from the Rust backend, and tracks window state. Windows only:
// this script is only ever injected on Windows targets.
(function () {
  "use strict";
  if (window.top !== window.self) return;

  // noinspection JSUnresolvedReference
  const TAURI = window.__TAURI_INTERNALS__;

  if (!TAURI || document.getElementById("tbo-controls")) return;

  const PLUGIN = "window-controls";
  const GLYPH = { min: "\uE921", max: "\uE922", restore: "\uE923", close: "\uE8BB" };
  const CLOSE_HOVER = "#C42B1C";
  const CLOSE_PRESSED = "#C42B1CCC";

  const DEFAULTS = {
    light: { default: "transparent", symbol: "#000000", hover: "#0000000F", pressed: "#0000000A", inactive: "transparent" },
    dark: { default: "transparent", symbol: "#FFFFFF", hover: "#FFFFFF0F", pressed: "#FFFFFF0A", inactive: "transparent" },
  };

  function invoke(cmd, args) {
    return TAURI.invoke("plugin:" + PLUGIN + "|" + cmd, args || {});
  }

  function merge(base, over) {
    const out = {};
    for (const k in base) out[k] = (over && over[k]) || base[k];
    return out;
  }

  function vars(c) {
    return (
      "--tbo-symbol:" + c.symbol + ";--tbo-default:" + c.default +
      ";--tbo-hover:" + c.hover + ";--tbo-pressed:" + c.pressed +
      ";--tbo-inactive:" + c.inactive + ";"
    );
  }

  function injectStyle(height, light, dark) {
    const css =
      ":root{--title-bar-height:" + height + "px;--title-bar-controls-width:138px;" + vars(light) + "}" +
      "@media (prefers-color-scheme:dark){:root{" + vars(dark) + "}}" +
      "#tbo-controls{position:fixed;top:0;right:0;display:flex;z-index:2147483647;-webkit-user-select:none;user-select:none;}" +
      "#tbo-controls .tbo-btn{width:46px;height:var(--title-bar-height);display:grid;place-items:center;" +
        "border:0;margin:0;padding:0;background:var(--tbo-default);color:var(--tbo-symbol);" +
        "cursor:default;transition:background-color .15s,color .1s;}" +
      "#tbo-controls .tbo-btn:hover{background:var(--tbo-hover);}" +
      "#tbo-controls .tbo-btn:hover:active{background:var(--tbo-pressed);transition:none;}" +
      "#tbo-controls .tbo-close:hover{background:" + CLOSE_HOVER + ";color:#fff;}" +
      "#tbo-controls .tbo-close:hover:active{background:" + CLOSE_PRESSED + ";color:#fff;transition:none;}" +
      // maximize hover/press come from the snap overlay (events -> classes), since
      // the native overlay sits over the button and swallows DOM: hover/:active.
      "#tbo-controls .tbo-btn.tbo-hover{background:var(--tbo-hover);}" +
      "#tbo-controls .tbo-btn.tbo-active{background:var(--tbo-pressed);transition:none;}" +
      "#tbo-controls .tbo-btn[disabled]{pointer-events:none;}" +
      "#tbo-controls .tbo-btn[disabled] .tbo-glyph{opacity:.36;}" +
      "#tbo-controls[data-active=\"false\"] .tbo-btn:not(:hover):not(.tbo-hover) .tbo-glyph{opacity:.32;}" +
      "#tbo-controls .tbo-glyph{width:10px;height:10px;line-height:0;}" +
      "#tbo-controls .tbo-glyph svg{display:block;width:100%;height:100%;}" +
      "#tbo-controls .tbo-min .tbo-glyph svg{shape-rendering:crispEdges;}";

    const style = document.createElement("style");
    style.id = "tbo-style";
    style.textContent = css;
    document.head.appendChild(style);
  }

  // Backend normalizes every glyph into this viewBox; fonts are nonzero-winding
  // (SVG default), so no fill-rule is needed. Paths are fetched once and reused.
  const SVGNS = "http://www.w3.org/2000/svg";
  const VIEW_BOX = "0 0 100 100";
  const PATHS = {};

  function glyphPath(ch) {
    if (ch in PATHS) return Promise.resolve(PATHS[ch]);
    return invoke("get_glyph_path", { text: ch })
      .then(function (d) {
        PATHS[ch] = d || "";
        return PATHS[ch];
      })
      .catch(function () {
        return "";
      });
  }

  async function setGlyph(span, ch) {
    const d = await glyphPath(ch);
    if (!d) return;

    const svg = document.createElementNS(SVGNS, "svg");
    svg.setAttribute("viewBox", VIEW_BOX);
    svg.setAttribute("preserveAspectRatio", "xMidYMid meet");

    const path = document.createElementNS(SVGNS, "path");
    path.setAttribute("d", d);
    path.setAttribute("fill", "currentColor");
    svg.appendChild(path);

    span.textContent = "";
    span.appendChild(svg);
  }

  function button(kind, action, label) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "tbo-btn tbo-" + kind;
    btn.setAttribute("aria-label", label);

    const glyph = document.createElement("span");
    glyph.className = "tbo-glyph";
    btn.appendChild(glyph);

    btn.addEventListener("mousedown", function (e) {
      e.preventDefault();
      e.stopPropagation();
    });

    btn.addEventListener("click", function () {
      invoke("window_command", { action: action });
    });

    btn._glyph = glyph;
    return btn;
  }

  function applyState(bar, btns, s) {
    bar.setAttribute("data-active", s.focused ? "true" : "false");

    btns.min.disabled = !s.minimizable;
    btns.max.disabled = !s.maximizable;
    btns.close.disabled = !s.closable;
    btns.max.setAttribute("aria-label", s.maximized ? "Restore" : "Maximize");

    void setGlyph(btns.max._glyph, s.maximized ? GLYPH.restore : GLYPH.max);
  }

  // Events are scoped to THIS webview. A bare "Any" target receives every
  // window's events (and the Rust side now emits per-window via emit_to), so we
  // must filter to our own label — otherwise one window's maximize hover would
  // light up every window's button. Falls back to "Any" only if the label is
  // somehow unavailable.
  function selfTarget() {
    const meta = TAURI.metadata || {};
    const label =
      (meta.currentWebview && meta.currentWebview.label) ||
      (meta.currentWindow && meta.currentWindow.label);
    return label ? { kind: "WebviewWindow", label: label } : { kind: "Any" };
  }

  function listen(event, handler) {
    try {
      const cb = TAURI.transformCallback(function (e) {
        handler(e);
      });

      TAURI.invoke("plugin:event|listen", { event: event, target: selfTarget(), handler: cb }).catch(
        function () {}
      );

    } catch (_) {}
  }

  function start() {
    const height = window.__TBO_HEIGHT__ || 32;
    const colors = window.__TBO_COLORS__ || {};

    injectStyle(height, merge(DEFAULTS.light, colors.light), merge(DEFAULTS.dark, colors.dark));

    const bar = document.createElement("div");
    bar.id = "tbo-controls";
    bar.setAttribute("data-active", "true");

    const btns = {
      min: button("min", "minimize", "Minimize"),
      max: button("max", "toggle-maximize", "Maximize"),
      close: button("close", "close", "Close"),
    };

    bar.appendChild(btns.min);
    bar.appendChild(btns.max);
    bar.appendChild(btns.close);
    document.body.appendChild(bar);

    void setGlyph(btns.min._glyph, GLYPH.min);
    void setGlyph(btns.close._glyph, GLYPH.close);

    // The native snap overlay is only installed once the window is actually
    // maximizable. A non-maximizable window must not get the hit-test child
    // (it would otherwise show the snap flyout and hover over a dead button).
    let snapInstalled = false;
    function ensureSnap(maximizable) {
      if (!maximizable || snapInstalled) return;
      snapInstalled = true;
      invoke("enable_snap", { height: height }).catch(function () {
        snapInstalled = false;
      });
    }

    function refresh() {
      invoke("window_state")
        .then(function (s) {
          applyState(bar, btns, s);
          ensureSnap(s.maximizable);
        })
        .catch(function () {});
    }

    refresh();
    listen("tauri://resize", refresh);

    listen("tauri://focus", function () {
      bar.setAttribute("data-active", "true");
    });
    listen("tauri://blur", function () {
      bar.setAttribute("data-active", "false");
    });

    // Native snap-layout overlay over the Maximize button. It returns
    // HTMAXBUTTON (so Windows shows the snap flyout) and bridges hover/press/
    // click back as events, because it covers the DOM button. Installed lazily
    // by ensureSnap() once the window is confirmed maximizable. The disabled
    // guards below keep a stale overlay (window turned non-maximizable after
    // install) from lighting up or actioning a dead button.
    listen("window-controls://snap-enter", function () {
      if (btns.max.disabled) return;
      btns.max.classList.add("tbo-hover");
    });
    listen("window-controls://snap-leave", function () {
      btns.max.classList.remove("tbo-hover", "tbo-active");
    });
    listen("window-controls://snap-down", function () {
      if (btns.max.disabled) return;
      btns.max.classList.add("tbo-active");
    });
    listen("window-controls://snap-up", function () {
      btns.max.classList.remove("tbo-active");
    });
    listen("window-controls://snap-click", function () {
      if (btns.max.disabled) return;
      invoke("window_command", { action: "toggle-maximize" });
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", start);
  } else {
    start();
  }
})();

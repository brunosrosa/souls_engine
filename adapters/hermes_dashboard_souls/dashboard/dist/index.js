/**
 * Souls Engine v7 Dashboard — Hermes Agent Web Plugin Component
 * Packaged as an IIFE compatible with window.__HERMES_PLUGIN_SDK__ and React container.
 */
(function () {
  const PLUGIN_ID = "souls_dashboard";
  const TARGET_URL = "http://127.0.0.1:9123/dashboard";

  function render(container, sdk) {
    if (!container) return;

    // Limpa o container
    container.innerHTML = "";

    // Wrapper principal
    const wrapper = document.createElement("div");
    wrapper.style.width = "100%";
    wrapper.style.height = "100%";
    wrapper.style.minHeight = "calc(100vh - 64px)";
    wrapper.style.display = "flex";
    wrapper.style.flexDirection = "column";
    wrapper.style.background = "#060911";
    wrapper.style.position = "relative";

    // Header informativo do Hermes
    const header = document.createElement("div");
    header.style.display = "flex";
    header.style.alignItems = "center";
    header.style.justifyContent = "space-between";
    header.style.padding = "8px 16px";
    header.style.background = "rgba(12, 18, 32, 0.9)";
    header.style.borderBottom = "1px solid rgba(255, 255, 255, 0.08)";
    header.style.fontSize = "12px";
    header.style.color = "#94a3b8";

    header.innerHTML = `
      <div style="display:flex;align-items:center;gap:8px;">
        <span style="display:inline-block;width:8px;height:8px;border-radius:50%;background:#10b981;box-shadow:0 0 8px #10b981;"></span>
        <strong style="color:#f1f5f9;">Souls Engine v7 Daemon</strong>
        <span style="font-family:monospace;color:#06b6d4;">(127.0.0.1:9123)</span>
      </div>
      <div>
        <a href="${TARGET_URL}" target="_blank" rel="noopener noreferrer" style="color:#06b6d4;text-decoration:none;font-weight:600;">
          Abrir no Navegador Externo ↗
        </a>
      </div>
    `;

    // IFrame incorporado
    const iframe = document.createElement("iframe");
    iframe.src = TARGET_URL;
    iframe.style.width = "100%";
    iframe.style.flex = "1";
    iframe.style.border = "none";
    iframe.style.background = "#060911";
    iframe.title = "Souls Engine Dashboard";

    // Overlay de erro/offline em caso de falha de conexão
    iframe.onerror = function () {
      wrapper.innerHTML = `
        <div style="padding:40px;text-align:center;color:#ef4444;font-family:sans-serif;">
          <h2>Souls Engine Daemon Offline</h2>
          <p style="color:#94a3b8;margin-top:8px;">Certifique-se de que o souls_server.exe está em execução em 127.0.0.1:9123.</p>
        </div>
      `;
    };

    wrapper.appendChild(header);
    wrapper.appendChild(iframe);
    container.appendChild(wrapper);
  }

  // Registro no SDK global do Hermes se disponível
  if (window.__HERMES_PLUGIN_SDK__ && window.__HERMES_PLUGIN_SDK__.registerPlugin) {
    window.__HERMES_PLUGIN_SDK__.registerPlugin(PLUGIN_ID, {
      name: "Souls Engine",
      render: render,
    });
  } else {
    // Exposição global para montagem dinâmica
    window.SoulsDashboardPlugin = { render: render };
  }
})();

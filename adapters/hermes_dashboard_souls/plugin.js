/**
 * Souls Engine v7 — Official Hermes Desktop Integration Plugin
 * Integrates the bare-metal Souls Engine dashboard natively into Hermes Desktop:
 * 1. Left Sidebar Navigation button/tab ('sidebar.nav')
 * 2. Dedicated Workspace Page ('routes' at /souls)
 * 3. Status Bar pill ('statusBar.right')
 * 4. Command Palette action (Ctrl+K -> 'Souls Engine')
 * 5. Keyboard shortcut (Ctrl+Alt+S / Cmd+Alt+S)
 * 6. Settings Page in Settings ▸ Plugins ▸ Souls Engine
 */

import {
  host,
  ROUTES_AREA,
  SIDEBAR_NAV_AREA,
  STATUSBAR_AREAS,
  PALETTE_AREA,
  KEYBINDS_AREA,
} from '@hermes/plugin-sdk';
import { jsx, jsxs } from 'react/jsx-runtime';
import * as React from 'react';

const DASHBOARD_URL = 'http://127.0.0.1:9123/';
const SOULS_ROUTE = '/souls';

// Robust fallbacks for area IDs across different Hermes Desktop releases
const AREA_ROUTES = ROUTES_AREA || 'routes';
const AREA_SIDEBAR_NAV = SIDEBAR_NAV_AREA || 'sidebar.nav';
const AREA_STATUSBAR_RIGHT = STATUSBAR_AREAS?.right || 'statusBar.right';
const AREA_PALETTE = PALETTE_AREA || 'palette';
const AREA_KEYBINDS = KEYBINDS_AREA || 'keybinds';

/**
 * Full page view rendered inside Hermes Desktop at route `/souls`.
 */
function SoulsDashboardPageView({ ctx }) {
  const [iframeKey, setIframeKey] = React.useState(0);
  const [isRefreshing, setIsRefreshing] = React.useState(false);

  const handleRefresh = () => {
    setIsRefreshing(true);
    setIframeKey((prev) => prev + 1);
    setTimeout(() => setIsRefreshing(false), 600);
  };

  const handleOpenExternal = () => {
    if (ctx?.os?.openExternal) {
      ctx.os.openExternal(DASHBOARD_URL);
    } else if (typeof window !== 'undefined') {
      window.open(DASHBOARD_URL, '_blank');
    }
  };

  return jsxs('div', {
    style: {
      width: '100%',
      height: '100%',
      minHeight: '100%',
      display: 'flex',
      flexDirection: 'column',
      background: '#060911',
      position: 'relative',
      overflow: 'hidden',
    },
    children: [
      // Top Navigation / Header Bar inside Hermes Desktop
      jsxs('div', {
        style: {
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '6px 14px',
          background: 'rgba(12, 18, 32, 0.98)',
          borderBottom: '1px solid rgba(6, 182, 212, 0.25)',
          fontSize: '12px',
          color: '#94a3b8',
          flexShrink: 0,
          zIndex: 10,
        },
        children: [
          // Left: Identity & Live Badges
          jsxs('div', {
            style: { display: 'flex', alignItems: 'center', gap: '10px' },
            children: [
              jsx('span', {
                style: {
                  display: 'inline-block',
                  width: '8px',
                  height: '8px',
                  borderRadius: '50%',
                  background: '#10b981',
                  boxShadow: '0 0 8px #10b981',
                  animation: 'pulse 2s infinite',
                },
              }),
              jsx('strong', {
                style: {
                  color: '#f1f5f9',
                  fontFamily: 'system-ui, -apple-system, sans-serif',
                  fontWeight: 600,
                  fontSize: '12px',
                },
                children: '⚡ Souls Engine v7 Bare-Metal Daemon',
              }),
              jsx('span', {
                style: {
                  fontFamily: 'monospace',
                  fontSize: '11px',
                  color: '#06b6d4',
                  background: 'rgba(6, 182, 212, 0.1)',
                  padding: '2px 6px',
                  borderRadius: '4px',
                  border: '1px solid rgba(6, 182, 212, 0.25)',
                },
                children: '127.0.0.1:9123',
              }),
              jsx('span', {
                style: {
                  fontSize: '10px',
                  color: '#10b981',
                  background: 'rgba(16, 185, 129, 0.1)',
                  padding: '2px 6px',
                  borderRadius: '4px',
                  border: '1px solid rgba(16, 185, 129, 0.25)',
                },
                children: 'RTX 2060m • WDDM Safe (5.2GB)',
              }),
            ],
          }),

          // Right: Action Buttons
          jsxs('div', {
            style: { display: 'flex', gap: '8px', alignItems: 'center' },
            children: [
              jsx('button', {
                onClick: handleRefresh,
                title: 'Recarregar painel local',
                style: {
                  background: 'rgba(255, 255, 255, 0.05)',
                  border: '1px solid rgba(255, 255, 255, 0.12)',
                  color: isRefreshing ? '#06b6d4' : '#cbd5e1',
                  borderRadius: '5px',
                  padding: '3px 8px',
                  fontSize: '11px',
                  cursor: 'pointer',
                  fontWeight: 500,
                  display: 'flex',
                  alignItems: 'center',
                  gap: '4px',
                  transition: 'all 0.2s',
                },
                children: isRefreshing ? 'Recarregando...' : '🔄 Recarregar',
              }),
              jsx('button', {
                onClick: handleOpenExternal,
                title: 'Abrir em aba de navegador externo',
                style: {
                  background: 'rgba(6, 182, 212, 0.15)',
                  border: '1px solid rgba(6, 182, 212, 0.4)',
                  color: '#06b6d4',
                  borderRadius: '5px',
                  padding: '3px 9px',
                  fontSize: '11px',
                  cursor: 'pointer',
                  fontWeight: 600,
                  display: 'flex',
                  alignItems: 'center',
                  gap: '4px',
                  transition: 'all 0.2s',
                },
                children: 'Abrir no Navegador ↗',
              }),
            ],
          }),
        ],
      }),

      // The embedded Dashboard iframe
      jsx('iframe', {
        key: iframeKey,
        src: DASHBOARD_URL,
        style: {
          width: '100%',
          height: 'calc(100% - 37px)',
          flex: 1,
          border: 'none',
          background: '#060911',
        },
        title: 'Souls Engine Bare-Metal Dashboard',
      }),
    ],
  });
}

/**
 * Status bar pill rendered in `statusBar.right`.
 */
function SoulsStatusBarPill() {
  const handleClick = () => {
    if (typeof host?.navigate === 'function') {
      host.navigate(SOULS_ROUTE);
    } else if (typeof window !== 'undefined') {
      window.open(DASHBOARD_URL, '_blank');
    }
  };

  return jsx('button', {
    onClick: handleClick,
    style: {
      display: 'inline-flex',
      alignItems: 'center',
      gap: '4px',
      height: '100%',
      padding: '0 8px',
      background: 'rgba(6, 182, 212, 0.12)',
      border: 'none',
      borderLeft: '1px solid rgba(6, 182, 212, 0.25)',
      borderRight: '1px solid rgba(6, 182, 212, 0.25)',
      color: '#06b6d4',
      fontSize: '0.6875rem',
      fontFamily: 'monospace',
      fontWeight: 600,
      cursor: 'pointer',
      transition: 'background 0.2s, color 0.2s',
    },
    title: '⚡ Souls Engine: Clique para abrir o Dashboard Bare-Metal (Ctrl+Alt+S)',
    children: [
      jsx('span', {
        style: {
          display: 'inline-block',
          width: '6px',
          height: '6px',
          borderRadius: '50%',
          background: '#10b981',
          boxShadow: '0 0 6px #10b981',
        },
      }),
      jsx('span', { children: 'Souls Engine' }),
    ],
  });
}

export default {
  id: 'hermes_dashboard_souls',
  name: 'Souls Engine',
  description: 'Painel Bare-Metal, Governança, Cloud Arena e Telemetria de Silício (RTX 2060m)',
  defaultEnabled: true,

  register(ctx) {
    const navigateToSouls = () => {
      if (typeof host?.navigate === 'function') {
        host.navigate(SOULS_ROUTE);
      } else if (typeof window !== 'undefined') {
        window.open(DASHBOARD_URL, '_blank');
      }
    };

    // 1. ABA / BOTÃO NA BARRA LATERAL (SIDEBAR_NAV_AREA: 'sidebar.nav')
    // Adiciona o botão com ícone 'pulse' e rótulo 'Souls Engine' na navegação lateral do Hermes Desktop
    ctx.register({
      id: 'nav',
      area: AREA_SIDEBAR_NAV,
      order: 55,
      data: {
        codicon: 'pulse',
        label: 'Souls Engine',
        path: SOULS_ROUTE,
      },
    });

    // 2. PÁGINA COMPLETA INTEGRADA NO HERMES DESKTOP (ROUTES_AREA: 'routes')
    // Monta o dashboard bare-metal na rota /souls do Hermes Desktop
    ctx.register({
      id: 'page',
      area: AREA_ROUTES,
      title: 'Souls Engine Bare-Metal',
      data: {
        path: SOULS_ROUTE,
      },
      render: () => jsx(SoulsDashboardPageView, { ctx }),
    });

    // 3. PILL / CHIP NA BARRA DE STATUS (STATUSBAR_AREAS.right: 'statusBar.right')
    // Exibe o status bare-metal no rodapé direito do Hermes Desktop
    ctx.register({
      id: 'status-pill',
      area: AREA_STATUSBAR_RIGHT,
      order: 75,
      render: () => jsx(SoulsStatusBarPill, {}),
    });

    // 4. COMANDO NA PALETA (Ctrl+K / ⌘K -> 'Souls Engine')
    ctx.register({
      id: 'palette-open',
      area: AREA_PALETTE,
      data: {
        id: 'souls.open',
        label: 'Souls Engine: Abrir Dashboard de Silício e Governança',
        keywords: ['souls', 'gpu', 'telemetry', 'dashboard', 'silicon', 'arena', 'vram', 'pareto'],
        run: navigateToSouls,
      },
    });

    // 5. ATALHO DE TECLADO (Ctrl+Alt+S / Cmd+Alt+S)
    ctx.register({
      id: 'keybind-open',
      area: AREA_KEYBINDS,
      data: {
        id: 'souls.open',
        category: 'view',
        defaults: ['mod+alt+s'],
        label: 'Souls Engine: Abrir Dashboard',
        run: navigateToSouls,
      },
    });

    // 6. PÁGINA DEDICADA EM CONFIGURAÇÕES ▸ PLUGINS ▸ SOULS ENGINE
    ctx.registerSettingsPage?.({
      id: 'souls-engine-settings',
      title: 'Souls Engine',
      icon: 'cpu',
      render: function SoulsSettingsPane() {
        return jsxs('div', {
          style: {
            padding: '24px',
            color: '#f1f5f9',
            fontFamily: 'system-ui, -apple-system, sans-serif',
            maxWidth: '640px',
          },
          children: [
            jsx('h3', {
              style: {
                color: '#06b6d4',
                fontSize: '1.25rem',
                marginBottom: '8px',
                fontWeight: 600,
              },
              children: '⚡ Souls Engine v7 — Bare-Metal Daemon',
            }),
            jsx('p', {
              style: {
                color: '#94a3b8',
                fontSize: '0.875rem',
                lineHeight: '1.6',
                marginBottom: '16px',
              },
              children:
                'O daemon bare-metal está ativo em 127.0.0.1:9123 com teto de VRAM protegido na NVIDIA GeForce RTX 2060m (5.2GB WDDM), Two-Stage Pareto Switchboard e memória ontológica no FrankenSQLite STRICT WAL.',
            }),
            jsx('div', {
              style: {
                display: 'flex',
                gap: '12px',
                marginTop: '20px',
              },
              children: [
                jsx('button', {
                  onClick: navigateToSouls,
                  style: {
                    background: '#06b6d4',
                    color: '#060911',
                    border: 'none',
                    borderRadius: '6px',
                    padding: '8px 16px',
                    fontWeight: 700,
                    cursor: 'pointer',
                    fontSize: '0.85rem',
                    transition: 'opacity 0.2s',
                  },
                  children: 'Abrir Aba Souls Engine ⚡',
                }),
                jsx('button', {
                  onClick: () => {
                    if (ctx?.os?.openExternal) {
                      ctx.os.openExternal(DASHBOARD_URL);
                    } else if (typeof window !== 'undefined') {
                      window.open(DASHBOARD_URL, '_blank');
                    }
                  },
                  style: {
                    background: 'rgba(255, 255, 255, 0.08)',
                    color: '#f1f5f9',
                    border: '1px solid rgba(255, 255, 255, 0.2)',
                    borderRadius: '6px',
                    padding: '8px 16px',
                    fontWeight: 600,
                    cursor: 'pointer',
                    fontSize: '0.85rem',
                  },
                  children: 'Abrir no Navegador Externo ↗',
                }),
              ],
            }),
          ],
        });
      },
    });
  },
};

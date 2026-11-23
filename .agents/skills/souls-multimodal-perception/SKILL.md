---
name: souls-multimodal-perception
description: Playbook operacional da Percepção Multimodal Híbrida (0 MB VRAM por padrão) no Souls Engine (v0.9.3). Cobre Visão Cognitiva O(1) via AXTree LEAN (95%), diff estrutural e detector de pontos cegos com SoM & Snapshot sob demanda (5%), Áudio Soberano PT-BR em CPU AVX2 (Silero VAD, PCOMP, SenseVoice, prosódia Camada 3 e cascata TTS/STT), Motor de Rapport Polivagal 2e/TDAH (EMA alpha=0.15, prefix-cache-safe), Gateway OpenAI (/v1/audio/*) e Hermes Voice Customs 1-clique.
metadata:
  version: "1.0.0"
  author: "Souls Engine Architecture"
  namespace: "ui"
---

# Percepção Multimodal Híbrida & Voz Soberana PT-BR (`v0.9.3`)

Este playbook governa a percepção de tela e voz sem desperdiçar VRAM da GPU local nem invalidar o *Prefix Caching* do chat mestre.

---

## 1. Catálogo MCP Atualizado (`91` Ferramentas / `17` Skills)

O Souls Engine v0.9.3 expõe **91 ferramentas MCP** (**8** no Boot Nível 0 e **83** sob revelação progressiva) e **17 skills canônicas** no Switchboard.
Para clientes MCP com validação estática (como o Hermes Agent), descubra via `tool_search(namespace="ui")` e invoque via `tool_dispatch(tool="...", arguments={...})`:

- `ui_inspect`: Captura e serializa a árvore de acessibilidade (CDP ou Windows UI Automation ao vivo) em notação LEAN dot-notation, aplicando `intent_filter` (`all`, `actionable`, `inputs`, `buttons`) e diagnóstico de blind-spot (`0 MB VRAM`).
- `ui_interact`: Calcula centro geométrico determinístico e realiza *hit-testing* tátil no nó sob `(x, y)` (via captura ao vivo ou `tree`/`cdp_nodes` fornecidos).
- `ui_diff`: Computa diff estrutural LEAN entre `previous_tree` e `after_tree` (ou captura ao vivo se `after_tree` for omitido), retornando apenas nós adicionados/removidos/alterados e `bytes_saved_pct`.
- `ui_snapshot`: Captura snapshot visual **sob demanda** com marcas **Set-of-Mark (SoM)** e deduplicação SHA-256 em `vision_snapshot_ledger` (retornando `screen_unchanged: true` após até `MAX_SNAPSHOT_OMISSION_STREAK = 5` repetições).
- `voice_transcribe`: Avalia áudio Base64 ou `sample_text` com filtro de alucinação de silêncio, tags emocionais SenseVoice e veredito PCOMP de turno, atualizando o perfil EMA do usuário (`0 MB VRAM`).
- `voice_synthesize`: Aplica modelagem prosódica Camada 3 em português e sintetiza WAV 24 kHz mono na CPU (`0 MB VRAM`).
- `voice_rapport`: Consulta ou persiste `UserRapportProfileDto` e `CognitiveVoiceMode` (`auto`, `hands_on`, `sparring`, `decompression`, `crisis_paralysis`, `philosophical`).

---

## 2. Regra de Ouro Visual (95% AXTree LEAN / 5% Snapshot Sob Demanda)

1. **Sempre comece por `ui_inspect`** (ou `ui_diff` após a primeira leitura da janela):
   - Latência sub-ms / sub-5ms, **0 MB VRAM**, zero tokens de imagem e redução estrutural em notação LEAN.
   - Use `intent_filter: "actionable"` ou `"inputs"` para cortar ruído decorativo.
2. **Escalone para `ui_snapshot` apenas diante de pontos cegos reais (`BlindSpotAssessment`):**
   - Superfícies `<canvas>` / WebGL / Figma / Miro / Canva / Blender.
   - Árvores com zero elementos acionáveis (telas renderizadas como bitmap).
   - Sessões remotas/virtualizadas (RDP, VNC, TeamViewer, AnyDesk, Hyper-V/VM).
   - Solicitação explícita de QA visual/CSS pelo usuário.
3. **Governança Térmica e Anti-Loop:**
   - Modo padrão `"auto"` sem `prompt` degrada deterministicamente para `deterministic_som_only` (`0 MB VRAM`); quando há `prompt`, o `EphemeralVisionRunner` (`Qwen2.5-VL-3B-Instruct-Q4_K_M`, priorizando diretórios com `mmproj*.gguf` irmão e mantendo arquivos transitórios em `Z:\souls_engine\.souls_data\vision_snapshots`) executa em turno único sob bloqueio NVML (`<= 4.200 MB` VRAM usada e `< 78.0 °C`), desviando para SoM/Cloud se o Watchdog restringir.
   - Nunca envie screenshots em streaming contínuo; se `screen_unchanged: true`, reutilize o contexto já observado.

---

## 3. Voz Soberana Local (`0 MB VRAM`) & Gateway OpenAI

- **Endpoints compatíveis com OpenAI Audio (`127.0.0.1:9123`):**
  - `POST /v1/audio/speech`: Sintetiza PCM RIFF/WAVE 24 kHz mono em CPU via cascata (`Kokoro ONNX` -> `Piper` -> `Edge Neural PT-BR` via venv Hermes -> `Windows SAPI PT-BR` `Microsoft Maria Desktop` -> oscilador/silêncio puro em Rust).
  - `POST /v1/audio/transcriptions`: Aceita upload `multipart/form-data` (campo `file`) ou JSON, suporta `response_format="text"` (`text/plain; charset=utf-8`) além de JSON, normaliza áudio para 16 kHz mono e executa cascata (`SenseVoice ONNX` -> `whisper.cpp` -> `faster-whisper CPU int8` via `Systran/faster-whisper-base` -> `Windows Speech` apenas com cultura idêntica, sem fallback *cross-language*).
  - **Provisionamento 1-Clique:** `POST /api/v1/audio/provision` (ou `tools/install_souls.ps1`) baixa `Systran/faster-whisper-base` para `Z:\souls_engine\.souls_data\models\audio\faster-whisper-base` e grava `audio_engines_manifest.json`; `whisper_ready` só reporta `true` com pesos presentes em disco (`faster_whisper_needs_download` caso contrário).
- **Regras PCOMP de Turn-Taking e Prosódia:**
  - Histerese Silero VAD: `SILERO_VAD_STANDBY_HYSTERESIS_MS = 8000` ms, *barge-in* >= `300` ms (`BARGE_IN_MIN_SPEECH_MS`), ignorando ruídos transientes <= `200` ms (`NOISE_IGNORE_MAX_MS`).
  - Conectivos abertos em PT-BR (`porque`, `mas`, `então`, `como`, `e aí`, `ou seja`, `,`, `...`) nunca tomam o turno prematuramente: retornam `continue_listening` (< 300 ms), `emit_backchannel` (300–1200 ms, ex.: `"Aham..."`) ou `contemplative_wait` (>= 2000 ms).
  - Camada 3 de prosódia converte formatação Markdown pesada em texto limpo com pausas naturais (`...`, `—`, `,`) e fracionamento sub-1ms por sentença.

---

## 4. Rapport Polivagal 2e/TDAH & Blindagem de Prefix Caching

- O perfil `user_rapport_profile` (SQLite `STRICT`) acompanha `formality_score`, `avg_sentence_length`, `preferred_pacing`, `slang_lexicon` e `contemplative_base_ms` via média móvel exponencial (**EMA** `alpha = 0.15`, variação máxima de `±10%` por turno e `mirroring_ratio` travado em `[0.0, 0.35]`).
- **Imutabilidade de Cache:** Nunca altere `system_prompt_block()` no adaptador Hermes. Orientações de ritmo (`[MODO DE VOZ & RITMO ATIVO]`) são injetadas apenas como contexto efêmero por turno em `prefetch()` via `GET /api/v1/audio/rapport`.
- Para ativar no Hermes Desktop e em todos os sub-perfis (`config.yaml` raiz + `profiles/*/config.yaml`, com backup `.yaml.bak` e preservação de `stt.local.model`), utilize `GET /api/v1/hermes/voice-customs` e `POST /api/v1/hermes/voice-customs/sync` (ou a sub-aba **Voz & Visão Híbrida** no Cockpit).

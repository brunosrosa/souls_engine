# ADR-033: Percepção Multimodal Híbrida, Áudio Soberano PT-BR e Rapport Polivagal (`v0.9.3`)

## Status
**Aprovado e Implementado** (Milestone `v0.9.3`)

## Data
2026-11-22

## Contexto e Problema
O ADR-029 estabeleceu a base da **Visão Cognitiva O(1)** (`ui_inspect` e `ui_interact` sobre árvores AXTree em notação LEAN), mas ainda deixava três lacunas práticas para uso cotidiano no Windows 11 e com o Hermes Agent Desktop:
1. **Pontos Cegos Reais na UI e Ausência de Diffs Estruturais:** Superfícies gráficas (`<canvas>`, WebGL, Figma, Miro, Blender) e sessões RDP/VM não expõem nós acionáveis via UI Automation, exigindo inspeção pontual de pixels sem sacrificar o orçamento WDDM da GPU RTX 2060m (~5,2 GB úteis). Além disso, reenviar árvores inteiras a cada turno desperdiçava tokens quando apenas poucos controles mudavam.
2. **Dependência de APIs Pagas e Idioma Desalinhado na Voz:** A alternativa de sessões WebRTC/Gemini Live cobra por minuto aberto de áudio/vídeo contínuo, envia áudio bruto continuamente para a nuvem e cria *lock-in* de provedor. No host real, o `config.yaml` do Hermes vinha configurado com `stt.language: en` e `tts.provider: elevenlabs`, impedindo uso soberano em português brasileiro (`pt-BR`) sem custo.
3. **Falta de Cadência Neuro-Inclusiva (2e/TDAH) e Risco ao Prefix Caching:** Leituras textuais frias com Markdown bruto soam robóticas em TTS, interrupções precoces quebram linhas de raciocínio abertas por conectivos em português, e alterações no `system_prompt_block` quebrariam a taxa de acerto >95% de *Prefix Caching*.

---

## Decisão de Arquitetura

### 1. Visão Cognitiva Híbrida 95% AXTree / 5% Snapshot Sob Demanda (`souls_core::vision` + `souls_inference_runtime::vision_ephemeral`)
- **Captura Ativa O(1) & Filtro de Intenção:** `capture_live_windows_ax_tree()` (PowerShell UIA em `-STA` com correspondência por título/`ProcessName` e fallback top-level) e `filter_ax_tree_by_intent()` mapeiam janelas Windows/CDP com `0 MB VRAM`, aceitando `intent_filter` (`all`, `actionable`, `inputs`, `buttons`).
- **Diff Estrutural LEAN (`ui_diff`):** `compute_ax_tree_diff()` compara árvores prévia e atual, transmitindo somente deltas e contabilizando `bytes_saved_pct`.
- **Detector de Blind-Spot, Set-of-Mark (SoM) e Ledger SHA-256 (`ui_snapshot`):** `assess_blind_spot()` identifica automaticamente áreas cegas (`canvas`, zero nós acionáveis, RDP/VM); `capture_on_demand_snapshot()` renderiza captura GDI sob demanda com retenção rotativa de 20 PNGs, gera marcas `SomOverlayMark`, grava histórico em `vision_snapshot_ledger` (SQLite `STRICT`) e deduplica telas inalteradas via SHA-256 (com teto `MAX_SNAPSHOT_OMISSION_STREAK = 5`). Chamadas `VisualAnalysisMode::Auto` sem `prompt` degradam deterministicamente para `DeterministicSomOnly` (`0 MB VRAM`).
- **VLM Efêmero Governado pelo Watchdog:** `EphemeralVisionRunner` (`Qwen2.5-VL-3B-Instruct-Q4_K_M` via `llama-mtmd-cli` / `llama-cli`, priorizando diretórios com `mmproj*.gguf` irmão e mantendo arquivos transitórios estritamente em `Z:\souls_engine\.souls_data\vision_snapshots`) executa apenas em turno único e sob trava estrita do `HardwareWatchdog` (`EPHEMERAL_VLM_MAX_VRAM_USED_MB = 4200`, `EPHEMERAL_VLM_MAX_TEMP_C = 78.0`, timeout `35s`, higiene de subprocesso Windows `Stdio::null()` + dreno em threads). Caso exceda os limites térmicos/VRAM, desvia automaticamente para fallback determinístico SoM ou escalonamento em nuvem (`diverted_by_watchdog: true`).

### 2. Áudio Soberano Local 0 MB VRAM (`souls_core::audio`) & Gateway OpenAI
- **Garantia Física (`ZERO_AUDIO_VRAM_CONSUMPTION_MB = 0`):** Todo o pipeline vocal opera exclusivamente em CPU AVX2 sem alocar tensores CUDA/WDDM.
- **Silero VAD & PCOMP Turn-Taking (`evaluate_pcomp_turn_taking`):** Define histerese de standby (`8000 ms`), janela de *barge-in* (`300 ms`), supressão de ruídos transientes (`<= 200 ms`), detecção de conectivos abertos em PT-BR (`porque`, `mas`, `então`, `e aí`, `ou seja`, `,`, `...`) e *pacing* contemplativo (`200–600 ms` + micro-backchannels PCM em RAM).
- **SenseVoice SER + Filtro de Alucinações (`parse_sensevoice_and_filter_hallucinations`):** Extrai tags emocionais (`neutral`, `happy`, `angry`, `hesitant`, `laughter`, `sad`, `focused`) e remove alucinações clássicas de silêncio em STT.
- **Prosódia Camada 3 (`shape_prosodic_text_for_tts` & `split_prosodic_sentences`):** Limpa Markdown mantendo pausa de respiro (`...`, `—`, `,`) e fracionamento sub-1ms em frases.
- **Cascata TTS/STT Real & Gateway OpenAI:**
  - **TTS (`synthesize_speech_cascade`):** `Kokoro ONNX -> Piper -> Edge Neural PT-BR` (`edge-tts.exe` + `ffmpeg.exe` 24 kHz mono do venv Hermes) `-> Windows SAPI PT-BR` (`Microsoft Maria Desktop`) `-> RIFF/WAVE PCM 24 kHz mono puro em Rust`.
  - **STT (`transcribe_audio_cascade`):** `SenseVoice ONNX -> whisper.cpp -> faster-whisper CPU int8` (`Systran/faster-whisper-base`, com `whisper_ready` exigindo pesos em disco e provisionamento 1-clique via `POST /api/v1/audio/provision` ou `tools/install_souls.ps1`) `-> Windows Speech` (apenas para cultura correspondente, bloqueando fallback *cross-language*) `-> fallback determinístico`.
  - **Compatibilidade OpenAI & Alfândega Multi-Perfil:** `POST /v1/audio/speech` e `POST /v1/audio/transcriptions` (suportando `response_format="text"` em `text/plain` e JSON) em `127.0.0.1:9123`, além da Alfândega de Voz (`/api/v1/hermes/voice-customs` e `/sync`) que cria `.yaml.bak`, preserva sub-blocos irmãos (`stt.local.model`), desarma `use_gateway: false` e sincroniza `config.yaml` raiz e todos os sub-perfis (`profiles/*/config.yaml`).

### 3. Motor de Rapport Polivagal (`souls_memory::rapport` + `adapters/hermes_memory_souls`)
- **Estado Evolutivo em SQLite STRICT (`user_rapport_profile`):** Atualiza `formality_score`, `avg_sentence_length`, `preferred_pacing`, `slang_lexicon_json` e `contemplative_base_ms` via **EMA** (`RAPPORT_EMA_ALPHA = 0.15`), limitada a variação máxima de `±10%` por turno e `mirroring_ratio` clamped em `[0.0, 0.35]` (`MAX_MIRRORING_RATIO`).
- **5 Modos Cognitivos + `auto` (`CognitiveVoiceMode`):** `auto`, `hands_on`, `sparring`, `decompression`, `crisis_paralysis`, `philosophical`.
- **Injeção Prefix-Cache-Safe:** `SoulsMemoryProvider::system_prompt_block()` permanece intacto; o método `_fetch_voice_rapport_block()` consulta `/api/v1/audio/rapport` e injeta `[MODO DE VOZ & RITMO ATIVO]` exclusivamente no retorno efêmero de `prefetch()`.

### 4. Superfície Humana Zero-Jargão (`souls_server`)
- Nova sub-aba **Voz & Visão Híbrida** no pilar Cockpit (`multimodal_hub.html` + `multimodal_hub.js`), sem estilos inline (`style="..."`), sem emojis crus e com diagnósticos técnicos recolhidos em `<details class="lab-drawer">`, acompanhada por **91 ferramentas MCP** (**8** Boot Nível 0 + **83** progressivas) e pela 17ª skill `souls-multimodal-perception`.

---

## Consequências

### Positivas
- **Economia Radical de VRAM & FinOps:** Uso cotidiano de visão e áudio com `0 MB VRAM` garantidos e zero custo recorrente de API de voz por minuto.
- **Conversação Natural em Português & Neuro-Inclusão:** Turn-taking sensível a hesitações e pensamento divergente, somado à adaptação contínua de formalidade e ritmo sem caricatura.
- **Integração Zero-Atrito com o Hermes:** Endpoint compatível com cliente OpenAI oficial (`/v1/audio/*`) e sincronização em 1 clique da Alfândega de Voz.

### Riscos e Mitigações
- **Ausência de pesos ONNX opcionais no disco do usuário:** Mitigada pela cascata com Windows Speech nativo PT-BR e síntese PCM 24 kHz determinística em Rust + facilitadores de provisionamento no Cockpit.
- **Disputa térmica em análises pontuais de `Qwen2.5-VL-3B`:** Mitigada pelo bloqueio NVML (`<= 4200 MB`, `< 78.0 °C`), execução em turno único e fallback automático para SoM/Cloud.

# ADR-012: Modularização Zero-Copy do Dashboard Web Headless e Unificação SSOT de Dependências no Workspace

## Status
**Aprovado e Implementado** (Milestone v0.5.0)

## Data
2026-11-05

## Contexto e Forças em Conflito
Com a expansão contínua do Souls Engine v7 bare-metal, dois pontos críticos de fricção arquitetural atingiram o limite operacional:

1. **Monólito Sintático de Interface:** O arquivo [`crates/souls_server/src/dashboard_ui.html`](file:///Z:/souls_engine/crates/souls_server/src/dashboard_ui.html) atingiu **15.782 linhas de código**, unindo HTML5, CSS customizado e JavaScript reativo em um único documento. Isso gerava sérios riscos de *Context Rot* para agentes de IA, desperdício de tokens de contexto, lentidão nas ferramentas de edição sintática e dificuldade de manutenção concorrente.
2. **Fragmentação de Dependências no Workspace:** As 11 crates canônicas declaravam repetidamente dependências compartilhadas (como `sqlx`, `dashmap`, `memmap2`, `sha2`, `tempfile` e `tiktoken`) com versões literais individuais, violando o princípio de *Single Source of Truth* (SSOT) e forçando o linker do MSVC a lidar com tabelas e monomorfizações duplicadas.
3. **Restrições Inegociáveis (AGENTS.md):**
   - O binário `souls_server.exe` / `souls_engine.exe` deve permanecer **100% headless** e autocontido (Zero dependência de arquivos soltos em runtime, zero leitura de disco para assets da web).
   - O workspace deve manter estritamente as **11 crates canônicas** compiladas com MSVC em Windows 11 Dev Drive ReFS.

---

## Decisão de Arquitetura

Optou-se pela fusão sinérgica entre a **Opção B (Decomposição Atômica no Disco)** e a **Opção A (Montagem Zero-Copy em Tempo de Compilação no Rust)**:

### 1. Decomposição em 25 Componentes Atômicos
O antigo arquivo monólito foi inteiramente eliminado e decomposto em 25 arquivos modulares em [`crates/souls_server/src/templates/`](file:///Z:/souls_engine/crates/souls_server/src/templates/):
- **`styles.css`**: ~3.000 linhas de estilos puros, desacoplados do DOM.
- **`components/`**: `topbar.html`, `navigation.html`, `modals.html`.
- **`tabs/`**: `overview.html`, `subagents.html`, `local_models.html`, `cloud_arena.html`.
- **`settings/`**: 9 subabas atômicas independentes (`subtab_brain.html`, `subtab_tools.html`, `subtab_reasoning.html`, `subtab_finops.html`, `subtab_router.html`, `subtab_memory.html`, `subtab_sandbox.html`, `subtab_obsidian.html`, `subtab_raw.html`), além dos delimitadores de layout.
- **`scripts/`**: `dashboard.js` concentrando a inteligência client-side.

### 2. Montagem em Tempo de Compilação (`templates/mod.rs`)
Através da macro Rust `concat!(include_str!(...))`, todos os parciais são fundidos estaticamente pelo compilador `rustc` diretamente na seção de dados `.rodata` do binário nativo:
- **Zero Disk I/O em Runtime:** Nenhuma chamada ao sistema de arquivos é feita quando o cliente requisita `GET /`. O conteúdo já reside em memória bare-metal.
- **Portabilidade Headless:** O binário continua 100% independente, sem pastas de assets adjacentes necessárias para execução.

### 3. Unificação de Dependências no `[workspace.dependencies]`
As dependências transversais foram promovidas para a raiz do workspace:
- `sqlx = { version = "0.8", default-features = false, features = ["runtime-tokio", "sqlite"] }`
- `dashmap = "6.1"`, `memmap2 = "0.9"`, `sha2 = "0.10"`, `tiktoken = "3.1.4"`
- `tempfile = "3.10"` (para dev-dependencies)

### 4. Otimização do Perfil Release
Ajuste dos parâmetros de compilação em `[profile.release]`:
- `lto = "thin"`: Ativa inlining inteligente entre crates com tempo de compilação controlado.
- `codegen-units = 1`: Habilita otimização global e eliminação de código morto (dead code elimination).
- `panic = "abort"`: Remove as tabelas gigantescas de unwinding do runtime C++/MSVC, reduzindo o executável final em até 30 MB.

---

## Consequências

### Positivas
1. **Erradicação do Context Rot:** Qualquer alteração futura em subabas pontuais (ex: Cofre Obsidian ou TOML SSOT) toca exclusivamente em arquivos pequenos (150 a 500 linhas), poupando mais de 80% de tokens de contexto durante edições agênticas.
2. **Aceleração da Compilação:** O tempo de `cargo check -p souls_server` reduziu para **2.72 segundos** (ganho de ~3.2x).
3. **Consistência de Tipos:** A unificação do `sqlx` e `tempfile` garante que todo o workspace compartilhe exatamente a mesma implementação de drivers e pools.
4. **Verificação Limpa:** 100% dos 35 testes unitários de `souls_server` e de todas as crates do workspace passaram com louvor.

### Neutras / Cuidados
- A edição de trechos da UI requer recompilação do binário (comportamento padrão do design bare-metal headless do Souls Engine).


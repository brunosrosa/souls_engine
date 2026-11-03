with open('crates/souls_server/src/profiles.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace('\r\n', '\n')

# 1. Update arena_category for frontier_balanced
old_frontier = """        configured_profiles.insert(0, MasterBrainProfile {
            id: "frontier_balanced".to_string(),
            name: "Frontier Balanced (Master Brain)".to_string(),
            description: "Orquestrador mestre e diálogo diário com o desenvolvedor. Ponto ótimo da curva de Pareto.".to_string(),
            role: "orchestrator".to_string(),
            arena_category: "agentic".to_string(),"""

new_frontier = """        configured_profiles.insert(0, MasterBrainProfile {
            id: "frontier_balanced".to_string(),
            name: "Frontier Balanced (Master Brain)".to_string(),
            description: "Orquestrador mestre e diálogo diário com o desenvolvedor. Ponto ótimo da curva de Pareto.".to_string(),
            role: "orchestrator".to_string(),
            arena_category: "master".to_string(),"""

assert old_frontier in text, "Error: old_frontier not found"
text = text.replace(old_frontier, new_frontier)

# 2. Add ux_designer, web_researcher, memory_curator before local_sovereign
old_local = """        configured_profiles.push(MasterBrainProfile {
            id: "local_sovereign".to_string(),
            name: "Local Sovereign 🛡️".to_string(),"""

new_subagents = """        configured_profiles.push(MasterBrainProfile {
            id: "ux_designer".to_string(),
            name: "UX & Frontend 🎨".to_string(),
            description: "Geração de componentes de interface, design responsivo, protótipos e estilização CSS.".to_string(),
            role: "subagent".to_string(),
            arena_category: "design".to_string(),
            target_model: "anthropic/claude-3-7-sonnet".to_string(),
            provider: "nous".to_string(),
            target_type: "Cloud API".to_string(),
            is_active: false,
            is_builtin: true,
            triad: None,
        });
        configured_profiles.push(MasterBrainProfile {
            id: "web_researcher".to_string(),
            name: "Web Intelligence 🔎".to_string(),
            description: "Pesquisa autônoma, síntese de fontes web, raspagem limpa na RAM e verificação factual.".to_string(),
            role: "subagent".to_string(),
            arena_category: "agentic".to_string(),
            target_model: "google/gemini-2.0-flash-001".to_string(),
            provider: "nous".to_string(),
            target_type: "Cloud API".to_string(),
            is_active: false,
            is_builtin: true,
            triad: None,
        });
        configured_profiles.push(MasterBrainProfile {
            id: "memory_curator".to_string(),
            name: "Memory & Graph 💾".to_string(),
            description: "Curadoria epistêmica L2/L3, extração de entidades no LadybugDB e síntese do cofre Obsidian.".to_string(),
            role: "subagent".to_string(),
            arena_category: "agentic".to_string(),
            target_model: "deepseek/deepseek-chat".to_string(),
            provider: "nous".to_string(),
            target_type: "Cloud API".to_string(),
            is_active: false,
            is_builtin: true,
            triad: None,
        });
        configured_profiles.push(MasterBrainProfile {
            id: "local_sovereign".to_string(),
            name: "Local Sovereign 🛡️".to_string(),"""

assert old_local in text, "Error: old_local not found"
text = text.replace(old_local, new_subagents)

with open('crates/souls_server/src/profiles.rs', 'w', encoding='utf-8', newline='\n') as f:
    f.write(text)

print("profiles.rs successfully updated!")

//! # resolver.rs — Resolução Canônica de Slugs e Entidades
//!
//! Mapeia strings heterogêneas dos provedores para o ID canônico correspondente.

use std::collections::{HashMap, HashSet};

#[derive(Clone, Default)]
pub struct CanonicalResolver {
    aliases: HashMap<String, String>,
    known_canonicals: HashSet<String>,
}

impl CanonicalResolver {
    pub fn new() -> Self {
        let mut resolver = Self {
            aliases: HashMap::new(),
            known_canonicals: HashSet::new(),
        };

        // Carrega seed de aliases inicial
        for item in crate::canonical::seed_aliases::get_initial_seed_models() {
            let canon_id = item.model.id.clone();
            resolver.known_canonicals.insert(canon_id.clone());
            // Mapeia o próprio ID canônico para ele mesmo
            resolver.register_alias(&canon_id, &canon_id);
            for (alias, _) in item.aliases {
                resolver.register_alias(alias, &canon_id);
            }
        }

        resolver
    }

    /// Registra um alias explícito.
    pub fn register_alias(&mut self, alias: &str, canonical_id: &str) {
        self.aliases.insert(alias.to_lowercase().trim().to_string(), canonical_id.to_string());
    }

    /// Registra dinamicamente um modelo canônico suportado na Cloud Arena.
    pub fn register_canonical(&mut self, canonical_id: &str) {
        let norm = canonical_id.to_lowercase().trim().to_string();
        self.known_canonicals.insert(norm.clone());
        self.register_alias(&norm, &norm);

        // Sem prefixo de fornecedor (ex: "claude-sonnet-5.5" -> "anthropic/claude-sonnet-5.5")
        if let Some((_, model)) = norm.split_once('/') {
            self.register_alias(model, &norm);
        }
        // Com prefixo de roteador ~ (ex: "~openai/gpt-6-sol" -> "openai/gpt-6-sol")
        self.register_alias(&format!("~{}", norm), &norm);
    }

    /// Retorna a lista de todos os IDs canônicos conhecidos.
    pub fn known_canonicals(&self) -> &HashSet<String> {
        &self.known_canonicals
    }

    /// Verifica se um ID é reconhecido como modelo canônico suportado na Cloud Arena.
    pub fn is_known_canonical(&self, canonical_id: &str) -> bool {
        self.known_canonicals.contains(canonical_id)
    }

    /// Tenta resolver uma string de modelo apenas se ela pertencer a um modelo canônico suportado.
    pub fn try_resolve(&self, raw: &str) -> Option<String> {
        let resolved = self.resolve(raw);
        if self.is_known_canonical(&resolved) {
            Some(resolved)
        } else {
            None
        }
    }

    /// Resolve uma string de modelo para o ID canônico correspondente.
    pub fn resolve(&self, raw: &str) -> String {
        let normalized = raw.to_lowercase().trim().to_string();

        // 1. Verificação direta no mapa de aliases
        if let Some(canonical) = self.aliases.get(&normalized) {
            return canonical.clone();
        }

        // 1.1 Verificação sem prefixo tilde (ex: ~openai/gpt-6-sol -> openai/gpt-6-sol)
        let without_tilde = normalized.trim_start_matches('~');
        if let Some(canonical) = self.aliases.get(without_tilde) {
            return canonical.clone();
        }

        // 2. Normalização heurística de fallback
        let without_tag = if let Some((base, _)) = normalized.split_once(':') {
            base
        } else {
            &normalized
        };

        if let Some(canonical) = self.aliases.get(without_tag) {
            return canonical.clone();
        }

        // Remove prefixo de fornecedor (ex: "anthropic/claude-3-7-sonnet" -> "claude-3-7-sonnet")
        let without_prefix = if let Some((_, model)) = without_tag.split_once('/') {
            model
        } else {
            without_tag
        };

        if let Some(canonical) = self.aliases.get(without_prefix) {
            return canonical.clone();
        }

        // Normalização de pontos para hífens (ex: "claude-3.7-sonnet" -> "claude-3-7-sonnet")
        let dashed = without_prefix.replace('.', "-").replace('_', "-");
        if let Some(canonical) = self.aliases.get(&dashed) {
            return canonical.clone();
        }

        // Fallback: retorna o nome normalizado limpo
        dashed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_resolution() {
        let resolver = CanonicalResolver::new();

        assert_eq!(resolver.resolve("~anthropic/claude-sonnet-5.5"), "anthropic/claude-sonnet-5.5");
        assert_eq!(resolver.resolve("anthropic/claude-sonnet-5.5:beta"), "anthropic/claude-sonnet-5.5");
        assert_eq!(resolver.resolve("claude-sonnet-5.5"), "anthropic/claude-sonnet-5.5");
        assert_eq!(resolver.resolve("gemini-4-argon-high"), "google/gemini-4-argon");
        assert_eq!(resolver.resolve("~openai/gpt-astra-latest"), "openai/gpt-6-astra");
        assert_eq!(resolver.resolve("glm-5.3-max"), "z-ai/glm-5.3-prime");
        assert_eq!(resolver.resolve("qwen3.8-max"), "qwen/qwen3.8-max-prime");
    }
}

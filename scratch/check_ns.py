import re

with open('crates/souls_server/src/mcp/registry.rs', encoding='utf-8') as f:
    text = f.read()

matches = re.findall(r'namespace:\s*"([^"]+)"', text)
print('Namespaces in registry.rs:', sorted(list(set(matches))))


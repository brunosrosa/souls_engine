with open('crates/souls_server/src/routes/dashboard.rs', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i, line in enumerate(lines):
    if "v_trim.starts_with" in line and "ends_with" in line:
        lines[i] = '            if (v_trim.starts_with(\'"\') && !v_trim.ends_with(\'"\')) || (v_trim.starts_with("\'") && !v_trim.ends_with("\'")) {\n'
        print(f"Replaced line {i+1} with clean string literals!")

with open('crates/souls_server/src/routes/dashboard.rs', 'w', encoding='utf-8') as f:
    f.writelines(lines)


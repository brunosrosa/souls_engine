with open(r'C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f\subagents_annotator.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i, l in enumerate(lines):
    if 'loadImage' in l or 'img.onload' in l or 'img.src' in l:
        print(f"at line {i+1}:")
        for j in range(max(0, i-5), min(len(lines), i + 25)):
            print(f"{j+1}: {lines[j]}", end='')
        break


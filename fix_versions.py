import os, re
for root, _, files in os.walk(r'D:\framework\threadloom'):
    if 'Cargo.toml' in files:
        path = os.path.join(root, 'Cargo.toml')
        with open(path, 'r', encoding='utf-8') as f:
            content = f.read()
        new_content = re.sub(r'(threadloom[a-zA-Z0-9-]*\s*=\s*\{\s*path\s*=\s*\"[^\"]+\")\s*\}', r'\1, version = "0.1.0" }', content)
        if new_content != content:
            with open(path, 'w', encoding='utf-8') as f:
                f.write(new_content)

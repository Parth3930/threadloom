import os, re

for root, _, files in os.walk(r'D:\framework\threadloom'):
    if 'Cargo.toml' in files:
        path = os.path.join(root, 'Cargo.toml')
        with open(path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        def replacer(match):
            m = match.group(0)
            if 'version' not in m:
                # Append version before the closing brace
                inner = m[:-1].strip()
                if not inner.endswith(','):
                    inner += ','
                return inner + ' version = "0.1.0" }'
            return m
        
        new_content = re.sub(r'threadloom[a-zA-Z0-9-]*\s*=\s*\{\s*path\s*=\s*\"[^\"]+\"[^\}]*\}', replacer, content)
        if new_content != content:
            with open(path, 'w', encoding='utf-8') as f:
                f.write(new_content)

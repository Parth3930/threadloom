import os
import re

def bump_version(ver_str):
    parts = ver_str.split('.')
    parts[-1] = str(int(parts[-1]) + 1)
    return '.'.join(parts)

versions_to_bump = {}

# Pass 1: Find all crate versions
for root, dirs, files in os.walk('.'):
    if 'target' in root or '.git' in root:
        continue
    for file in files:
        if file == 'Cargo.toml':
            path = os.path.join(root, file)
            with open(path, 'r', encoding='utf-8') as f:
                content = f.read()
            # find package name
            m_name = re.search(r'^name\s*=\s*"([^"]+)"', content, flags=re.MULTILINE)
            m_ver = re.search(r'^version\s*=\s*"([^"]+)"', content, flags=re.MULTILINE)
            if m_name and m_ver:
                versions_to_bump[m_name.group(1)] = (m_ver.group(1), bump_version(m_ver.group(1)))

print("Bumping versions:", versions_to_bump)

# Pass 2: Replace versions in all Cargo.toml
for root, dirs, files in os.walk('.'):
    if 'target' in root or '.git' in root:
        continue
    for file in files:
        if file == 'Cargo.toml':
            path = os.path.join(root, file)
            with open(path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            # replace package version
            def repl_pkg_ver(m):
                return f'version = "{bump_version(m.group(1))}"'
            content = re.sub(r'^version\s*=\s*"([^"]+)"', repl_pkg_ver, content, count=1, flags=re.MULTILINE)
            
            # replace dependencies versions
            for pkg, (old_v, new_v) in versions_to_bump.items():
                pattern = '(' + re.escape(pkg) + r'\s*=\s*\{[^}]*version\s*=\s*"=?)' + re.escape(old_v) + r'(")'
                content = re.sub(pattern, r'\g<1>' + new_v + r'\g<2>', content)
                
            with open(path, 'w', encoding='utf-8') as f:
                f.write(content)

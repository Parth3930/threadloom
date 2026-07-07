import os

for root, _, files in os.walk(r'D:\framework\threadloom'):
    if 'Cargo.toml' in files:
        path = os.path.join(root, 'Cargo.toml')
        with open(path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        if '[package]' in content:
            changed = False
            if 'description =' not in content:
                content = content.replace('[package]', '[package]\ndescription = "A meticulously crafted Rust full-stack framework."')
                changed = True
            if 'license =' not in content:
                content = content.replace('[package]', '[package]\nlicense = "MIT"')
                changed = True
            
            if changed:
                with open(path, 'w', encoding='utf-8') as f:
                    f.write(content)

import os
import re

components_dir = r"D:\framework\threadloom\crates\threadloom-ui\src\components"

for filename in os.listdir(components_dir):
    if not filename.endswith(".rs"): continue
    path = os.path.join(components_dir, filename)
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()
    
    # 1. Find all `pub struct XYZProps { ... }` and extract fields
    struct_pattern = re.compile(r'pub struct (\w+)Props\s*\{([^}]*)\}')
    structs = {}
    for match in struct_pattern.finditer(content):
        name = match.group(1)
        body = match.group(2)
        fields = []
        for line in body.split('\n'):
            line = line.strip()
            # Handle inline docs: remove them
            line = re.sub(r'///.*', '', line).strip()
            if not line: continue
            if line.startswith('pub '):
                # e.g. pub gap: i32, pub p: i32, pub px: i32,
                parts = line.split(',')
                for part in parts:
                    part = part.strip()
                    if part.startswith('pub '):
                        field_decl = part[4:].strip()
                        if ':' in field_decl:
                            fields.append(field_decl)
                    elif ':' in part: # handles inline comma separated fields
                        fields.append(part.strip())
        structs[name] = fields

    # 2. For each struct, find the corresponding `pub fn Name(props: NameProps)` and insert `/// **Props:**\n/// - \n` above it
    new_content = content
    for name, fields in structs.items():
        if name == "Modal": continue # it's an alias
        
        # build the docs block
        docs = "///\n/// **Props:**\n"
        for f in fields:
            # e.g. "gap: i32"
            docs += f"/// - `{f}`\n"
        
        # We need to find the function signature and its preceding docs
        # We look for something like:
        # /// Renders a Card component.
        # #[allow(non_snake_case)]
        # pub fn Card(props: CardProps)
        
        # It's tricky because there might be other attributes or different doc styles.
        # Let's find `pub fn Name(props: NameProps)` and insert docs right before it if it doesn't already have `**Props:**`.
        
        # Regex to find the function, including any doc comments and attributes right before it
        func_pattern = re.compile(r'((?:///.*\n)*)(#\[[^\]]+\]\n)*pub fn ' + name + r'\s*\(')
        
        def replacer(m):
            existing_docs = m.group(1)
            attrs = m.group(2) or ""
            # If it already has **Props:**, remove the old ones to replace them
            if "**Props:**" in existing_docs:
                existing_docs = re.sub(r'///\s*\*\*Props:\*\*.*$', '', existing_docs, flags=re.DOTALL)
            
            # Ensure existing docs end with a newline
            if existing_docs and not existing_docs.endswith('\n'):
                existing_docs += '\n'
                
            return existing_docs + docs + attrs + f"pub fn {name}("
            
        new_content = func_pattern.sub(replacer, new_content)
        
    with open(path, "w", encoding="utf-8") as f:
        f.write(new_content)
    print(f"Updated {filename}")

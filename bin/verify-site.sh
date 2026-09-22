#!/bin/sh
# =============================================================================
# CCCP Documentation Site Integrity Validator
# Verifies all local links, scripts, stylesheets, and images across site/
# =============================================================================

set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SITE_DIR="$PROJECT_ROOT/site"

if [ ! -d "$SITE_DIR" ]; then
    echo "Error: Directory '$SITE_DIR' does not exist." >&2
    exit 1
fi

echo "★ State Quality Control: Verifying static site integrity..."

python3 - << 'EOF'
import os
import sys
import re

site_dir = os.path.abspath("site")
broken = []
total_checked = 0

pattern = re.compile(r'(?:href|src)=["\']([^"\']+)["\']')

for root, _, files in os.walk(site_dir):
    for f in files:
        if not f.endswith(".html"):
            continue
        html_path = os.path.join(root, f)
        rel_html = os.path.relpath(html_path, site_dir)
        
        with open(html_path, "r", encoding="utf-8") as fp:
            content = fp.read()
            
        for match in pattern.findall(content):
            target = match.strip()
            # Ignore anchor-only or external URLs or mailto/javascript
            if not target or target.startswith(("#", "http://", "https://", "mailto:", "javascript:", "data:")):
                continue
                
            # Strip query params or hash fragments
            clean_target = target.split("?")[0].split("#")[0]
            if not clean_target:
                continue
                
            total_checked += 1
            current_dir = os.path.dirname(html_path)
            resolved = os.path.normpath(os.path.join(current_dir, clean_target))
            
            if not os.path.exists(resolved):
                broken.append((rel_html, target, resolved))

if broken:
    print(f"❌ Found {len(broken)} broken local references:")
    for src, target, resolved in broken:
        print(f"   In '{src}': target '{target}' -> missing: '{resolved}'")
    sys.exit(1)
else:
    print(f"✔ All {total_checked} local asset and page references verified successfully! Zero broken links.")
EOF

echo "★ Site verification passed with socialist distinction!"

#!/usr/bin/env python3

from pathlib import Path
import re
import sys

def generate_anchor(heading: str) -> str:
    # GitHub anchor rules:
    # - Convert to lowercase
    # - Strip surrounding whitespace
    # - Remove punctuation
    # - Replace spaces with dashes
    anchor = heading.strip().lower()
    anchor = re.sub(r'[^\w\s-]', '', anchor)
    anchor = re.sub(r'\s+', '-', anchor)
    return anchor

def build_toc(markdown_path: Path) -> str:
    assert markdown_path.is_file(), f"File does not exist: {markdown_path}"
    lines = markdown_path.read_text(encoding="utf-8").splitlines()

    toc_lines = []
    seen = {}

    for line in lines:
        match = re.match(r'^(#{1,6})\s+(.*)$', line)
        if not match:
            continue

        level, title = match.groups()
        if title.strip().lower() == "table of contents":
            continue  # skip existing TOC title if present

        base_anchor = generate_anchor(title)
        # ensure uniqueness like GitHub does (by suffixing -1, -2, etc.)
        anchor = base_anchor
        if anchor in seen:
            i = seen[anchor] + 1
            while f"{base_anchor}-{i}" in seen:
                i += 1
            anchor = f"{base_anchor}-{i}"
            seen[base_anchor] = i
        seen[anchor] = 0

        indent = "  " * (len(level) - 1)
        toc_lines.append(f"{indent}- [{title}](#{anchor})")

    return "\n".join(toc_lines)

if __name__ == "__main__":
    assert len(sys.argv) == 2, "Usage: build_toc.py README.md"
    path = Path(sys.argv[1])
    toc = build_toc(path)
    print("## Table of Contents\n")
    print(toc)

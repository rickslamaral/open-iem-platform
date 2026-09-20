#!/usr/bin/env python3
"""Validate an interlinked Markdown LLM wiki without third-party dependencies."""
from __future__ import annotations

import argparse
import hashlib
import os
import re
import sys
from pathlib import Path

FRONTMATTER = re.compile(r"\A---\n(.*?)\n---\n", re.S)
WIKILINK = re.compile(r"\[\[([^]|#]+)(?:[|#][^]]*)?\]\]")
FIELD = re.compile(r"^([A-Za-z_][\w-]*):\s*(.*)$")
TAG = re.compile(r"[a-z0-9]+(?:-[a-z0-9]+)*")
TAXONOMY_ITEM = re.compile(r"^\s*-\s*`?([a-z0-9]+(?:-[a-z0-9]+)*)`?\s*(?:#.*)?$")


def safe_read_text(path: Path, wiki: Path) -> str:
    parts = path.relative_to(wiki).parts
    root_fd = os.open(wiki, os.O_RDONLY | os.O_DIRECTORY | getattr(os, "O_NOFOLLOW", 0))
    fd = root_fd
    try:
        for part in parts[:-1]:
            next_fd = os.open(part, os.O_RDONLY | os.O_DIRECTORY | getattr(os, "O_NOFOLLOW", 0), dir_fd=fd)
            if fd != root_fd:
                os.close(fd)
            fd = next_fd
        file_fd = os.open(parts[-1], os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0), dir_fd=fd)
        try:
            return os.fdopen(file_fd, encoding="utf-8").read()
        except BaseException:
            os.close(file_fd)
            raise
    finally:
        os.close(fd)
        if fd != root_fd:
            os.close(root_fd)


def fields(text: str) -> dict[str, str] | None:
    match = FRONTMATTER.match(text)
    if not match:
        return None
    result: dict[str, str] = {}
    for line in match.group(1).splitlines():
        item = FIELD.match(line)
        if not item or not item.group(2).strip():
            return None
        if item.group(1) in result:
            return None
        result[item.group(1)] = item.group(2).strip()
    return result


def slug_candidates(wiki: Path, target: str) -> set[str]:
    normalized = target.strip().replace("\\", "/")
    path = Path(normalized)
    if path.is_absolute() or ".." in path.parts or any(part == "" for part in path.parts):
        return set()
    candidates = {normalized, normalized.removesuffix(".md")}
    if path.suffix != ".md":
        candidates.add(f"{normalized}.md")
    return {str((wiki / candidate).resolve()) for candidate in candidates}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--wiki", required=True, type=Path)
    args = parser.parse_args()
    wiki = args.wiki.expanduser().resolve()
    errors: list[str] = []
    warnings: list[str] = []
    if not wiki.is_dir():
        print(f"ERROR: wiki directory missing: {wiki}")
        return 1

    schema = wiki / "SCHEMA.md"
    index = wiki / "index.md"
    log = wiki / "log.md"
    safe_files = {schema, index, log}
    for candidate in (schema, index, log):
        if candidate.exists() and (candidate.is_symlink() or not candidate.resolve().is_relative_to(wiki)):
            errors.append(f"unsafe path: {candidate.relative_to(wiki)}")
    if not schema.is_file(): errors.append("missing SCHEMA.md")
    if not index.is_file(): errors.append("missing index.md")
    if not log.is_file(): errors.append("missing log.md")

    def walk_error(exc: OSError) -> None:
        errors.append(f"unable to traverse {exc.filename or wiki}: {exc.strerror or exc}")

    for root, dirs, files in os.walk(wiki, followlinks=False, onerror=walk_error):
        for name in [*dirs, *files]:
            entry = Path(root) / name
            if entry.is_symlink() or not entry.resolve().is_relative_to(wiki):
                errors.append(f"unsafe path: {entry.relative_to(wiki)}")
        dirs[:] = [name for name in dirs if not (Path(root) / name).is_symlink()]

    taxonomy: set[str] = set()
    taxonomy_found = False
    if schema.is_file() and not schema.is_symlink() and schema in safe_files and schema.resolve().is_relative_to(wiki):
        try:
            schema_text = safe_read_text(schema, wiki)
        except (OSError, UnicodeError) as exc:
            errors.append(f"SCHEMA.md: unable to read file: {exc}")
        else:
            in_taxonomy = False
            for line in schema_text.splitlines():
                if line.strip().lower().startswith("## tag taxonomy"):
                    taxonomy_found = True
                    in_taxonomy = True
                elif in_taxonomy and line.startswith("## "):
                    in_taxonomy = False
                elif in_taxonomy:
                    item = TAXONOMY_ITEM.fullmatch(line)
                    if item:
                        taxonomy.add(item.group(1))
    if not taxonomy_found or not taxonomy:
        errors.append("SCHEMA.md: missing or empty Tag Taxonomy")

    markdown_files: list[Path] = []
    for root, dirs, files in os.walk(wiki, followlinks=False, onerror=walk_error):
        dirs[:] = [name for name in dirs if not (Path(root) / name).is_symlink()]
        markdown_files.extend(Path(root) / name for name in files if name.endswith(".md"))
    pages = [p for p in markdown_files if not p.is_symlink() and p.resolve().is_relative_to(wiki) and p.is_file() and "raw" not in p.relative_to(wiki).parts and "_archive" not in p.relative_to(wiki).parts and p.name not in {"SCHEMA.md", "index.md", "log.md"}]
    known = {str(p.resolve()) for p in pages if p.resolve().is_relative_to(wiki)} | {str((wiki / "index.md").resolve())}
    inbound: dict[str, int] = {str(p.resolve()): 0 for p in pages}
    indexed = ""
    if index.is_file() and not index.is_symlink() and index in safe_files and index.resolve().is_relative_to(wiki):
        try:
            indexed = safe_read_text(index, wiki)
        except (OSError, UnicodeError) as exc:
            errors.append(f"index.md: unable to read file: {exc}")

    for page in pages:
        try:
            text = safe_read_text(page, wiki)
        except (OSError, UnicodeError) as exc:
            errors.append(f"{page.relative_to(wiki)}: unable to read file: {exc}")
            continue
        fm = fields(text)
        rel = page.relative_to(wiki)
        if not re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*\.md", page.name):
            errors.append(f"{rel}: filename must be lowercase kebab-case")
        if fm is None:
            errors.append(f"{rel}: missing or malformed frontmatter")
        else:
            for required in ("title", "created", "updated", "type", "tags", "sources"):
                if required not in fm:
                    errors.append(f"{rel}: missing frontmatter field {required}")
            if taxonomy and "tags" in fm:
                used = set(TAG.findall(fm["tags"]))
                unknown = sorted(used - taxonomy)
                if unknown:
                    errors.append(f"{rel}: tags outside taxonomy: {', '.join(unknown)}")
        if len(text.splitlines()) > 200:
            warnings.append(f"{rel}: over 200 lines; consider splitting")
        link_count = len(WIKILINK.findall(text))
        if link_count < 2:
            warnings.append(f"{rel}: fewer than two outbound wikilinks; record legitimate exception")
        for target in WIKILINK.findall(text):
            matches = slug_candidates(wiki, target)
            existing = next((candidate for candidate in matches if candidate in known), None)
            if existing is None:
                errors.append(f"{rel}: broken wikilink [[{target}]]")
            elif existing in inbound:
                inbound[existing] += 1
        index_links = set(WIKILINK.findall(indexed))
        page_targets = {str(rel).replace("\\", "/"), str(rel.with_suffix("")).replace("\\", "/")}
        if not any(target in page_targets for target in index_links):
            errors.append(f"{rel}: missing from index.md")

    for target in WIKILINK.findall(indexed):
        if not any(candidate in known for candidate in slug_candidates(wiki, target)):
            errors.append(f"index.md: broken wikilink [[{target}]]")

    for path, count in inbound.items():
        if count == 0:
            warnings.append(f"{Path(path).relative_to(wiki)}: orphan page")

    raw_files: list[Path] = []
    raw_root = wiki / "raw"
    if raw_root.is_dir() and not raw_root.is_symlink():
        for root, dirs, files in os.walk(raw_root, followlinks=False, onerror=walk_error):
            dirs[:] = [name for name in dirs if not (Path(root) / name).is_symlink()]
            raw_files.extend(Path(root) / name for name in files)
    for raw in raw_files:
        if raw.is_symlink() or not raw.resolve().is_relative_to(wiki):
            errors.append(f"{raw.relative_to(wiki)}: unsafe path")
            continue
        if raw.suffix != ".md":
            errors.append(f"{raw.relative_to(wiki)}: unsupported raw source format; use Markdown")
            continue
        if not re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*\.md", raw.name):
            errors.append(f"{raw.relative_to(wiki)}: filename must be lowercase kebab-case")
        try:
            text = safe_read_text(raw, wiki)
        except (OSError, UnicodeError) as exc:
            errors.append(f"{raw.relative_to(wiki)}: unable to read file: {exc}")
            continue
        fm = fields(text)
        if not fm or any(field not in fm for field in ("source_url", "ingested", "sha256")):
            errors.append(f"{raw.relative_to(wiki)}: raw source missing source_url, ingested, or sha256")
            continue
        body = text.split("\n---\n", 1)[1] if "\n---\n" in text else ""
        digest = hashlib.sha256(body.encode()).hexdigest()
        if digest != fm["sha256"]:
            errors.append(f"{raw.relative_to(wiki)}: sha256 mismatch")

    print(f"WIKI: {wiki}")
    print(f"PAGES: {len(pages)}")
    for item in errors: print(f"ERROR: {item}")
    for item in warnings: print(f"WARNING: {item}")
    print(f"ERRORS: {len(errors)}")
    print(f"WARNINGS: {len(warnings)}")
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())

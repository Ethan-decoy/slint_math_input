"""Generate the self-contained Slint catalog from the shared public JSON source."""
import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# The keypad is an input menu, not an exhaustive formatting toolbar. Keep the
# notation API intact while presenting a smaller set of directly selectable topics.
KEYPAD_GROUPS = [
    ("结构", ("基本结构", "重音与注记")),
    ("常用", ("常用函数与数论",)),
    ("微积分", ("微积分",)),
    ("三角", ("三角函数", "双曲与反双曲函数")),
    ("统计概率", ("统计与列表", "概率与分布")),
    ("矩阵", ("线性代数函数", "矩阵与分段")),
    ("希腊字母", ("希腊字母",)),
    ("关系集合", ("关系与集合",)),
    ("符号常数", ("箭头与运算", "常数与字母集")),
]
CORE_ONLY_IDS = {
    "bold", "blackboard", "calligraphic", "fraktur", "italic", "sans-serif", "monospace",
    "underline", "fn-sign", "fn-factorial",
    "fn-asinh", "fn-acosh", "fn-atanh", "fn-asech", "fn-acsch", "fn-acoth",
}


def catalog():
    entries = json.loads((ROOT / "assets/math-catalog.json").read_text(encoding="utf-8"))
    ids = [entry["id"] for entry in entries]
    if len(ids) != len(set(ids)):
        raise ValueError("Duplicate catalog identifiers")
    if not CORE_ONLY_IDS <= set(ids):
        raise ValueError("Unknown core-only catalog identifiers")
    groups = {title: [] for title, _ in KEYPAD_GROUPS}
    target = {source: title for title, sources in KEYPAD_GROUPS for source in sources}
    for entry in entries:
        if entry["id"] not in CORE_ONLY_IDS:
            groups[target[entry["category"]]].append(entry)
    return entries, groups


def render():
    _, groups = catalog()
    quote = lambda value: json.dumps(value, ensure_ascii=False)
    lines = [
        "// Generated from assets/math-catalog.json by tools/generate_catalog.py.",
        "// Keep catalog metadata in the JSON source; regenerate after changes.",
        "export struct CatalogKey { label: string, value: string, description: string }",
        "export struct CatalogSection { title: string, keys: [CatalogKey] }",
        "export global MathCatalog {",
        "    out property <[CatalogSection]> sections: [",
    ]
    for title, items in groups.items():
        lines.append(f"        {{title: {quote(title)}, keys: [")
        for item in items:
            values = {"label": item["label"], "value": "catalog:" + item["id"], "description": item["description"]}
            lines.append("            {" + ", ".join(f"{key}: {quote(value)}" for key, value in values.items()) + "},")
        lines.append("        ]},")
    lines.extend(["    ];", "}", ""])
    return "\n".join(lines)


def reference():
    entries, groups = catalog()
    visible = sum(len(items) for items in groups.values())
    lines = ["# Mathematical input catalog", "",
             f"The **函数** keypad has {visible} buttons in {len(groups)} directly selectable categories. These actions edit notation; they do not calculate or graph.", "",
             "Click a category to show its entries; scroll within that category. Switching categories preserves the formula and restores input focus.",
             "Font-formatting controls and redundant function aliases are omitted from the keypad.", "",
             f"The core retains {len(entries)} notation actions, including the omitted formatting and aliases, for programmatic input and LaTeX interchange.",
             "Inspect `catalog::entries()` in a Rust host.",
             "`Editor::insert_catalog(id)` performs one undoable insertion. The Slint controller accepts `catalog:<id>` tokens.", "",
             "The authoritative metadata is [assets/math-catalog.json](../assets/math-catalog.json).",
             "See [supported editing and interchange](support.md) for behavior, LaTeX and platform verification.", ""]
    for title, items in groups.items():
        lines += [f"## {title}", "", "| Button | Identifier | Description |", "| --- | --- | --- |"]
        for item in items:
            label = item["label"].replace("|", "\\|")
            description = item["description"].replace("|", "\\|")
            lines.append(f"| {label} | `{item['id']}` | {description} |")
        lines.append("")
    lines += ["## Core-only formatting and aliases", "",
              "These entries are available through the core API, not the mouse keypad.", "",
              "| Notation | Identifier | Description |", "| --- | --- | --- |"]
    for item in entries:
        if item["id"] in CORE_ONLY_IDS:
            lines.append(f"| {item['label']} | `{item['id']}` | {item['description']} |")
    lines.append("")
    return "\n".join(lines)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    for relative, source in [("ui/math-catalog.slint", render()), ("docs/catalog.md", reference())]:
        target = ROOT / relative
        if args.check:
            if not target.is_file() or target.read_text(encoding="utf-8") != source:
                raise SystemExit(f"{relative} differs; run tools/generate_catalog.py")
        else:
            target.write_text(source, encoding="utf-8", newline="\n")

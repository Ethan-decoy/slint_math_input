"""Regenerate the deterministic catalog-test font with FontTools 4.60.2.

Source (OFL-1.1): https://github.com/notofonts/noto-cjk/tree/523d033d6cb47f4a80c58a35753646f5c3608a78
Download Sans/OTF/SimplifiedChinese/NotoSansCJKsc-Regular.otf from that revision,
then run: python tools/generate_test_font.py --source /path/to/the/font.otf
The renamed subset is for tests; it is not used by the library at runtime.
"""
import argparse
import hashlib
import json
from pathlib import Path
import fontTools
from fontTools import subset
from fontTools.ttLib import TTFont

ROOT = Path(__file__).resolve().parent.parent
SOURCE_SHA256 = "2c76254f6fc379fddfce0a7e84fb5385bb135d3e399294f6eeb6680d0365b74b"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    args = parser.parse_args()
    assert fontTools.__version__ == "4.60.2", "Use FontTools 4.60.2 for reproducible output."
    assert hashlib.sha256(args.source.read_bytes()).hexdigest() == SOURCE_SHA256
    catalog = json.loads((ROOT / "assets/math-catalog.json").read_text(encoding="utf-8"))
    text = "".join(chr(code) for code in range(32, 127))
    text += "".join(entry["label"] + entry["category"] for entry in catalog)
    text += "".join(path.read_text(encoding="utf-8") for path in (ROOT / "ui").glob("*.slint"))
    options = subset.Options()
    options.name_IDs = ["*"]
    options.name_languages = ["*"]
    options.name_legacy = True
    font = TTFont(args.source, recalcTimestamp=False)
    subsetter = subset.Subsetter(options=options)
    subsetter.populate(text=text)
    subsetter.subset(font)
    # Renaming keeps this modified test fixture distinct from the original font.
    names = {1: "SMI Test CJK", 2: "Regular", 3: "SMITestCJK-2.004", 4: "SMI Test CJK Regular", 6: "SMITestCJK-Regular", 16: "SMI Test CJK", 17: "Regular", 18: "SMI Test CJK Regular", 21: "SMI Test CJK", 22: "Regular"}
    for record in font["name"].names:
        if record.nameID in names:
            record.string = names[record.nameID].encode(record.getEncoding())
    cff = font["CFF "].cff
    cff.fontNames = ["SMITestCJK-Regular"]
    cff.topDictIndex[0].FamilyName = "SMI Test CJK"
    cff.topDictIndex[0].FullName = "SMI Test CJK Regular"
    output = ROOT / "tests/fixtures/noto_sans_cjk_test.otf"
    output.parent.mkdir(parents=True, exist_ok=True)
    font.save(output)
    print(f"Generated {output.name}: {output.stat().st_size} bytes, sha256={hashlib.sha256(output.read_bytes()).hexdigest()}")


if __name__ == "__main__":
    main()

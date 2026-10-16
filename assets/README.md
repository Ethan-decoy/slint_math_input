# Font provenance

`STIXTwoMath-Regular.ttf` is the unmodified STIX Two Math font distributed by
[Google Fonts](https://github.com/google/fonts/tree/main/ofl/stixtwomath), obtained
2026-10-16. Its SHA-256 is
`562551b15b836e6e01d1b7350909baf3c8c8d83260c1190fbf4544333e6936de`.

The copyright and SIL Open Font License accompany it in `OFL-STIX.txt`.
The font is embedded in the Rust layout/renderer so drawing and hit testing use
the same glyph advances. Latin mathematical variables use the font's mathematical
italic glyphs; function names and text spans use upright glyphs.

The Slint keypad also imports this font for its mathematical catalog labels, so
symbols such as double/triple integrals do not depend on an installed UI font.
Ordinary controls retain the host UI font; Slint resolves Chinese catalog captions
through its system font fallback.

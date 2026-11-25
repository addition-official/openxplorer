#!/usr/bin/env python3
"""Runs OpenXplorer's symbol-font reader checks on a font file and says which
one fails. Usage: python3 check-symbol-font.py /path/to/SegoeIcons.ttf
Reads the file only; prints facts about its tables, never its glyph data."""
import struct, sys

def u16(b, at): return struct.unpack_from('>H', b, at)[0]
def i16(b, at): return struct.unpack_from('>h', b, at)[0]
def u32(b, at): return struct.unpack_from('>I', b, at)[0]

path = sys.argv[1]
data = open(path, 'rb').read()
print('size', len(data), '(reader cap 33554432)')
version = data[:4]
print('sfnt version', version.hex(), repr(version))
if version not in (b'\x00\x01\x00\x00', b'true'):
    sys.exit('FAILS: not a TrueType font (the reader takes 00010000 or "true"; "OTTO" means CFF outlines)')
count = u16(data, 4)
tables = {}
for i in range(count):
    rec = 12 + 16 * i
    tables[data[rec:rec+4].decode('latin1')] = (u32(data, rec + 8), u32(data, rec + 12))
print('tables', sorted(tables), '(reader cap 64)')
for need in ('head', 'hhea', 'maxp', 'cmap', 'loca', 'glyf', 'hmtx'):
    if need not in tables:
        sys.exit(f'FAILS: no {need} table')
    off, length = tables[need]
    if off + length > len(data):
        sys.exit(f'FAILS: {need} runs past the end')
head = data[tables['head'][0]:]
upem, loc = u16(head, 18), i16(head, 50)
hhea = data[tables['hhea'][0]:]
asc, desc, metrics = i16(hhea, 4), i16(hhea, 6), u16(hhea, 34)
glyphs = u16(data, tables['maxp'][0] + 4)
print('unitsPerEm', upem, 'indexToLocFormat', loc, 'ascender', asc, 'descender', desc,
      'numberOfHMetrics', metrics, 'numGlyphs', glyphs)
if not 16 <= upem <= 16384: sys.exit('FAILS: unitsPerEm out of 16..16384')
if loc not in (0, 1): sys.exit('FAILS: indexToLocFormat not 0 or 1')
if metrics == 0 or asc <= desc: sys.exit('FAILS: no metrics, or ascender <= descender')
cmap = data[tables['cmap'][0]:]
subs = []
for i in range(min(u16(cmap, 2), 64)):
    rec = 4 + 8 * i
    pid, eid, off = u16(cmap, rec), u16(cmap, rec + 2), u32(cmap, rec + 4)
    subs.append((pid, eid, u16(cmap, off)))
print('cmap subtables (platform, encoding, format):', subs)
rank = {(3, 10): 3, (0, 4): 3, (0, 6): 3, (3, 1): 2, (0, 0): 2, (0, 1): 2, (0, 2): 2, (0, 3): 2, (3, 0): 1}
best = max(((rank[(p, e)], f) for p, e, f in subs if (p, e) in rank), default=None)
print('subtable the reader uses: format', best and best[1])
if not best or best[1] not in (4, 12):
    sys.exit('FAILS: no Unicode cmap subtable in format 4 or 12')
try:
    from fontTools.ttLib import TTFont
except ImportError:
    print('fontTools not installed: glyph checks skipped (pip install fonttools for them)')
    sys.exit(0)
font = TTFont(path)
cm = font.getBestCmap()
for cp in (0xE710, 0xECC8, 0xE8C6, 0xE8C8, 0xE77F, 0xE8AC, 0xE72D, 0xE74D, 0xE8CB, 0xE712, 0xE72B, 0xE711):
    name = cm.get(cp)
    if name is None:
        print(f'U+{cp:04X}: not in the cmap'); continue
    g = font['glyf'][name]
    kind = 'composite' if g.isComposite() else f'{g.numberOfContours} contours'
    extra = ''
    if g.isComposite():
        extra = ' flags ' + ','.join(hex(c.flags) for c in g.components)
    print(f'U+{cp:04X}: {name}, {kind}{extra}')
print('All header checks pass.')

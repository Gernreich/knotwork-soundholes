#!/usr/bin/env python3
"""The thirteen rosettes in 3D, each cut through a patch of 3mm plywood.

    python3 viewer.py [--out=turn.html]

A rosette is a cut-out, not a part: every closed path in the cut layer is
waste, and the ribbon is the board that is left. So the cut file has no outline
for lasermade-tools/turn3d.py to find -- it reads each waste region as a part of
its own. Here they become the holes in a square of soundboard instead, at the
3mm the writeups assume (3mm Baltic birch), and the blue engrave lines that mark
the over/under at each crossing are drawn on the face.

The square runs a margin past the rim (MARGIN, or a fifth of the radius on the
big ones) so the anchors read as board running on into board, which is the
point the previews make in two golds.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, '..', 'lasermade-tools'))
import turn3d as t   # noqa: E402

ROSETTES = [('2 × 3 trefoil', '2-lead_3-bight_knot_radius30mm.svg'),
            ('2 × 5 cinquefoil', '2-lead_5-bight_knot_radius30mm.svg'),
            ('2 × 7 septafoil', '2-lead_7-bight_knot_radius30mm.svg'),
            ('3 × 2', '3-lead_2-bight_knot_radius30mm.svg'),
            ('3 × 4', '3-lead_4-bight_knot_radius30mm.svg'),
            ('3 × 5', '3-lead_5-bight_knot_radius30mm.svg'),
            ('4 × 3 · 39mm', '4-lead_3-bight_knot_radius39mm.svg'),
            ('4 × 5 · 39mm', '4-lead_5-bight_knot_radius39mm.svg'),
            ('5 × 4 · 60mm', '5-lead_4-bight_knot_radius60mm.svg'),
            ('9 × 11 · 300mm', '9-lead_11-bight_knot_radius300mm.svg'),
            ('plait · 8', '8-crossing_plait_radius30mm.svg'),
            ('plait · 10', '10-crossing_plait_radius30mm.svg'),
            ('plait · 10 · 50mm', '10-crossing_plait_radius50mm.svg')]
MARGIN = 12   # mm of board past the rim


def board(svg):
    """-> one part: a square of soundboard with the cut layer's waste as its holes."""
    waste = t.parts(svg)
    # a waste region with a hole in it would be an island of ribbon that drops
    # out with it -- the one thing these designs are built not to do
    assert all(not w['h'] for w in waste), svg
    root = t.ET.parse(svg).getroot()
    reach = float(root.get('viewBox').split()[2]) / 2   # rim + bite + 0.5, as preview.js reads it
    s = round(reach + max(MARGIN, reach / 5))
    engr = [[(x, -y) for x, y in pts] for c, pts, _ in t.fpc.collect(root) if c == t.ENGRAVE]
    return {'o': [(-s, -s), (s, -s), (s, s), (-s, s)], 'h': [w['o'] for w in waste],
            'e': engr, 'box': (-s, -s, s, s)}, len(waste)


def build():
    out = {}
    for name, f in ROSETTES:
        b, n = board(os.path.join(HERE, f))
        out[name] = {'parts': [t.placed(b, t.T(0, 0, 0))], 'ex': 0, 'view': [-0.35, 0.75]}
    return out


def main(argv):
    o = [a.split('=', 1)[1] for a in argv if a.startswith('--out=')]
    path = o[0] if o else os.path.join(HERE, 'turn.html')
    t.write(path, 'Knotwork Sound Holes', build(),
            'Drag to turn; right-drag, shift-drag or move two fingers to slide the view; '
            'scroll or pinch to zoom in on whatever is under the pointer; double-click to turn '
            'round that point, and Reset view to go back. Pick a rosette above. Each is its cut '
            'file’s waste taken out of a square of 3mm plywood soundboard, with the engrave lines '
            'that mark the over/under on the face. The ribbon is the board that is left, and it '
            'runs on into the board round the rim at every anchor.',
            'knotwork-soundholes/viewer.py')
    print(f'  wrote {os.path.relpath(path)}')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))

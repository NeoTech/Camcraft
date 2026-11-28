#!/usr/bin/env python3
"""Regenerate rest.obj, the model of the catalog case 3d/rest.

The part is an original CamCraft fixture: a 60 x 50 x 4 mm plate with two
parallel ribs on top. Each rib is 13 mm wide (X), 30 mm long (Y) and 10 mm
high, so its top is at Z14. The ribs stand at X -17..-4 and X 4..17. The
channel between them is 8 mm wide: a 12 mm end mill cannot enter it, a
6 mm end mill can.

Method: the top is a set of flat rectangles on the rectilinear grid of the
feature breakpoints. Each cell is the plate top (Z4) or a rib top (Z14).
Where two neighbour cells have different heights, a vertical wall joins
them. Four side walls and a bottom fan (Z0) close the solid. The script
checks that each edge is used exactly twice, in opposite directions,
before it writes.

Usage: python gen_rest_obj.py [output.obj]
"""

import sys

XS = [-30.0, -17.0, -4.0, 4.0, 17.0, 30.0]   # plate edges and rib walls
YS = [-25.0, -15.0, 15.0, 25.0]              # plate edges and rib ends
PLATE = 4.0                                  # plate top
RIB = 14.0                                   # rib top


def cell_height(i, j):
    """Top Z of grid cell (i, j): the ribs are cells X 1 and 3, Y 1."""
    return RIB if i in (1, 3) and j == 1 else PLATE


def build():
    nx, ny = len(XS) - 1, len(YS) - 1
    tris = []

    def quad(a, b, c, d):
        tris.extend([(a, b, c), (a, c, d)])

    # Cell tops, counter-clockwise from +Z.
    for i in range(nx):
        for j in range(ny):
            z = cell_height(i, j)
            x0, x1, y0, y1 = XS[i], XS[i + 1], YS[j], YS[j + 1]
            quad((x0, y0, z), (x1, y0, z), (x1, y1, z), (x0, y1, z))
    # Vertical walls between neighbour cells of different heights. The
    # wall faces the low cell.
    for i in range(nx - 1):
        for j in range(ny):
            za, zb = cell_height(i, j), cell_height(i + 1, j)
            if za == zb:
                continue
            x, y0, y1 = XS[i + 1], YS[j], YS[j + 1]
            if za < zb:   # wall faces -X
                quad((x, y1, za), (x, y0, za), (x, y0, zb), (x, y1, zb))
            else:         # wall faces +X
                quad((x, y0, zb), (x, y1, zb), (x, y1, za), (x, y0, za))
    for i in range(nx):
        for j in range(ny - 1):
            za, zb = cell_height(i, j), cell_height(i, j + 1)
            if za == zb:
                continue
            y, x0, x1 = YS[j + 1], XS[i], XS[i + 1]
            if za < zb:   # wall faces -Y
                quad((x0, y, za), (x1, y, za), (x1, y, zb), (x0, y, zb))
            else:         # wall faces +Y
                quad((x1, y, zb), (x0, y, zb), (x0, y, za), (x1, y, za))
    # Side walls and bottom fan: walk the boundary counter-clockwise seen
    # from +Z. The boundary cells are all plate cells (Z4).
    ring = ([(XS[i], YS[0]) for i in range(nx)] + [(XS[nx], YS[j]) for j in range(ny)]
            + [(XS[i], YS[ny]) for i in range(nx, 0, -1)] + [(XS[0], YS[j]) for j in range(ny, 0, -1)])
    centre = (0.0, 0.0, 0.0)
    for n, (x0, y0) in enumerate(ring):
        x1, y1 = ring[(n + 1) % len(ring)]
        quad((x0, y0, 0.0), (x1, y1, 0.0), (x1, y1, PLATE), (x0, y0, PLATE))
        tris.append((centre, (x1, y1, 0.0), (x0, y0, 0.0)))   # faces -Z
    return tris


def check(tris):
    edges = {}
    for t in tris:
        for n in range(3):
            e = (t[n], t[(n + 1) % 3])
            edges[e] = edges.get(e, 0) + 1
    for (p, q), count in edges.items():
        if count != 1 or edges.get((q, p)) != 1:
            raise SystemExit(f"mesh is not closed at edge {p} -> {q}")


def write(tris, path):
    verts, index, faces = [], {}, []
    for t in tris:
        face = []
        for p in t:
            if p not in index:
                index[p] = len(verts) + 1
                verts.append(p)
            face.append(index[p])
        faces.append(face)
    with open(path, "w", newline="\n") as f:
        f.write("# rest: 60 x 50 x 4 mm plate with two 13 x 30 x 10 mm ribs and an 8 mm channel\n")
        for v in verts:
            f.write("v {:g} {:g} {:g}\n".format(*v))
        for face in faces:
            f.write("f " + " ".join(str(n) for n in face) + "\n")


if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "rest.obj"
    mesh = build()
    check(mesh)
    write(mesh, out)

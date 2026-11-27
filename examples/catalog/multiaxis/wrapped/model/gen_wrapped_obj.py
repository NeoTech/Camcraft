#!/usr/bin/env python3
# Provenance: original grooved cylinder: a round-bottom ring groove that wraps once about the X axis.
"""Generate a closed cylinder along X with one wrapped ring groove."""
import math
from pathlib import Path

RADIUS = 10.0         # cylinder radius, mm
HALF_LENGTH = 10.0    # the cylinder spans X-10 to X10
GROOVE_X = 0.0        # the groove centre plane, x = GROOVE_X
GROOVE_RADIUS = 5.0   # radius of the round groove profile
GROOVE_DEPTH = 1.5    # radial depth at the groove centre
SECTORS = 36          # 10 degree steps around the axis
FINE = 1.0            # axial step where the groove runs


def x_stations():
    """Coarse steps on the plain ends, fine steps over the groove band."""
    band = GROOVE_RADIUS + 1.0
    fine = [GROOVE_X - band + FINE * i for i in range(int(round(2 * band / FINE)) + 1)]
    coarse = [-HALF_LENGTH, -HALF_LENGTH + 5.0, HALF_LENGTH - 5.0, HALF_LENGTH]
    return sorted(set(round(x, 9) for x in coarse + fine))


def radius(x):
    """The skin radius: the cylinder, less the round groove profile."""
    distance = abs(x - GROOVE_X)
    if distance >= GROOVE_RADIUS:
        return RADIUS
    sink = math.sqrt(GROOVE_RADIUS ** 2 - distance ** 2) - (GROOVE_RADIUS - GROOVE_DEPTH)
    return RADIUS - max(0.0, sink)


def point(x, j):
    theta = 2.0 * math.pi * j / SECTORS
    r = radius(x)
    return (round(x, 8), round(r * math.cos(theta), 8), round(r * math.sin(theta), 8))


def mesh():
    xs = x_stations()
    grid = [[point(x, j) for j in range(SECTORS)] for x in xs]
    triangles = []
    for i in range(len(xs) - 1):
        for j in range(SECTORS):
            k = (j + 1) % SECTORS
            a, b, c, d = grid[i][j], grid[i][k], grid[i + 1][k], grid[i + 1][j]
            # Outward normals: theta first, then +X.
            triangles.append((a, b, c))
            triangles.append((a, c, d))
    low, high = (xs[0], 0.0, 0.0), (xs[-1], 0.0, 0.0)
    for j in range(SECTORS):
        k = (j + 1) % SECTORS
        triangles.append((high, grid[-1][j], grid[-1][k]))
        triangles.append((low, grid[0][k], grid[0][j]))
    check(triangles)
    return triangles


def check(triangles):
    """Every directed edge occurs once and has one reverse edge; normals face out."""
    edges = set()
    for tri in triangles:
        u = [tri[1][i] - tri[0][i] for i in range(3)]
        v = [tri[2][i] - tri[0][i] for i in range(3)]
        n = (u[1]*v[2]-u[2]*v[1], u[2]*v[0]-u[0]*v[2], u[0]*v[1]-u[1]*v[0])
        if sum(c * c for c in n) < 1e-14:
            raise ValueError("degenerate triangle")
        centre = [sum(p[i] for p in tri) / 3.0 for i in range(3)]
        outward = (centre[0] if abs(abs(centre[0]) - HALF_LENGTH) < 1e-9 else 0.0,
                   centre[1], centre[2])
        if sum(n[i] * outward[i] for i in range(3)) <= 0.0:
            raise ValueError("inward facing triangle")
        for edge in zip(tri, tri[1:] + tri[:1]):
            if edge in edges:
                raise ValueError("repeated directed edge")
            edges.add(edge)
    if any((q, p) not in edges for p, q in edges):
        raise ValueError("mesh has an open edge")


def main():
    triangles = mesh()
    points = sorted(set(p for tri in triangles for p in tri))
    ids = {p: i + 1 for i, p in enumerate(points)}
    lines = ["# Original grooved cylinder: 20 mm x 20 mm along X; "
             "ring groove at X0, 5 mm profile radius, 1.5 mm deep."]
    lines += ["v %.8f %.8f %.8f" % p for p in points]
    lines += ["f " + " ".join(str(ids[p]) for p in tri) for tri in triangles]
    out = Path(__file__).with_name("wrapped.obj")
    out.write_text("\n".join(lines) + "\n")
    print(f"{out.name}: {len(points)} vertices, {len(triangles)} triangles; closed oriented mesh")


if __name__ == "__main__":
    main()

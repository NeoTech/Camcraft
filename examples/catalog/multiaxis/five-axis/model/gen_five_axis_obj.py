#!/usr/bin/env python3
"""Provenance: original faceted cylinder from analytic geometry."""
import math
from pathlib import Path


def write_mesh(name, triangles):
    lines = ["# Original five-axis catalog geometry, millimetres."]
    vertices = {}
    faces = []
    for triangle in triangles:
        face = []
        for x, y, z in triangle:
            key = tuple(round(value, 9) for value in (x, y, z))
            if key not in vertices:
                vertices[key] = len(vertices) + 1
            face.append(vertices[key])
        faces.append(face)
    lines.extend(f"v {x:.9f} {y:.9f} {z:.9f}" for x, y, z in vertices)
    lines.extend(f"f {a} {b} {c}" for a, b, c in faces)
    Path(__file__).with_name(name + ".obj").write_text("\n".join(lines) + "\n")


def wall():
    """Closed radius-12 cylinder with straight rulings parallel to X."""
    def point(j, i):
        angle = math.tau * j / 16
        return (-24 + 48 * i / 6, 12 * math.cos(angle), 12 * math.sin(angle))

    triangles = []
    for j in range(16):
        for i in range(6):
            a, b = point(j, i), point((j + 1) % 16, i)
            c, d = point((j + 1) % 16, i + 1), point(j, i + 1)
            triangles.extend([(a, b, c), (a, c, d)])
        triangles.extend([
            ((24, 0, 0), point(j, 6), point((j + 1) % 16, 6)),
            ((-24, 0, 0), point((j + 1) % 16, 0), point(j, 0)),
        ])
    return triangles


if __name__ == "__main__":
    write_mesh("five-axis", wall())

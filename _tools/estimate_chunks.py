"""Y早期スキップ後のチャンク数を見積もる (改善版: 地表面チャンク+上下1層)"""
import math

CHUNK_SIZE = 8.0
DRAW_DIST = 20

def terrain_height(gx, gz):
    h_base = math.sin(gx * 0.08) * 5.0 + math.cos(gz * 0.08) * 5.0
    h_hills = math.sin(gx * 0.2) * math.cos(gz * 0.15) * 3.0
    h_detail = math.sin(gx * 0.4) * 1.5 + math.cos(gz * 0.35) * 1.5
    h_micro = math.sin(gx * 0.8) * 0.5 + math.cos(gz * 0.7) * 0.5
    mountain = abs(math.sin(gx * 0.03) * math.cos(gz * 0.04)) * 12.0
    return 6.0 + h_base + h_hills + h_detail + h_micro + mountain

center_cx = 1
center_cz = 1

total_chunks = 0
xz_positions = 0
y_layers_hist = {}

for dx in range(-DRAW_DIST, DRAW_DIST + 1):
    for dz in range(-DRAW_DIST, DRAW_DIST + 1):
        if dx*dx + dz*dz > DRAW_DIST * DRAW_DIST:
            continue

        cx = center_cx + dx
        cz = center_cz + dz
        xz_positions += 1

        wx = cx * CHUNK_SIZE
        wz = cz * CHUNK_SIZE
        half = CHUNK_SIZE * 0.5
        max_h = -100
        min_h = 100
        for sx in [wx, wx + half, wx + CHUNK_SIZE]:
            for sz in [wz, wz + half, wz + CHUNK_SIZE]:
                h = terrain_height(sx, sz)
                max_h = max(max_h, h)
                min_h = min(min_h, h)

        cy_surface_min = math.floor(min_h / CHUNK_SIZE)
        cy_surface_max = math.floor(max_h / CHUNK_SIZE)
        cy_bottom = max(cy_surface_min - 1, -1)
        cy_top = cy_surface_max + 1

        layers = cy_top - cy_bottom + 1
        total_chunks += layers
        y_layers_hist[layers] = y_layers_hist.get(layers, 0) + 1

print(f"XZ positions: {xz_positions}")
print(f"Total chunks (surface-based skip): {total_chunks}")
print(f"Total chunks (old, 1 layer cy=0): {xz_positions}")
print(f"Ratio vs old: {total_chunks / xz_positions:.1f}x")
print(f"Average Y layers per XZ: {total_chunks / xz_positions:.1f}")
print(f"\nY layer distribution:")
for layers in sorted(y_layers_hist.keys()):
    print(f"  {layers} layers: {y_layers_hist[layers]} positions")

fps = 60
mesh_per_frame = 8
print(f"\nLoad time estimate:")
print(f"  {total_chunks} chunks / {mesh_per_frame} per frame / {fps} fps = {total_chunks / mesh_per_frame / fps:.1f} sec")
print(f"  (old cy=0: {xz_positions} / 1 / {fps} = {xz_positions / 1 / fps:.1f} sec)")

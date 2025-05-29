with open("palettes/smooth.pal", "rb") as f:
    data = f.read()

if len(data) < 192:
    raise ValueError(f"Palette file too short: {len(data)} bytes")

palette = [
    (data[i], data[i+1], data[i+2])
    for i in range(0, 192, 3)
]

print("pub const NES_PALETTE: [(u8, u8, u8); 64] = [")
for (r, g, b) in palette:
    print(f"    ({r}, {g}, {b}),")
print("];")

palette = 'base'

with open(f"palettes/{palette}.pal", "rb") as f:
    data = f.read()

if len(data) < 192:
    raise ValueError(f"Palette file too short: {len(data)} bytes")

palette = [
    (data[i], data[i+1], data[i+2])
    for i in range(0, 192, 3)
]

print("// THIS FILE IS GENERATED AT BUILD TIME. ANY CHANGES MADE WILL BE LOST.")
print()
print("// THIS PROJECT CURRENTLY USES base.pal FROM\n\
// https://en.wikipedia.org/wiki/List_of_video_game_console_palettes.")
print()
print("// TO CHANGE THE PALETTE (e.g., \"test.pal\"),")
print("// 1. COPY test.pal INTO /palettes/")
print("// 2. EDIT /scripts/load-palette.py TO SET `palette = \"test\"`")
print()
print("// TODO: MAKE PALETTE SWAPPING EASIER.")
print()
print("pub const NES_PALETTE: [(u8, u8, u8); 64] = [")
for (r, g, b) in palette:
    print(f"    ({r}, {g}, {b}),")
print("];")

from PIL import Image
import numpy as np
import sys

if len(sys.argv) != 2:
    print("Usage: python image-to-data.py <image-path>")
    exit(1)

try:
    im = Image.open(sys.argv[1])
except:
    print("Unable to open image file")
    exit(1)

if im.mode != "P":
    print("Image is not in palette mode")
    exit(1)

palette = np.reshape(np.array(im.getpalette()), (-1, 3))
print(palette)

pixels = im.getdata()

with open("../assets/sprite.mif", "w") as file:
    file.write(f"""WIDTH = 3;
DEPTH = {len(pixels)};
ADDRESS_RADIX = HEX;
DATA_RADIX = BIN;

CONTENT BEGIN

""")
    for i, pixel in enumerate(pixels):
        file.write(f"{i:05x} : {pixel:03b};\n")
    file.write("\nEND;")

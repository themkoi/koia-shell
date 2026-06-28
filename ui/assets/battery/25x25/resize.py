import os
import re
import sys

TARGET_WIDTH = "25"
TARGET_HEIGHT = "25"

# optional: update viewBox if it's exactly 20x20
VIEWBOX_PATTERN = re.compile(r'viewBox="0\s+0\s+20\s+20"')

def update_svg(file_path):
    with open(file_path, "r", encoding="utf-8") as f:
        content = f.read()

    original = content

    # Replace width
    content = re.sub(r'width="[^"]+"', f'width="{TARGET_WIDTH}"', content)

    # Replace height
    content = re.sub(r'height="[^"]+"', f'height="{TARGET_HEIGHT}"', content)

    # Update viewBox if it's exactly 20x20
    content = VIEWBOX_PATTERN.sub('viewBox="0 0 24 24"', content)

    # If no width/height exist, inject them into <svg ...>
    if "<svg" in content and "width=" not in original:
        content = content.replace("<svg", f'<svg width="{TARGET_WIDTH}" height="{TARGET_HEIGHT}"', 1)

    with open(file_path, "w", encoding="utf-8") as f:
        f.write(content)

    print(f"Updated: {file_path}")


def process_folder(folder):
    for root, _, files in os.walk(folder):
        for name in files:
            if name.lower().endswith(".svg"):
                update_svg(os.path.join(root, name))


if __name__ == "__main__":
    folder = sys.argv[1] if len(sys.argv) > 1 else "."
    process_folder(folder)

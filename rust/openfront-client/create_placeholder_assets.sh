#!/bin/bash

# Create placeholder sprite images using ImageMagick convert command
# If ImageMagick is not installed, these will be simple colored squares

cd assets/sprites

# Create placeholder sprites (64x64 colored squares)
for sprite in capital city camp tower fort port airport radar bridge; do
    # Try to use ImageMagick if available
    if command -v convert &> /dev/null; then
        case $sprite in
            capital) color="red" ;;
            city) color="orange" ;;
            camp) color="green" ;;
            tower) color="gray" ;;
            fort) color="darkgray" ;;
            port) color="blue" ;;
            airport) color="skyblue" ;;
            radar) color="purple" ;;
            bridge) color="brown" ;;
        esac
        convert -size 64x64 xc:$color $sprite.png
    else
        # Create a minimal 1x1 PNG as placeholder
        echo -ne '\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x02\x00\x00\x00\x90wS\xde\x00\x00\x00\x0cIDATx\x9cc\xf8\x0f\x00\x00\x01\x01\x00\x05W\xcd\xca+\x00\x00\x00\x00IEND\xaeB`\x82' > $sprite.png
    fi
done

echo "Placeholder sprites created in assets/sprites/"

# Download or create a font file
cd ../fonts
if command -v curl &> /dev/null; then
    # Try to download a free font
    curl -L "https://github.com/mozilla/Fira/raw/master/ttf/FiraSans-Bold.ttf" -o FiraSans-Bold.ttf 2>/dev/null || {
        echo "Could not download font, creating placeholder"
        touch FiraSans-Bold.ttf
    }
else
    touch FiraSans-Bold.ttf
fi

echo "Font placeholder created in assets/fonts/"
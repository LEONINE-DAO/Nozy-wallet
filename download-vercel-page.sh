#!/bin/bash
# Bash script to download Vercel landing page and assets

VERCEL_URL="https://nozy-wallet.vercel.app/"
OUTPUT_DIR="vercel-landing"

echo "📥 Downloading landing page from Vercel..."

# Create output directory
mkdir -p "$OUTPUT_DIR/assets"
HTML_FILE="$OUTPUT_DIR/index.html"

# Download HTML
echo "Downloading HTML..."
if curl -L "$VERCEL_URL" -o "$HTML_FILE"; then
    echo "✅ HTML downloaded to $HTML_FILE"
else
    echo "❌ Failed to download HTML"
    exit 1
fi

# Extract and download CSS files
echo ""
echo "📦 Extracting CSS files..."
CSS_URLS=$(grep -oP 'href=["'"'"']([^"'"'"']*\.css[^"'"'"']*)["'"'"']' "$HTML_FILE" | sed -E "s/href=[\"']([^\"']+)['\"]/\1/" | sort -u)
CSS_COUNT=$(echo "$CSS_URLS" | grep -c . || echo "0")
echo "Found $CSS_COUNT CSS files"

echo "$CSS_URLS" | while read -r css_url; do
    if [ -n "$css_url" ]; then
        # Make absolute URL if relative
        if [[ ! "$css_url" =~ ^http ]]; then
            css_url="https://nozy-wallet.vercel.app$css_url"
        fi
        css_file=$(basename "$css_url")
        echo "  Downloading: $css_file"
        curl -L "$css_url" -o "$OUTPUT_DIR/assets/$css_file" 2>/dev/null || echo "    ⚠️  Failed"
    fi
done

# Extract and download JS files
echo ""
echo "📦 Extracting JS files..."
JS_URLS=$(grep -oP 'src=["'"'"']([^"'"'"']*\.js[^"'"'"']*)["'"'"']' "$HTML_FILE" | sed -E "s/src=[\"']([^\"']+)['\"]/\1/" | sort -u)
JS_COUNT=$(echo "$JS_URLS" | grep -c . || echo "0")
echo "Found $JS_COUNT JS files"

echo "$JS_URLS" | while read -r js_url; do
    if [ -n "$js_url" ]; then
        # Make absolute URL if relative
        if [[ ! "$js_url" =~ ^http ]]; then
            js_url="https://nozy-wallet.vercel.app$js_url"
        fi
        js_file=$(basename "$js_url")
        echo "  Downloading: $js_file"
        curl -L "$js_url" -o "$OUTPUT_DIR/assets/$js_file" 2>/dev/null || echo "    ⚠️  Failed"
    fi
done

# Extract and download images
echo ""
echo "📦 Extracting image files..."
IMG_URLS=$(grep -oP 'src=["'"'"']([^"'"'"']*\.(png|jpg|jpeg|svg|gif|webp)[^"'"'"']*)["'"'"']' "$HTML_FILE" | sed -E "s/src=[\"']([^\"']+)['\"]/\1/" | sort -u)
IMG_COUNT=$(echo "$IMG_URLS" | grep -c . || echo "0")
echo "Found $IMG_COUNT image files"

echo "$IMG_URLS" | while read -r img_url; do
    if [ -n "$img_url" ]; then
        # Make absolute URL if relative
        if [[ ! "$img_url" =~ ^http ]]; then
            img_url="https://nozy-wallet.vercel.app$img_url"
        fi
        img_file=$(basename "$img_url")
        echo "  Downloading: $img_file"
        curl -L "$img_url" -o "$OUTPUT_DIR/assets/$img_file" 2>/dev/null || echo "    ⚠️  Failed"
    fi
done

echo ""
echo "✅ Download complete!"
echo "Files saved to: $OUTPUT_DIR"
echo ""
echo "Next steps:"
echo "1. Review the files in $OUTPUT_DIR"
echo "2. Copy index.html to repo root (replace existing)"
echo "3. Copy assets to assets/ folder"
echo "4. Update paths in index.html if needed"
echo "5. Commit and push"


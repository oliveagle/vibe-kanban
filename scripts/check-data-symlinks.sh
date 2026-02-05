#!/bin/bash
# Check for symlinks in VK data directory that point outside the directory
# These symlinks will break inside the container because the target path doesn't exist

set -e

VK_DATA_DIR="${1:-$HOME/.local/share/vibe-kanban}"

if [ ! -d "$VK_DATA_DIR" ]; then
    echo "❌ Data directory does not exist: $VK_DATA_DIR"
    exit 1
fi

echo "Checking for problematic symlinks in: $VK_DATA_DIR"
echo ""

FOUND_ISSUES=0

find "$VK_DATA_DIR" -type l 2>/dev/null | while IFS= read -r symlink; do
    if [ -z "$symlink" ]; then
        continue
    fi
    
    target=$(readlink "$symlink")
    abs_target=$(cd "$(dirname "$symlink")" 2>/dev/null && realpath -m "$target" 2>/dev/null || echo "$target")
    
    case "$abs_target" in
        "$VK_DATA_DIR"*)
            echo "✅ OK: $symlink → $target (within data directory)"
            ;;
        *)
            echo "❌ PROBLEM: $symlink → $target"
            echo "   Target is OUTSIDE the data directory. This will break in the container!"
            FOUND_ISSUES=$((FOUND_ISSUES + 1))
            ;;
    esac
done

echo ""

if [ $FOUND_ISSUES -eq 0 ]; then
    echo "✅ No problematic symlinks found. All symlinks are within the data directory."
    exit 0
else
    echo "❌ Found $FOUND_ISSUES problematic symlink(s)."
    echo ""
    echo "To fix these issues, you can:"
    echo "  1. Replace symlinks with actual files/directories"
    echo "  2. Move the target into the data directory and recreate the symlink"
    echo "  3. Use bind mounts instead of symlinks (for directories)"
    echo ""
    echo "Example fix for a file:"
    echo "  cp \$(readlink symlink_name) symlink_name"
    echo ""
    echo "Example fix for a directory:"
    echo "  cp -rL \$(readlink symlink_name) symlink_name"
    exit 1
fi

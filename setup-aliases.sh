#!/bin/bash

# Cube.js Aliases Setup Script
# This script automatically sets up the .cube-aliases in your shell configuration

set -e

COLOR_BLUE='\033[0;34m'
COLOR_GREEN='\033[0;32m'
COLOR_YELLOW='\033[1;33m'
COLOR_RED='\033[0;31m'
COLOR_RESET='\033[0m'

echo -e "${COLOR_BLUE}╔════════════════════════════════════════════════════════════════╗${COLOR_RESET}"
echo -e "${COLOR_BLUE}║       Cube.js Development Aliases Setup                      ║${COLOR_RESET}"
echo -e "${COLOR_BLUE}╚════════════════════════════════════════════════════════════════╝${COLOR_RESET}"
echo ""

# Detect shell
SHELL_NAME=$(basename "$SHELL")
ALIASES_FILE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/.cube-aliases"

echo -e "${COLOR_YELLOW}Detected shell: $SHELL_NAME${COLOR_RESET}"
echo -e "${COLOR_YELLOW}Aliases file: $ALIASES_FILE${COLOR_RESET}"
echo ""

# Verify aliases file exists
if [ ! -f "$ALIASES_FILE" ]; then
  echo -e "${COLOR_RED}Error: .cube-aliases file not found at $ALIASES_FILE${COLOR_RESET}"
  exit 1
fi

echo -e "${COLOR_GREEN}✓ Aliases file found${COLOR_RESET}"
echo ""

# Function to add source to config file
add_source_to_config() {
  local config_file=$1
  local shell_name=$2

  if [ ! -f "$config_file" ]; then
    echo -e "${COLOR_YELLOW}Creating $config_file...${COLOR_RESET}"
    touch "$config_file"
  fi

  # Check if already added
  if grep -q "source.*\.cube-aliases" "$config_file" 2>/dev/null; then
    echo -e "${COLOR_YELLOW}Already configured in $config_file${COLOR_RESET}"
    return 0
  fi

  # Add source command
  cat >> "$config_file" <<EOF

# Cube.js Development Aliases
if [ -f "$ALIASES_FILE" ]; then
  source "$ALIASES_FILE"
fi
EOF

  echo -e "${COLOR_GREEN}✓ Added to $config_file${COLOR_RESET}"
  return 0
}

# Set up based on detected shell
case "$SHELL_NAME" in
  bash)
    CONFIG_FILE="$HOME/.bashrc"
    add_source_to_config "$CONFIG_FILE" "bash"
    RELOAD_CMD="source ~/.bashrc"
    ;;
  zsh)
    CONFIG_FILE="$HOME/.zshrc"
    add_source_to_config "$CONFIG_FILE" "zsh"
    RELOAD_CMD="source ~/.zshrc"
    ;;
  fish)
    CONFIG_FILE="$HOME/.config/fish/config.fish"
    mkdir -p "$HOME/.config/fish"
    if [ ! -f "$CONFIG_FILE" ]; then
      touch "$CONFIG_FILE"
    fi
    if ! grep -q "source.*\.cube-aliases" "$CONFIG_FILE" 2>/dev/null; then
      cat >> "$CONFIG_FILE" <<EOF

# Cube.js Development Aliases
if test -f "$ALIASES_FILE"
  source "$ALIASES_FILE"
end
EOF
      echo -e "${COLOR_GREEN}✓ Added to $CONFIG_FILE${COLOR_RESET}"
    else
      echo -e "${COLOR_YELLOW}Already configured in $CONFIG_FILE${COLOR_RESET}"
    fi
    RELOAD_CMD="source ~/.config/fish/config.fish"
    ;;
  *)
    echo -e "${COLOR_YELLOW}Unsupported shell: $SHELL_NAME${COLOR_RESET}"
    echo "Please manually add the following to your shell configuration:"
    echo ""
    echo "if [ -f \"$ALIASES_FILE\" ]; then"
    echo "  source \"$ALIASES_FILE\""
    echo "fi"
    echo ""
    exit 1
    ;;
esac

echo ""
echo -e "${COLOR_GREEN}✓ Setup complete!${COLOR_RESET}"
echo ""
echo "To use the aliases, run:"
echo -e "  ${COLOR_BLUE}$RELOAD_CMD${COLOR_RESET}"
echo ""
echo "Or restart your terminal."
echo ""
echo "Then verify it worked with:"
echo -e "  ${COLOR_BLUE}cube-help${COLOR_RESET}"
echo ""

#!/bin/bash

echo "🚀 Bootstrapping Graph-OS development environment..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "📦 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo "📦 Installing Node.js 20..."
    # For macOS with Homebrew
    if command -v brew &> /dev/null; then
        brew install node@20
    else
        echo "❌ Please install Node.js 20 manually from https://nodejs.org/"
        exit 1
    fi
fi

# Check if pnpm is installed
if ! command -v pnpm &> /dev/null; then
    echo "📦 Installing pnpm..."
    npm install -g pnpm@9
fi

# Install Rust tools
echo "🔧 Installing Rust development tools..."
cargo install cargo-watch

# Install dependencies
echo "📦 Installing Node dependencies..."
pnpm install

# Build Rust crates
echo "🔨 Building Rust crates..."
cargo build --all

echo "✅ Bootstrap complete!"
echo ""
echo "🎯 Next steps:"
echo "  1. Initialize the todo system: cargo run -p todo-cli -- init"
echo "  2. Add your first task: cargo run -p todo-cli -- add \"Bootstrap repo skeleton\""
echo "  3. List tasks: cargo run -p todo-cli -- ls"
echo "  4. Start desktop app: pnpm --filter desktop-tauri tauri dev"
echo ""

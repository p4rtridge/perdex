default:
    just --list

# Lint
lint:
    cargo clippy & cargo check


# Install dev tools
install-tools:
    sudo apt install valgrind & \
    sudo apt install gnuplot & \
    sudo apt install massif-visualizer & \
    cargo install just & \
    cargo install --locked cargo-llvm-cov & \
    cargo install --locked cargo-nextest

# Test with coverage
test package="" *args:
    @if [ -z "{{package}}" ]; then \
        cargo llvm-cov nextest --workspace {{args}}; \
    else \
        cargo llvm-cov nextest -p {{package}} {{args}}; \
    fi
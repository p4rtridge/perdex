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
    cargo install --locked cargo-tarpaulin

# Test with coverage
test package="" *args:
    @if [ -z "{{package}}" ]; then \
        cargo tarpaulin --workspace {{args}}; \
    else \
        cargo tarpaulin -p {{package}} {{args}}; \
    fi
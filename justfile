default:
    just --list

# Lint
lint:
    cargo clippy & cargo check

# Sort deps
sort:
    cargo sort --workspace --grouped

# Check unused deps
unused:
    cargo machete

# Test with coverage
test package="" *args:
    @if [ -z "{{package}}" ]; then \
        cargo llvm-cov nextest --workspace {{args}}; \
    else \
        cargo llvm-cov nextest -p {{package}} {{args}}; \
    fi

# Install dev tools
install-tools:
    sudo apt install valgrind & \
    sudo apt install gnuplot & \
    sudo apt install massif-visualizer & \
    cargo install just & \
    cargo install --locked cargo-llvm-cov & \
    cargo install --locked cargo-nextest & \
    cargo install cargo-sort & \
    cargo install cargo-machete
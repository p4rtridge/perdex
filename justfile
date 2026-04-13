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
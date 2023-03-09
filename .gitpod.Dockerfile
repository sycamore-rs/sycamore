FROM gitpod/workspace-full

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Install trunk
RUN bash -cl "wget -qO- https://github.com/thedodd/trunk/releases/download/v0.16.0/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf- && sudo mv ./trunk /usr/bin/"

# Install Rust with wasm32-unknown-unknown target on nightly toolchain
RUN bash -cl "rustup toolchain install nightly && rustup default nightly && rustup target add wasm32-unknown-unknown"

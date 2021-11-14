FROM gitpod/workspace-full

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Install trunk
RUN bash -cl "wget -qO- https://github.com/thedodd/trunk/releases/download/v0.14.0/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf-"
RUN bash -cl "sudo mv ./trunk /usr/bin/"

# Install wasm32-unknown-unknown target
RUN rustup target add wasm32-unknown-unknown
# Install rust nightly
RUN rustup toolchain add nightly
RUN rustup component add rustfmt --toolchain nightly

FROM gitpod/workspace-full

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Install trunk
RUN bash -cl "wget -qO- https://github.com/thedodd/trunk/releases/download/v0.11.0/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf-"
RUN bash -cl "sudo mv ./trunk /usr/bin/"

# Install wasm-bindgen
RUN bash -cl "wget -O wasm-bindgen.tar.gz https://github.com/rustwasm/wasm-bindgen/releases/download/0.2.74/wasm-bindgen-0.2.74-x86_64-unknown-linux-musl.tar.gz \
    && tar -xf wasm-bindgen.tar.gz \
    && mv wasm-bindgen-0.2.74-x86_64-unknown-linux-musl wasm-bindgen"
RUN bash -cl "sudo mv ./wasm-bindgen/wasm-bindgen /usr/bin/"
RUN bash -cl "sudo mv ./wasm-bindgen/wasm-bindgen-test-runner /usr/bin/"
RUN bash -cl "rm -r ./wasm-bindgen/"

# Install wasm32-unknown-unknown target
RUN rustup target add wasm32-unknown-unknown

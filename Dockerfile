FROM archlinux
RUN pacman --noconfirm -Syu npm
RUN pacman --noconfirm -Syu curl
RUN pacman --noconfirm -Syu perl
RUN pacman --noconfirm -Syu make
#RUN pacman --noconfirm -Syu rust
#RUN pacman --noconfirm -Syu cargo
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o rust.sh
RUN chmod +x rust.sh
RUN ./rust.sh -y
RUN npm install -g @cloudflare/wrangler
RUN npm install -g miniflare
RUN pacman --noconfirm -Syu gcc
ENV PATH=/root/.cargo/bin:$PATH
RUN cargo install -q worker-build
RUN cargo install -q wasm-pack
RUN rustup target add wasm32-unknown-unknown

VOLUME /workdir
WORKDIR /workdir
CMD miniflare -e .env -w

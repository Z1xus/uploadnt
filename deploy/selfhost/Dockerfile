FROM docker.io/library/rust:1.98.1-slim-trixie AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates crates
RUN cargo build --release -p lobby --bin lobby

FROM gcr.io/distroless/cc-debian13:nonroot
COPY --from=build /src/target/release/lobby /lobby
COPY web/build /site
ENV SITE=/site ADDR=0.0.0.0:8787
ENTRYPOINT ["/lobby"]

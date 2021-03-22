FROM rust:1.45

# BUILD CACHING
FROM lukemathwalker/cargo-chef as planner
WORKDIR app
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM lukemathwalker/cargo-chef as cacher
WORKDIR app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build project
FROM rust as builder
WORKDIR /usr/src/source-code-parser
COPY . .
# Copy over the cached dependencies
COPY --from=cacher /app/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
RUN cargo build --release

FROM rust as runtime
WORKDIR app
COPY --from=builder /usr/src/source-code-parser/target/release/source-code-parser-web /usr/local/bin
ENTRYPOINT [ "./target/release/source-code-parser-web", "--host", "0.0.0.0" ]

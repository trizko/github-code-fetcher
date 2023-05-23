# Build step
FROM rust:1.68.2-buster as build-env

ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL='sparse'

WORKDIR /app
COPY . /app
RUN cargo build --release

# Final image
FROM gcr.io/distroless/cc

ENV HOST=0.0.0.0
ENV PORT=3000
EXPOSE 3000

COPY --from=build-env /app/target/release/github-code-fetcher /
COPY ./src/static /src/static

ENTRYPOINT ["/github-code-fetcher"]

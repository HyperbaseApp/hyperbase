FROM rust:1.79 AS build
WORKDIR /app
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
WORKDIR /app
COPY --from=build /app/target/release/hyperbase .
CMD ["./hyperbase"]
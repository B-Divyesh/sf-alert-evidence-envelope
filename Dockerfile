FROM node:22-alpine AS frontend
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY frontend ./frontend
RUN npm run build

FROM rust:1.98-alpine AS backend
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release --locked

FROM alpine:3.21
RUN apk add --no-cache ca-certificates && addgroup -S envelope && adduser -S -G envelope envelope
WORKDIR /app
COPY --from=backend /app/target/release/alert-evidence-envelope /usr/local/bin/alert-evidence-envelope
COPY --from=frontend /app/dist ./dist
RUN mkdir -p /data && chown envelope:envelope /data
USER envelope
ENV PORT=8080 DATABASE_URL=sqlite:/data/envelopes.db?mode=rwc STATIC_DIR=/app/dist
EXPOSE 8080
ENTRYPOINT ["alert-evidence-envelope"]

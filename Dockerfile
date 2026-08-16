FROM rust:1-bookworm AS wasm
WORKDIR /build
RUN cargo install wasm-pack --locked
COPY core/Cargo.toml core/Cargo.lock* ./core/
COPY core/src ./core/src
WORKDIR /build/core
RUN wasm-pack build --release --target web --out-dir /build/wasm --out-name kosmos

FROM node:22-alpine AS web
WORKDIR /app
COPY web/package.json web/package-lock.json* ./
RUN npm ci --include=dev
COPY web/ ./
COPY --from=wasm /build/wasm ./src/wasm
RUN rm -f src/wasm/.gitignore && npm run build

FROM nginx:alpine
COPY --from=web /app/dist /usr/share/nginx/html
COPY deploy/nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
HEALTHCHECK --interval=30s --timeout=3s CMD wget -q --spider http://localhost/ || exit 1

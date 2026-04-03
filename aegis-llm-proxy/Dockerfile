# AEGIS — zokastech.fr — image minimale (< ~50MB)
FROM golang:1.22-alpine AS build
RUN apk add --no-cache ca-certificates
WORKDIR /src
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -trimpath -ldflags="-s -w" -o /out/aegis-llm-proxy ./cmd/aegis-llm-proxy

FROM scratch
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=build /out/aegis-llm-proxy /aegis-llm-proxy
ENTRYPOINT ["/aegis-llm-proxy"]
CMD ["-config", "/etc/aegis/llm-proxy.yaml"]

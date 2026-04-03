# AEGIS — zokastech.fr — Apache 2.0 / MIT

.PHONY: tidy test grpc-gen run

tidy:
	go mod tidy

test:
	go test ./...

# Nécessite protoc + include google/protobuf (ex. brew install protobuf).
grpc-gen:
	protoc -I. -I$$(dirname $$(which protoc))/../include \
		--go_out=paths=source_relative:. \
		--go-grpc_out=paths=source_relative:. \
		api/grpc/aegis.proto 2>/dev/null || \
		protoc -I. -I/usr/local/include \
		--go_out=paths=source_relative:. \
		--go-grpc_out=paths=source_relative:. \
		api/grpc/aegis.proto

run:
	go run ./cmd/aegis-gateway

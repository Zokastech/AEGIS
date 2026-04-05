// AEGIS — zokastech.fr — Apache 2.0 / MIT

package grpcapi

import (
	"context"
	"encoding/json"
	"net"
	"testing"
	"time"

	"github.com/zokastech/aegis/aegis-gateway/api/rest"
	"github.com/zokastech/aegis/aegis-gateway/bridge"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/status"
	"google.golang.org/grpc/test/bufconn"
	"google.golang.org/protobuf/types/known/wrapperspb"
)

func testGRPCServer(t *testing.T) (*grpc.Server, *grpc.ClientConn, func()) {
	t.Helper()
	pool := bridge.NewPool(2, 2*time.Second, func() (bridge.Engine, error) {
		return bridge.NewMockEngine(), nil
	})
	svc := &rest.Services{
		Pool:          pool,
		Breaker:       bridge.NewDefaultBreaker(),
		EngineTimeout: 5 * time.Second,
		Loader:        nil,
	}
	svc.AnalyzeUC = rest.NewAnalyzeUseCase(svc)
	s := grpc.NewServer()
	RegisterAegisGatewayServer(s, &Server{Services: svc})
	lis := bufconn.Listen(1024 * 1024)
	go func() {
		_ = s.Serve(lis)
	}()
	conn, err := grpc.DialContext(context.Background(), "bufnet",
		grpc.WithContextDialer(func(context.Context, string) (net.Conn, error) {
			return lis.Dial()
		}),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	if err != nil {
		t.Fatal(err)
	}
	cleanup := func() {
		_ = conn.Close()
		s.Stop()
		_ = lis.Close()
	}
	return s, conn, cleanup
}

func TestGRPCHealth(t *testing.T) {
	_, conn, done := testGRPCServer(t)
	defer done()
	out := new(wrapperspb.StringValue)
	err := conn.Invoke(context.Background(), aegisGatewayHealthFullMethodName, wrapperspb.String("{}"), out)
	if err != nil {
		t.Fatal(err)
	}
	var h rest.HealthResponse
	if err := json.Unmarshal([]byte(out.GetValue()), &h); err != nil {
		t.Fatal(err)
	}
	if h.Status != "ok" {
		t.Fatal(h)
	}
}

func TestGRPCAnalyze(t *testing.T) {
	_, conn, done := testGRPCServer(t)
	defer done()
	payload, _ := json.Marshal(rest.AnalyzeRequest{Text: "x@y.co"})
	out := new(wrapperspb.StringValue)
	err := conn.Invoke(context.Background(), aegisGatewayAnalyzeFullMethodName, wrapperspb.String(string(payload)), out)
	if err != nil {
		t.Fatal(err)
	}
	if out.GetValue() == "" {
		t.Fatal("empty")
	}
}

func TestGRPCDeanonymizeUnimplemented(t *testing.T) {
	_, conn, done := testGRPCServer(t)
	defer done()
	payload, _ := json.Marshal(rest.DeanonymizeRequest{AnonymizedResultJSON: "{}"})
	out := new(wrapperspb.StringValue)
	err := conn.Invoke(context.Background(), aegisGatewayDeanonymizeFullMethodName, wrapperspb.String(string(payload)), out)
	if err == nil {
		t.Fatal("expected error")
	}
	st, ok := status.FromError(err)
	if !ok || st.Code() != codes.Unimplemented {
		t.Fatalf("got %v", err)
	}
	_ = out
}

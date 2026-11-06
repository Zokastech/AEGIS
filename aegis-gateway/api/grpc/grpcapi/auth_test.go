// AEGIS — zokastech.fr — Apache 2.0 / MIT

package grpcapi

import (
	"context"
	"encoding/json"
	"net"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/zokastech/aegis/aegis-gateway/api/rest"
	"github.com/zokastech/aegis/aegis-gateway/bridge"
	"github.com/zokastech/aegis/aegis-gateway/config"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/reflection"
	"google.golang.org/grpc/status"
	"google.golang.org/grpc/test/bufconn"
	"google.golang.org/protobuf/types/known/wrapperspb"
)

type securedGRPC struct {
	conn    *grpc.ClientConn
	secrets map[string]string
	gw      *rest.GatewaySecurity
}

func newSecuredGRPC(t *testing.T, mutate func(*config.SecurityConfig), allowReflection bool) *securedGRPC {
	t.Helper()
	dir := t.TempDir()
	sc := config.DefaultSecurity()
	sc.APIKeys.FilePath = filepath.Join(dir, "k.yaml")
	sc.APIKeys.Pepper = "grpc-test-pepper"
	sc.RBAC.FilePath = filepath.Join(dir, "rbac.yaml")
	sc.Audit.FilePath = ""
	if mutate != nil {
		mutate(&sc)
	}
	gw, err := rest.BootstrapGatewaySecurity(config.Config{APIKeyHeader: "X-API-Key", Security: sc})
	if err != nil {
		t.Fatal(err)
	}
	secrets := map[string]string{}
	for id, role := range map[string]string{"admin-1": "admin", "admin-2": "admin", "viewer-1": "viewer", "operator-1": "operator"} {
		secrets[id] = "s-" + id
		if err := gw.APIKeys.Generate(id, role, secrets[id]); err != nil {
			t.Fatal(err)
		}
	}
	pool := bridge.NewPool(2, 2*time.Second, func() (bridge.Engine, error) { return bridge.NewMockEngine(), nil })
	svc := &rest.Services{Pool: pool, Breaker: bridge.NewDefaultBreaker(), EngineTimeout: 5 * time.Second, Loader: config.NewLoader(), Gateway: gw}
	authn := &Authenticator{Gateway: gw, APIKeyHeader: "X-API-Key", AllowReflection: allowReflection}
	s := grpc.NewServer(grpc.ChainUnaryInterceptor(authn.Unary()), grpc.ChainStreamInterceptor(authn.Stream()))
	RegisterAegisGatewayServer(s, &Server{Services: svc})
	reflection.Register(s)
	lis := bufconn.Listen(1 << 20)
	go func() { _ = s.Serve(lis) }()
	conn, err := grpc.NewClient("passthrough:///bufnet",
		grpc.WithContextDialer(func(context.Context, string) (net.Conn, error) { return lis.Dial() }),
		grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = conn.Close(); s.Stop(); _ = lis.Close() })
	return &securedGRPC{conn: conn, secrets: secrets, gw: gw}
}

func (g *securedGRPC) call(md map[string]string, method, payload string) (string, error) {
	ctx := context.Background()
	if len(md) > 0 {
		ctx = metadata.NewOutgoingContext(ctx, metadata.New(md))
	}
	out := new(wrapperspb.StringValue)
	err := g.conn.Invoke(ctx, method, wrapperspb.String(payload), out)
	return out.GetValue(), err
}

func code(err error) codes.Code { return status.Code(err) }

func analyzePayload() string {
	b, _ := json.Marshal(rest.AnalyzeRequest{Text: "x@y.co"})
	return string(b)
}

// Regression: the gRPC listener used to have NO authentication at all (Deanonymize/UpdateConfig open).
func TestGRPC_UnauthenticatedRejected(t *testing.T) {
	g := newSecuredGRPC(t, nil, false)
	for _, m := range []string{
		aegisGatewayAnalyzeFullMethodName,
		aegisGatewayAnonymizeFullMethodName,
		aegisGatewayUpdateConfigFullMethodName,
		aegisGatewayHealthFullMethodName,
	} {
		if _, err := g.call(nil, m, analyzePayload()); code(err) != codes.Unauthenticated {
			t.Errorf("%s sans auth: %v", m, err)
		}
	}
	if _, err := g.call(nil, aegisGatewayDeanonymizeFullMethodName, "{}"); code(err) != codes.PermissionDenied {
		t.Errorf("Deanonymize sans approbation: %v", err)
	}
}

func TestGRPC_RBAC(t *testing.T) {
	g := newSecuredGRPC(t, nil, false)
	viewer := map[string]string{"x-api-key": g.secrets["viewer-1"]}
	if _, err := g.call(viewer, aegisGatewayAnalyzeFullMethodName, analyzePayload()); err != nil {
		t.Fatalf("viewer Analyze: %v", err)
	}
	if _, err := g.call(viewer, aegisGatewayAnonymizeFullMethodName, `{"text":"a@b.c"}`); code(err) != codes.PermissionDenied {
		t.Fatalf("viewer Anonymize: %v", err)
	}
	if _, err := g.call(viewer, aegisGatewayUpdateConfigFullMethodName, "pool_size: 3\n"); code(err) != codes.PermissionDenied {
		t.Fatalf("viewer UpdateConfig: %v", err)
	}
	admin := map[string]string{"x-api-key": g.secrets["admin-1"]}
	if _, err := g.call(admin, aegisGatewayUpdateConfigFullMethodName, "pool_size: 3\n"); err != nil {
		t.Fatalf("admin UpdateConfig: %v", err)
	}
	if _, err := g.call(admin, aegisGatewayUpdateConfigFullMethodName, "security:\n  development:\n    disable_auth: true\n"); code(err) != codes.InvalidArgument {
		t.Fatalf("admin UpdateConfig security: %v", err)
	}
	if _, err := g.call(map[string]string{"x-api-key": "bogus"}, aegisGatewayAnalyzeFullMethodName, analyzePayload()); code(err) != codes.Unauthenticated {
		t.Fatalf("clé invalide: %v", err)
	}
	if _, err := g.call(map[string]string{"authorization": "Bearer nope"}, aegisGatewayAnalyzeFullMethodName, analyzePayload()); code(err) != codes.Unauthenticated {
		t.Fatalf("bearer invalide: %v", err)
	}
}

func TestGRPC_DeanonymizeDualApproval(t *testing.T) {
	g := newSecuredGRPC(t, func(sc *config.SecurityConfig) { sc.Deanon.MaxPerHour = 1 }, false)
	single := map[string]string{"x-api-key": g.secrets["admin-1"]}
	if _, err := g.call(single, aegisGatewayDeanonymizeFullMethodName, "{}"); code(err) != codes.PermissionDenied {
		t.Fatalf("clé unique: %v", err)
	}
	same := map[string]string{"x-aegis-approval-1": g.secrets["admin-1"], "x-aegis-approval-2": g.secrets["admin-1"]}
	if _, err := g.call(same, aegisGatewayDeanonymizeFullMethodName, "{}"); code(err) != codes.PermissionDenied {
		t.Fatalf("même clé: %v", err)
	}
	nonAdmin := map[string]string{"x-aegis-approval-1": g.secrets["admin-1"], "x-aegis-approval-2": g.secrets["viewer-1"]}
	if _, err := g.call(nonAdmin, aegisGatewayDeanonymizeFullMethodName, "{}"); code(err) != codes.PermissionDenied {
		t.Fatalf("non-admin: %v", err)
	}
	ok := map[string]string{"x-aegis-approval-1": g.secrets["admin-1"], "x-aegis-approval-2": g.secrets["admin-2"]}
	// Mock engine: deanonymize not implemented → Unimplemented means auth passed.
	if _, err := g.call(ok, aegisGatewayDeanonymizeFullMethodName, `{"anonymized_result_json":"{}"}`); code(err) != codes.Unimplemented {
		t.Fatalf("double approbation: %v", err)
	}
	if _, err := g.call(ok, aegisGatewayDeanonymizeFullMethodName, `{"anonymized_result_json":"{}"}`); code(err) != codes.ResourceExhausted {
		t.Fatalf("limite horaire: %v", err)
	}
}

func TestGRPC_StreamRequiresAuth(t *testing.T) {
	g := newSecuredGRPC(t, nil, false)
	desc := &grpc.StreamDesc{StreamName: "StreamAnalyze", ServerStreams: true, ClientStreams: true}
	open := func(md map[string]string) error {
		ctx := context.Background()
		if md != nil {
			ctx = metadata.NewOutgoingContext(ctx, metadata.New(md))
		}
		st, err := g.conn.NewStream(ctx, desc, aegisGatewayStreamAnalyzeFullMethodName)
		if err != nil {
			return err
		}
		if err := st.SendMsg(wrapperspb.String(`{"log_line":"x@y.co"}`)); err != nil {
			return err
		}
		_ = st.CloseSend()
		return st.RecvMsg(new(wrapperspb.StringValue))
	}
	if err := open(nil); code(err) != codes.Unauthenticated {
		t.Fatalf("stream sans auth: %v", err)
	}
	if err := open(map[string]string{"x-api-key": g.secrets["viewer-1"]}); err != nil {
		t.Fatalf("stream viewer: %v", err)
	}
}

func TestGRPC_UnknownMethodAndReflection(t *testing.T) {
	g := newSecuredGRPC(t, nil, false)
	if _, err := g.call(map[string]string{"x-api-key": g.secrets["admin-1"]}, "/aegis.gateway.v1.AegisGateway/Nope", "{}"); err == nil {
		t.Fatal("méthode inconnue acceptée")
	}
	refl := "/grpc.reflection.v1.ServerReflection/ServerReflectionInfo"
	desc := &grpc.StreamDesc{StreamName: "ServerReflectionInfo", ServerStreams: true, ClientStreams: true}
	st, err := g.conn.NewStream(context.Background(), desc, refl)
	if err == nil {
		err = st.RecvMsg(new(wrapperspb.StringValue))
	}
	if code(err) != codes.PermissionDenied {
		t.Fatalf("réflexion accessible sans opt-in: %v", err)
	}
}

func TestGRPC_EntityFilteringByRole(t *testing.T) {
	g := newSecuredGRPC(t, nil, false)
	g.gw.RBAC.RoleEntityTypes["viewer"] = []string{"PERSON"}
	out, err := g.call(map[string]string{"x-api-key": g.secrets["viewer-1"]}, aegisGatewayAnalyzeFullMethodName, analyzePayload())
	if err != nil {
		t.Fatal(err)
	}
	if strings.Contains(out, `"EMAIL"`) {
		t.Fatalf("filtrage RBAC des entités contourné via gRPC: %s", out)
	}
}

func TestGRPC_DisableAuthDevMode(t *testing.T) {
	g := newSecuredGRPC(t, func(sc *config.SecurityConfig) { sc.Production = false; sc.Development.DisableAuth = true }, false)
	if _, err := g.call(nil, aegisGatewayHealthFullMethodName, "{}"); err != nil {
		t.Fatalf("dev: %v", err)
	}
}

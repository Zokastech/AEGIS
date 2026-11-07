// AEGIS — zokastech.fr — Apache 2.0 / MIT

package grpcapi

import (
	"context"
	"errors"
	"net/http"
	"strings"

	"github.com/labstack/echo/v4"
	"github.com/zokastech/aegis/aegis-gateway/api/rest"
	"github.com/zokastech/aegis/aegis-gateway/auth"
	"github.com/zokastech/aegis/aegis-gateway/rbac"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/peer"
	"google.golang.org/grpc/status"
)

const dualApproval = "@dual-approval"

// methodPermissions maps every RPC to the RBAC permission it requires (same matrix as REST).
// Unknown methods are denied (fail closed).
var methodPermissions = map[string]string{
	aegisGatewayAnalyzeFullMethodName:         rbac.PermAnalyzeExecute,
	aegisGatewayAnalyzeBatchFullMethodName:    rbac.PermAnalyzeBatch,
	aegisGatewayAnonymizeFullMethodName:       rbac.PermAnonymizeExecute,
	aegisGatewayDeanonymizeFullMethodName:     dualApproval,
	aegisGatewayListRecognizersFullMethodName: rbac.PermMetaRead,
	aegisGatewayListEntityTypesFullMethodName: rbac.PermMetaRead,
	aegisGatewayUpdateConfigFullMethodName:    rbac.PermConfigWrite,
	aegisGatewayHealthFullMethodName:          rbac.PermMetaRead,
	aegisGatewayStreamAnalyzeFullMethodName:   rbac.PermAnalyzeExecute,
}

type principalCtxKey struct{}

// PrincipalFromContext returns the authenticated principal set by the interceptors.
func PrincipalFromContext(ctx context.Context) *auth.Principal {
	p, _ := ctx.Value(principalCtxKey{}).(*auth.Principal)
	return p
}

// Authenticator enforces authentication + RBAC on the gRPC listener, mirroring the REST stack.
type Authenticator struct {
	Gateway      *rest.GatewaySecurity
	APIKeyHeader string
	// AllowReflection lets unauthenticated clients call the reflection service (dev only).
	AllowReflection bool
}

func (a *Authenticator) md(ctx context.Context, key string) string {
	md, ok := metadata.FromIncomingContext(ctx)
	if !ok {
		return ""
	}
	v := md.Get(strings.ToLower(key))
	if len(v) == 0 {
		return ""
	}
	return v[0]
}

func verifiedCN(ctx context.Context) string {
	p, ok := peer.FromContext(ctx)
	if !ok || p.AuthInfo == nil {
		return ""
	}
	ti, ok := p.AuthInfo.(credentials.TLSInfo)
	if !ok {
		return ""
	}
	return auth.VerifiedClientCN(&ti.State)
}

// authorize returns the context enriched with the principal, or a gRPC status error.
func (a *Authenticator) authorize(ctx context.Context, method string) (context.Context, *auth.Principal, error) {
	g := a.Gateway
	if g == nil {
		return nil, nil, status.Error(codes.Unavailable, "sécurité gateway non initialisée")
	}
	if g.Config.Development.DisableAuth {
		p := &auth.Principal{Subject: "dev:bypass", Role: rbac.RoleAdmin, AuthMethod: "disabled"}
		return context.WithValue(ctx, principalCtxKey{}, p), p, nil
	}
	if strings.HasPrefix(method, "/grpc.reflection.") && a.AllowReflection {
		return ctx, nil, nil
	}
	perm, known := methodPermissions[method]
	if !known {
		return nil, nil, status.Error(codes.PermissionDenied, "méthode non autorisée")
	}
	var (
		p   *auth.Principal
		err error
	)
	if perm == dualApproval {
		h1, h2 := g.ApprovalHeaders()
		p, err = g.ResolveDualApproval(a.md(ctx, h1), a.md(ctx, h2))
	} else {
		header := a.APIKeyHeader
		if header == "" {
			header = "X-API-Key"
		}
		p, err = g.Resolve(ctx, rest.Credentials{
			ClientCN:      verifiedCN(ctx),
			Authorization: a.md(ctx, "authorization"),
			APIKey:        a.md(ctx, header),
		})
		if err == nil {
			err = g.Authorize(p, perm)
		}
	}
	if err != nil {
		return nil, nil, toStatus(err)
	}
	return context.WithValue(ctx, principalCtxKey{}, p), p, nil
}

// Unary is the unary server interceptor.
func (a *Authenticator) Unary() grpc.UnaryServerInterceptor {
	return func(ctx context.Context, req interface{}, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (interface{}, error) {
		nctx, p, err := a.authorize(ctx, info.FullMethod)
		if err != nil {
			return nil, err
		}
		resp, herr := handler(nctx, req)
		if herr == nil && p != nil && p.AuthMethod == "dual_api_key" && a.Gateway.Deanon != nil {
			a.Gateway.Deanon.RecordSuccess()
		}
		return resp, herr
	}
}

type wrappedStream struct {
	grpc.ServerStream
	ctx context.Context
}

func (w *wrappedStream) Context() context.Context { return w.ctx }

// Stream is the streaming server interceptor.
func (a *Authenticator) Stream() grpc.StreamServerInterceptor {
	return func(srv interface{}, ss grpc.ServerStream, info *grpc.StreamServerInfo, handler grpc.StreamHandler) error {
		nctx, _, err := a.authorize(ss.Context(), info.FullMethod)
		if err != nil {
			return err
		}
		return handler(srv, &wrappedStream{ServerStream: ss, ctx: nctx})
	}
}

func toStatus(err error) error {
	var ae *rest.AuthError
	if errors.As(err, &ae) {
		return status.Error(httpToCode(ae.Status), ae.Message)
	}
	var he *echo.HTTPError
	if errors.As(err, &he) {
		msg, _ := he.Message.(string)
		return status.Error(httpToCode(he.Code), msg)
	}
	return status.Error(codes.PermissionDenied, "accès refusé")
}

func httpToCode(st int) codes.Code {
	switch st {
	case http.StatusUnauthorized:
		return codes.Unauthenticated
	case http.StatusForbidden:
		return codes.PermissionDenied
	case http.StatusTooManyRequests:
		return codes.ResourceExhausted
	case http.StatusServiceUnavailable:
		return codes.Unavailable
	default:
		return codes.PermissionDenied
	}
}

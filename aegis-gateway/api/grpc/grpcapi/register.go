// AEGIS — zokastech.fr — Apache 2.0 / MIT

// Hand-written code (protoc-gen-go-grpc equivalent) for google.protobuf.StringValue only.

package grpcapi

import (
	"context"

	"google.golang.org/grpc"
	"google.golang.org/protobuf/types/known/wrapperspb"
)

// GatewayServer must be implemented by the application server.
type GatewayServer interface {
	Analyze(context.Context, *wrapperspb.StringValue) (*wrapperspb.StringValue, error)
	AnalyzeBatch(context.Context, *wrapperspb.StringValue) (*wrapperspb.StringValue, error)
	Anonymize(context.Context, *wrapperspb.StringValue) (*wrapperspb.StringValue, error)
	Deanonymize(context.Context, *wrapperspb.StringValue) (*wrapperspb.StringValue, error)
	ListRecognizers(context.Context, *wrapperspb.StringValue) (*wrapperspb.StringValue, error)
	ListEntityTypes(context.Context, *wrapperspb.StringValue) (*wrapperspb.StringValue, error)
	UpdateConfig(context.Context, *wrapperspb.StringValue) (*wrapperspb.StringValue, error)
	Health(context.Context, *wrapperspb.StringValue) (*wrapperspb.StringValue, error)
	StreamAnalyze(Gateway_StreamAnalyzeServer) error
}

const (
	aegisGatewayAnalyzeFullMethodName         = "/aegis.gateway.v1.AegisGateway/Analyze"
	aegisGatewayAnalyzeBatchFullMethodName    = "/aegis.gateway.v1.AegisGateway/AnalyzeBatch"
	aegisGatewayAnonymizeFullMethodName       = "/aegis.gateway.v1.AegisGateway/Anonymize"
	aegisGatewayDeanonymizeFullMethodName     = "/aegis.gateway.v1.AegisGateway/Deanonymize"
	aegisGatewayListRecognizersFullMethodName = "/aegis.gateway.v1.AegisGateway/ListRecognizers"
	aegisGatewayListEntityTypesFullMethodName = "/aegis.gateway.v1.AegisGateway/ListEntityTypes"
	aegisGatewayUpdateConfigFullMethodName    = "/aegis.gateway.v1.AegisGateway/UpdateConfig"
	aegisGatewayHealthFullMethodName        = "/aegis.gateway.v1.AegisGateway/Health"
)

// RegisterAegisGatewayServer registers the service (grpc.Server or xds.GRPCServer).
func RegisterAegisGatewayServer(s grpc.ServiceRegistrar, srv GatewayServer) {
	s.RegisterService(&AegisGateway_ServiceDesc, srv)
}

func _AegisGateway_Analyze_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(wrapperspb.StringValue)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(GatewayServer).Analyze(ctx, in)
	}
	info := &grpc.UnaryServerInfo{Server: srv, FullMethod: aegisGatewayAnalyzeFullMethodName}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(GatewayServer).Analyze(ctx, req.(*wrapperspb.StringValue))
	}
	return interceptor(ctx, in, info, handler)
}

func _AegisGateway_AnalyzeBatch_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(wrapperspb.StringValue)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(GatewayServer).AnalyzeBatch(ctx, in)
	}
	info := &grpc.UnaryServerInfo{Server: srv, FullMethod: aegisGatewayAnalyzeBatchFullMethodName}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(GatewayServer).AnalyzeBatch(ctx, req.(*wrapperspb.StringValue))
	}
	return interceptor(ctx, in, info, handler)
}

func _AegisGateway_Anonymize_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(wrapperspb.StringValue)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(GatewayServer).Anonymize(ctx, in)
	}
	info := &grpc.UnaryServerInfo{Server: srv, FullMethod: aegisGatewayAnonymizeFullMethodName}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(GatewayServer).Anonymize(ctx, req.(*wrapperspb.StringValue))
	}
	return interceptor(ctx, in, info, handler)
}

func _AegisGateway_Deanonymize_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(wrapperspb.StringValue)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(GatewayServer).Deanonymize(ctx, in)
	}
	info := &grpc.UnaryServerInfo{Server: srv, FullMethod: aegisGatewayDeanonymizeFullMethodName}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(GatewayServer).Deanonymize(ctx, req.(*wrapperspb.StringValue))
	}
	return interceptor(ctx, in, info, handler)
}

func _AegisGateway_ListRecognizers_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(wrapperspb.StringValue)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(GatewayServer).ListRecognizers(ctx, in)
	}
	info := &grpc.UnaryServerInfo{Server: srv, FullMethod: aegisGatewayListRecognizersFullMethodName}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(GatewayServer).ListRecognizers(ctx, req.(*wrapperspb.StringValue))
	}
	return interceptor(ctx, in, info, handler)
}

func _AegisGateway_ListEntityTypes_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(wrapperspb.StringValue)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(GatewayServer).ListEntityTypes(ctx, in)
	}
	info := &grpc.UnaryServerInfo{Server: srv, FullMethod: aegisGatewayListEntityTypesFullMethodName}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(GatewayServer).ListEntityTypes(ctx, req.(*wrapperspb.StringValue))
	}
	return interceptor(ctx, in, info, handler)
}

func _AegisGateway_UpdateConfig_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(wrapperspb.StringValue)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(GatewayServer).UpdateConfig(ctx, in)
	}
	info := &grpc.UnaryServerInfo{Server: srv, FullMethod: aegisGatewayUpdateConfigFullMethodName}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(GatewayServer).UpdateConfig(ctx, req.(*wrapperspb.StringValue))
	}
	return interceptor(ctx, in, info, handler)
}

func _AegisGateway_Health_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(wrapperspb.StringValue)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(GatewayServer).Health(ctx, in)
	}
	info := &grpc.UnaryServerInfo{Server: srv, FullMethod: aegisGatewayHealthFullMethodName}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(GatewayServer).Health(ctx, req.(*wrapperspb.StringValue))
	}
	return interceptor(ctx, in, info, handler)
}

// Gateway_StreamAnalyzeServer is the bidirectional stream server API.
type Gateway_StreamAnalyzeServer interface {
	Send(*wrapperspb.StringValue) error
	Recv() (*wrapperspb.StringValue, error)
	grpc.ServerStream
}

type aegisGatewayStreamAnalyzeServer struct {
	grpc.ServerStream
}

func (x *aegisGatewayStreamAnalyzeServer) Send(m *wrapperspb.StringValue) error {
	return x.ServerStream.SendMsg(m)
}

func (x *aegisGatewayStreamAnalyzeServer) Recv() (*wrapperspb.StringValue, error) {
	m := new(wrapperspb.StringValue)
	if err := x.ServerStream.RecvMsg(m); err != nil {
		return nil, err
	}
	return m, nil
}

func _AegisGateway_StreamAnalyze_Handler(srv interface{}, stream grpc.ServerStream) error {
	return srv.(GatewayServer).StreamAnalyze(&aegisGatewayStreamAnalyzeServer{stream})
}

// AegisGateway_ServiceDesc is the gRPC service descriptor.
var AegisGateway_ServiceDesc = grpc.ServiceDesc{
	ServiceName: "aegis.gateway.v1.AegisGateway",
	// Must be the server interface (like protoc-gen-go-grpc), not the *Server struct:
	// grpc.RegisterService uses reflect.TypeOf(HandlerType).Elem() then st.Implements(ht).
	HandlerType: (*GatewayServer)(nil),
	Methods: []grpc.MethodDesc{
		{MethodName: "Analyze", Handler: _AegisGateway_Analyze_Handler},
		{MethodName: "AnalyzeBatch", Handler: _AegisGateway_AnalyzeBatch_Handler},
		{MethodName: "Anonymize", Handler: _AegisGateway_Anonymize_Handler},
		{MethodName: "Deanonymize", Handler: _AegisGateway_Deanonymize_Handler},
		{MethodName: "ListRecognizers", Handler: _AegisGateway_ListRecognizers_Handler},
		{MethodName: "ListEntityTypes", Handler: _AegisGateway_ListEntityTypes_Handler},
		{MethodName: "UpdateConfig", Handler: _AegisGateway_UpdateConfig_Handler},
		{MethodName: "Health", Handler: _AegisGateway_Health_Handler},
	},
	Streams: []grpc.StreamDesc{{
		StreamName:    "StreamAnalyze",
		Handler:       _AegisGateway_StreamAnalyze_Handler,
		ServerStreams: true,
		ClientStreams: true,
	}},
	Metadata: "aegis.proto",
}

// AEGIS — zokastech.fr — Apache 2.0 / MIT

package grpcapi

import (
	"context"
	"encoding/json"
	"errors"
	"io"
	"net/http"
	"strings"
	"time"

	"github.com/zokastech/aegis/aegis-gateway/api/rest"
	"github.com/zokastech/aegis/aegis-gateway/bridge"
	"github.com/zokastech/aegis/aegis-gateway/metrics"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/wrapperspb"
)

// Server implements GatewayServer using the same pool as REST.
type Server struct {
	Services *rest.Services
}

func str(in *wrapperspb.StringValue) string {
	if in == nil {
		return ""
	}
	return in.GetValue()
}

func okJSON(b []byte, err error) (*wrapperspb.StringValue, error) {
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}
	return wrapperspb.String(string(b)), nil
}

// Analyze implements the RPC (JSON payload in StringValue).
func (s *Server) Analyze(ctx context.Context, in *wrapperspb.StringValue) (*wrapperspb.StringValue, error) {
	started := time.Now()
	st := http.StatusOK
	analysisJSON := ""
	var resultRaw []byte
	defer func() {
		metrics.ObserveAnalyze("grpc.Analyze", "grpc", st, analysisJSON, started, resultRaw)
	}()

	var req rest.AnalyzeRequest
	if err := json.Unmarshal([]byte(str(in)), &req); err != nil {
		st = http.StatusBadRequest
		return nil, status.Error(codes.InvalidArgument, err.Error())
	}
	analysisJSON = req.AnalysisJSON
	if strings.TrimSpace(req.Text) == "" {
		st = http.StatusBadRequest
		return nil, status.Error(codes.InvalidArgument, "text requis")
	}
	ctx, cancel := context.WithTimeout(ctx, s.Services.EngineTimeout)
	defer cancel()
	var out string
	err := bridge.WithCircuitBreaker(s.Services.Breaker, func() error {
		return s.Services.Pool.With(ctx, func(ctx context.Context, eng bridge.Engine) error {
			var e error
			out, e = eng.Analyze(ctx, req.Text, req.AnalysisJSON)
			return e
		})
	})
	if err != nil {
		st = http.StatusInternalServerError
		return nil, status.Error(codes.Internal, err.Error())
	}
	resultRaw = []byte(out)
	return wrapperspb.String(out), nil
}

func (s *Server) AnalyzeBatch(ctx context.Context, in *wrapperspb.StringValue) (*wrapperspb.StringValue, error) {
	started := time.Now()
	st := http.StatusOK
	batchLen := 0
	var metricItems [][]byte
	defer func() {
		metrics.ObserveAnalyzeBatch("grpc.AnalyzeBatch", "grpc", st, batchLen, "", started, metricItems)
	}()

	var req rest.AnalyzeBatchRequest
	if err := json.Unmarshal([]byte(str(in)), &req); err != nil {
		st = http.StatusBadRequest
		return nil, status.Error(codes.InvalidArgument, err.Error())
	}
	if len(req.Texts) == 0 {
		st = http.StatusBadRequest
		return nil, status.Error(codes.InvalidArgument, "texts requis")
	}
	page := req.Page
	if page < 1 {
		page = 1
	}
	ps := req.PageSize
	if ps < 1 {
		ps = 20
	}
	if ps > 100 {
		ps = 100
	}
	total := len(req.Texts)
	start := (page - 1) * ps
	if start > total {
		start = total
	}
	end := start + ps
	if end > total {
		end = total
	}
	slice := req.Texts[start:end]
	batchLen = len(slice)
	ctx, cancel := context.WithTimeout(ctx, s.Services.EngineTimeout*time.Duration(1+len(slice)/10))
	defer cancel()
	var raw string
	err := bridge.WithCircuitBreaker(s.Services.Breaker, func() error {
		return s.Services.Pool.With(ctx, func(ctx context.Context, eng bridge.Engine) error {
			var e error
			raw, e = eng.AnalyzeBatch(ctx, slice)
			return e
		})
	})
	if err != nil {
		st = http.StatusInternalServerError
		return nil, status.Error(codes.Internal, err.Error())
	}
	var items []json.RawMessage
	if err := json.Unmarshal([]byte(raw), &items); err != nil {
		st = http.StatusInternalServerError
		return nil, status.Error(codes.Internal, "batch invalide")
	}
	for i := range items {
		b, merr := items[i].MarshalJSON()
		if merr != nil {
			continue
		}
		metricItems = append(metricItems, append([]byte(nil), b...))
	}
	type batchOut struct {
		Items    []json.RawMessage `json:"items"`
		Total    int               `json:"total"`
		Page     int               `json:"page"`
		PageSize int               `json:"page_size"`
		HasMore  bool              `json:"has_more"`
	}
	b, _ := json.Marshal(batchOut{Items: items, Total: total, Page: page, PageSize: ps, HasMore: end < total})
	return wrapperspb.String(string(b)), nil
}

func (s *Server) Anonymize(ctx context.Context, in *wrapperspb.StringValue) (*wrapperspb.StringValue, error) {
	st := http.StatusOK
	var resultRaw []byte
	defer func() {
		metrics.ObserveAnonymizeRequest("grpc.Anonymize", "grpc", st, resultRaw)
	}()

	var req rest.AnonymizeRequest
	if err := json.Unmarshal([]byte(str(in)), &req); err != nil {
		st = http.StatusBadRequest
		return nil, status.Error(codes.InvalidArgument, err.Error())
	}
	if strings.TrimSpace(req.Text) == "" {
		st = http.StatusBadRequest
		return nil, status.Error(codes.InvalidArgument, "text requis")
	}
	ctx, cancel := context.WithTimeout(ctx, s.Services.EngineTimeout)
	defer cancel()
	var out string
	err := bridge.WithCircuitBreaker(s.Services.Breaker, func() error {
		return s.Services.Pool.With(ctx, func(ctx context.Context, eng bridge.Engine) error {
			var e error
			out, e = eng.Anonymize(ctx, req.Text, req.ConfigJSON)
			return e
		})
	})
	if err != nil {
		st = http.StatusInternalServerError
		return nil, status.Error(codes.Internal, err.Error())
	}
	resultRaw = []byte(out)
	return wrapperspb.String(out), nil
}

func (s *Server) Deanonymize(ctx context.Context, in *wrapperspb.StringValue) (*wrapperspb.StringValue, error) {
	var req rest.DeanonymizeRequest
	if err := json.Unmarshal([]byte(str(in)), &req); err != nil {
		return nil, status.Error(codes.InvalidArgument, err.Error())
	}
	ctx, cancel := context.WithTimeout(ctx, s.Services.EngineTimeout)
	defer cancel()
	body, _ := json.Marshal(req)
	var out string
	err := s.Services.Pool.With(ctx, func(ctx context.Context, eng bridge.Engine) error {
		var e error
		out, e = eng.Deanonymize(ctx, string(body))
		return e
	})
	if err != nil {
		if errors.Is(err, bridge.ErrNotImplemented) {
			return nil, status.Error(codes.Unimplemented, bridge.ErrNotImplemented.Error())
		}
		return nil, status.Error(codes.Internal, err.Error())
	}
	if strings.TrimSpace(out) != "" {
		metrics.IncDeanonymize()
	}
	b, _ := json.Marshal(rest.DeanonymizeResponse{Text: out})
	return wrapperspb.String(string(b)), nil
}

func (s *Server) ListRecognizers(context.Context, *wrapperspb.StringValue) (*wrapperspb.StringValue, error) {
	rs := bridge.DefaultRecognizers()
	dto := make([]rest.RecognizerDTO, len(rs))
	for i, r := range rs {
		dto[i] = rest.RecognizerDTO{Name: r.Name, Kind: r.Kind, Enabled: r.Enabled}
	}
	b, err := json.Marshal(rest.RecognizersResponse{Recognizers: dto})
	return okJSON(b, err)
}

func (s *Server) ListEntityTypes(context.Context, *wrapperspb.StringValue) (*wrapperspb.StringValue, error) {
	b, err := json.Marshal(rest.EntityTypesResponse{EntityTypes: bridge.SupportedEntityTypes()})
	return okJSON(b, err)
}

func (s *Server) UpdateConfig(ctx context.Context, in *wrapperspb.StringValue) (*wrapperspb.StringValue, error) {
	y := str(in)
	if strings.TrimSpace(y) == "" {
		return nil, status.Error(codes.InvalidArgument, "yaml requis")
	}
	if s.Services.Loader == nil {
		return nil, status.Error(codes.FailedPrecondition, "loader absent")
	}
	if err := s.Services.Loader.MergeYAML([]byte(y)); err != nil {
		return nil, status.Error(codes.InvalidArgument, err.Error())
	}
	b, err := json.Marshal(rest.UpdateConfigResponse{Status: "merged"})
	return okJSON(b, err)
}

func (s *Server) Health(ctx context.Context, _ *wrapperspb.StringValue) (*wrapperspb.StringValue, error) {
	ctx, cancel := context.WithTimeout(ctx, 2*time.Second)
	defer cancel()
	ver := ""
	_ = s.Services.Pool.With(ctx, func(ctx context.Context, eng bridge.Engine) error {
		ver = eng.Version()
		return nil
	})
	b, err := json.Marshal(rest.HealthResponse{Status: "ok", RustVersion: ver})
	return okJSON(b, err)
}

func (s *Server) StreamAnalyze(stream Gateway_StreamAnalyzeServer) error {
	for {
		in, err := stream.Recv()
		if err == io.EOF {
			return nil
		}
		if err != nil {
			return err
		}
		line := str(in)
		var sub struct {
			LogLine   string `json:"log_line"`
			RequestID string `json:"request_id"`
		}
		if json.Unmarshal([]byte(line), &sub) == nil && sub.LogLine != "" {
			line = sub.LogLine
		}
		ctx, cancel := context.WithTimeout(stream.Context(), s.Services.EngineTimeout)
		var out string
		err = bridge.WithCircuitBreaker(s.Services.Breaker, func() error {
			return s.Services.Pool.With(ctx, func(ctx context.Context, eng bridge.Engine) error {
				var e error
				out, e = eng.Analyze(ctx, line, "")
				return e
			})
		})
		cancel()
		if err != nil {
			out = `{"error":` + jsonString(err.Error()) + `}`
		}
		type chunk struct {
			RequestID    string `json:"request_id,omitempty"`
			AnalysisJSON string `json:"analysis_json"`
		}
		b, _ := json.Marshal(chunk{RequestID: sub.RequestID, AnalysisJSON: out})
		if err := stream.Send(wrapperspb.String(string(b))); err != nil {
			return err
		}
	}
}

func jsonString(s string) string {
	b, _ := json.Marshal(s)
	return string(b)
}

use hyper::Version;
use opentelemetry::{
    global::{BoxedSpan, BoxedTracer},
    trace::{Span, StatusCode, Tracer},
    KeyValue,
};

/// Create an OpenTelemetry Span for the given HTTP request, according to the OpenTelemetry
/// semantic conventions for HTTP traffic.
/// See https://github.com/open-telemetry/opentelemetry-specification/blob/v0.5.0/specification/trace/semantic_conventions/http.md
pub fn span_for_request<T>(tracer: &BoxedTracer, req: &hyper::Request<T>) -> BoxedSpan {
    let span = tracer.start(&format!(
        "HTTP {} {}",
        req.method(),
        req.uri().host().unwrap_or("<unknown>")
    ));
    span.set_attribute(KeyValue::new("span.kind", "client"));
    span.set_attribute(KeyValue::new("http.method", req.method().to_string()));
    span.set_attribute(KeyValue::new("http.url", req.uri().to_string()));
    if let Some(path_and_query) = req.uri().path_and_query() {
        span.set_attribute(KeyValue::new("http.target", path_and_query.to_string()));
    }
    if let Some(host) = req.uri().host() {
        span.set_attribute(KeyValue::new("http.host", host.to_owned()));
    }
    if let Some(scheme) = req.uri().scheme_str() {
        span.set_attribute(KeyValue::new("http.scheme", scheme.to_string()));
    }

    // Using strings from https://github.com/open-telemetry/opentelemetry-specification/blob/v0.5.0/specification/trace/semantic_conventions/http.md#common-attributes
    let serialized_version = match req.version() {
        Version::HTTP_10 => "1.0",
        Version::HTTP_11 => "1.1",
        Version::HTTP_2 => "2",
        Version::HTTP_3 => "3",
        _ => "unknown",
    };
    span.set_attribute(KeyValue::new("http.flavor", serialized_version));

    // TODO: Emit UserAgent
    // TODO: Expose non-HTTP specific attributes https://github.com/open-telemetry/opentelemetry-specification/blob/v0.5.0/specification/trace/semantic_conventions/span-general.md#general-network-connection-attributes

    span
}

/// Annotate a span that has previously been created given the HTTP response.
/// The passed in span must have been created for the HTTP request for which we got the response.
pub fn annotate_span_for_response<T>(span: &BoxedSpan, response: &hyper::Response<T>) {
    let status = response.status();

    span.set_attribute(KeyValue::new(
        "http.status_code",
        status.as_u16().to_string(),
    ));
    if let Some(canonical_reason) = status.canonical_reason() {
        span.set_attribute(KeyValue::new(
            "http.status_text",
            canonical_reason.to_owned(),
        ));
    }

    if status != hyper::StatusCode::OK {
        span.set_status(StatusCode::Error, status.as_str().to_owned());
    }
}
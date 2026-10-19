//! Local HTTP transport for the zero-JavaScript HTML workbench.
//! HTML templates and CSS live in web/; calculations and SVG stay in Rust.
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

mod forms;
mod svg;
mod workbench;

struct Response {
    status: u16,
    content_type: &'static str,
    body: String,
    filename: Option<&'static str>,
}
impl Response {
    fn new(status: u16, content_type: &'static str, body: String) -> Self {
        Self {
            status,
            content_type,
            body,
            filename: None,
        }
    }
    fn html(status: u16, body: String) -> Self {
        Self::new(status, "text/html; charset=utf-8", body)
    }
    fn text(status: u16, body: String) -> Self {
        Self::new(status, "text/plain; charset=utf-8", body)
    }
    fn download(content_type: &'static str, body: String, filename: &'static str) -> Self {
        Self {
            status: 200,
            content_type,
            body,
            filename: Some(filename),
        }
    }
    fn encode(&self) -> String {
        let disposition = self
            .filename
            .map(|name| format!("Content-Disposition: attachment; filename=\"{name}\"\r\n"))
            .unwrap_or_default();
        format!(
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nContent-Security-Policy: script-src 'none'; object-src 'none'; base-uri 'none'\r\n{disposition}Connection: close\r\n\r\n{}",
            self.status, reason_phrase(self.status), self.content_type, self.body.len(), self.body,
        )
    }
}

fn route(method: &str, path: &str, body: &str) -> Response {
    let path = path.split('?').next().unwrap_or(path);
    match (method, path) {
        ("GET", "/" | "/index.html" | "/workbench") => workbench::initial(),
        ("GET", "/assets/styles.css") => {
            Response::new(200, "text/css; charset=utf-8", workbench::CSS.into())
        }
        ("GET", "/assets/fonts.css") => Response::new(
            200,
            "text/css; charset=utf-8",
            crate::presentation::fonts::css(),
        ),
        ("GET", "/api/health") => Response::text(200, "ok".into()),
        ("POST", "/open") => workbench::upload(body),
        (
            "POST",
            "/workbench"
            | "/blueprint"
            | "/export/design.ssr"
            | "/export/blueprint.svg"
            | "/export/report.html",
        ) => workbench::submit(path, body),
        _ => Response::text(404, "Not found".into()),
    }
}

/// Maximum accepted HTTP request body.
///
/// The console is meant for hand-pasted `.ssr`/CSV text; anything larger is a
/// mistake or an attempt to exhaust memory, so it is rejected early.
pub const MAX_BODY_BYTES: usize = 8 * 1024 * 1024;

/// Maximum simultaneous active client connections allowed by the server.
/// Exceeding this bound immediately returns 503 Service Unavailable to protect
/// file descriptors and thread resources.
pub const MAX_CONCURRENT_CONNECTIONS: usize = 64;

/// Read/write timeout for a single connection.  Without this a client that
/// announces a body and then stalls would occupy a connection worker indefinitely.
const CONNECTION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

/// A managed loopback service, owned by the native launcher window.
pub struct LocalServer {
    pub url: String,
    stop: Arc<AtomicBool>,
    worker: Option<std::thread::JoinHandle<io::Result<()>>>,
}
impl LocalServer {
    pub fn start() -> io::Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", 0))?;
        let url = format!("http://{}/", listener.local_addr()?);
        listener.set_nonblocking(true)?;
        let stop = Arc::new(AtomicBool::new(false));
        let signal = Arc::clone(&stop);
        let worker = std::thread::spawn(move || serve(listener, signal));
        Ok(Self {
            url,
            stop,
            worker: Some(worker),
        })
    }
}
impl Drop for LocalServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
pub fn run_server(port: u16) -> io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    println!("SpringSharp web console: http://{}", listener.local_addr()?);
    listener.set_nonblocking(true)?;
    serve(listener, Arc::new(AtomicBool::new(false)))
}
fn serve(listener: TcpListener, stop: Arc<AtomicBool>) -> io::Result<()> {
    let active_connections = Arc::new(AtomicUsize::new(0));

    while !stop.load(Ordering::Relaxed) {
        let mut stream = match listener.accept() {
            Ok((stream, _)) => stream,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(25));
                continue;
            }
            Err(error) => return Err(error),
        };

        // Accepted sockets can inherit nonblocking mode on Windows.
        stream.set_nonblocking(false)?;
        let current = active_connections.load(Ordering::Relaxed);
        if current >= MAX_CONCURRENT_CONNECTIONS {
            let _ = write_plain(
                &mut stream,
                503,
                "Service Unavailable: connection limit reached",
            );
            continue;
        }

        active_connections.fetch_add(1, Ordering::SeqCst);
        let counter = Arc::clone(&active_connections);

        // Handle each connection on its own thread so one slow/stalled client
        // cannot block the whole console.
        std::thread::spawn(move || {
            struct ConnGuard(Arc<AtomicUsize>);
            impl Drop for ConnGuard {
                fn drop(&mut self) {
                    self.0.fetch_sub(1, Ordering::SeqCst);
                }
            }
            let _guard = ConnGuard(counter);

            // A panic inside a request handler must not take down the server.
            let result = std::panic::catch_unwind(|| handle_connection(stream));
            match result {
                Ok(Ok(())) => {}
                Ok(Err(e)) => eprintln!("connection error: {e}"),
                Err(_) => eprintln!("connection handler panicked"),
            }
        });
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> io::Result<()> {
    stream.set_read_timeout(Some(CONNECTION_TIMEOUT))?;
    stream.set_write_timeout(Some(CONNECTION_TIMEOUT))?;

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    // Bound the request line itself: a client that never sends a newline must
    // not be able to grow this String without limit.
    if request_line.len() > 8 * 1024 {
        return write_plain(&mut stream, 400, "Bad Request");
    }

    let mut content_length = 0_usize;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        if line.len() > 8 * 1024 {
            return write_plain(&mut stream, 400, "Bad Request");
        }
        let lower = line.to_ascii_lowercase();
        if let Some(v) = lower.strip_prefix("content-length:") {
            // Reject absurd lengths outright instead of allocating for them.
            content_length = v.trim().parse().unwrap_or(0);
            if content_length > MAX_BODY_BYTES {
                return write_plain(&mut stream, 413, "Payload Too Large");
            }
        }
    }

    let mut body = vec![0_u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }

    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("GET");
    let path = parts.next().unwrap_or("/");
    if method == "GET" {
        if let Some(file) = path.strip_prefix("/assets/fonts/") {
            if let Some(file_path) = crate::presentation::fonts::font_path(file) {
                return match std::fs::read(file_path) {
                    Ok(bytes) => {
                        let mime = if file.ends_with(".otf") {
                            "font/otf"
                        } else {
                            "font/ttf"
                        };
                        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nX-Content-Type-Options: nosniff\r\nCache-Control: public, max-age=3600\r\nConnection: close\r\n\r\n", bytes.len())?;
                        stream.write_all(&bytes)
                    }
                    Err(_) => write_plain(&mut stream, 404, "Bundled font missing"),
                };
            }
        }
    }
    let body_text = match std::str::from_utf8(&body) {
        Ok(text) => text,
        Err(_) => return write_plain(&mut stream, 400, "Request must be UTF-8"),
    };

    let response = route(method, path, body_text);
    stream.write_all(response.encode().as_bytes())?;
    stream.flush()
}

fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        413 => "Payload Too Large",
        503 => "Service Unavailable",
        _ => "Error",
    }
}

/// Minimal response used by the header-parse guards before a route is chosen.
fn write_plain(stream: &mut TcpStream, status: u16, message: &str) -> io::Result<()> {
    let response = format!(
        "HTTP/1.1 {status} {}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        reason_phrase(status),
        message.len(),
        message
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::serialize;

    fn encoded(pairs: &[(&str, &str)]) -> String {
        fn component(value: &str) -> String {
            value
                .bytes()
                .map(|b| {
                    if b.is_ascii_alphanumeric() || b"-._".contains(&b) {
                        (b as char).to_string()
                    } else {
                        format!("%{b:02X}")
                    }
                })
                .collect()
        }
        pairs
            .iter()
            .map(|(k, v)| format!("{}={}", component(k), component(v)))
            .collect::<Vec<_>>()
            .join("&")
    }

    #[test]
    fn managed_server_serves_fonts_and_stops() {
        let server = LocalServer::start().unwrap();
        let address = server
            .url
            .trim_start_matches("http://")
            .trim_end_matches('/');
        let mut stream = TcpStream::connect(address).unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();
        stream
            .write_all(
                b"GET /assets/fonts/NotoSans-Regular.ttf HTTP/1.1\r\nHost: localhost\r\n\r\n",
            )
            .unwrap();
        let mut bytes = Vec::new();
        stream.read_to_end(&mut bytes).unwrap();
        let split = bytes.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
        assert!(std::str::from_utf8(&bytes[..split])
            .unwrap()
            .contains("Content-Type: font/ttf"));
        assert_eq!(&bytes[split + 4..split + 8], &[0, 1, 0, 0]);
        let address = address.to_string();
        drop(server);
        assert!(TcpStream::connect(address).is_err());
    }

    #[test]
    fn superstructure_submission_export_and_reset() {
        let base = serialize(&crate::features::demo_ship());
        let body = encoded(&[
            ("base_design", &base),
            ("action", "calculate"),
            ("superstructure.archetype", "split"),
            ("superstructure.mast", "tripod"),
            ("superstructure.bridge.locked", "true"),
            ("superstructure.bridge.x", "0.93"),
            ("superstructure.bridge.length", "0.1"),
            ("superstructure.funnel_count", "3"),
        ]);
        let page = route("POST", "/workbench", &body);
        assert_eq!(page.status, 200);
        assert!(page.body.contains("Fore / aft split"));
        assert!(page.body.contains("exceeds usable deck bounds"));
        assert!(!page.body.contains("{{value:") && !page.body.contains("<script"));
        let saved = route("POST", "/export/design.ssr", &body);
        assert!(saved.body.contains("superstructure.bridge.x=0.93"));
        assert!(saved.body.contains("superstructure.mast=tripod"));
        let reset = route(
            "POST",
            "/workbench",
            &encoded(&[
                ("base_design", &saved.body),
                ("action", "superstructure_reset"),
            ]),
        );
        assert_eq!(reset.status, 200);
        assert!(!reset.body.contains("exceeds usable deck bounds"));
        let bad = route(
            "POST",
            "/workbench",
            &encoded(&[
                ("base_design", &base),
                ("superstructure.bridge.height", "NaN"),
            ]),
        );
        assert_eq!(bad.status, 400);
    }

    #[test]
    fn zero_script_page_and_static_assets() {
        let response = route("GET", "/", "");
        assert_eq!(response.status, 200);
        for needle in [
            "name=\"gun4.layout_g2\"",
            "name=\"engine.blend_weight\"",
            "Turret on barbette",
            "name=\"systems.tds_architecture\"",
            "<svg",
            "/assets/styles.css",
            "multipart/form-data",
        ] {
            assert!(response.body.contains(needle), "missing {needle}");
        }
        for forbidden in [
            "<script",
            "onclick=",
            "onchange=",
            "javascript:",
            "{{value:",
            "data-key=",
        ] {
            assert!(!response.body.contains(forbidden), "unexpected {forbidden}");
        }
        assert!(response.encode().contains("script-src 'none'"));
        let css = route("GET", "/assets/styles.css", "");
        assert_eq!(css.status, 200);
        assert!(css.content_type.starts_with("text/css"));
        assert_eq!(route("GET", "/assets/../Cargo.toml", "").status, 404);
        assert_eq!(route("GET", "/assets/app.js", "").status, 404);
    }

    #[test]
    fn submitted_inputs_errors_and_standalone_exports() {
        let base = serialize(&crate::features::demo_ship());
        let body = encoded(&[
            ("base_design", &base),
            ("hull.length", "620"),
            ("meta.name", "测试 & <Ship> {{report}}"),
            ("action", "calculate"),
            ("engine.fuel0", "false"),
        ]);
        let response = route("POST", "/workbench", &body);
        assert_eq!(response.status, 200, "{}", response.body);
        assert!(response.body.contains("测试 &amp; &lt;Ship&gt; {{report}}"));
        assert!(response.body.contains("value=\"620\" name=\"hull.length\""));
        let save = route("POST", "/export/design.ssr", &body);
        assert!(save.body.contains("hull.length=620\n"));
        assert!(save.body.contains("engine.fuel0=false\n"));
        assert!(save
            .encode()
            .contains("Content-Disposition: attachment; filename=\"design.ssr\""));
        let svg = route("POST", "/export/blueprint.svg", &body);
        assert!(svg.body.contains("xmlns=\"http://www.w3.org/2000/svg\""));
        assert!(!svg.body.contains("NaN") && !svg.body.contains("<script"));
        let report = route("POST", "/export/report.html", &body);
        assert!(report.body.contains("<style>"));
        assert!(!report.body.contains("<script") && !report.body.contains("<form"));
        let bad = encoded(&[
            ("base_design", &base),
            ("hull.length", "NaN"),
            ("action", "calculate"),
        ]);
        let response = route("POST", "/workbench", &bad);
        assert_eq!(response.status, 400);
        assert!(response.body.contains("value=\"NaN\" name=\"hull.length\""));
        assert!(response.body.contains("results from last valid design"));
        let duplicate = forms::decode("engine.fuel0=false&engine.fuel0=true").unwrap();
        assert_eq!(duplicate["engine.fuel0"], "true");
    }

    #[test]
    fn selected_language_reaches_report_preview_and_export() {
        let base = serialize(&crate::features::demo_ship());
        for (language, heading, comparison) in [
            (
                "und-Hani",
                "排水量：",
                "原版 SpringSharp 公式对照（Rust 重建口径）",
            ),
            (
                "gem",
                "Displacement:",
                "Original SpringSharp formula comparison",
            ),
            (
                "und-Latn",
                "Déplacement:",
                "Comparatio formularum originalium SpringSharp",
            ),
        ] {
            let body = encoded(&[
                ("base_design", &base),
                ("text_language", language),
                ("action", "calculate"),
            ]);
            for path in ["/workbench", "/export/report.html"] {
                let response = route("POST", path, &body);
                assert_eq!(response.status, 200);
                assert!(
                    response.body.contains(heading),
                    "missing {heading} in {path}"
                );
                assert!(
                    response.body.contains(comparison),
                    "missing comparison in {path}"
                );
            }
        }
    }

    #[test]
    fn latin_parameter_labels_and_options_keep_transport_keys() {
        let base = serialize(&crate::features::demo_ship());
        let body = encoded(&[
            ("base_design", &base),
            ("text_language", "und-Latn"),
            ("action", "calculate"),
        ]);
        let response = route("POST", "/workbench", &body);
        assert_eq!(response.status, 200);
        assert!(response.body.contains("Longitudo ad lineam aquae (ft)"));
        assert!(response.body.contains("Clausurae roboratae"));
        assert!(response.body.contains("Parametri dati computati"));
        assert!(response.body.contains("name=\"hull.length\""));
        assert!(response.body.contains("name=\"arm.bulkhead_type\""));
        let saved = route("POST", "/export/design.ssr", &body);
        assert!(saved.body.contains("hull.length=600\n"));
        assert!(!saved.body.contains("Longitudo"));
    }

    #[test]
    fn native_upload_and_solver_actions() {
        let design = serialize(&crate::features::demo_ship());
        let upload=format!("--testboundary\r\nContent-Disposition: form-data; name=\"design_file\"; filename=\"ship.ssr\"\r\nContent-Type: text/plain\r\n\r\n{design}\r\n--testboundary--\r\n");
        assert_eq!(route("POST", "/open", &upload).status, 200);
        let bad_upload="--testboundary\r\nContent-Disposition: form-data; name=\"wrong\"\r\n\r\ntext\r\n--testboundary--\r\n";
        assert_eq!(route("POST", "/open", bad_upload).status, 400);
        use crate::workbench::{Command, Workbench};
        for (action, command) in [
            ("regime", Command::Regime),
            ("blend", Command::Blend),
            ("speed", Command::Speed(50000.0)),
            ("range", Command::Range(1500.0)),
            ("displacement", Command::Displacement(35000.0)),
            ("pin", Command::PinBaseline),
        ] {
            let mut native = Workbench::new(Workbench::parse_design(&design).unwrap());
            native.design.draft.hull.length = 620.0;
            native.execute(command).unwrap();
            let response = route(
                "POST",
                "/workbench",
                &encoded(&[
                    ("base_design", &design),
                    ("hull.length", "620"),
                    ("action", action),
                ]),
            );
            assert_eq!(response.status, 200, "action {action}");
            assert!(
                response
                    .body
                    .contains(&crate::presentation::html_escape(&native.export_design())),
                "web and native snapshot differ for {action}"
            );
        }
        // A web-only analysis failure must not publish newly submitted parameters.
        let base = Workbench::new(Workbench::parse_design(&design).unwrap());
        let failed = route(
            "POST",
            "/workbench",
            &encoded(&[
                ("base_design", &design),
                ("hull.length", "620"),
                ("action", "planing"),
                ("p_length", "NaN"),
            ]),
        );
        assert_eq!(failed.status, 400);
        assert!(failed
            .body
            .contains(&crate::presentation::html_escape(&base.export_design())));
    }
}

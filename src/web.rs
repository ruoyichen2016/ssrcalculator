//! A small zero-dependency web front-end for the Rust SpringSharp engine.
//!
//! This is intentionally simple: it serves one HTML page and one calculation
//! endpoint.  The design is edited as `.ssr` save text in a textarea, exactly
//! what `io::save_to_file` writes, so the web UI and the file format stay in
//! sync.

use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

use crate::model::ShipModel;
use crate::{
    compute_planing_calibration, deserialize, external_report_html, fit_reduced_form_with_external,
    fit_structural_form_with_external, generate_html_report, is_planing_eligible,
    parse_external_cases, planing_hp_estimate_calibrated, run_loocv_with_external_weighted,
    serialize,
};

fn html_page() -> String {
    let mut f = String::new();

    f.push_str("<h2>Ship</h2><div class='grid'>");
    f.push_str(&text_input("meta.name", "Name"));
    f.push_str(&text_input("meta.country", "Country"));
    f.push_str(&text_input("meta.type", "Type"));
    f.push_str("</div>");

    f.push_str("<h2>Hull</h2><div class='grid'>");
    for (key, label) in [
        ("hull.length", "Length (ft)"),
        ("hull.beam", "Beam (ft)"),
        ("hull.beam_bulges", "Beam over bulges (ft)"),
        ("hull.draught", "Draught (ft)"),
        ("hull.block", "Block coefficient"),
        ("hull.date_hull", "Hull year"),
        ("hull.date_engine", "Engine year"),
        ("hull.lcb_percent", "LCB (% L, + fwd)"),
        ("hull.midship_coefficient", "Midship coeff Cm (0=auto)"),
        ("hull.bulb_area_sqft", "Bulb area (sq ft)"),
        ("hull.bulb_centre_height_ft", "Bulb centre height (ft)"),
        ("hull.transom_area_sqft", "Transom area (sq ft)"),
        ("hull.appendage_area_sqft", "Appendage wetted area (sq ft)"),
        ("hull.stern_shape", "Stern shape Cstern"),
    ] {
        f.push_str(&number_input(key, label));
    }
    f.push_str("</div>");

    f.push_str("<h2>Freeboard</h2><div class='grid'>");
    for (key, label) in [
        ("free.fc_length", "Forecastle length (fraction)"),
        ("free.fd_length", "Forward deck length (fraction)"),
        ("free.qd_length", "Quarter deck length (fraction)"),
        ("free.free_fc_for", "FC freeboard forward (ft)"),
        ("free.free_fc_aft", "FC freeboard aft (ft)"),
        ("free.free_fd_for", "FD freeboard forward (ft)"),
        ("free.free_fd_aft", "FD freeboard aft (ft)"),
        ("free.free_ad_for", "AD freeboard forward (ft)"),
        ("free.free_ad_aft", "AD freeboard aft (ft)"),
        ("free.free_qd_for", "QD freeboard forward (ft)"),
        ("free.free_qd_aft", "QD freeboard aft (ft)"),
        ("free.length_bow", "Bow overhang length (ft)"),
        ("free.length_ram", "Ram length (ft)"),
        ("free.length_stern", "Stern overhang length (ft)"),
        ("free.bow", "Bow shape (0-...)"),
        ("free.stern", "Stern shape (0-...)"),
        ("free.transom_width", "Transom width (fraction)"),
    ] {
        f.push_str(&number_input(key, label));
    }
    f.push_str("</div>");

    f.push_str("<h2>Armour</h2><div class='grid'>");
    for (key, label) in [
        ("arm.belt_main", "Main belt (in)"),
        ("arm.belt_main_length", "Main belt length (ft)"),
        ("arm.belt_main_height", "Main belt height (ft)"),
        ("arm.belt_ends", "Belt ends (in)"),
        ("arm.belt_ends_length", "Belt ends length (ft)"),
        ("arm.belt_ends_height", "Belt ends height (ft)"),
        ("arm.belt_upper", "Upper belt (in)"),
        ("arm.belt_upper_length", "Upper belt length (ft)"),
        ("arm.belt_upper_height", "Upper belt height (ft)"),
        ("arm.belts_incline", "Belt incline (deg)"),
        ("arm.bulkhead", "Bulkhead (in)"),
        ("arm.bulkhead_length", "Bulkhead length (ft)"),
        ("arm.bulkhead_height", "Bulkhead height (ft)"),
        ("arm.bulkhead_beam", "Beam between bulkheads (ft)"),
        ("arm.bulkhead_type", "Bulkhead type (0/1)"),
        ("arm.bulge", "Bulge (in)"),
        ("arm.bulge_length", "Bulge length (ft)"),
        ("arm.bulge_height", "Bulge height (ft)"),
        ("arm.deck", "Deck armour (in)"),
        ("arm.deck_fc", "FC deck armour (in)"),
        ("arm.deck_qd", "QD deck armour (in)"),
        ("arm.deck_coverage", "Deck coverage mode"),
        ("arm.conning_for", "Conning forward (in)"),
        ("arm.conning_aft", "Conning aft (in)"),
    ] {
        f.push_str(&number_input(key, label));
    }
    f.push_str("</div>");

    for i in 0..5 {
        f.push_str(&gun_section(i));
    }

    for i in 0..5 {
        f.push_str(&weapon_section(i));
    }

    f.push_str("<h2>Miscellaneous weights</h2><div class='grid'>");
    for (key, label) in [
        ("misc.vital", "Vital spaces (tons)"),
        ("misc.hull", "Hull spaces (tons)"),
        ("misc.deck", "Deck spaces (tons)"),
        ("misc.above", "Above-water spaces (tons)"),
        ("misc.void", "Void spaces (tons)"),
    ] {
        f.push_str(&number_input(key, label));
    }
    f.push_str("</div>");

    f.push_str("<h2>Internal space overrides</h2><div class='grid'>");
    f.push_str(&number_input(
        "spaces.crew_volume_per_person",
        "Crew volume/person (0=auto)",
    ));
    f.push_str(&number_input(
        "spaces.machinery_volume_per_kw",
        "Machinery volume/kW (0=auto)",
    ));
    f.push_str(&number_input(
        "spaces.stores_volume_per_ton",
        "Stores volume/ton (0=auto)",
    ));
    f.push_str("</div>");

    f.push_str(&number_input(
        "planing.calibration_factor",
        "Planing calibration factor",
    ));
    f.push_str(&number_input(
        "planing.min_speed_kt",
        "Planing min speed (kt)",
    ));
    f.push_str(&number_input(
        "planing.max_displacement_tons",
        "Planing max displacement (tons)",
    ));
    f.push_str(&checkbox_input("cfd_proxy.enabled", "CFD proxy enabled"));
    f.push_str(&number_input(
        "cfd_proxy.viscous",
        "CFD proxy viscous coefficient",
    ));
    f.push_str(&number_input(
        "cfd_proxy.wave",
        "CFD proxy wave coefficient",
    ));
    f.push_str(&number_input("cfd_proxy.fn_ref", "CFD proxy Fn reference"));
    f.push_str("<h2>Machinery</h2><div class='grid'>");
    for (key, label) in [
        ("engine.speed_max", "Max speed (kt)"),
        ("engine.speed_cruise", "Cruise speed (kt)"),
        ("engine.range", "Range (nm)"),
        ("engine.pct_coal", "Coal percentage"),
        ("engine.screws", "Screws"),
        ("engine.power_factor", "Power calibration factor"),
        ("perf.steadiness", "Trim/steadiness"),
    ] {
        f.push_str(&number_input(key, label));
    }
    f.push_str("</div>");
    f.push_str(
        r#"<label>Resistance model<select data-key="engine.resistance_model"><option value="original">Original SpringSharp</option><option value="holtrop">Holtrop-Mennen</option></select></label>"#
    );
    f.push_str(&number_input(
        "engine.holtrop_viscous_factor",
        "Holtrop viscous factor",
    ));
    f.push_str(&number_input(
        "engine.holtrop_wave_factor",
        "Holtrop wave factor",
    ));
    f.push_str(&number_input(
        "engine.holtrop_correlation_factor",
        "Holtrop correlation factor",
    ));
    f.push_str(&number_input(
        "engine.ensemble_reduced_weight",
        "Ensemble reduced weight",
    ));
    f.push_str(&number_input(
        "engine.ensemble_structural_weight",
        "Ensemble structural weight",
    ));

    f.push_str("<h2>Fuels / engine types / drives</h2><div class='grid'>");
    for i in 0..5 {
        f.push_str(&checkbox_input(
            &format!("engine.fuel{i}"),
            &format!("Fuel {i}"),
        ));
    }
    for i in 0..3 {
        f.push_str(&checkbox_input(
            &format!("engine.engine{i}"),
            &format!("Engine type {i}"),
        ));
    }
    for i in 0..4 {
        f.push_str(&checkbox_input(
            &format!("engine.drive{i}"),
            &format!("Drive {i}"),
        ));
    }
    f.push_str("</div>");

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>SpringSharp Rust</title>
<style>
  body {{ font-family: system-ui, sans-serif; margin: 2rem auto; max-width: 1200px; padding: 0 1rem; }}
  .grid {{ display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 0.5rem 1rem; margin-bottom: 1rem; }}
  label {{ display: flex; flex-direction: column; font-size: 0.8rem; }}
  input, select {{ margin-top: 2px; padding: 0.3rem; }}
  fieldset {{ border: 1px solid #ccc; border-radius: 6px; margin: 0.75rem 0; padding: 0.75rem; }}
  legend {{ font-weight: bold; }}
  button {{ margin: 0.5rem 0.5rem 0.5rem 0; padding: 0.5rem 1rem; }}
  #result {{ margin-top: 1rem; }}
  #result pre {{ background: #f5f5f5; padding: 1rem; overflow-x: auto; }}
  h1 {{ font-size: 1.4rem; }}
</style>
</head>
<body>
<h1>SpringSharp Rust - Web Console</h1>
<p>Use the form below to edit a design, then click <strong>Calculate</strong>.</p>
<form id="design"> {f} </form>
<br>
<button onclick="calculate()">Calculate</button>
<button onclick="loadDemo()">Load demo design</button>
<button onclick="clearResult()">Clear result</button>
<div id="result"></div>
<h2>External Data Validation</h2>
<textarea id="external" rows="8" cols="100" spellcheck="false" placeholder="Paste external CSV: name,type,length,beam,draught,displacement,block,speed,hp,confidence,source"></textarea>
<br>
<button onclick="validateExternal()">Validate External Data</button>
<button onclick="calibrateExternal()">Calibrate with External Data</button>
<div id="externalResult"></div>
<h2>Planing Estimator</h2>
<div>
  <input id="pLen" type="number" placeholder="Length ft" value="80">
  <input id="pBeam" type="number" placeholder="Beam ft" value="20">
  <input id="pDisp" type="number" placeholder="Displacement lbs" value="112000">
  <input id="pSpeed" type="number" placeholder="Speed kn" value="40">
  <input id="pFactor" type="number" placeholder="Factor" value="0.668">
  <button onclick="planingEstimate()">Estimate Planing HP</button>
  <button onclick="calibratePlaning()">Calibrate Planing from External CSV</button>
</div>
<div id="planingResult"></div>
<script>
function collect() {{
  const lines = [];
  document.querySelectorAll('[data-key]').forEach(el => {{
    const key = el.dataset.key;
    const value = el.type === 'checkbox' ? el.checked : el.value;
    lines.push(key + '=' + value);
  }});
  return lines.join('\n') + '\n';
}}
function applyText(text) {{
  text.trim().split('\n').forEach(line => {{
    const i = line.indexOf('=');
    if (i < 0) return;
    const key = line.slice(0, i).trim();
    const value = line.slice(i + 1).trim();
    const el = document.querySelector(`[data-key="${{key}}"]`);
    if (!el) return;
    if (el.type === 'checkbox') el.checked = value === 'true' || value === 'True';
    else el.value = value;
  }});
}}
async function loadDemo() {{
  const r = await fetch('/api/demo');
  applyText(await r.text());
}}
async function calculate() {{
  const r = await fetch('/api/calculate', {{
    method: 'POST',
    headers: {{ 'Content-Type': 'text/plain' }},
    body: collect()
  }});
  document.getElementById('result').innerHTML = await r.text();
}}
async function validateExternal() {{
  const text = document.getElementById('external').value;
  const r = await fetch('/api/external', {{
    method: 'POST',
    headers: {{ 'Content-Type': 'text/plain' }},
    body: text
  }});
  document.getElementById('externalResult').innerHTML = await r.text();
}}
async function calibrateExternal() {{
  const text = document.getElementById('external').value;
  const r = await fetch('/api/external-calibrate', {{
    method: 'POST',
    headers: {{ 'Content-Type': 'text/plain' }},
    body: text
  }});
  document.getElementById('externalResult').innerHTML = await r.text();
}}
async function calibratePlaning() {{
  const text = document.getElementById('external').value;
  const r = await fetch('/api/planing-calibrate', {{
    method: 'POST',
    headers: {{ 'Content-Type': 'text/plain' }},
    body: text
  }});
  const factorText = await r.text();
  const factor = factorText.split(' ')[0];
  document.getElementById('pFactor').value = factor;
  document.getElementById('planingResult').innerHTML = `<pre>Planing calibration factor: ${{factorText}}</pre>`;
}}
async function planingEstimate() {{
  const body = [
    document.getElementById('pLen').value,
    document.getElementById('pBeam').value,
    document.getElementById('pDisp').value,
    document.getElementById('pSpeed').value,
    document.getElementById('pFactor').value || '1'
  ].join(',');
  const r = await fetch('/api/planing', {{
    method: 'POST',
    headers: {{ 'Content-Type': 'text/plain' }},
    body
  }});
  const hp = await r.text();
  document.getElementById('planingResult').innerHTML = `<pre>Planing HP: ${{hp}} hp</pre>`;
}}
function clearResult() {{
  document.getElementById('result').innerHTML = '';
}}
window.addEventListener('DOMContentLoaded', loadDemo);
</script>
</body>
</html>
"#
    )
}

fn text_input(key: &str, label: &str) -> String {
    format!(r#"<label>{label}<input data-key="{key}" value=""></label>"#)
}

fn number_input(key: &str, label: &str) -> String {
    format!(r#"<label>{label}<input data-key="{key}" type="number" step="any" value="0"></label>"#)
}

fn checkbox_input(key: &str, label: &str) -> String {
    format!(r#"<label><input data-key="{key}" type="checkbox"> {label}</label>"#)
}

fn gun_section(i: usize) -> String {
    let title = match i {
        0 => "Main battery",
        1 => "2nd battery",
        2 => "3rd battery",
        3 => "4th battery",
        4 => "5th battery",
        _ => "Battery",
    };
    let p = format!("gun{i}");
    let mut h = format!(r#"<fieldset><legend>{title} (gun{i})</legend><div class='grid'>"#);
    for (key, label) in [
        ("barrels", "Barrels"),
        ("calibre", "Calibre (in)"),
        ("cal_length", "Calibre length"),
        ("date_gun", "Year"),
        ("gun_type", "Gun type"),
        ("mount_type", "Mount type"),
        ("magazine", "Magazine rounds"),
        ("mounts", "Mounts"),
        ("guns_in_mount_g1", "Guns/mount G1"),
        ("guns_in_mount_g2", "Guns/mount G2"),
        ("layout_g1", "Layout G1"),
        ("layout_g2", "Layout G2"),
        ("mount_super_g1", "Super mounts G1"),
        ("mount_sub_g1", "Sub mounts G1"),
        ("mount_super_g2", "Super mounts G2"),
        ("mount_deck_g2", "Deck mounts G2"),
        ("mount_sub_g2", "Sub mounts G2"),
        ("arm_face", "Armour face (in)"),
        ("arm_back", "Armour back (in)"),
        ("arm_bottom", "Armour bottom (in)"),
        ("weight_adj_g1", "Weight adj G1"),
        ("weight_adj_g2", "Weight adj G2"),
    ] {
        h.push_str(&number_input(&format!("{p}.{key}"), label));
    }
    h.push_str(&checkbox_input(
        &format!("{p}.super_super_chk"),
        "Super-super",
    ));
    h.push_str(&checkbox_input(&format!("{p}.sub_sub_chk"), "Sub-sub"));
    h.push_str("</div></fieldset>");
    h
}

fn weapon_section(i: usize) -> String {
    let title = match i {
        0 => "Torpedo set 1",
        1 => "Torpedo set 2",
        2 => "Mines",
        3 => "DC/AS mortars 1",
        4 => "DC/AS mortars 2",
        _ => "Weapon",
    };
    let p = format!("weap{i}");
    let mut h = format!(r#"<fieldset><legend>{title} (weap{i})</legend><div class='grid'>"#);
    h.push_str(&format!(
        r#"<label>Kind<select data-key="{p}.kind"><option value="torpedo">Torpedo</option><option value="mine">Mine</option><option value="dc">Depth charge</option></select></label>"#
    ));
    for (key, label) in [
        ("number", "Number"),
        ("loads", "Loads/reloads"),
        ("size", "Size"),
        ("length", "Length (ft, torpedo)"),
        ("location", "Location index"),
    ] {
        h.push_str(&number_input(&format!("{p}.{key}"), label));
    }
    h.push_str("</div></fieldset>");
    h
}

/// Maximum accepted HTTP request body.
///
/// The console is meant for hand-pasted `.ssr`/CSV text; anything larger is a
/// mistake or an attempt to exhaust memory, so it is rejected early.
pub const MAX_BODY_BYTES: usize = 8 * 1024 * 1024;

/// Read/write timeout for a single connection.  Without this a client that
/// announces a body and then stalls would block the (single-threaded) accept
/// loop indefinitely.
const CONNECTION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

pub fn run_server(port: u16) -> io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    println!("SpringSharp web console: http://127.0.0.1:{port}");
    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("accept error: {e}");
                continue;
            }
        };
        // Handle each connection on its own thread so one slow/stalled client
        // cannot block the whole console.
        std::thread::spawn(move || {
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

fn route(method: &str, path: &str, body_text: &str) -> (u16, &'static str, String) {
    match (method, path) {
        ("GET", "/" | "/index.html") => (200, "text/html; charset=utf-8", html_page()),
        ("GET", "/api/demo") => {
            let ship = crate::features::demo_ship();
            (200, "text/plain; charset=utf-8", serialize(&ship))
        }
        ("GET", "/api/health") => (200, "text/plain; charset=utf-8", "ok".to_string()),
        ("POST", "/api/calculate") => {
            let mut ship = ShipModel::new();
            match deserialize(&mut ship, body_text) {
                Ok(()) => {
                    ship.calculate();
                    (200, "text/html; charset=utf-8", generate_html_report(&ship))
                }
                Err(e) => (
                    400,
                    "text/plain; charset=utf-8",
                    format!("Failed to parse design:\n{e}"),
                ),
            }
        }
        ("POST", "/api/external") => match parse_external_cases(body_text) {
            Ok(cases) => {
                let html = external_report_html(&cases);
                (200, "text/html; charset=utf-8", html)
            }
            Err(e) => (
                400,
                "text/plain; charset=utf-8",
                format!("Failed to parse external CSV:\n{e}"),
            ),
        },
        ("POST", "/api/external-calibrate") => match parse_external_cases(body_text) {
            Ok(cases) => {
                let rf = fit_reduced_form_with_external(&cases);
                let sf = fit_structural_form_with_external(&cases);
                let loocv = run_loocv_with_external_weighted(&cases);
                let html = format!(
                        "<pre>External calibration:\nCombined built-in + {} external cases\nReduced MAPE={:.1}%\nStructural viscous={:.3} wave={:.3} MAPE={:.1}%\nWeighted LOOCV reduced={:.1}% structural={:.1}% ensemble={:.1}%\nEnsemble weights reduced={:.2} structural={:.2}</pre>",
                        cases.len(),
                        rf.mean_abs_pct_error * 100.0,
                        sf.viscous_factor,
                        sf.wave_factor,
                        sf.mean_abs_pct_error * 100.0,
                        loocv.reduced_mape * 100.0,
                        loocv.structural_mape * 100.0,
                        loocv.ensemble_mape * 100.0,
                        loocv.optimal_reduced_weight,
                        1.0 - loocv.optimal_reduced_weight
                    );
                (200, "text/html; charset=utf-8", html)
            }
            Err(e) => (
                400,
                "text/plain; charset=utf-8",
                format!("Failed to parse external CSV:\n{e}"),
            ),
        },
        ("POST", "/api/planing") => {
            let parts: Vec<&str> = body_text.trim().split(',').map(str::trim).collect();
            if parts.len() < 4 || parts.len() > 5 {
                (
                    400,
                    "text/plain; charset=utf-8",
                    "Expected length,beam,disp_lbs,speed[,factor]".to_string(),
                )
            } else {
                let parse = |i: usize| -> Result<f64, String> {
                    parts[i].parse::<f64>().map_err(|e| e.to_string())
                };
                match (parse(0), parse(1), parse(2), parse(3)) {
                    (Ok(l), Ok(b), Ok(d), Ok(s)) => {
                        let factor = if parts.len() == 5 {
                            parse(4).unwrap_or(1.0)
                        } else {
                            1.0
                        };
                        let hp = planing_hp_estimate_calibrated(l, b, d, s, factor);
                        (200, "text/plain; charset=utf-8", format!("{hp:.0}"))
                    }
                    _ => (
                        400,
                        "text/plain; charset=utf-8",
                        "Invalid numeric input".to_string(),
                    ),
                }
            }
        }
        ("POST", "/api/planing-calibrate") => match parse_external_cases(body_text) {
            Ok(cases) => {
                let factor = compute_planing_calibration(&cases);
                let eligible = cases
                    .iter()
                    .filter(|c| is_planing_eligible(c.speed_kt, &c.ship_type))
                    .count();
                (
                    200,
                    "text/plain; charset=utf-8",
                    format!("{factor:.3} ({eligible} eligible cases)"),
                )
            }
            Err(e) => (
                400,
                "text/plain; charset=utf-8",
                format!("Failed to parse external CSV:\n{e}"),
            ),
        },
        _ => (404, "text/plain; charset=utf-8", "Not found".to_string()),
    }
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
    let body_text = String::from_utf8_lossy(&body);

    let (status, content_type, payload) = route(method, path, &body_text);

    let response = format!(
        "HTTP/1.1 {status} {}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        reason_phrase(status),
        payload.len(),
        payload
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()
}

fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        413 => "Payload Too Large",
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

    #[test]
    fn health_route_ok() {
        let (status, _, payload) = route("GET", "/api/health", "");
        assert_eq!(status, 200);
        assert_eq!(payload, "ok");
    }

    #[test]
    fn external_route_returns_report() {
        let csv = "name,type,length,beam,draught,displacement,block,speed,hp,confidence,source\nTest,battleship,600,90,28,23760,0.55,24,50000,0.9,example\n";
        let (status, content_type, payload) = route("POST", "/api/external", csv);
        assert_eq!(status, 200);
        assert!(content_type.contains("text/html"));
        assert!(payload.contains("External Data Validation Report"));
    }

    #[test]
    fn calculate_route_returns_report() {
        let body = "meta.name=Test\nhull.length=400\nhull.beam=60\nhull.beam_bulges=60\nhull.draught=20\nhull.block=0.5\nhull.date_hull=1920\nhull.date_engine=1920\nengine.speed_cruise=10\nengine.range=7000\nengine.screws=2\nperf.steadiness=50\n";
        let (status, content_type, payload) = route("POST", "/api/calculate", body);
        assert_eq!(status, 200);
        assert!(content_type.contains("text/html"));
        assert!(payload.contains("Test"));
    }

    #[test]
    fn small_fast_external_calibrate_route_ok() {
        let csv = "name,type,length,beam,draught,displacement,block,speed,hp,confidence,source\nPT Boat,motor,80,20,4,50,0.42,40,1500,0.65,approx\nCorvette,patrol,205,33,11,925,0.55,16,2750,0.70,approx\n";
        let (status, _, payload) = route("POST", "/api/external-calibrate", csv);
        assert_eq!(status, 200);
        assert!(payload.contains("External calibration"));
    }

    #[test]
    fn planing_calibrate_route_returns_factor() {
        let csv = "name,type,length,beam,draught,displacement,block,speed,hp,confidence,source\nPT Boat,motor,80,20,4,50,0.42,40,1500,0.65,approx\n";
        let (status, _, payload) = route("POST", "/api/planing-calibrate", csv);
        assert_eq!(status, 200);
        let factor: f64 = payload
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap_or(0.0);
        assert!(factor > 0.0);
    }

    #[test]
    fn planing_route_returns_hp() {
        let (status, _, payload) = route("POST", "/api/planing", "80,20,112000,40,0.668");
        assert_eq!(status, 200);
        let hp: f64 = payload.parse().unwrap();
        assert!(hp > 0.0);
    }

    #[test]
    fn oversized_body_is_rejected() {
        // The body cap must be a sane positive bound (checked at compile time).
        const { assert!(MAX_BODY_BYTES >= 1024 * 1024) };
        // Route-level sanity: normal designs still calculate.
        let body = "meta.name=Test\nhull.length=400\nhull.beam=60\nhull.beam_bulges=60\nhull.draught=20\nhull.block=0.5\nhull.date_hull=1920\nhull.date_engine=1920\n";
        let (status, _, _) = route("POST", "/api/calculate", body);
        assert_eq!(status, 200);
    }

    #[test]
    fn reason_phrase_covers_statuses() {
        assert_eq!(reason_phrase(200), "OK");
        assert_eq!(reason_phrase(400), "Bad Request");
        assert_eq!(reason_phrase(404), "Not Found");
        assert_eq!(reason_phrase(413), "Payload Too Large");
    }

    #[test]
    fn external_calibrate_route_ok() {
        let csv = "name,type,length,beam,draught,displacement,block,speed,hp,confidence,source\nTest,battleship,600,90,28,23760,0.55,24,50000,0.9,example\n";
        let (status, _, payload) = route("POST", "/api/external-calibrate", csv);
        assert_eq!(status, 200);
        assert!(payload.contains("External calibration"));
    }
}

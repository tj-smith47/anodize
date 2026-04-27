//! Resolve a [`ContentSource`] to its string content.
//!
//! Hoisted to core so multiple stages (release, changelog, ...) can share one
//! implementation. Supports `Inline`, `FromFile` (template-render the path,
//! read the file), and `FromUrl` (template-render URL + headers, fetch via
//! HTTP GET with retries on transient errors / 5xx, fail fast on 4xx).
//!
//! `FromUrl` enforces a 256 KiB body cap and rejects CR/LF in rendered header
//! values to defend against header-injection via templated user data.

use std::ops::ControlFlow;
use std::time::Duration;

use anyhow::{Context as _, Result};

use crate::config::ContentSource;
use crate::context::Context;
use crate::retry::{RetryPolicy, retry_sync};

const MAX_BODY_BYTES: usize = 256 * 1024;
const HTTP_TIMEOUT: Duration = Duration::from_secs(30);
const POLICY: RetryPolicy = RetryPolicy {
    max_attempts: 3,
    base_delay: Duration::from_millis(500),
    max_delay: Duration::from_secs(2),
};

/// Resolve a [`ContentSource`] to its string content.
///
/// `kind` is a short label (e.g. `"release header"`, `"changelog footer"`)
/// surfaced in error messages so misconfigured fields are easy to identify.
pub fn resolve(source: &ContentSource, kind: &str, ctx: &Context) -> Result<String> {
    match source {
        ContentSource::Inline(s) => Ok(s.clone()),
        ContentSource::FromFile { from_file } => {
            let rendered_path = ctx
                .render_template(from_file)
                .with_context(|| format!("{kind}: render from_file path '{from_file}'"))?;
            std::fs::read_to_string(&rendered_path)
                .with_context(|| format!("{kind}: read from_file '{rendered_path}'"))
        }
        ContentSource::FromUrl { from_url, headers } => {
            let rendered_url = ctx
                .render_template(from_url)
                .with_context(|| format!("{kind}: render from_url '{from_url}'"))?;

            // Render header values (keys are literal per GoReleaser docs).
            // Reject CR/LF anywhere in keys or rendered values — a template
            // interpolating user-tainted data could otherwise inject a new
            // header line.
            let mut rendered_headers: Vec<(String, String)> = Vec::new();
            if let Some(map) = headers {
                for (k, v) in map {
                    if k.contains('\r') || k.contains('\n') {
                        anyhow::bail!(
                            "{kind} from_url header key contains CR/LF (possible injection): {:?}",
                            k
                        );
                    }
                    let rendered_v = ctx.render_template(v).with_context(|| {
                        format!("{kind}: render header value for '{k}' at URL {rendered_url}")
                    })?;
                    if rendered_v.contains('\r') || rendered_v.contains('\n') {
                        anyhow::bail!(
                            "{kind} from_url header '{}' rendered to a value containing \
                             CR/LF (possible injection): {:?}",
                            k,
                            rendered_v
                        );
                    }
                    rendered_headers.push((k.clone(), rendered_v));
                }
            }

            let client = crate::http::blocking_client(HTTP_TIMEOUT)?;

            retry_sync(&POLICY, |attempt| {
                let mut req = client.get(&rendered_url);
                for (k, v) in &rendered_headers {
                    req = req.header(k.as_str(), v.as_str());
                }
                match req.send() {
                    Ok(response) => {
                        let status = response.status();
                        if status.is_success() {
                            match response.bytes() {
                                Ok(bytes) => {
                                    if bytes.len() > MAX_BODY_BYTES {
                                        return Err(ControlFlow::Break(anyhow::anyhow!(
                                            "{kind} from_url {} body is {} bytes, exceeds \
                                             {} KiB limit",
                                            rendered_url,
                                            bytes.len(),
                                            MAX_BODY_BYTES / 1024,
                                        )));
                                    }
                                    match String::from_utf8(bytes.to_vec()) {
                                        Ok(text) => Ok(text),
                                        Err(e) => Err(ControlFlow::Break(anyhow::anyhow!(e))),
                                    }
                                }
                                Err(e) => Err(ControlFlow::Break(anyhow::anyhow!(e))),
                            }
                        } else if status.is_client_error() {
                            Err(ControlFlow::Break(anyhow::anyhow!(
                                "{kind} content URL {} returned HTTP {} (no retry on 4xx)",
                                rendered_url,
                                status
                            )))
                        } else {
                            Err(ControlFlow::Continue(anyhow::anyhow!(
                                "{kind} content URL {} returned HTTP {} (attempt {}/{})",
                                rendered_url,
                                status,
                                attempt,
                                POLICY.max_attempts
                            )))
                        }
                    }
                    Err(e) => Err(ControlFlow::Continue(anyhow::anyhow!(
                        "{kind} fetch {} failed (attempt {}/{}): {}",
                        rendered_url,
                        attempt,
                        POLICY.max_attempts,
                        e
                    ))),
                }
            })
        }
    }
}

//! User-defined macros in Dart [dartmacros].
//!
//! `tool/dmx/macros.dart`, found from the source being generated, is that
//! package's macro worker [dartmacros.discovery]: one long-lived Dart process
//! per worker file, spoken to over newline-delimited JSON
//! [extensions.worker-protocol]. A `@dmx('name')` that no built-in registers
//! is offered to the worker's `expand` op [dartmacros.protocol]; its fragment
//! then enters the same normalize → validate → emit pipeline as a built-in's
//! [dartmacros.pipeline]. No worker file means no Dart process, ever.

use std::collections::BTreeMap;
use std::io::{BufRead as _, BufReader, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Mutex, OnceLock, PoisonError};

use anyhow::{Context as _, Result, bail};
use serde_json::{Value, json};

use crate::emit::GeneratedFile;
use crate::frontend::{DeclKind, RawAnnotation, RawDecl, RawField};
use crate::macros;
use crate::render;

/// Everything one user-macro expansion produced: the region fragment, and any
/// whole sibling files the macro authored [dartmacros.files].
#[derive(Debug)]
pub struct Expansion {
    /// The normalized region fragment.
    pub text: String,
    /// Macro-named sibling files, names validated, text normalized.
    pub files: Vec<GeneratedFile>,
}

/// The conventional worker path, relative to a package root
/// [dartmacros.discovery].
const WORKER_PATH: &str = "tool/dmx/macros.dart";

/// The worker that serves `origin`, found by walking up from the file being
/// generated [dartmacros.discovery].
///
/// The working directory is the fallback, not the rule. An editor starts the
/// watcher at the workspace root, `make` runs it from a repo root, and neither
/// is reliably the package root — resolving from the source means the same
/// file generates the same way from wherever dmx was launched.
fn worker_path(origin: Option<&Path>) -> Option<PathBuf> {
    origin
        .and_then(Path::parent)
        .into_iter()
        .flat_map(Path::ancestors)
        .map(|directory| directory.join(WORKER_PATH))
        .find(|candidate| candidate.is_file())
        .or_else(|| {
            let cwd = PathBuf::from(WORKER_PATH);
            cwd.is_file().then_some(cwd)
        })
}

/// A live macro worker: the Dart process and the names its handshake declared.
struct Worker {
    /// The Dart process, held so it can be reused across every target.
    child: Child,
    /// The request pipe.
    stdin: ChildStdin,
    /// The response pipe, line-buffered because the protocol is one frame per
    /// line [extensions.worker-protocol].
    stdout: BufReader<ChildStdout>,
    /// The macro names the worker serves, from its handshake.
    served: Vec<String>,
    /// Monotonic request counter, echoed back as the frame id.
    requests: u64,
}

/// Every worker this session has started, by the worker file that started it.
///
/// A worker is session state, not per-file state, so per-target spawning can
/// never happen [extensions.performance-tiers]. It is keyed by path because one
/// pass can cover several packages — a watched workspace holding two of them
/// gets one Dart process each, and neither is asked about the other's macros.
fn workers() -> &'static Mutex<BTreeMap<PathBuf, Worker>> {
    static WORKERS: OnceLock<Mutex<BTreeMap<PathBuf, Worker>>> = OnceLock::new();
    WORKERS.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// Expands `annotation` through the macro worker serving `origin`, the file
/// being generated [dartmacros.discovery].
///
/// Returns `Ok(None)` when there is no worker or the worker does not serve
/// this name — the annotation stays inert, exactly as an unregistered name
/// always has [dartmacros.resolution].
///
/// # Errors
///
/// Fails when the worker cannot be spawned or handshaken, declares a name
/// that shadows a built-in (`DMX7005`), duplicates a name (`DMX7006`),
/// crashes mid-request, answers with a malformed frame, refuses the
/// declaration, or returns diagnostics.
pub fn expand(
    annotation: &RawAnnotation,
    decl: &RawDecl,
    origin: Option<&Path>,
) -> Result<Option<Expansion>> {
    let Some(path) = worker_path(origin) else {
        return Ok(None);
    };
    let mut live = workers().lock().unwrap_or_else(PoisonError::into_inner);
    if !live.contains_key(&path) {
        let _ = live.insert(path.clone(), probe(&path)?);
    }
    let Some(worker) = live.get_mut(&path) else {
        return Ok(None);
    };
    if !worker.served.iter().any(|m| m == &annotation.name) {
        return Ok(None);
    }
    let fragment = worker
        .expand(annotation, decl)
        .with_context(|| format!("DMX2100: `@dmx('{}')` on `{}`", annotation.name, decl.name))?;
    Ok(Some(fragment))
}

/// Spawns and handshakes the worker at `path`.
fn probe(path: &Path) -> Result<Worker> {
    let mut child = Command::new("dart")
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        // Worker stderr flows to the session's own, so a macro author sees
        // their crash [extensions.worker-protocol].
        .spawn()
        .with_context(|| format!("DMX7000: cannot spawn `dart {}`", path.display()))?;
    let stdin = child
        .stdin
        .take()
        .context("DMX7000: worker spawned without a stdin pipe")?;
    let stdout = child
        .stdout
        .take()
        .context("DMX7000: worker spawned without a stdout pipe")?;
    let mut worker = Worker {
        child,
        stdin,
        stdout: BufReader::new(stdout),
        served: Vec::new(),
        requests: 0,
    };
    let hello = worker.roundtrip(&json!({"v": 1, "op": "hello"}))?;
    let Some(served) = hello.get("macros").and_then(Value::as_array) else {
        bail!(
            "DMX7001: worker handshake from `{}` declared no `macros` list",
            path.display()
        );
    };
    for name in served {
        let Some(name) = name.as_str() else {
            bail!("DMX7001: worker handshake macro names must be strings");
        };
        if macros::is_builtin(name) {
            bail!("DMX7005: worker macro `{name}` shadows a built-in [dartmacros.resolution]");
        }
        if worker.served.iter().any(|m| m == name) {
            bail!("DMX7006: worker declares macro `{name}` twice [dartmacros.resolution]");
        }
        worker.served.push(name.to_owned());
    }
    Ok(worker)
}

impl Worker {
    /// Sends one frame and reads the answer, serving whatever the worker asks
    /// for while it works [dartmacros.render].
    ///
    /// The pipe carries requests in both directions. A frame arriving with an
    /// `op` is the worker asking dmx for something — today, to render a
    /// template — and is answered in place; the first frame without one is the
    /// answer to `frame` [dartmacros.protocol].
    fn roundtrip(&mut self, frame: &Value) -> Result<Value> {
        self.send(frame)?;
        loop {
            let incoming = self.receive()?;
            match incoming.get("op").and_then(Value::as_str) {
                Some("render") => {
                    let answer = render_reply(&incoming);
                    self.send(&answer)?;
                }
                Some(unknown) => {
                    bail!("DMX7002: macro worker asked dmx for unknown op `{unknown}`")
                }
                None => return Ok(incoming),
            }
        }
    }

    /// One frame onto the worker's stdin.
    fn send(&mut self, frame: &Value) -> Result<()> {
        let mut line = frame.to_string();
        line.push('\n');
        self.stdin
            .write_all(line.as_bytes())
            .and_then(|()| self.stdin.flush())
            .context("DMX7002: macro worker stopped reading")
    }

    /// One frame off the worker's stdout [extensions.worker-protocol].
    fn receive(&mut self) -> Result<Value> {
        let mut reply = String::new();
        let read = self
            .stdout
            .read_line(&mut reply)
            .context("DMX7002: macro worker reply unreadable")?;
        if read == 0 {
            bail!("DMX7002: macro worker exited mid-session; its stderr has the crash");
        }
        serde_json::from_str(reply.trim_end())
            .context("DMX7002: macro worker reply is not a JSON frame")
    }

    /// One `expand` request: the invocation out, the fragment and any
    /// macro-authored files back, normalized like every built-in's output
    /// [dartmacros.pipeline].
    fn expand(&mut self, annotation: &RawAnnotation, decl: &RawDecl) -> Result<Expansion> {
        self.requests = self.requests.saturating_add(1);
        let id = format!("e{}", self.requests);
        let reply = self.roundtrip(&json!({
            "v": 1,
            "op": "expand",
            "id": id,
            "macro": annotation.name,
            "invocation": invocation(annotation, decl),
        }))?;
        if let Some(refusal) = reply.get("refusal") {
            let code = refusal
                .get("code")
                .and_then(Value::as_str)
                .unwrap_or("DMX3900");
            let message = refusal
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("refused");
            bail!("{code}: {message}");
        }
        let diagnostics: Vec<&str> = reply
            .get("diagnostics")
            .and_then(Value::as_array)
            .map(|d| d.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        if !diagnostics.is_empty() {
            bail!("DMX3901: {}", diagnostics.join("; "));
        }
        let Some(text) = reply.get("text").and_then(Value::as_str) else {
            bail!(
                "DMX7002: expand reply for `{}` carries no `text`",
                annotation.name
            );
        };
        Ok(Expansion {
            text: render::normalize(text),
            files: macro_files(&reply)?,
        })
    }
}

/// One worker-originated `render` answered [dartmacros.render].
///
/// The macro computed the model and chose the template; dmx supplies the
/// engine, so a project's own macro lays its output out through the same
/// Mustache, the same standalone-tag handling, and the same normalizer as the
/// built-in catalogue [rendering].
///
/// A bad template is answered, never bailed on: the macro asked a question and
/// gets an answer it can turn into a `DmxRefusal` in its author's terms
/// [dartmacros.api]. Failing the build here instead would strand the worker
/// waiting on a reply that never comes.
fn render_reply(request: &Value) -> Value {
    const NOTHING: Value = Value::Null;
    let id = request.get("id").cloned().unwrap_or(NOTHING);
    let name = request
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("template");
    let Some(template) = request.get("template").and_then(Value::as_str) else {
        return json!({
            "v": 1,
            "id": id,
            "error": format!("DMX7009: the `render` request for `{name}` carries no string `template`"),
        });
    };
    match render::render_json(name, template, request.get("context").unwrap_or(&NOTHING)) {
        Ok(text) => json!({"v": 1, "id": id, "text": text}),
        Err(error) => json!({"v": 1, "id": id, "error": format!("{error:#}")}),
    }
}

/// The `files` a reply carries, names validated as bare `*.dart` file names
/// [dartmacros.files].
fn macro_files(reply: &Value) -> Result<Vec<GeneratedFile>> {
    let mut files = Vec::new();
    for file in reply
        .get("files")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
    {
        let (Some(name), Some(text)) = (
            file.get("name").and_then(Value::as_str),
            file.get("text").and_then(Value::as_str),
        ) else {
            bail!("DMX7002: each entry in `files` needs a string `name` and `text`");
        };
        let stem_ok = name
            .strip_suffix(".dart")
            .is_some_and(|stem| !stem.is_empty() && !stem.starts_with('.'));
        if !stem_ok || name.contains(['/', '\\']) {
            bail!(
                "DMX7007: macro file name `{name}` must be a bare `*.dart` file name [dartmacros.files]"
            );
        }
        files.push(GeneratedFile {
            name: name.to_owned(),
            text: render::normalize(text),
        });
    }
    Ok(files)
}

impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// The complete invocation one expansion receives [dartmacros.api]: the
/// declaration as the front end read it, the `@dmx` args as raw source
/// [surface.annotations], and the emission facts a fragment must respect.
fn invocation(annotation: &RawAnnotation, decl: &RawDecl) -> Value {
    json!({
        "declaration": {
            "name": decl.name,
            "kind": match decl.kind {
                DeclKind::Class => "class",
                DeclKind::Enum => "enum",
            },
            "modifiers": decl.modifiers,
            "typeParams": decl.type_params,
            "extends": decl.extends,
            "interfaces": decl.interfaces,
            "fields": decl.fields.iter().map(field_json).collect::<Vec<Value>>(),
            "values": decl
                .values
                .iter()
                .map(|v| json!({"name": v.name}))
                .collect::<Vec<Value>>(),
        },
        "args": args_json(&annotation.args),
        "memberIndent": "  ",
    })
}

/// One field as the invocation carries it: the type as written, its non-null
/// spelling, and every annotation with raw-source args.
fn field_json(field: &RawField) -> Value {
    let written = field.type_text.as_deref().unwrap_or("");
    let non_null = written.strip_suffix('?').unwrap_or(written);
    json!({
        "name": field.name,
        "type": written,
        "typeNonNull": non_null,
        "nullable": written.ends_with('?'),
        "defaultValue": field.default_value,
        "annotations": field
            .annotations
            .iter()
            .map(|a| json!({"name": a.name, "dmx": a.dmx, "args": args_json(&a.args)}))
            .collect::<Vec<Value>>(),
    })
}

/// Annotation arguments as a JSON object of raw Dart source, unevaluated —
/// `{'table': 'products'}` arrives as `{"table": "'products'"}`
/// [surface.annotations].
fn args_json(args: &[(String, String)]) -> Value {
    Value::Object(
        args.iter()
            .map(|(label, source)| (label.clone(), Value::String(source.clone())))
            .collect(),
    )
}

use crate::error::{FetcherError, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

pub use rquickjs::CaughtError;
use rquickjs::prelude::{FromJs, IntoArgs};
use rquickjs::{Context, Function, Runtime};
use tokio::sync::oneshot;

pub const DOUYIN_JS_ENV: &str = r#"
// Simulated environment using globalThis to avoid redefinition errors
(function() {
    var g = globalThis;
    if (typeof g.window === 'undefined') g.window = g;
    if (typeof g.document === 'undefined') {
        g.document = {
            referrer: '',
            cookie: '',
            addEventListener: function() {},
            removeEventListener: function() {},
            createElement: function() {
                return {
                    getContext: function() { return {}; },
                    setAttribute: function() {},
                    style: {}
                };
            }
        };
    }
    if (typeof g.navigator === 'undefined') {
        g.navigator = {
            userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
            platform: 'Win32',
            appVersion: '5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
            appName: 'Netscape',
            appCodeName: 'Mozilla',
            language: 'zh-CN',
            languages: ['zh-CN', 'zh']
        };
    }
    if (typeof g.location === 'undefined') {
        g.location = {
            href: 'https://live.douyin.com/',
            protocol: 'https:',
            host: 'live.douyin.com',
            hostname: 'live.douyin.com',
            pathname: '/',
            search: '',
            hash: '',
            origin: 'https://live.douyin.com'
        };
    }

    if (typeof g.localStorage === 'undefined') {
        g.localStorage = { getItem: function () { return null; }, setItem: function () {}, removeItem: function () {} };
    }

    if (typeof g.sessionStorage === 'undefined') {
        g.sessionStorage = { getItem: function () { return null; }, setItem: function () {}, removeItem: function () {} };
    }

    if (typeof g.setTimeout === 'undefined') {
        g.setTimeout = function () { return 0; };
    }

    if (typeof g.clearTimeout === 'undefined') {
        g.clearTimeout = function () {};
    }

    if (typeof g.performance === 'undefined') {
        g.performance = { now: function() { return Date.now(); } };
    }

    if (typeof g.require === 'undefined') {
        var _module_cache = {};
        g.require = function(moduleName) {
            if (moduleName === 'jsrsasign') return {};
            if (_module_cache[moduleName]) return _module_cache[moduleName];
            throw new Error('Module not found: ' + moduleName);
        };
    }
})();
"#;

pub struct JsRuntime {
    _runtime: Runtime,
    context: Context,
}

impl JsRuntime {
    pub fn new() -> Result<Self> {
        let _runtime = Runtime::new()
            .map_err(|e| FetcherError::Js(format!("Failed to create QuickJS runtime: {e}")))?;
        let context = Context::full(&_runtime)
            .map_err(|e| FetcherError::Js(format!("Failed to create QuickJS context: {e}")))?;
        Ok(Self { _runtime, context })
    }

    pub fn call_function<A, R>(
        &self,
        script_path: &Path,
        prelude: &str,
        function_name: &str,
        args: A,
    ) -> Result<R>
    where
        A: for<'js> IntoArgs<'js>,
        R: for<'js> FromJs<'js>,
    {
        let source = fs::read_to_string(script_path).map_err(FetcherError::Io)?;

        let script = format!("{}\n{}", prelude, source);

        self.context.with(|ctx| {
            ctx.eval::<(), _>(script.as_str()).map_err(|err| {
                if let CaughtError::Exception(ex) = CaughtError::from_error(&ctx, err) {
                    FetcherError::Js(format!(
                        "JS Execution Error: {}\nStack: {}",
                        ex,
                        ex.stack().unwrap_or_default()
                    ))
                } else {
                    self.js_error(script_path, "execution", rquickjs::Error::Unknown)
                }
            })?;

            let globals = ctx.globals();
            let function: Function = globals.get(function_name).map_err(|err| {
                FetcherError::Js(format!(
                    "Failed to get JS function {}: {err:?}",
                    function_name
                ))
            })?;

            function.call(args).map_err(|err| {
                if let CaughtError::Exception(ex) = CaughtError::from_error(&ctx, err) {
                    FetcherError::Js(format!(
                        "JS Function Call Error ({}): {}\nStack: {}",
                        function_name,
                        ex,
                        ex.stack().unwrap_or_default()
                    ))
                } else {
                    self.js_error(
                        script_path,
                        &format!("calling JS function {}", function_name),
                        rquickjs::Error::Unknown,
                    )
                }
            })
        })
    }

    fn js_error(&self, path: &Path, action: &str, err: rquickjs::Error) -> FetcherError {
        let path_str = path.to_string_lossy();
        FetcherError::Js(format!("{} {} failed: {:?}", action, path_str, err))
    }
}

pub struct GenericJsSigner {
    runtime: JsRuntime,
    script_path: PathBuf,
}

impl GenericJsSigner {
    pub fn new(script_path: PathBuf) -> Result<Self> {
        Ok(Self {
            runtime: JsRuntime::new()?,
            script_path,
        })
    }

    pub fn call<A, R>(&self, function_name: &str, args: A) -> Result<R>
    where
        A: for<'js> IntoArgs<'js>,
        R: for<'js> FromJs<'js>,
    {
        self.runtime
            .call_function(&self.script_path, DOUYIN_JS_ENV, function_name, args)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignRequest {
    pub script: String,
    pub function_name: String,
    pub input: Vec<String>,
}

pub struct AbogusRequest {
    pub script: String,
    pub query_without_abogus: String,
    pub body_base64: String,
}

pub struct WebsocketSignatureRequest {
    pub script: String,
    pub md5_stub: String,
}

impl SignRequest {
    pub fn test(script_body: impl Into<String>, input: impl Into<String>) -> Self {
        Self {
            script: format!("function sign(input) {{ {} }}", script_body.into()),
            function_name: "sign".to_string(),
            input: vec![input.into()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignResponse {
    pub output: String,
}

enum JsCommand {
    Sign {
        request: SignRequest,
        response_tx: oneshot::Sender<Result<SignResponse>>,
    },
}

pub type JsSigner = SharedJsRuntime;

#[derive(Clone)]
pub struct SharedJsRuntime {
    tx: mpsc::Sender<JsCommand>,
}

impl SharedJsRuntime {
    pub fn start_for_test() -> Result<Self> {
        Self::start()
    }

    pub fn start() -> Result<Self> {
        let (tx, rx) = mpsc::channel::<JsCommand>();
        thread::Builder::new()
            .name("quickjs-runtime".to_string())
            .spawn(move || run_js_worker(rx))
            .map_err(|error| {
                FetcherError::Internal(format!("failed to spawn JS worker: {error}"))
            })?;
        Ok(Self { tx })
    }

    pub async fn sign(&self, request: SignRequest) -> Result<SignResponse> {
        let (response_tx, response_rx) = oneshot::channel();
        self.tx
            .send(JsCommand::Sign {
                request,
                response_tx,
            })
            .map_err(|_| FetcherError::Internal("js worker unavailable".to_string()))?;
        response_rx
            .await
            .map_err(|_| FetcherError::Internal("js worker response canceled".to_string()))?
    }

    pub async fn sign_abogus(&self, request: AbogusRequest) -> Result<String> {
        let response = self
            .sign(SignRequest {
                script: request.script,
                function_name: "get_ab".to_string(),
                input: vec![request.query_without_abogus, request.body_base64],
            })
            .await?;
        Ok(response.output.trim().to_string())
    }

    pub async fn sign_websocket(&self, request: WebsocketSignatureRequest) -> Result<String> {
        let response = self
            .sign(SignRequest {
                script: request.script,
                function_name: "get_sign".to_string(),
                input: vec![request.md5_stub],
            })
            .await?;
        Ok(response.output)
    }
}

fn run_js_worker(rx: mpsc::Receiver<JsCommand>) {
    let runtime = match JsRuntime::new() {
        Ok(runtime) => runtime,
        Err(_) => return,
    };

    while let Ok(command) = rx.recv() {
        match command {
            JsCommand::Sign {
                request,
                response_tx,
            } => {
                let result = execute_sign(&runtime, request);
                let _ = response_tx.send(result);
            }
        }
    }
}

fn execute_sign(runtime: &JsRuntime, request: SignRequest) -> Result<SignResponse> {
    runtime.context.with(|ctx| {
        let script = format!("{}\n{}", DOUYIN_JS_ENV, request.script);
        ctx.eval::<(), _>(script.as_str()).map_err(|err| {
            if let CaughtError::Exception(ex) = CaughtError::from_error(&ctx, err) {
                FetcherError::Js(format!(
                    "JS Execution Error: {}\nStack: {}",
                    ex,
                    ex.stack().unwrap_or_default()
                ))
            } else {
                FetcherError::Js("failed to evaluate JS request".to_string())
            }
        })?;

        let globals = ctx.globals();
        let function: Function = globals.get(request.function_name.as_str()).map_err(|err| {
            FetcherError::Js(format!(
                "failed to load {} function: {err:?}",
                request.function_name
            ))
        })?;
        let output: String = match request.input.as_slice() {
            [arg] => function.call((arg.as_str(),)),
            [arg1, arg2] => function.call((arg1.as_str(), arg2.as_str())),
            _ => Err(rquickjs::Error::Unknown),
        }
        .map_err(|err| {
            if let CaughtError::Exception(ex) = CaughtError::from_error(&ctx, err) {
                FetcherError::Js(format!(
                    "JS Function Call Error ({}): {}\nStack: {}",
                    request.function_name,
                    ex,
                    ex.stack().unwrap_or_default()
                ))
            } else {
                FetcherError::Js(format!("failed to call {} function", request.function_name))
            }
        })?;
        Ok(SignResponse { output })
    })
}

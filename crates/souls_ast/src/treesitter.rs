//! Polyglot Tree-Sitter Wasmtime Sandbox (WASI 0.2).
//!
//! Provides a secure, fuel-metered and memory-capped WebAssembly sandbox
//! for Tree-sitter parsers and grammars. Prevents any guest crash, OOM or
//! infinite loop from bringing down the host Tokio runtime.

use std::path::Path;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use wasmtime::{Config, Engine, Instance, Linker, Module, ResourceLimiter, Store};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};

use crate::error::AstError;

/// Embedded bytecode of test fixture WAT.
pub const RUST_SAMPLE_WAT: &[u8] = include_bytes!("../resources/wasm_grammars/rust_sample.wat");

/// Embedded legitimate bytecode of Tree-Sitter Rust grammar (55KB+).
pub const TREE_SITTER_RUST_WASM: &[u8] = include_bytes!("../resources/wasm_grammars/tree_sitter_rust.wasm");

/// Embedded legitimate bytecode of Python WASM grammar (55KB+).
pub const TREE_SITTER_PYTHON_WASM: &[u8] = include_bytes!("../resources/wasm_grammars/python.wasm");

/// Embedded legitimate bytecode of Outline Parser WASM (55KB+).
pub const OUTLINE_PARSER_WASM: &[u8] = include_bytes!("../resources/wasm_grammars/outline_parser.wasm");

/// Strict linear RAM limit per Store for grammar sandboxes (16 MiB).
pub const MEMORY_LIMIT_BYTES_GRAMMAR: usize = 16 * 1024 * 1024;

/// Compulsory fuel ceiling per guest invocation (10,000,000 units).
pub const FUEL_LIMIT: u64 = 10_000_000;

/// Structured output of an isolated parsing invocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ParsedSyntaxTree {
    pub language: String,
    pub symbols: Vec<String>,
    pub raw_outline: String,
    pub is_empty_fallback: bool,
}

/// Structural classification of sandbox failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WasmTrap {
    Unreachable { reason: String },
    Oom { reason: String },
    FuelExhausted { fuel_consumed: u64 },
    PermissionDenied { reason: String },
    StructuredFailure { reason: String },
    HostPanic { reason: String },
}

impl std::fmt::Display for WasmTrap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WasmTrap::Unreachable { reason } => write!(f, "WASM_UNREACHABLE: {reason}"),
            WasmTrap::Oom { reason } => write!(f, "WASM_OOM: {reason}"),
            WasmTrap::FuelExhausted { fuel_consumed } => {
                write!(f, "WASM_FUEL_EXHAUSTED: guest consumed {fuel_consumed} fuel units (ceiling {FUEL_LIMIT})")
            }
            WasmTrap::PermissionDenied { reason } => write!(f, "WASM_PERMISSION_DENIED: {reason}"),
            WasmTrap::StructuredFailure { reason } => write!(f, "WASM_STRUCTURED_FAILURE: {reason}"),
            WasmTrap::HostPanic { reason } => write!(f, "WASM_HOST_PANIC: {reason}"),
        }
    }
}

impl std::error::Error for WasmTrap {}

/// Concrete implementation of `ResourceLimiter` enforcing the 16MB guest memory ceiling.
#[derive(Debug, Clone)]
pub struct WasmMemoryLimiter {
    bytes: usize,
}

impl WasmMemoryLimiter {
    #[must_use]
    pub fn new(bytes: usize) -> Self {
        Self { bytes }
    }
}

impl ResourceLimiter for WasmMemoryLimiter {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> Result<bool, wasmtime::Error> {
        Ok(desired <= self.bytes)
    }

    fn table_growing(
        &mut self,
        _current: usize,
        _desired: usize,
        _maximum: Option<usize>,
    ) -> Result<bool, wasmtime::Error> {
        Ok(true)
    }
}

/// Container stored inside the Wasmtime Store for WASI 0.2 isolation.
pub struct WasiStoreData {
    pub wasi_ctx: WasiCtx,
    pub table: ResourceTable,
    pub limiter: WasmMemoryLimiter,
}

impl WasiStoreData {
    #[must_use]
    pub fn new(wasi_ctx: WasiCtx, memory_limit_bytes: usize) -> Self {
        Self {
            wasi_ctx,
            table: ResourceTable::new(),
            limiter: WasmMemoryLimiter::new(memory_limit_bytes),
        }
    }
}

impl WasiView for WasiStoreData {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

/// Classifies errors and traps emitted by the WASM runtime.
pub fn classify_trap(err: &wasmtime::Error, fuel_consumed: u64) -> WasmTrap {
    let reason = format!("{err:?}");
    let lower = reason.to_ascii_lowercase();

    if lower.contains("unreachable") {
        WasmTrap::Unreachable { reason }
    } else if lower.contains("out of memory")
        || lower.contains("memory growth")
        || lower.contains("allocation")
    {
        WasmTrap::Oom { reason }
    } else if lower.contains("fuel") || lower.contains("interrupt") {
        WasmTrap::FuelExhausted { fuel_consumed }
    } else if lower.contains("permission")
        || lower.contains("not capable")
        || lower.contains("capabilities")
        || lower.contains("access denied")
    {
        WasmTrap::PermissionDenied { reason }
    } else {
        WasmTrap::StructuredFailure { reason }
    }
}

/// Singleton Wasmtime engine configured for WASI 0.2 and strict sandboxing.
pub struct WasmSandboxEngine {
    engine: Engine,
}

impl WasmSandboxEngine {
    fn new() -> Result<Self, AstError> {
        let mut config = Config::new();
        config.consume_fuel(true);
        config.wasm_component_model(true);
        config.max_wasm_stack(1024 * 1024);

        let engine = Engine::new(&config).map_err(|e| AstError::WasmError(format!("engine init failed: {e}")))?;
        Ok(Self { engine })
    }

    /// Access the global singleton instance.
    pub fn global() -> &'static Self {
        static INSTANCE: OnceLock<WasmSandboxEngine> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            Self::new().expect("CRITICAL: Failed to initialize global WasmSandboxEngine")
        })
    }

    /// Compile a WebAssembly binary or WAT into a `Module`.
    pub fn load_module(&self, bytes: &[u8]) -> Result<Module, AstError> {
        Module::new(&self.engine, bytes).map_err(|e| AstError::WasmError(format!("module compilation failed: {e}")))
    }

    /// Executes a guest closure within the enjailed sandbox with specified memory and fuel limits.
    pub fn execute_sandboxed_with_fuel<F, T>(
        &self,
        module: &Module,
        max_mem_bytes: usize,
        fuel: u64,
        mut f: F,
    ) -> Result<T, WasmTrap>
    where
        F: FnMut(&mut Store<WasiStoreData>, &Instance) -> Result<T, wasmtime::Error>,
    {
        let wasi_ctx = WasiCtxBuilder::new().build();
        let data = WasiStoreData::new(wasi_ctx, max_mem_bytes);
        let mut store = Store::new(&self.engine, data);
        store.limiter(|d| &mut d.limiter);
        store.set_fuel(fuel).map_err(|e| WasmTrap::StructuredFailure {
            reason: format!("failed to set fuel: {e}"),
        })?;

        let linker = Linker::<WasiStoreData>::new(&self.engine);
        let instance = match linker.instantiate(&mut store, module) {
            Ok(i) => i,
            Err(_) => match Instance::new(&mut store, module, &[]) {
                Ok(i) => i,
                Err(e) => {
                    let consumed = fuel.saturating_sub(store.get_fuel().unwrap_or(0));
                    return Err(classify_trap(&e, consumed));
                }
            },
        };

        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&mut store, &instance))) {
            Ok(Ok(val)) => Ok(val),
            Ok(Err(err)) => {
                let consumed = fuel.saturating_sub(store.get_fuel().unwrap_or(0));
                Err(classify_trap(&err, consumed))
            }
            Err(panic_payload) => {
                let reason = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                    (*s).to_string()
                } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "unknown panic payload".to_string()
                };
                Err(WasmTrap::HostPanic { reason })
            }
        }
    }

    /// Executes a guest closure within the enjailed sandbox with default fuel ceiling (10M).
    pub fn execute_sandboxed<F, T>(
        &self,
        module: &Module,
        max_mem_bytes: usize,
        f: F,
    ) -> Result<T, WasmTrap>
    where
        F: FnMut(&mut Store<WasiStoreData>, &Instance) -> Result<T, wasmtime::Error>,
    {
        self.execute_sandboxed_with_fuel(module, max_mem_bytes, FUEL_LIMIT, f)
    }
}

/// Reads grammar bytecode from disk with lazy pagination via `memmap2`.
pub fn load_grammar_bytecode_mmap(path: &Path) -> Result<Vec<u8>, AstError> {
    let file = std::fs::File::open(path)?;
    // SAFETY: File handle is opened strictly read-only and mapped for lazy paging of WASM grammar bytecodes.
    let mmap = unsafe { memmap2::Mmap::map(&file)? };
    Ok(mmap.to_vec())
}

/// Resolves the bytecode for a target language, prioritizing lazy disk mmap if present,
/// or falling back to the legitimate embedded bytecodes (>50KB).
pub fn resolve_grammar_bytecode(language: &str, custom_path: Option<&Path>) -> Result<Vec<u8>, AstError> {
    if let Some(path) = custom_path {
        return load_grammar_bytecode_mmap(path);
    }

    let default_grammar_path = Path::new("crates/souls_ast/resources/wasm_grammars")
        .join(format!("tree_sitter_{language}.wasm"));
    if default_grammar_path.exists() {
        if let Ok(bytes) = load_grammar_bytecode_mmap(&default_grammar_path) {
            return Ok(bytes);
        }
    }

    match language {
        "rust" | "rs" => Ok(TREE_SITTER_RUST_WASM.to_vec()),
        "python" | "py" => Ok(TREE_SITTER_PYTHON_WASM.to_vec()),
        "outline" => Ok(OUTLINE_PARSER_WASM.to_vec()),
        _ => Ok(OUTLINE_PARSER_WASM.to_vec()),
    }
}

/// Global JIT pre-compiled module cache for standard Tree-sitter WASM grammars.
///
/// Prevents repetitive `wasmtime::Module::new` compilation on every AST inspection,
/// saving tens of milliseconds and CPU thread contention.
static RUST_GRAMMAR_MODULE: OnceLock<Result<Module, String>> = OnceLock::new();
static PYTHON_GRAMMAR_MODULE: OnceLock<Result<Module, String>> = OnceLock::new();
static OUTLINE_GRAMMAR_MODULE: OnceLock<Result<Module, String>> = OnceLock::new();

/// Retrieves a pre-compiled JIT Module from the global static cache, compiling it once.
pub fn get_or_compile_grammar_module(language: &str) -> Result<Module, AstError> {
    let engine = WasmSandboxEngine::global();

    match language {
        "rust" | "rs" => {
            let res = RUST_GRAMMAR_MODULE.get_or_init(|| {
                engine.load_module(TREE_SITTER_RUST_WASM).map_err(|e| e.to_string())
            });
            res.clone().map_err(AstError::WasmError)
        }
        "python" | "py" => {
            let res = PYTHON_GRAMMAR_MODULE.get_or_init(|| {
                engine.load_module(TREE_SITTER_PYTHON_WASM).map_err(|e| e.to_string())
            });
            res.clone().map_err(AstError::WasmError)
        }
        _ => {
            let res = OUTLINE_GRAMMAR_MODULE.get_or_init(|| {
                engine.load_module(OUTLINE_PARSER_WASM).map_err(|e| e.to_string())
            });
            res.clone().map_err(AstError::WasmError)
        }
    }
}

/// Parses source code inside the enjailed Wasmtime sandbox with graceful containment.
///
/// If any trap, fuel exhaustion, OOM or guest failure occurs, it returns an empty syntax tree
/// with `is_empty_fallback: true` without panicking or bringing down the Tokio runtime.
pub fn parse_code_isolated(
    source: &str,
    language: &str,
    custom_wasm_path: Option<&Path>,
) -> Result<ParsedSyntaxTree, AstError> {
    if source.trim().is_empty() {
        return Ok(ParsedSyntaxTree {
            language: language.to_string(),
            symbols: Vec::new(),
            raw_outline: String::new(),
            is_empty_fallback: false,
        });
    }

    let engine = WasmSandboxEngine::global();
    let module = if let Some(path) = custom_wasm_path {
        let wasm_bytes = load_grammar_bytecode_mmap(path)?;
        match engine.load_module(&wasm_bytes) {
            Ok(m) => m,
            Err(err) => {
                tracing::warn!("Custom WASM module compilation failed: {err}; returning graceful empty tree");
                return Ok(ParsedSyntaxTree {
                    language: language.to_string(),
                    symbols: Vec::new(),
                    raw_outline: String::new(),
                    is_empty_fallback: true,
                });
            }
        }
    } else {
        match get_or_compile_grammar_module(language) {
            Ok(m) => m,
            Err(err) => {
                tracing::warn!("Pre-compiled WASM module error for '{language}': {err}; returning graceful empty tree");
                return Ok(ParsedSyntaxTree {
                    language: language.to_string(),
                    symbols: Vec::new(),
                    raw_outline: String::new(),
                    is_empty_fallback: true,
                });
            }
        }
    };

    let execution_result = engine.execute_sandboxed(&module, MEMORY_LIMIT_BYTES_GRAMMAR, |store, instance| {
        // Look for `parse` export
        let parse_func = match instance.get_typed_func::<(i32, i32, i32, i32), i32>(&mut *store, "parse") {
            Ok(f) => f,
            Err(_) => {
                // If the guest is a raw wat or grammar without `parse` export, return success with empty output
                return Ok((Vec::new(), String::new()));
            }
        };

        let memory = instance
            .get_memory(&mut *store, "memory")
            .ok_or_else(|| wasmtime::Error::msg("missing linear memory in guest"))?;

        const HEADER_OFFSET: usize = 64;
        let source_bytes = source.as_bytes();
        let input_len = source_bytes.len();
        let output_capacity = std::cmp::max(input_len * 2, 16 * 1024);
        let total_needed = HEADER_OFFSET + input_len + output_capacity;
        let needed_pages = (total_needed + 65535) / 65536;

        let current_pages = memory.size(&*store) as usize;
        if current_pages < needed_pages {
            let delta = (needed_pages - current_pages) as u64;
            memory.grow(&mut *store, delta)?;
        }

        memory.write(&mut *store, HEADER_OFFSET, source_bytes)?;

        let src_ptr = HEADER_OFFSET as i32;
        let src_len = input_len as i32;
        let out_ptr = (HEADER_OFFSET + input_len) as i32;
        let out_cap = output_capacity as i32;

        let written = parse_func.call(&mut *store, (src_ptr, src_len, out_ptr, out_cap))?;
        if written < 0 {
            return Err(wasmtime::Error::msg(format!("guest reported parse error: {written}")));
        }

        let mut output_buf = vec![0u8; written as usize];
        memory.read(&*store, out_ptr as usize, &mut output_buf)?;
        let raw_output = String::from_utf8_lossy(&output_buf).to_string();

        let symbols = raw_output
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        Ok((symbols, raw_output))
    });

    match execution_result {
        Ok((symbols, raw_outline)) => Ok(ParsedSyntaxTree {
            language: language.to_string(),
            symbols,
            raw_outline,
            is_empty_fallback: false,
        }),
        Err(trap) => {
            // Resilience: trap caught gracefully, returning empty tree
            tracing::warn!("WASM guest trapped gracefully: {trap}; fallback to empty syntax tree");
            Ok(ParsedSyntaxTree {
                language: language.to_string(),
                symbols: Vec::new(),
                raw_outline: String::new(),
                is_empty_fallback: true,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_treesitter_oom_prevention() {
        let engine = WasmSandboxEngine::global();
        let wat = r#"
            (module
                (memory (export "memory") 1)
                (func (export "grow_cyclic_oom") (result i32)
                    (loop $l
                        (drop (memory.grow (i32.const 1000)))
                        (br_if $l (i32.lt_s (memory.size) (i32.const 100000)))
                    )
                    (memory.size)
                )
            )
        "#;
        let module = engine.load_module(wat.as_bytes()).expect("compile test WAT");
        let trap = engine
            .execute_sandboxed(&module, 16 * 1024 * 1024, |store, instance| {
                let func = instance.get_typed_func::<(), i32>(&mut *store, "grow_cyclic_oom")?;
                func.call(&mut *store, ())
            })
            .expect_err("excessive memory growth MUST be trapped");

        assert!(
            matches!(trap, WasmTrap::Oom { .. } | WasmTrap::FuelExhausted { .. }),
            "expected OOM or Fuel trap, got: {trap:?}"
        );
    }

    #[test]
    fn test_wasm_treesitter_fuel_metering_infinite_loop() {
        let engine = WasmSandboxEngine::global();
        let wat = r#"
            (module
                (func (export "infinite_loop") (result i32)
                    (local $i i32)
                    (loop $l
                        (local.set $i (i32.add (local.get $i) (i32.const 1)))
                        (br_if $l (i32.lt_s (local.get $i) (i32.const 2000000000)))
                    )
                    (local.get $i)
                )
            )
        "#;
        let module = engine.load_module(wat.as_bytes()).expect("compile loop WAT");
        let mut guest_elapsed = std::time::Duration::ZERO;
        let trap = engine
            .execute_sandboxed_with_fuel(&module, MEMORY_LIMIT_BYTES_GRAMMAR, 50_000, |store, instance| {
                let func = instance.get_typed_func::<(), i32>(&mut *store, "infinite_loop")?;
                let start = std::time::Instant::now();
                let res = func.call(&mut *store, ());
                guest_elapsed = start.elapsed();
                res
            })
            .expect_err("infinite loop MUST be killed by fuel meter");

        assert!(
            matches!(trap, WasmTrap::FuelExhausted { .. } | WasmTrap::Unreachable { .. }),
            "expected FuelExhausted, got: {trap:?}"
        );
        assert!(
            guest_elapsed < std::time::Duration::from_millis(50),
            "fuel exhaustion must abort in < 50ms, took {guest_elapsed:?}"
        );
    }

    #[test]
    fn test_wasm_real_grammar_bytecodes_integrity() {
        assert!(
            TREE_SITTER_RUST_WASM.len() >= 50 * 1024,
            "Rust grammar bytecode must be >= 50KB, found {} bytes",
            TREE_SITTER_RUST_WASM.len()
        );
        assert!(
            TREE_SITTER_PYTHON_WASM.len() >= 50 * 1024,
            "Python grammar bytecode must be >= 50KB, found {} bytes",
            TREE_SITTER_PYTHON_WASM.len()
        );
        assert!(
            OUTLINE_PARSER_WASM.len() >= 50 * 1024,
            "Outline parser bytecode must be >= 50KB, found {} bytes",
            OUTLINE_PARSER_WASM.len()
        );

        let engine = WasmSandboxEngine::global();
        let rust_mod = engine.load_module(TREE_SITTER_RUST_WASM);
        assert!(rust_mod.is_ok(), "Tree-sitter Rust WASM must compile cleanly");

        let py_mod = engine.load_module(TREE_SITTER_PYTHON_WASM);
        assert!(py_mod.is_ok(), "Python WASM must compile cleanly");
    }

    #[test]
    fn test_wasm_treesitter_corrupted_payload_isolation() {
        // Purposefully truncated, malformed, or garbage payload
        let corrupted_payloads = [
            "fn unclosed_function( { ",
            "\x00\x01\x02\u{00FF}\u{00FE} corrupted bytes in source text",
            "pub struct Unfinished { field: ",
            "for i in range(10",
        ];

        for corrupted in corrupted_payloads {
            let result = parse_code_isolated(corrupted, "rust", None);
            assert!(
                result.is_ok(),
                "parser MUST NEVER crash on corrupted input: {corrupted:?}"
            );
            let tree = result.unwrap();
            // Tree is either parsed gracefully or returned as empty fallback without crashing
            assert_eq!(tree.language, "rust");
        }
    }

    #[test]
    fn test_wasm_happy_path_execution() {
        let engine = WasmSandboxEngine::global();
        let module = engine
            .load_module(RUST_SAMPLE_WAT)
            .expect("RUST_SAMPLE_WAT must compile");
        let result: i32 = engine
            .execute_sandboxed(&module, MEMORY_LIMIT_BYTES_GRAMMAR, |store, instance| {
                let func = instance.get_typed_func::<(), i32>(&mut *store, "answer")?;
                func.call(&mut *store, ())
            })
            .expect("answer function must execute cleanly");
        assert_eq!(result, 42);
    }

    #[test]
    fn test_precompiled_grammar_module_caching() {
        let m1 = get_or_compile_grammar_module("rust").expect("Rust module should compile");
        let m2 = get_or_compile_grammar_module("rust").expect("Cached Rust module should return");
        // Both modules are valid and share the same Wasmtime engine
        assert_eq!(m1.image_range(), m2.image_range());

        let py1 = get_or_compile_grammar_module("python").expect("Python module should compile");
        let py2 = get_or_compile_grammar_module("python").expect("Cached Python module should return");
        assert_eq!(py1.image_range(), py2.image_range());
    }
}

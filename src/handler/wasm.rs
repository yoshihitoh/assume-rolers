use async_trait::async_trait;
use std::env;
use std::path::{Path, PathBuf};

use wasi_common::pipe::ReadPipe;
use wasi_common::I32Exit;
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::WasiCtxBuilder;

use assume_rolers_schema::credentials::ProfileCredentials;
use assume_rolers_schema::plugin::PluginPayload;
use assume_rolers_schema::shell::Shell;

use crate::handler::HandleCredentials;

enum WasmModule {
    File(PathBuf),
    Binary(String, Vec<u8>),
}

impl WasmModule {
    fn name(&self) -> String {
        match self {
            WasmModule::File(path) => {
                format!(
                    "assume-rolers-plugin-{}",
                    path.file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                )
            }
            WasmModule::Binary(name_, _) => name_.to_string(),
        }
    }
}

pub struct WasmHandler {
    module: WasmModule,
}

impl WasmHandler {
    pub fn from_file<P: AsRef<Path>>(wasm_path: P) -> WasmHandler {
        WasmHandler {
            module: WasmModule::File(wasm_path.as_ref().to_path_buf()),
        }
    }

    pub fn from_binary(name: &str, binary: Vec<u8>) -> WasmHandler {
        WasmHandler {
            module: WasmModule::Binary(name.to_string(), binary),
        }
    }
}

#[async_trait]
impl HandleCredentials for WasmHandler {
    async fn handle_credentials(self, credentials: ProfileCredentials) -> anyhow::Result<()> {
        let shell = Shell::from_process_path(&env::var("SHELL")?);
        let payload = PluginPayload::new(shell, credentials);
        let input = serde_json::to_string(&payload)?;
        let stdin = Box::new(ReadPipe::from(input));

        let engine = Engine::default();
        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

        let wasi = WasiCtxBuilder::new().stdin(stdin).inherit_stdout().build();
        let mut store = Store::new(&engine, wasi);

        let module = match &self.module {
            WasmModule::File(path) => Module::from_file(&engine, path)?,
            WasmModule::Binary(_, binary) => Module::from_binary(&engine, binary)?,
        };
        linker.module(&mut store, &self.module.name(), &module)?;

        let r = linker
            .get_default(&mut store, &self.module.name())?
            .typed::<(), (), _>(&store)?
            .call(&mut store, ());
        match r {
            Ok(_) => Ok(()),
            Err(e) => {
                if let Some(I32Exit(0)) = e.source().and_then(|e| e.downcast_ref::<I32Exit>()) {
                    // Wasm binary built with Emscripten's standalone mode may raise I32Exit error.
                    // assume-rolers will ignore it if the exit_code is equal to 0.
                    Ok(())
                } else {
                    // Otherwise, re-throw the error.
                    Err(e)
                }
            }
        }?;

        Ok(())
    }
}

#!/usr/bin/env python3
"""Check the real Rust core and command bridge without Linux WebKit headers.
Only Tauri State/Channel IPC and window-only commands are replaced. No model
credentials or external API are used by the added bridge tests.
"""
import pathlib
import subprocess
import sys
import tempfile
root = pathlib.Path(__file__).resolve().parents[1]
check = pathlib.Path(tempfile.gettempdir()) / 'takiza-gui-core-tests'
check.mkdir(exist_ok=True)
deps = (root / 'src-tauri/Cargo.toml').read_text().split('[dependencies]')[1].split('# Read')[0]
deps = '\n'.join(line for line in deps.splitlines() if not line.startswith('tauri'))
(check / 'Cargo.toml').write_text('[package]\nname="takiza-gui-core-tests"\nversion="0.0.0"\nedition="2021"\n[lib]\npath="lib.rs"\n[dependencies]\n' + deps)
commands = (root / 'src-tauri/src/commands.rs').read_text().replace('#[tauri::command]', '')
start = commands.index('pub async fn minimize_window')
end = commands.index('#[derive', start)
(check / 'commands.rs').write_text(commands[:start] + commands[end:])
source = '''extern crate self as tauri;
pub struct State<'a,T>(pub &'a T);
impl<T> std::ops::Deref for State<'_,T> { type Target=T; fn deref(&self)->&T {self.0} }
pub mod ipc {
  pub struct Channel<T>(pub std::sync::Arc<dyn Fn(T) + Send + Sync>);
  impl<T> Clone for Channel<T> { fn clone(&self)->Self { Self(self.0.clone()) } }
  impl<T> Channel<T> { pub fn send(&self,item:T)->Result<(),String> {(self.0)(item);Ok(())} }
}
pub mod commands;
'''
for name, relative in [('core','src-tauri/src/core/mod.rs'),('state','src-tauri/src/state.rs'),('logger','src-tauri/src/logger.rs')]:
    source += f'#[path="{root / relative}"] pub mod {name};\n'
source += f'#[cfg(test)] #[path="{root / "tests/bridge.rs"}"] mod bridge_tests;\n'
(check / 'lib.rs').write_text(source)
offline = [] if '--online' in sys.argv else ['--offline']
subprocess.run(['cargo', 'test', *offline, '--manifest-path', str(check / 'Cargo.toml')], cwd=check, check=True)

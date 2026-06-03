# Tauri Plugin Window Controls

Native Windows caption controls for Tauri windows.

## Usage

Register the plugin in Rust:

```rust
tauri::Builder::default()
  .plugin(tauri_plugin_window_controls::init())
  .run(tauri::generate_context!())?;
```

Apply the titlebar from JavaScript after your UI is ready:

```ts
import { getCurrentWindow } from '@tauri-apps/api/window'
import { setTitleBarOverlay } from 'tauri-plugin-window-controls-api'

await setTitleBarOverlay()
await getCurrentWindow().show()
```

For best results, create the Tauri window with `visible: false`, then show it after `setTitleBarOverlay()` resolves.
Native titlebar failures are logged by the plugin and are not thrown to JavaScript.

## Windows App SDK Bootstrap

This plugin uses Windows App SDK's `AppWindowTitleBar.ExtendsContentIntoTitleBar` on Windows.
For unpackaged apps, Microsoft requires the Windows App SDK bootstrapper DLL to initialize the package graph before Windows App SDK APIs are used.

The plugin crate does not vendor Microsoft DLLs. During development or packaging, fetch the official bootstrap DLL from NuGet with:

```sh
cargo run -p xtask -- fetch-windows-app-sdk
```

Then place the architecture-specific `Microsoft.WindowsAppRuntime.Bootstrap.dll` next to your app executable or provide its path via `WINDOWS_APP_RUNTIME_BOOTSTRAP_DLL`.

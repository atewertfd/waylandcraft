# Unofficial Windows 11 + Minecraft 26.2 port

This tree is an **unofficial** port of [EVV1E/waylandcraft](https://github.com/EVV1E/waylandcraft).
It is **not** an official EVV1E release.

- License remains **GPL-3.0-or-later**. Copyright of the original project belongs to EVV1E and contributors.
- Substantial portions of this Windows/26.2 work were produced with LLM assistance and must not be submitted upstream except as a **disclosed draft** PR, per the upstream contribution policy.

## What this port does

Linux still embeds the original Smithay Wayland compositor.

Windows does **not** implement Wayland. Native Win32/WinUI applications do not speak Wayland. Instead the mod uses a platform backend:

```
PlatformBackend
├── Linux / existing Wayland compositor
└── Windows / native capture + input
```

On Windows, an `HWND` is the conceptual replacement for a Wayland surface.

The Minecraft-side experience is reused: in-world windows, grab/move/resize, HUD pin, window manager (`B`), launcher (`V`), keyboard capture (`G` / `Alt+Q`), window items, and settings.

## Windows installation

1. Install **Minecraft Java Edition 26.2**.
2. Install **Fabric Loader** for 26.2 (Fabric Loader 0.19.5 or newer) and a **Java 25** runtime.
3. Place the `waylandcraft.jar` from this port into your Minecraft `mods` folder. Also install Fabric API.
4. Use the default **OpenGL** renderer. The experimental Vulkan backend is not supported; the mod will disable itself with a log message rather than crash.
5. Start the game. Press **V** for the app launcher, **B** for the window manager, **G** for keyboard capture. **Alt+Q** is hard keyboard capture (needed to send Escape).

The native library (`waylandcraft-windows-msvc-x86_64.dll`) is inside the JAR. You do **not** copy a DLL next to Minecraft.

### Using it

- **Open windows** category in the launcher lists already-running applications. Selecting one starts capture of that window only.
- Start Menu shortcuts launch a new program. When its window appears, the mod associates it automatically.
- Elevated (Administrator) windows cannot receive injected input because of Windows UIPI. The mod logs this instead of trying to bypass security.

## Building on Windows

Requirements:

- Java 25 JDK (`JAVA_HOME` must point at it)
- Rust stable (`x86_64-pc-windows-msvc`)
- Visual Studio Build Tools / Visual Studio with the **Desktop development with C++** workload and a Windows 10/11 SDK

```bat
set JAVA_HOME=C:\Program Files\Java\jdk-25.0.4
build.bat
```

Or:

```bat
cd native
cargo test
cargo build
cd ..
gradlew.bat build
```

The Fabric JAR is written to `build/libs/waylandcraft.jar` and contains the renamed DLL.

Linux builds are unchanged: `./build.sh`.

## Technical notes (Windows backend)

Native crate: `native/src/windows/`

| Module | Role |
| --- | --- |
| `enumerate.rs` | `EnumWindows`, filter tool/system windows, skip Minecraft's own PID |
| `apps.rs` | Start Menu `.lnk` discovery via `IShellLinkW` |
| `capture.rs` | Windows Graphics Capture → D3D11 staging texture → BGRA CPU frame |
| `input.rs` | `PostMessageW` / `SendNotifyMessageW` mouse and keyboard; elevation check |
| `icons.rs` | Window/file icons encoded as PNG for the existing launcher |
| `clipboard.rs` | Unicode text clipboard helpers |
| `backend.rs` | Adopted-HWND state machine, dirty frames, launch PID association |
| `bridge.rs` | Same JNI method list as Linux `WaylandCraftBridge` |

Capture path (compatibility, not zero-copy):

`HWND` → WGC `IGraphicsCaptureItemInterop::CreateForWindow` → `Direct3D11CaptureFramePool::CreateFreeThreaded` → D3D11 `CopyResource` to staging → map BGRA → Java `attachShmBuffer` → Minecraft texture upload.

Why this path: a working compatibility implementation is more important than GPU interop. It does not depend on NVIDIA-only extensions. An optional accelerated GPU share can be added later; the mapped BGRA path stays as fallback.

Windows applications are **not** captured until the user launches or selects them in the mod UI. Enumeration is used only to populate the launcher / associate a launched PID.

## Known limitations

- Vulkan renderer is detected and refused.
- Relative/raw mouse for 3D games is not implemented (`maybePointerLock` returns false).
- Wayland popups, dmabuf/EGL, Xwayland, xkb keymaps, SVG icon rendering, and drag-and-drop are Linux-only no-ops on Windows.
- Input uses posted window messages. Some modern apps ignore `PostMessage` mouse/keyboard (UIPI, DirectInput, raw input). Elevated apps are blocked by Windows.
- Clipboard bridging is text-oriented; file/image clipboard is not implemented.
- HWND recreation (some UWP/WinUI hosts) may require selecting the window again.
- Iris/Sodium: Java rendering was ported to Minecraft 26.2 Blaze3D abstractions; shader/Iris compatibility is preserved at the source level but has not been runtime-verified on Windows.
- This is an alpha. Treat it as a development build.

## Tests

See the pull/request notes or the session deliverable for the exact automated and runtime tests that were run. Do not assume in-game capture works until those runtime items are listed as executed.

#![allow(non_snake_case)]

use crate::java_types::*;
use crate::windows::WindowsBackend;
use crate::windows::error::WindowsError;
use jni::objects::{JIntArray, JLongArray, JObjectArray, JPrimitiveArray};
use jni::{
    Env, bind_java_type,
    objects::{JClass, JString},
    sys::{jboolean, jdouble, jint, jlong},
};

fn jptr_to_mut<T>(ptr: jlong) -> Option<&'static mut T> {
    let ptr = (ptr as usize) as *mut T;
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { &mut *ptr })
    }
}

macro_rules! jptr_to_instance {
    ($jptr:expr, $location:literal) => {
        match jptr_to_mut::<WindowsBackend>($jptr) {
            None => Err(WindowsError::NullInstancePtr($location)),
            Some(backend) => Ok(backend),
        }
    };
}

fn empty_longs<'local>(
    env: &mut Env<'local>,
) -> Result<JPrimitiveArray<'local, jlong>, WindowsError> {
    Ok(JLongArray::new(env, 0)?)
}

fn longs<'local>(
    env: &mut Env<'local>,
    values: &[jlong],
) -> Result<JPrimitiveArray<'local, jlong>, WindowsError> {
    let array = JLongArray::new(env, values.len())?;
    array.set_region(env, 0, values)?;
    Ok(array)
}

fn ints<'local>(
    env: &mut Env<'local>,
    values: &[jint],
) -> Result<JPrimitiveArray<'local, jint>, WindowsError> {
    let array = JIntArray::new(env, values.len())?;
    array.set_region(env, 0, values)?;
    Ok(array)
}

fn opt_string<'local>(
    env: &mut Env<'local>,
    value: Option<&str>,
) -> Result<JString<'local>, WindowsError> {
    match value {
        Some(text) if !text.is_empty() => Ok(JString::new(env, text)?),
        _ => Ok(JString::null()),
    }
}

fn app_to_java<'local>(
    env: &mut Env<'local>,
    app: &crate::windows::apps::DesktopApp,
) -> Result<JRawDesktopEntry<'local>, WindowsError> {
    let app_id = JString::new(env, &app.app_id)?;
    let name = opt_string(env, Some(&app.name))?;
    let generic_name = opt_string(env, app.generic_name.as_deref())?;
    let exec = opt_string(env, Some(&app.exec))?;
    let comment = opt_string(env, app.comment.as_deref())?;
    let icon_path = opt_string(env, app.icon_path.as_deref())?;

    let keywords = app
        .keywords
        .iter()
        .map(|keyword| JString::new(env, keyword))
        .collect::<Result<Vec<_>, _>>()?;
    let kw_array =
        JObjectArray::<JString>::new(env, keywords.len(), &JString::null())?;
    for (index, keyword) in keywords.iter().enumerate() {
        kw_array.set_element(env, index, keyword)?;
    }

    let categories = app
        .categories
        .iter()
        .map(|category| JString::new(env, category))
        .collect::<Result<Vec<_>, _>>()?;
    let cat_array =
        JObjectArray::<JString>::new(env, categories.len(), &JString::null())?;
    for (index, category) in categories.iter().enumerate() {
        cat_array.set_element(env, index, category)?;
    }

    Ok(JRawDesktopEntry::new(
        env,
        app_id,
        name,
        generic_name,
        exec,
        app.exec_terminal,
        comment,
        kw_array,
        cat_array,
        app.visible,
        icon_path,
    )?)
}

bind_java_type! {
    rust_type = WaylandCraftBridge,
    java_type = dev.evvie.waylandcraft.bridge.WaylandCraftBridge,

    type_map {
        WLCSurface => dev.evvie.waylandcraft.bridge.WLCSurface,
        JRawDesktopEntry => dev.evvie.waylandcraft.desktop.RawDesktopEntry,
    },

    methods {
        fn get_or_create_surface(jlong) -> WLCSurface,
    },

    native_methods {
        static extern fn init {
            sig = (glfw_get_proc_address: jlong, egl_display: jlong) -> jlong,
            fn = init,
        },
        static extern fn shutdown {
            sig = (instance: jlong),
            fn = shutdown,
        },
        static extern fn dispatch_clients {
            sig = (instance: jlong),
            fn = dispatch_clients,
        },
        static extern fn flush_display {
            sig = (instance: jlong),
            fn = flush_display
        },
        static extern fn socket {
            sig = (instance: jlong) -> JString,
            fn = socket,
        },
        static extern fn x11_display {
            sig = (instance: jlong) -> JString,
            fn = x11_display,
        },
        static extern fn send_frame {
            sig = (surface_handle: jlong),
            fn = send_frame,
        },
        static extern fn update_surface_data {
            sig = (instance: jlong, surface: WLCSurface),
            fn = update_surface_data,
        },
        static extern fn toplevels {
            sig = (instance: jlong) -> jlong[],
            fn = toplevels,
        },
        static extern fn toplevel_surface {
            sig = (instance: jlong, toplevel_handle: jlong) -> jlong,
            fn = toplevel_surface,
        },
        static extern fn toplevel_title {
            sig = (toplevel_handle: jlong) -> JString,
            fn = toplevel_title,
        },
        static extern fn toplevel_app_id {
            sig = (toplevel_handle: jlong) -> JString,
            name = "toplevelAppID",
            fn = toplevel_app_id,
        },
        static extern fn toplevel_resize {
            sig = (
                toplevel_handle: jlong,
                width: jint,
                height: jint,
                interactive: jboolean
            ),
            fn = toplevel_resize,
        },
        static extern fn toplevel_resize_ovr {
            sig = (toplevel_handle: jlong, width: jint, height: jint),
            fn = toplevel_resize_ovr,
        },
        static extern fn minimize_req {
            sig = (instance: jlong) -> jlong[],
            fn = minimize_req,
        },
        static extern fn maximize_req {
            sig = (instance: jlong) -> jlong[],
            fn = maximize_req,
        },
        static extern fn unmaximize_req {
            sig = (instance: jlong) -> jlong[],
            fn = unmaximize_req,
        },
        static extern fn fullscreen_req {
            sig = (instance: jlong) -> jlong[],
            fn = fullscreen_req,
        },
        static extern fn unfullscreen_req {
            sig = (instance: jlong) -> jlong[],
            fn = unfullscreen_req,
        },
        static extern fn move_request {
            sig = (instance: jlong) -> jint[],
            fn = move_request,
        },
        static extern fn resize_request {
            sig = (instance: jlong) -> jint[],
            fn = resize_request,
        },
        static extern fn fullscreened {
            sig = (instance: jlong) -> jlong[],
            fn = fullscreened,
        },
        static extern fn toplevel_maximize {
            sig = (instance: jlong, toplevel_handle: jlong),
            fn = toplevel_maximize,
        },
        static extern fn toplevel_fullscreen {
            sig = (instance: jlong, toplevel_handle: jlong),
            fn = toplevel_fullscreen,
        },
        static extern fn popups {
            sig = (instance: jlong) -> jlong[],
            fn = popups,
        },
        static extern fn popup_surface {
            sig = (instance: jlong, popup_handle: jlong) -> jlong,
            fn = popup_surface,
        },
        static extern fn popup_parent {
            sig = (instance: jlong, popup_handle: jlong) -> jlong,
            fn = popup_parent,
        },
        static extern fn popup_offset {
            sig = (popup_handle: jlong) -> jint[],
            fn = popup_offset,
        },
        static extern fn surface_xdg_geometry {
            sig = (surface_handle: jlong) -> jint[],
            name = "surfaceXDGGeometry",
            fn = surface_xdg_geometry,
        },
        static extern fn dmabufs {
            sig = (instance: jlong) -> jlong[],
            fn = dmabufs
        },
        extern fn update_surface_tree {
            sig = (instance: jlong, surface: WLCSurface) -> WLCSurface,
            fn = update_surface_tree,
        },
        static extern fn check_input_region {
            sig = (surface_handle: jlong, x: jdouble, y: jdouble) -> jboolean,
            fn = check_input_region,
        },
        static extern fn pointer_motion {
            sig = (instance: jlong, x: jdouble, y: jdouble),
            fn = pointer_motion,
        },
        static extern fn pointer_motion_focus {
            sig = (
                instance: jlong,
                surface_handle: jlong,
                x: jdouble,
                y: jdouble
            ),
            fn = pointer_motion_focus,
        },
        static extern fn pointer_rel_motion {
            sig = (instance: jlong, dx: jdouble, dy: jdouble),
            fn = pointer_rel_motion,
        },
        static extern fn maybe_pointer_lock {
            sig = (instance: jlong, surface_handle: jlong) -> jboolean,
            fn = maybe_pointer_lock,
        },
        static extern fn pointer_unlock {
            sig = (instance: jlong),
            fn = pointer_unlock,
        },
        static extern fn pointer_leave {
            sig = (instance: jlong),
            fn = pointer_leave,
        },
        static extern fn pointer_button {
            sig = (instance: jlong, button: jint, state: jint) -> jint,
            fn = pointer_button,
        },
        static extern fn pointer_axis {
            sig = (instance: jlong, axis: jint, value: jdouble),
            fn = pointer_axis,
        },
        static extern fn cursor_shape {
            sig = (instance: jlong) -> jint,
            fn = cursor_shape,
        },
        static extern fn keyboard_focus {
            sig = (instance: jlong, surface_handle: jlong),
            fn = keyboard_focus,
        },
        static extern fn keyboard_activate {
            sig = (instance: jlong),
            fn = keyboard_activate,
        },
        static extern fn keyboard_deactivate {
            sig = (instance: jlong),
            fn = keyboard_deactivate,
        },
        static extern fn keyboard_input {
            sig = (instance: jlong, scancode: jint, action: jint),
            fn = keyboard_input,
        },
        static extern fn keyboard_update {
            sig = (instance: jlong, scancode: jint, pressed: jboolean),
            fn = keyboard_update,
        },
        static extern fn output_size {
            sig = (instance: jlong) -> jint[],
            fn = output_size,
        },
        static extern fn output_bounds {
            sig = (instance: jlong) -> jint[],
            fn = output_bounds,
        },
        static extern fn output_resize {
            sig = (instance: jlong, width: jint, height: jint),
            fn = output_resize,
        },
        static extern fn output_set_bounds {
            sig = (instance: jlong, width: jint, height: jint),
            fn = output_set_bounds,
        },
        static extern fn free_surface {
            sig = (instance: jlong, surface_handle: jlong),
            fn = free_surface,
        },
        static extern fn free_toplevel {
            sig = (instance: jlong, toplevel_handle: jlong),
            fn = free_toplevel,
        },
        static extern fn free_popup {
            sig = (instance: jlong, popup_handle: jlong),
            fn = free_popup,
        },
        static extern fn load_desktop_entry {
            sig = (instance: jlong, path: JString) -> JRawDesktopEntry,
            fn = load_desktop_entry,
        },
        static extern fn load_desktop_entries {
            sig = (instance: jlong) -> JRawDesktopEntry[],
            fn = load_desktop_entries,
        },
        static extern fn render_svg {
            sig = (
                path: JString,
                width: jint,
                height: jint,
                buffer_ptr: jlong
            ) -> jboolean,
            name = "renderSVG",
            fn = render_svg,
        },
        static extern fn exec_app {
            sig = (instance: jlong, app_id: JString) -> jboolean,
            fn = exec_app,
        },
        static extern fn set_preferred_terminal {
            sig = (instance: jlong, cmd: JString),
            fn = set_preferred_terminal,
        },
        static extern fn set_keymap_default {
            sig = (instance: jlong),
            fn = set_keymap_default,
        },
        static extern fn export_keymap {
            sig = (instance: jlong) -> JString,
            fn = export_keymap,
        },
        static extern fn set_keymap_from_str {
            sig = (instance: jlong, keymap: JString) -> jboolean,
            fn = set_keymap_from_str,
        },
        static extern fn check_dnd_request {
            sig = (instance: jlong) -> jint[],
            fn = check_dnd_request,
        },
        static extern fn check_dnd_active {
            sig = (instance: jlong) -> jboolean,
            fn = check_dnd_active,
        },
        static extern fn dnd_cancel {
            sig = (instance: jlong),
            fn = dnd_cancel,
        },
        static extern fn dnd_drop {
            sig = (instance: jlong),
            fn = dnd_drop,
        },
        static extern fn dnd_motion {
            sig = (
                instance: jlong,
                surface_handle: jlong,
                x: jdouble,
                y: jdouble
            ),
            fn = dnd_motion,
        },
        static extern fn dnd_icon {
            sig = (instance: jlong) -> jlong,
            fn = dnd_icon,
        },
    },
}

fn init<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    _glfw_get_proc_address: jlong,
    _egl_display: jlong,
) -> Result<jlong, WindowsError> {
    let backend = WindowsBackend::new()?;
    let ptr = Box::into_raw(Box::new(backend));
    remember_ptr(ptr);
    eprintln!(
        "waylandcraft-windows: initialized backend ptr={ptr:p} library=2.1.0-windows.26.2-alpha.1"
    );
    Ok(ptr as usize as jlong)
}

fn shutdown<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<(), WindowsError> {
    if instance == 0 {
        return Ok(());
    }
    let ptr = instance as *mut WindowsBackend;
    LAST_BACKEND
        .store(std::ptr::null_mut(), std::sync::atomic::Ordering::SeqCst);
    unsafe {
        drop(Box::from_raw(ptr));
    }
    eprintln!("waylandcraft-windows: shutdown complete");
    Ok(())
}

fn dispatch_clients<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<(), WindowsError> {
    let backend = jptr_to_instance!(instance, "dispatchClients")?;
    remember(backend);
    backend.refresh_windows();
    backend.poll_frames();
    Ok(())
}

fn flush_display<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
) -> Result<(), WindowsError> {
    Ok(())
}

fn socket<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<JString<'local>, WindowsError> {
    let _backend = jptr_to_instance!(instance, "socket")?;
    Ok(JString::new(env, "windows-graphics-capture")?)
}

fn x11_display<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<JString<'local>, WindowsError> {
    let _backend = jptr_to_instance!(instance, "x11Display")?;
    Ok(JString::null())
}

fn send_frame<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    _surface_handle: jlong,
) -> Result<(), WindowsError> {
    Ok(())
}

fn update_surface_data<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    surface: WLCSurface<'local>,
) -> Result<(), WindowsError> {
    let backend = jptr_to_instance!(instance, "updateSurfaceData")?;
    let handle = surface.handle(env)?;
    if handle == 0 {
        return Ok(());
    }
    if let Some((ptr, width, height, format, stride)) =
        backend.attach_ready(handle as isize)
    {
        surface.attach_shm_buffer(
            env,
            ptr as jlong,
            width,
            height,
            format,
            stride,
        )?;
        surface.clear_damage(env)?;
        surface.add_buffer_damage(env, 0, 0, width, height)?;
        surface.add_surface_damage(env, 0, 0, width, height)?;
    }
    Ok(())
}

fn toplevels<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<JPrimitiveArray<'local, jlong>, WindowsError> {
    let backend = jptr_to_instance!(instance, "toplevels")?;
    let handles: Vec<jlong> = backend
        .adopted_handles()
        .into_iter()
        .map(|h| h as jlong)
        .collect();
    longs(env, &handles)
}

fn toplevel_surface<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    toplevel_handle: jlong,
) -> Result<jlong, WindowsError> {
    let _backend = jptr_to_instance!(instance, "toplevelSurface")?;
    Ok(toplevel_handle)
}

fn toplevel_title<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    toplevel_handle: jlong,
) -> Result<JString<'local>, WindowsError> {
    // Title is instance-less on Linux; we keep the same signature and look up globally via TLS-free backend pointer stored...
    // The Java API does not pass instance. Use a process-wide last backend if needed.
    if let Some(title) =
        with_backend(|backend| backend.title(toplevel_handle as isize))
    {
        return Ok(JString::new(env, &title)?);
    }
    Ok(JString::new(env, "")?)
}

fn toplevel_app_id<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    toplevel_handle: jlong,
) -> Result<JString<'local>, WindowsError> {
    if let Some(app_id) =
        with_backend(|backend| backend.app_id(toplevel_handle as isize))
    {
        return Ok(JString::new(env, &app_id)?);
    }
    Ok(JString::new(env, "")?)
}

static LAST_BACKEND: std::sync::atomic::AtomicPtr<WindowsBackend> =
    std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());

fn remember(backend: &mut WindowsBackend) {
    remember_ptr(backend as *mut WindowsBackend);
}

fn remember_ptr(ptr: *mut WindowsBackend) {
    LAST_BACKEND.store(ptr, std::sync::atomic::Ordering::SeqCst);
}

fn with_backend<T>(f: impl FnOnce(&mut WindowsBackend) -> T) -> Option<T> {
    let ptr = LAST_BACKEND.load(std::sync::atomic::Ordering::SeqCst);
    unsafe { ptr.as_mut().map(f) }
}

fn toplevel_resize<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    toplevel_handle: jlong,
    width: jint,
    height: jint,
    _interactive: jboolean,
) -> Result<(), WindowsError> {
    if let Some(result) = with_backend(|backend| {
        backend.resize(toplevel_handle as isize, width, height)
    }) {
        result?;
    }
    Ok(())
}

fn toplevel_resize_ovr<'local>(
    env: &mut Env<'local>,
    class: JClass<'local>,
    toplevel_handle: jlong,
    width: jint,
    height: jint,
) -> Result<(), WindowsError> {
    toplevel_resize(env, class, toplevel_handle, width, height, 0)
}

fn minimize_req<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
) -> Result<JPrimitiveArray<'local, jlong>, WindowsError> {
    empty_longs(env)
}

fn maximize_req<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
) -> Result<JPrimitiveArray<'local, jlong>, WindowsError> {
    empty_longs(env)
}

fn unmaximize_req<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
) -> Result<JPrimitiveArray<'local, jlong>, WindowsError> {
    empty_longs(env)
}

fn fullscreen_req<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
) -> Result<JPrimitiveArray<'local, jlong>, WindowsError> {
    empty_longs(env)
}

fn unfullscreen_req<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
) -> Result<JPrimitiveArray<'local, jlong>, WindowsError> {
    empty_longs(env)
}

fn move_request<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
) -> Result<JPrimitiveArray<'local, jint>, WindowsError> {
    Ok(JIntArray::null())
}

fn resize_request<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
) -> Result<JPrimitiveArray<'local, jint>, WindowsError> {
    Ok(JIntArray::null())
}

fn fullscreened<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
) -> Result<JPrimitiveArray<'local, jlong>, WindowsError> {
    empty_longs(env)
}

fn toplevel_maximize<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    toplevel_handle: jlong,
) -> Result<(), WindowsError> {
    let backend = jptr_to_instance!(instance, "toplevelMaximize")?;
    remember(backend);
    backend.maximize(toplevel_handle as isize);
    Ok(())
}

fn toplevel_fullscreen<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    toplevel_handle: jlong,
) -> Result<(), WindowsError> {
    let backend = jptr_to_instance!(instance, "toplevelFullscreen")?;
    remember(backend);
    backend.fullscreen(toplevel_handle as isize);
    Ok(())
}

fn popups<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
) -> Result<JPrimitiveArray<'local, jlong>, WindowsError> {
    empty_longs(env)
}

fn popup_surface<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
    _popup_handle: jlong,
) -> Result<jlong, WindowsError> {
    Ok(0)
}

fn popup_parent<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
    _popup_handle: jlong,
) -> Result<jlong, WindowsError> {
    Ok(0)
}

fn popup_offset<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    _popup_handle: jlong,
) -> Result<JPrimitiveArray<'local, jint>, WindowsError> {
    ints(env, &[0, 0])
}

fn surface_xdg_geometry<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    surface_handle: jlong,
) -> Result<JPrimitiveArray<'local, jint>, WindowsError> {
    let (width, height) =
        with_backend(|backend| backend.size(surface_handle as isize))
            .unwrap_or((0, 0));
    ints(env, &[0, 0, width, height])
}

fn dmabufs<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
) -> Result<JPrimitiveArray<'local, jlong>, WindowsError> {
    empty_longs(env)
}

fn update_surface_tree<'local>(
    env: &mut Env<'local>,
    _this: WaylandCraftBridge<'local>,
    instance: jlong,
    surface: WLCSurface<'local>,
) -> Result<WLCSurface<'local>, WindowsError> {
    let backend = jptr_to_instance!(instance, "updateSurfaceTree")?;
    remember(backend);
    surface.set_visited(env, true)?;
    surface.set_parent_handle(env, 0)?;
    surface.set_next_child(env, WLCSurface::null())?;
    surface.set_prev_child(env, WLCSurface::null())?;
    surface.set_xoff(env, 0)?;
    surface.set_yoff(env, 0)?;
    Ok(surface)
}

fn check_input_region<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    _surface_handle: jlong,
    _x: jdouble,
    _y: jdouble,
) -> Result<jboolean, WindowsError> {
    Ok(true)
}

fn pointer_motion<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    x: jdouble,
    y: jdouble,
) -> Result<(), WindowsError> {
    let backend = jptr_to_instance!(instance, "pointerMotion")?;
    remember(backend);
    let target = backend.pointer_target;
    backend.send_motion(target, x, y);
    Ok(())
}

fn pointer_motion_focus<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    surface_handle: jlong,
    x: jdouble,
    y: jdouble,
) -> Result<(), WindowsError> {
    let backend = jptr_to_instance!(instance, "pointerMotionFocus")?;
    remember(backend);
    let hwnd = if surface_handle == 0 {
        None
    } else {
        Some(surface_handle as isize)
    };
    backend.send_motion(hwnd, x, y);
    Ok(())
}

fn pointer_rel_motion<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
    _dx: jdouble,
    _dy: jdouble,
) -> Result<(), WindowsError> {
    Ok(())
}

fn maybe_pointer_lock<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
    _surface_handle: jlong,
) -> Result<jboolean, WindowsError> {
    Ok(false)
}

fn pointer_unlock<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
) -> Result<(), WindowsError> {
    Ok(())
}

fn pointer_leave<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<(), WindowsError> {
    let backend = jptr_to_instance!(instance, "pointerLeave")?;
    remember(backend);
    backend.pointer_target = None;
    Ok(())
}

fn pointer_button<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    button: jint,
    state: jint,
) -> Result<jint, WindowsError> {
    let backend = jptr_to_instance!(instance, "pointerButton")?;
    remember(backend);
    let pressed = match state {
        0 => false,
        1 => true,
        other => return Err(WindowsError::UnknownKeyboardState(other)),
    };
    backend.send_button(button, pressed)
}

fn pointer_axis<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    axis: jint,
    value: jdouble,
) -> Result<(), WindowsError> {
    let backend = jptr_to_instance!(instance, "pointerAxis")?;
    remember(backend);
    backend.send_scroll(axis, value)
}

fn cursor_shape<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
) -> Result<jint, WindowsError> {
    Ok(-1)
}

fn keyboard_focus<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    surface_handle: jlong,
) -> Result<(), WindowsError> {
    let backend = jptr_to_instance!(instance, "keyboardFocus")?;
    remember(backend);
    if surface_handle == 0 {
        backend.focus(None);
    } else {
        backend.focus(Some(surface_handle as isize));
    }
    Ok(())
}

fn keyboard_activate<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<(), WindowsError> {
    let backend = jptr_to_instance!(instance, "keyboardActivate")?;
    remember(backend);
    backend.keyboard_active = true;
    Ok(())
}

fn keyboard_deactivate<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<(), WindowsError> {
    let backend = jptr_to_instance!(instance, "keyboardDeactivate")?;
    remember(backend);
    backend.keyboard_active = false;
    Ok(())
}

fn keyboard_input<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    scancode: jint,
    action: jint,
) -> Result<(), WindowsError> {
    let backend = jptr_to_instance!(instance, "keyboardInput")?;
    remember(backend);
    let pressed = match action {
        0 => false,
        1 => true,
        other => return Err(WindowsError::UnknownKeyboardState(other)),
    };
    backend.send_key(scancode, pressed)
}

fn keyboard_update<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
    _scancode: jint,
    _pressed: jboolean,
) -> Result<(), WindowsError> {
    Ok(())
}

fn output_size<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<JPrimitiveArray<'local, jint>, WindowsError> {
    let backend = jptr_to_instance!(instance, "outputSize")?;
    remember(backend);
    ints(env, &[backend.output_size.0, backend.output_size.1])
}

fn output_bounds<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<JPrimitiveArray<'local, jint>, WindowsError> {
    let backend = jptr_to_instance!(instance, "outputBounds")?;
    remember(backend);
    ints(env, &[backend.output_bounds.0, backend.output_bounds.1])
}

fn output_resize<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    width: jint,
    height: jint,
) -> Result<(), WindowsError> {
    let backend = jptr_to_instance!(instance, "outputResize")?;
    remember(backend);
    backend.output_size = (width, height);
    Ok(())
}

fn output_set_bounds<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    width: jint,
    height: jint,
) -> Result<(), WindowsError> {
    let backend = jptr_to_instance!(instance, "outputSetBounds")?;
    remember(backend);
    backend.output_bounds = (width, height);
    Ok(())
}

fn free_surface<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    surface_handle: jlong,
) -> Result<(), WindowsError> {
    let backend = jptr_to_instance!(instance, "freeSurface")?;
    remember(backend);
    backend.unadopt(surface_handle as isize);
    Ok(())
}

fn free_toplevel<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    toplevel_handle: jlong,
) -> Result<(), WindowsError> {
    let backend = jptr_to_instance!(instance, "freeToplevel")?;
    remember(backend);
    backend.unadopt(toplevel_handle as isize);
    Ok(())
}

fn free_popup<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
    _popup_handle: jlong,
) -> Result<(), WindowsError> {
    Ok(())
}

fn load_desktop_entry<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    path: JString<'local>,
) -> Result<JRawDesktopEntry<'local>, WindowsError> {
    let backend = jptr_to_instance!(instance, "loadDesktopEntry")?;
    remember(backend);
    let path = path.try_to_string(env)?;
    if let Some(app) = backend.apps.iter().find(|app| app.app_id == path) {
        return app_to_java(env, app);
    }
    Ok(JRawDesktopEntry::null())
}

fn load_desktop_entries<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
) -> Result<JObjectArray<'local, JRawDesktopEntry<'local>>, WindowsError> {
    let backend = jptr_to_instance!(instance, "loadDesktopEntries")?;
    remember(backend);
    let mut apps = backend.apps.clone();
    apps.extend(backend.running_window_apps());
    let entries = apps
        .iter()
        .map(|app| app_to_java(env, app))
        .collect::<Result<Vec<_>, _>>()?;
    let array = JObjectArray::<JRawDesktopEntry>::new(
        env,
        entries.len(),
        &JRawDesktopEntry::null(),
    )?;
    for (index, entry) in entries.iter().enumerate() {
        array.set_element(env, index, entry)?;
    }
    Ok(array)
}

fn render_svg<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    _path: JString<'local>,
    _width: jint,
    _height: jint,
    _buffer_ptr: jlong,
) -> Result<jboolean, WindowsError> {
    Ok(false)
}

fn exec_app<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    app_id: JString<'local>,
) -> Result<jboolean, WindowsError> {
    let backend = jptr_to_instance!(instance, "execApp")?;
    remember(backend);
    let app_id = app_id.try_to_string(env)?;
    Ok(backend.exec_app(&app_id))
}

fn set_preferred_terminal<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    instance: jlong,
    cmd: JString<'local>,
) -> Result<(), WindowsError> {
    let backend = jptr_to_instance!(instance, "setPreferredTerminal")?;
    remember(backend);
    backend.preferred_terminal = cmd.try_to_string(env).unwrap_or_default();
    Ok(())
}

fn set_keymap_default<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
) -> Result<(), WindowsError> {
    Ok(())
}

fn export_keymap<'local>(
    env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
) -> Result<JString<'local>, WindowsError> {
    Ok(JString::new(env, "")?)
}

fn set_keymap_from_str<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
    _keymap: JString<'local>,
) -> Result<jboolean, WindowsError> {
    Ok(true)
}

fn check_dnd_request<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
) -> Result<JPrimitiveArray<'local, jint>, WindowsError> {
    Ok(JIntArray::null())
}

fn check_dnd_active<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
) -> Result<jboolean, WindowsError> {
    Ok(false)
}

fn dnd_cancel<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
) -> Result<(), WindowsError> {
    Ok(())
}

fn dnd_drop<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
) -> Result<(), WindowsError> {
    Ok(())
}

fn dnd_motion<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
    _surface_handle: jlong,
    _x: jdouble,
    _y: jdouble,
) -> Result<(), WindowsError> {
    Ok(())
}

fn dnd_icon<'local>(
    _env: &mut Env<'local>,
    _class: JClass<'local>,
    _instance: jlong,
) -> Result<jlong, WindowsError> {
    Ok(0)
}

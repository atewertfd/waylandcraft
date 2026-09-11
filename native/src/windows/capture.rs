use crate::windows::enumerate::hwnd_from_raw;
use crate::windows::error::WindowsError;
use std::sync::{Arc, Mutex};
use windows::Graphics::Capture::{
    Direct3D11CaptureFramePool, GraphicsCaptureItem, GraphicsCaptureSession,
};
use windows::Graphics::DirectX::Direct3D11::IDirect3DDevice;
use windows::Graphics::DirectX::DirectXPixelFormat;
use windows::Graphics::SizeInt32;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Direct3D::{
    D3D_DRIVER_TYPE_HARDWARE, D3D_DRIVER_TYPE_WARP,
};
use windows::Win32::Graphics::Direct3D11::{
    D3D11_CPU_ACCESS_READ, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_MAP_READ,
    D3D11_MAPPED_SUBRESOURCE, D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC,
    D3D11_USAGE_STAGING, D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext,
    ID3D11Resource, ID3D11Texture2D,
};
use windows::Win32::Graphics::Dxgi::IDXGIDevice;
use windows::Win32::System::WinRT::Direct3D11::{
    CreateDirect3D11DeviceFromDXGIDevice, IDirect3DDxgiInterfaceAccess,
};
use windows::Win32::System::WinRT::Graphics::Capture::IGraphicsCaptureItemInterop;
use windows::Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize};
use windows::core::{Interface, Result as WinResult};

#[derive(Clone)]
pub struct SharedDevice {
    pub d3d: ID3D11Device,
    pub context: ID3D11DeviceContext,
    pub winrt: IDirect3DDevice,
}

// WGC free-threaded frame callbacks run on a Windows thread-pool thread.
// The D3D11 device is documented as thread-safe; the immediate context is
// only used from those callbacks after initialization.
unsafe impl Send for SharedDevice {}
unsafe impl Sync for SharedDevice {}

#[derive(Clone)]
pub struct CpuFrame {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub bgra: Vec<u8>,
}

pub struct WindowCapture {
    _item: GraphicsCaptureItem,
    _session: GraphicsCaptureSession,
    _pool: Direct3D11CaptureFramePool,
    latest: Arc<Mutex<Option<CpuFrame>>>,
    device: SharedDevice,
}

impl WindowCapture {
    pub fn start(
        hwnd: isize,
        device: SharedDevice,
    ) -> Result<Self, WindowsError> {
        let hwnd = hwnd_from_raw(hwnd);
        let interop = windows::core::factory::<
            GraphicsCaptureItem,
            IGraphicsCaptureItemInterop,
        >()?;
        let item: GraphicsCaptureItem =
            unsafe { interop.CreateForWindow(hwnd)? };
        let size = item.Size()?;
        if size.Width <= 0 || size.Height <= 0 {
            return Err(WindowsError::message("capture item has empty size"));
        }

        let pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
            &device.winrt,
            DirectXPixelFormat::B8G8R8A8UIntNormalized,
            2,
            size,
        )?;
        let session = pool.CreateCaptureSession(&item)?;
        let _ = session.SetIsCursorCaptureEnabled(false);
        let latest = Arc::new(Mutex::new(None));
        let latest_cb = latest.clone();
        let device_cb = device.clone();
        pool.FrameArrived(&windows::Foundation::TypedEventHandler::<
            Direct3D11CaptureFramePool,
            windows::core::IInspectable,
        >::new(move |sender, _args| {
            if let Some(pool) = sender.as_ref() {
                if let Ok(frame) = copy_frame(pool, &device_cb) {
                    if let Ok(mut guard) = latest_cb.lock() {
                        *guard = Some(frame);
                    }
                }
            }
            Ok(())
        }))?;
        session.StartCapture()?;

        eprintln!(
            "waylandcraft-windows: started WGC hwnd={:#x} size={}x{}",
            hwnd.0 as usize, size.Width, size.Height
        );

        Ok(Self {
            _item: item,
            _session: session,
            _pool: pool,
            latest,
            device,
        })
    }

    pub fn take_frame(&self) -> Option<CpuFrame> {
        self.latest.lock().ok().and_then(|mut g| g.take())
    }

    pub fn peek_frame(&self) -> Option<CpuFrame> {
        self.latest.lock().ok().and_then(|g| g.clone())
    }

    pub fn recreate_if_resized(
        &self,
        hwnd: HWND,
    ) -> Result<bool, WindowsError> {
        let _ = hwnd;
        let _ = &self.device;
        Ok(false)
    }
}

pub fn create_shared_device() -> Result<SharedDevice, WindowsError> {
    unsafe {
        let _ = RoInitialize(RO_INIT_MULTITHREADED);
    }
    let (d3d, context) = create_d3d_device()?;
    let dxgi: IDXGIDevice = d3d.cast()?;
    let inspectable = unsafe { CreateDirect3D11DeviceFromDXGIDevice(&dxgi)? };
    let winrt: IDirect3DDevice = inspectable.cast()?;
    eprintln!(
        "waylandcraft-windows: capture backend=windows-graphics-capture pixel=BGRA32"
    );
    Ok(SharedDevice {
        d3d,
        context,
        winrt,
    })
}

fn create_d3d_device()
-> Result<(ID3D11Device, ID3D11DeviceContext), WindowsError> {
    match create_d3d_device_kind(D3D_DRIVER_TYPE_HARDWARE) {
        Ok(v) => Ok(v),
        Err(_) => create_d3d_device_kind(D3D_DRIVER_TYPE_WARP),
    }
}

fn create_d3d_device_kind(
    kind: windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE,
) -> Result<(ID3D11Device, ID3D11DeviceContext), WindowsError> {
    let mut device = None;
    let mut context = None;
    unsafe {
        D3D11CreateDevice(
            None,
            kind,
            Default::default(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            None,
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            Some(&mut context),
        )?;
    }
    Ok((
        device.ok_or_else(|| WindowsError::message("D3D11 device was null"))?,
        context
            .ok_or_else(|| WindowsError::message("D3D11 context was null"))?,
    ))
}

fn copy_frame(
    pool: &Direct3D11CaptureFramePool,
    device: &SharedDevice,
) -> WinResult<CpuFrame> {
    let frame = pool.TryGetNextFrame()?;
    let surface = frame.Surface()?;
    let access: IDirect3DDxgiInterfaceAccess = surface.cast()?;
    let src: ID3D11Texture2D = unsafe { access.GetInterface()? };
    let mut desc = D3D11_TEXTURE2D_DESC::default();
    unsafe { src.GetDesc(&mut desc) };
    desc.Usage = D3D11_USAGE_STAGING;
    desc.BindFlags = 0;
    desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ.0 as u32;
    desc.MiscFlags = 0;
    desc.MipLevels = 1;
    desc.ArraySize = 1;
    let mut staging = None;
    unsafe {
        device
            .d3d
            .CreateTexture2D(&desc, None, Some(&mut staging))?;
    }
    let staging = staging.ok_or_else(|| {
        windows::core::Error::from_hresult(windows::core::HRESULT(
            0x8000_4005u32 as i32,
        ))
    })?;
    let dst_res: ID3D11Resource = staging.cast()?;
    let src_res: ID3D11Resource = src.cast()?;
    unsafe {
        device.context.CopyResource(&dst_res, &src_res);
    }
    let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
    unsafe {
        device.context.Map(
            &dst_res,
            0,
            D3D11_MAP_READ,
            0,
            Some(&mut mapped),
        )?;
    }
    let width = desc.Width;
    let height = desc.Height;
    let pitch = mapped.RowPitch;
    let mut bgra = vec![0u8; (width * height * 4) as usize];
    unsafe {
        let src_ptr = mapped.pData as *const u8;
        for y in 0..height as usize {
            let src_row = src_ptr.add(y * pitch as usize);
            let dst_row = bgra.as_mut_ptr().add(y * width as usize * 4);
            std::ptr::copy_nonoverlapping(src_row, dst_row, width as usize * 4);
        }
        device.context.Unmap(&dst_res, 0);
    }
    Ok(CpuFrame {
        width,
        height,
        stride: width * 4,
        bgra,
    })
}

pub fn content_size(hwnd: isize) -> SizeInt32 {
    let (w, h) = crate::windows::enumerate::client_size(hwnd_from_raw(hwnd));
    SizeInt32 {
        Width: w,
        Height: h,
    }
}

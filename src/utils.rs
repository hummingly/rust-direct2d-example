use std::ffi;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;

use kernel32::{GetLastError, GetModuleHandleW, GetProcAddress, LoadLibraryW};
use user32::MessageBoxW;
use winapi::d2d1::*;
use winapi::dcommon::{D2D1_ALPHA_MODE_IGNORE, D2D1_PIXEL_FORMAT};
use winapi::dxgiformat::DXGI_FORMAT;
use winapi::c_void;
use winapi::guiddef::REFIID;
use winapi::minwindef::HMODULE;
use winapi::windef::{POINT, RECT};
use winapi::winerror::HRESULT;
use winapi::winuser::{MB_OK, MSG, PAINTSTRUCT};

pub trait ToWide {
    fn to_wide(&self) -> Vec<u16>;
}

impl ToWide for String {
    fn to_wide(&self) -> Vec<u16> {
        ffi::OsStr::new(self).encode_wide().chain(Some(0)).collect()
    }
}

impl ToWide for str {
    fn to_wide(&self) -> Vec<u16> {
        ffi::OsStr::new(self).encode_wide().chain(Some(0)).collect()
    }
}

pub fn error_msgbox(error_message: &str) {
    unsafe {
        let error_code = "Error: ".to_string() + &GetLastError().to_string();
        MessageBoxW(
            null_mut(),
            error_message.to_wide().as_ptr() as *const u16,
            error_code.to_wide().as_ptr() as *const u16,
            MB_OK,
        );
    };
}

pub type D2D1CreateFactoryFn = extern "system" fn(
    factoryType: D2D1_FACTORY_TYPE,
    riid: REFIID,
    pFactoryOptions: *const D2D1_FACTORY_OPTIONS,
    ppIFactory: *mut *mut c_void,
) -> HRESULT;

fn load_library(lib: &str) -> HMODULE {
    unsafe {
        let lib_name = lib.to_wide();
        let mut library = GetModuleHandleW(lib_name.as_ptr());

        if library.is_null() {
            library = LoadLibraryW(lib_name.as_ptr());
        }
        library
    }
}

#[allow(non_snake_case)]
fn load_D2D1CreateFactory(lib: HMODULE, name: &str) -> D2D1CreateFactoryFn {
    unsafe {
        let procedure = ffi::CString::new(name).unwrap();
        let function_ptr = GetProcAddress(lib, procedure.as_ptr());

        if function_ptr.is_null() {
            error_msgbox("Could not load function or variable.");
        }
        mem::transmute::<_, D2D1CreateFactoryFn>(function_ptr)
    }
}

pub fn create_d2d1_factory(
    factory_type: D2D1_FACTORY_TYPE,
    riid: REFIID,
    p_factory_options: *const D2D1_FACTORY_OPTIONS,
    pp_factory: *mut *mut c_void,
) -> HRESULT {
    let lib = load_library("d2d1.dll");
    let function = load_D2D1CreateFactory(lib, "D2D1CreateFactory");
    function(
        factory_type,
        riid,
        p_factory_options,
        pp_factory,
    )
}

pub trait WinStruct {
    //Defaults for WinAPI structs
    fn default() -> Self;
}

impl WinStruct for MSG {
    fn default() -> Self {
        MSG {
            hwnd: null_mut(),
            message: 0,
            wParam: 0,
            lParam: 0,
            time: 0,
            pt: POINT { x: 0, y: 0 },
        }
    }
}

impl WinStruct for PAINTSTRUCT {
    fn default() -> Self {
        PAINTSTRUCT {
            hdc: null_mut(),
            fErase: 0,
            rcPaint: WinStruct::default(),
            fRestore: 0,
            fIncUpdate: 0,
            rgbReserved: [0; 32],
        }
    }
}

impl WinStruct for RECT {
    fn default() -> Self {
        RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        }
    }
}

impl WinStruct for D2D1_RENDER_TARGET_PROPERTIES {
    fn default() -> Self {
        D2D1_RENDER_TARGET_PROPERTIES {
            _type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
            pixelFormat: WinStruct::default(),
            dpiX: 0.0,
            dpiY: 0.0,
            usage: D2D1_RENDER_TARGET_USAGE_GDI_COMPATIBLE,
            minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
        }
    }
}

impl WinStruct for D2D1_PIXEL_FORMAT {
    fn default() -> Self {
        D2D1_PIXEL_FORMAT {
            format: DXGI_FORMAT(87),
            alphaMode: D2D1_ALPHA_MODE_IGNORE,
        }
    }
}

impl WinStruct for D2D1_MATRIX_3X2_F {
    fn default() -> Self {
        D2D1_MATRIX_3X2_F {
            matrix: [[1.0, 0.0], [0.0, 1.0], [0.0, 0.0]],
        }
    }
}

impl WinStruct for D2D1_POINT_2F {
    fn default() -> Self {
        D2D1_POINT_2F { x: 0.0, y: 0.0 }
    }
}

pub trait Color {
    fn solid_color(red: f32, green: f32, blue: f32) -> Self;
}

impl Color for D2D1_COLOR_F {
    fn solid_color(red: f32, green: f32, blue: f32) -> Self {
        D2D1_COLOR_F {
            r: red,
            g: green,
            b: blue,
            a: 1.0,
        }
    }
}

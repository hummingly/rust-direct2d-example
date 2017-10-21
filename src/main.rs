extern crate kernel32;
extern crate user32;
extern crate winapi;
mod utils;

use utils::*;

use winapi::*;
use user32::*;
use kernel32::*;

use std::ptr::{null, null_mut};
use std::mem;

//ALTERNATIVE TO CALLING AN EXTERNAL FUNCTION
/*#[link(name = "d2d1")]
extern "system" {
    pub fn D2D1CreateFactory(
        factoryType: D2D1_FACTORY_TYPE,
        riid: REFIID,
        pFactoryOptions: *const D2D1_FACTORY_OPTIONS,
        ppIFactory: *mut *mut c_void,
    ) -> HRESULT;
}

fn create_d2d1_factory(
    factory_type: D2D1_FACTORY_TYPE,
    riid: REFIID,
    p_factory_options: *const D2D1_FACTORY_OPTIONS,
    pp_factory: *mut *mut c_void,
) -> HRESULT {
    unsafe { D2D1CreateFactory(factory_type, riid, p_factory_options, pp_factory) }
}*/

//STRUCTURES
pub struct Resources {
    render_target: *mut ID2D1HwndRenderTarget,
    brush1: *mut ID2D1SolidColorBrush,
    brush2: *mut ID2D1SolidColorBrush,
}

pub struct MyApp {
    resources: Resources,
    factory: *mut ID2D1Factory,
    hwnd: HWND,
}

//D2D1 SETUP
fn set_d2d1_factory(app: &mut MyApp) {
    let mut factory: *mut c_void = null_mut();
    let factory_options = D2D1_FACTORY_OPTIONS {
        debugLevel: D2D1_DEBUG_LEVEL_NONE,
    };

    let d2d1_factory = create_d2d1_factory(
        D2D1_FACTORY_TYPE_MULTI_THREADED,
        &UuidOfID2D1Factory,
        &factory_options as *const D2D1_FACTORY_OPTIONS,
        &mut factory,
    );

    if d2d1_factory != S_OK {
        error_msgbox("Could not create D2D1 factory.");
    } else {
        app.factory = factory as *mut ID2D1Factory;
    }
}

fn set_d2d_resources(app: &mut MyApp) {
    unsafe {
        if !app.resources.render_target.is_null() {
            return;
        } else if app.factory.is_null() {
            error_msgbox("There is no render target!")
        } else {
            let mut rect: RECT = WinStruct::default();

            let mut resources = Resources {
                render_target: null_mut(),
                brush1: null_mut(),
                brush2: null_mut(),
            };

            GetClientRect(app.hwnd, &mut rect as *mut RECT);

            let d2d_rect = D2D1_SIZE_U {
                width: (rect.right - rect.left) as u32,
                height: (rect.bottom - rect.top) as u32,
            };

            let pixel_format = D2D1_PIXEL_FORMAT {
                alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                ..WinStruct::default()
            };

            let render_properties = D2D1_RENDER_TARGET_PROPERTIES {
                pixelFormat: pixel_format,
                ..WinStruct::default()
            };

            let hwnd_render_properties = D2D1_HWND_RENDER_TARGET_PROPERTIES {
                hwnd: app.hwnd,
                pixelSize: d2d_rect,
                presentOptions: D2D1_PRESENT_OPTIONS_NONE,
            };

            let gray = Brush::solid_color(0.345, 0.423, 0.463);
            let red = Brush::solid_color(0.941, 0.353, 0.392);


            let factory: &mut ID2D1Factory = &mut *app.factory;

            if factory.CreateHwndRenderTarget(
                &render_properties,
                &hwnd_render_properties,
                &mut resources.render_target,
            ) != S_OK
            {
                error_msgbox("Could not create render target!");
            }
            let rt: &mut ID2D1HwndRenderTarget = &mut *resources.render_target;

            if rt.CreateSolidColorBrush(&gray, null(), &mut resources.brush1) != S_OK {
                error_msgbox("Could not create brush!");
            }

            if rt.CreateSolidColorBrush(&red, null(), &mut resources.brush2) != S_OK {
                error_msgbox("Could not create brush!");
            }
            app.resources = resources;
        }
    }
}

//RENDER METHOD
fn on_paint(app: &mut MyApp) -> HRESULT {
    unsafe {
        let d2d1_matrix: D2D1_MATRIX_3X2_F = WinStruct::default();

        let white = Brush::solid_color(255.0, 255.0, 255.0);

        let mut render_size = D2D1_SIZE_F {
            width: 0.0,
            height: 0.0,
        };

        let render = &mut *app.resources.render_target;
        render.BeginDraw();
        render.Clear(&white);
        render.SetTransform(&d2d1_matrix);
        render.GetSize(&mut render_size);

        let mut count: f32 = 0.0;
        while count < render_size.width {
            render.DrawLine(
                D2D1_POINT_2F { x: count, y: 0.0 },
                D2D1_POINT_2F {
                    x: count,
                    y: render_size.height,
                },
                &mut **app.resources.brush1 as *mut ID2D1Brush,
                0.5,
                null_mut(),
            );
            count += 10.0;
        }

        count = 0.0;
        while count < render_size.height {
            render.DrawLine(
                D2D_POINT_2F { x: 0.0, y: count },
                D2D_POINT_2F {
                    x: render_size.width,
                    y: count,
                },
                &mut **app.resources.brush1 as *mut ID2D1Brush,
                0.5,
                null_mut(),
            );
            count += 10.0;
        }

        // Draw two rectangles.
        let rx = render_size.width / 2.0;
        let ry = render_size.height / 2.0;

        let rect1 = D2D1_RECT_F {
            left: rx - 50.0,
            right: rx + 50.0,
            top: ry - 50.0,
            bottom: ry + 50.0,
        };
        let rect2 = D2D1_RECT_F {
            left: rx - 100.0,
            right: rx + 100.0,
            top: ry - 100.0,
            bottom: ry + 100.0,
        };

        render.FillRectangle(&rect1, &mut **app.resources.brush1 as *mut ID2D1Brush);
        render.DrawRectangle(
            &rect2,
            &mut **app.resources.brush2 as *mut ID2D1Brush,
            3.0,
            null_mut(),
        );

        render.EndDraw(null_mut(), null_mut())
    }
}

//RELEASE RESOURCES
fn release_resources(app: &mut MyApp) {
    unsafe {
        safe_release(app);

        if !app.factory.is_null() {
            (*app.factory).Release();
            app.factory = null_mut();
        }
    }
}

fn safe_release(app: &mut MyApp) {
    unsafe {
        if !app.resources.render_target.is_null() {
            (*app.resources.brush1).Release();
            (*app.resources.brush2).Release();
            (*app.resources.render_target).Release();

            app.resources.brush1 = null_mut();
            app.resources.brush2 = null_mut();
            app.resources.render_target = null_mut();
        }
    }
}

//MESSAGE PROCESSING
unsafe extern "system" fn wndproc(
    hwnd: HWND,
    message: UINT32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let app_ptr = GetWindowLongPtrW(hwnd, 0);
    let app: &mut MyApp = &mut *(app_ptr as *mut MyApp);
    match message {
        WM_PAINT => {
            set_d2d_resources(app);
            if on_paint(app) == D2DERR_RECREATE_TARGET {
                safe_release(app);
            }
            0
        }
        WM_CREATE => {
            SetWindowLongPtrW(hwnd, 0, 0);
            0
        }
        WM_SIZE => {
            if app_ptr != 0 {
                let width = GET_X_LPARAM(lparam) as u32;
                let height = GET_Y_LPARAM(lparam) as u32;
                let render_size = D2D_SIZE_U {
                    width: width,
                    height: height,
                };

                let render = &mut *app.resources.render_target;
                render.Resize(&render_size);
            }
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

//WINDOW CREATION
pub fn init_class() {
    unsafe {
        let class = "direct2d_example".to_wide();
        let wndcl = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as UINT32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: mem::size_of::<LONG_PTR>() as INT32,
            hInstance: GetModuleHandleW(null_mut()),
            hIcon: 0 as HICON,
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: COLOR_WINDOWFRAME as HBRUSH,
            lpszMenuName: null(),
            lpszClassName: class.as_ptr() as *const u16,
            hIconSm: 0 as HICON,
        };

        if RegisterClassExW(&wndcl) == 0 {
            error_msgbox("Could not register class!");
            PostQuitMessage(0);
        } else {
            RegisterClassExW(&wndcl);
        };
    }
}

fn create_window(app: &mut MyApp, class: &[u16], window: &[u16]) {
    unsafe {
        let hwnd = CreateWindowExW(
            WS_EX_COMPOSITED,
            class.as_ptr(),
            window.as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            600,
            400,
            null_mut(),
            null_mut(),
            GetModuleHandleW(null_mut()),
            null_mut(),
        );

        if hwnd.is_null() {
            error_msgbox("Could not create window!");
            PostQuitMessage(0);
        } else {
            app.hwnd = hwnd;
        }
    }
}

//ASSOCIATE STRUCTURES/DATA
fn set_window(app: &mut MyApp) {
    unsafe {
        SetWindowLongPtrW(app.hwnd, 0, app as *mut MyApp as LONG_PTR);
    }
}

fn main() {
    unsafe {
        let mut app = MyApp {
            factory: null_mut(),
            hwnd: null_mut(),
            resources: Resources {
                render_target: null_mut(),
                brush1: null_mut(),
                brush2: null_mut(),
            },
        };

        let class = "direct2d_example".to_wide();
        let window = "Hello World!".to_wide();

        init_class();
        create_window(&mut app, &class, &window);
        set_window(&mut app);

        set_d2d1_factory(&mut app);
        set_d2d_resources(&mut app);

        let mut msg: MSG = WinStruct::default();

        while GetMessageW(&mut msg as *mut MSG, 0 as HWND, 0, 0) != 0 {
            TranslateMessage(&msg as *const MSG);
            DispatchMessageW(&msg as *const MSG);
        }
        release_resources(&mut app);
    }
}

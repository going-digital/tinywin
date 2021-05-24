#![no_main]
#![no_std]
#![windows_subsystem = "windows"]
#[cfg(windows)]
extern crate winapi;

mod gl;
mod gl_util;

use core::mem::{size_of, MaybeUninit};
use core::panic::PanicInfo;
use gl::CVoid;
use winapi::shared::{minwindef, windef};
use winapi::um::{libloaderapi, winuser, wingdi};

pub unsafe extern "system" fn window_proc(
    hwnd: windef::HWND,
    msg: minwindef::UINT,
    w_param: minwindef::WPARAM,
    l_param: minwindef::LPARAM,
) -> minwindef::LRESULT {
    match msg {
        winapi::um::winuser::WM_DESTROY => {
            winuser::PostQuitMessage(0);
        }
        _ => {
            return winuser::DefWindowProcA(hwnd, msg, w_param, l_param);
        }
    }
    return 0;
}

fn show_error(message: *const i8) {
    unsafe {
        winuser::MessageBoxA(
            0 as windef::HWND,
            message,
            "Window::create\0".as_ptr() as *const i8,
            winuser::MB_ICONERROR,
        );
    }
}

// Create window function
// https://mariuszbartosik.com/opengl-4-x-initialization-in-windows-without-a-framework/
fn create_window() -> (windef::HWND, windef::HDC) {
    unsafe {
        let hinstance = libloaderapi::GetModuleHandleA(0 as *const i8);
        let wnd_class = winuser::WNDCLASSA {
            style: winuser::CS_OWNDC | winuser::CS_HREDRAW | winuser::CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            hInstance: hinstance, // The instance handle for our application which we can retrieve by calling GetModuleHandleW.
            lpszClassName: ".\0".as_ptr() as *const i8,
            cbClsExtra: 0,
            cbWndExtra: 0,
            hIcon: 0 as windef::HICON,
            hCursor: 0 as windef::HICON,
            hbrBackground: 0 as windef::HBRUSH,
            lpszMenuName: 0 as *const i8,
        };
        winuser::RegisterClassA(&wnd_class);

        // More info: https://msdn.microsoft.com/en-us/library/windows/desktop/ms632680(v=vs.85).aspx
        let h_wnd = winuser::CreateWindowExA(
            0,
            //WS_EX_APPWINDOW | WS_EX_WINDOWEDGE,                     // dwExStyle
            ".\0".as_ptr() as *const i8, // class we registered.
            ".\0".as_ptr() as *const i8,   // title
            winuser::WS_OVERLAPPEDWINDOW | winuser::WS_VISIBLE,  // dwStyle
            winuser::CW_USEDEFAULT,
            winuser::CW_USEDEFAULT,
            winuser::CW_USEDEFAULT,
            winuser::CW_USEDEFAULT, // size and position
            0 as windef::HWND,     // hWndParent
            0 as windef::HMENU,    // hMenu
            hinstance,     // hInstance
            0 as minwindef::LPVOID,
        ); // lpParam

        let h_dc: windef::HDC = winuser::GetDC(h_wnd); // Device Context

        let mut pfd: wingdi::PIXELFORMATDESCRIPTOR = core::mem::zeroed();
        pfd.nSize = core::mem::size_of::<wingdi::PIXELFORMATDESCRIPTOR>() as u16;
        pfd.nVersion = 1;
        pfd.dwFlags = wingdi::PFD_DRAW_TO_WINDOW | wingdi::PFD_SUPPORT_OPENGL | wingdi::PFD_DOUBLEBUFFER;
        pfd.iPixelType = wingdi::PFD_TYPE_RGBA;
        pfd.cColorBits = 32;
        pfd.cAlphaBits = 8;
        pfd.cDepthBits = 32;

        let pf_id: i32 = wingdi::ChoosePixelFormat(h_dc, &pfd);
        if pf_id == 0 {
            show_error("ChoosePixelFormat\0".as_ptr() as *const i8);
            return (0 as windef::HWND, h_dc);
        }

        if wingdi::SetPixelFormat(h_dc, pf_id, &pfd) == 0 {
            show_error("SetPixelFormat\0".as_ptr() as *const i8);
            return (0 as windef::HWND, h_dc);
        }

        let gl_context: windef::HGLRC = wingdi::wglCreateContext(h_dc); // Rendering Contex
        if gl_context == 0 as windef::HGLRC {
            show_error("wglCreateContext\0".as_ptr() as *const i8);
            return (0 as windef::HWND, h_dc);
        }

        if wingdi::wglMakeCurrent(h_dc, gl_context) == 0 {
            show_error("wglMakeCurrent\0".as_ptr() as *const i8);
            return (0 as windef::HWND, h_dc);
        }
        gl::init();
        gl::wglSwapIntervalEXT(1);
        (h_wnd, h_dc)
    }
}

// Create message handling function with which to link to hook window to Windows messaging system
// More info: https://msdn.microsoft.com/en-us/library/windows/desktop/ms644927(v=vs.85).aspx
fn handle_message(_window: windef::HWND) -> bool {
    unsafe {
        let mut msg: winuser::MSG = MaybeUninit::uninit().assume_init();
        loop {
            if winuser::PeekMessageA(&mut msg, 0 as windef::HWND, 0, 0, winuser::PM_REMOVE) == 0 {
                return true;
            }
            if msg.message == winapi::um::winuser::WM_QUIT {
                return false;
            }
            winuser::TranslateMessage(&msg);
            winuser::DispatchMessageA(&msg);
        }
    }
}

#[panic_handler]
#[no_mangle]
pub extern "C" fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub unsafe extern "C" fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *((dest as usize + i) as *mut u8) = c as u8;
        i += 1;
    }
    dest
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *((dest as usize + i) as *mut u8) = *((src as usize + i) as *const u8);
        i += 1;
    }
    dest
}

#[no_mangle]
pub extern "system" fn mainCRTStartup() {
    let (window, hdc) = create_window();
    let mut error_message: [i8; 1000] = [0; 1000];
    let vtx_shader_src: &'static str = concat!(include_str!("shader_min.vtx"), "\0");
    let frag_shader_src: &'static str = concat!(include_str!("shader_min.frag"), "\0\0");

    let vtx_coords: [[gl::GLfloat; 3]; 4] = [
        [-1.0, -1.0, 0.0],
        [1.0, -1.0, 0.0],
        [-1.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
    ];

    let vtx_shader =
        match gl_util::shader_from_source(vtx_shader_src, gl::VERTEX_SHADER, &mut error_message) {
            Some(shader) => shader,
            None => {
                show_error(error_message.as_ptr());
                0
            }
        };

    let frag_shader =
        match gl_util::shader_from_source(frag_shader_src, gl::FRAGMENT_SHADER, &mut error_message)
        {
            Some(shader) => shader,
            None => {
                show_error(error_message.as_ptr());
                0
            }
        };

    let shader_prog =
        match gl_util::program_from_shaders(vtx_shader, frag_shader, &mut error_message) {
            Some(prog) => prog,
            None => {
                show_error(error_message.as_ptr());
                0
            }
        };

    let mut vertex_buffer_id: gl::GLuint = 0;
    let mut vertex_array_id: gl::GLuint = 0;
    unsafe {
        // Generate 1 buffer, put the resulting identifier in vertexbuffer
        gl::GenBuffers(1, &mut vertex_buffer_id);
        // one vertex array to hold the vertex and its attributes
        gl::GenVertexArrays(1, &mut vertex_array_id);
        gl::BindVertexArray(vertex_array_id);
        // bind the buffer and load the vertices
        gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer_id);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            size_of::<gl::GLfloat>() as isize * 3 * 4,
            vtx_coords.as_ptr() as *const gl::CVoid,
            gl::STATIC_DRAW,
        );
        // enable and define vertex attributes
        gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * size_of::<gl::GLfloat>() as gl::GLint,
            0 as *const CVoid,
        );
    }

    let mut time: f32 = 0.0;
    loop {
        if !handle_message(window) {
            break;
        }
        unsafe {
            let rgba = &[0.4f32, 1.0, 0.9, 0.0];
            gl::ClearBufferfv(gl::COLOR, 0, rgba as *const _);

            gl::UseProgram(shader_prog);

            let time_loc: i32 = gl::GetUniformLocation(shader_prog, "iTime\0".as_ptr());
            gl::Uniform1f(time_loc, time);

            gl::BindVertexArray(vertex_array_id);
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
            wingdi::SwapBuffers(hdc);
            time += 1.0 / 60.0f32;
        }
    }
    unsafe {
        // Tying to exit normally seems to crash after certain APIs functions have been called. ( Like wingdi::ChoosePixelFormat )
        winapi::um::processthreadsapi::ExitProcess(0);
    }
}

// Compiling with no_std seems to require the following symbol to be set if there is any floating point code anywhere in the code
#[no_mangle]
pub static _fltused: i32 = 1;

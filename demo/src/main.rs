extern crate "kernel32-sys" as kernel32;
extern crate "gdi32-sys" as gdi32;
extern crate "user32-sys" as user32;
extern crate "mantle-sys" as ffi;
extern crate winapi;

use std::mem;
use std::ptr;

fn main() {
    let window = unsafe { create_window() };

    let gpus = unsafe {
        let mut appinfos: ffi::GR_APPLICATION_INFO = mem::zeroed();
        appinfos.apiVersion = ffi::GR_API_VERSION;

        let mut gpus = Vec::with_capacity(ffi::GR_MAX_PHYSICAL_GPUS);
        let mut gpus_count = 2;

        let result = ffi::grInitAndEnumerateGpus(&appinfos, ptr::null(), &mut gpus_count,
                                                 gpus.as_mut_ptr());
        check_result(result).unwrap();

        gpus.set_len(gpus_count as usize);
        gpus
    };

    let device = {
        let queue_info = ffi::GR_DEVICE_QUEUE_CREATE_INFO {
            queueType: ffi::GR_QUEUE_TYPE::GR_QUEUE_UNIVERSAL,
            queueCount: 1,
        };

        let device_info = ffi::GR_DEVICE_CREATE_INFO {
            queueRecordCount: 1,
            pRequestedQueues: &queue_info,
            extensionCount: 1,
            ppEnabledExtensionNames: unsafe { mem::transmute(&b"GR_WSI_WINDOWS\0") },
            maxValidationLevel: ffi::GR_VALIDATION_LEVEL::GR_VALIDATION_LEVEL_0,
            flags: 0,
        };

        let mut device: ffi::GR_DEVICE = 0;
        unsafe {
            check_result(ffi::grCreateDevice(gpus[0], &device_info, &mut device)).unwrap();
        }
        device
    };
}

fn check_result(value: ffi::GR_RESULT) -> Result<(), String> {
    match value {
        ffi::GR_RESULT::GR_SUCCESS => Ok(()),
        c => Err(format!("{:?}", c))
    }
}

unsafe fn create_window() {
    let class_name = register_window_class();

    let title: Vec<u16> = vec![b'M' as u16, b'a' as u16, b'n' as u16, b't' as u16,
                               b'l' as u16, b'e' as u16, 0];

    user32::CreateWindowExW(winapi::WS_EX_APPWINDOW | winapi::WS_EX_WINDOWEDGE, class_name.as_ptr(),
                            title.as_ptr() as winapi::LPCWSTR,
                            winapi::WS_OVERLAPPEDWINDOW | winapi::WS_CLIPSIBLINGS |
                            winapi::WS_VISIBLE,
                            winapi::CW_USEDEFAULT, winapi::CW_USEDEFAULT,
                            winapi::CW_USEDEFAULT, winapi::CW_USEDEFAULT,
                            ptr::null_mut(), ptr::null_mut(),
                            kernel32::GetModuleHandleW(ptr::null()),
                            ptr::null_mut());
}

unsafe fn register_window_class() -> Vec<u16> {
    let class_name: Vec<u16> = "Window Class".utf16_units().chain(Some(0).into_iter())
        .collect::<Vec<u16>>();

    let class = winapi::WNDCLASSEXW {
        cbSize: mem::size_of::<winapi::WNDCLASSEXW>() as winapi::UINT,
        style: winapi::CS_HREDRAW | winapi::CS_VREDRAW | winapi::CS_OWNDC,
        lpfnWndProc: Some(callback),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: kernel32::GetModuleHandleW(ptr::null()),
        hIcon: ptr::null_mut(),
        hCursor: ptr::null_mut(),
        hbrBackground: ptr::null_mut(),
        lpszMenuName: ptr::null(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: ptr::null_mut(),
    };

    user32::RegisterClassExW(&class);
    class_name
}

pub extern "system" fn callback(window: winapi::HWND, msg: winapi::UINT,
                                wparam: winapi::WPARAM, lparam: winapi::LPARAM) -> winapi::LRESULT
{
    unsafe { user32::DefWindowProcW(window, msg, wparam, lparam) }
}

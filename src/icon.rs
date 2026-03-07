pub fn load_tray_icon() -> windows_sys::Win32::UI::WindowsAndMessaging::HICON {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateIconFromResourceEx, IDI_APPLICATION, LoadIconW,
    };

    const ICON_BYTES: &[u8] = include_bytes!("../assets/icon.ico");

    if let Some(icon_image) = first_ico_image(ICON_BYTES) {
        let created: windows_sys::Win32::UI::WindowsAndMessaging::HICON = unsafe {
            CreateIconFromResourceEx(
                icon_image.as_ptr() as *mut u8,
                icon_image.len() as u32,
                1,
                0x0003_0000,
                0,
                0,
                0,
            ) as windows_sys::Win32::UI::WindowsAndMessaging::HICON
        };
        if !created.is_null() {
            return created;
        }
    }

    unsafe { LoadIconW(std::ptr::null_mut(), IDI_APPLICATION) }
}

fn first_ico_image(bytes: &[u8]) -> Option<&[u8]> {
    if bytes.len() < 6 {
        return None;
    }

    let reserved: u16 = read_u16_le(bytes, 0)?;
    let file_type: u16 = read_u16_le(bytes, 2)?;
    let count: u16 = read_u16_le(bytes, 4)?;
    if reserved != 0 || file_type != 1 || count == 0 {
        return None;
    }

    let entry_offset: usize = 6;
    let entry_size: usize = 16;
    if bytes.len() < entry_offset + entry_size {
        return None;
    }

    let image_size: usize = read_u32_le(bytes, entry_offset + 8)? as usize;
    let image_offset: usize = read_u32_le(bytes, entry_offset + 12)? as usize;
    let image_end: usize = image_offset.checked_add(image_size)?;
    if image_size == 0 || image_end > bytes.len() {
        return None;
    }

    Some(&bytes[image_offset..image_end])
}

fn read_u16_le(bytes: &[u8], offset: usize) -> Option<u16> {
    let chunk: &[u8] = bytes.get(offset..offset + 2)?;
    Some(u16::from_le_bytes([chunk[0], chunk[1]]))
}

fn read_u32_le(bytes: &[u8], offset: usize) -> Option<u32> {
    let chunk: &[u8] = bytes.get(offset..offset + 4)?;
    Some(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
}

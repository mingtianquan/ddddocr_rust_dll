use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::result::Result;
use std::vec::Vec;
use bson;
use ddddocr::Ddddocr;
use serde_json;
//初始化ocr识别
#[no_mangle]
pub extern "stdcall" fn initialize_OCR() -> *mut c_void {
    let c = ddddocr::ddddocr_classification().unwrap();
    let ocr : *mut Ddddocr = Box::into_raw(Box::new(c));
    ocr.cast()
}
//初始化目标检测
#[no_mangle]
pub extern "stdcall" fn initialize_detection() -> *mut c_void {
    let c = ddddocr::ddddocr_detection().unwrap();
    let ocr : *mut Ddddocr = Box::into_raw(Box::new(c));
    ocr.cast()
}
//ocr识别
#[no_mangle]
pub extern "stdcall" fn classification_byte_slice(c: *mut c_void,data: *const u8, len: usize) -> *const c_char {
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    let image_bytes = Vec::from(slice);
    let mut ocr: Box<Ddddocr> = unsafe{
        Box::from_raw(c.cast())
    };
    //cstr函数结束生命周期就结束了，指向的指针也就无效了
    let res = ocr.classification(image_bytes, false).unwrap();
    // 保证c不被释放
    Box::into_raw(ocr);
    // res.as_ptr()
    match CString::new(res) {
        Ok(s) => {

            s.into_raw()
        },

        Err(_) => {
            // 处理转换错误，例如返回空字符串或NULL指针
            return std::ptr::null();
        }
    }
}
//目标检测
#[no_mangle]
pub extern "stdcall" fn detection_byte_slice(c: *mut c_void,data: *const u8, len: usize) -> *const c_char {
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    let image_bytes = Vec::from(slice);
    let mut ocr: Box<Ddddocr> = unsafe{
        Box::from_raw(c.cast())
    };
    //cstr函数结束生命周期就结束了，指向的指针也就无效了
    // let res = ocr.classification(image_bytes, false).unwrap();
    let res = ocr.detection(image_bytes).unwrap();
    // 保证c不被释放
    Box::into_raw(ocr);
    // res.as_ptr()
    let json = serde_json::to_string(&res).unwrap();
    match CString::new(json) {
        Ok(s) => {

            s.into_raw()
        },

        Err(_) => {
            // 处理转换错误，例如返回空字符串或NULL指针
            return std::ptr::null();
        }
    }
}
// 滑块算法一
#[no_mangle]
pub extern "stdcall" fn slideral_gorithm_one_slide_match(target: *const u8, len: usize,background: *const u8, len2: usize) -> *const c_char {
    let slicetarget = unsafe { std::slice::from_raw_parts(target, len) };
    let slicebackground = unsafe { std::slice::from_raw_parts(background, len2) };
    let target_bytes = Vec::from(slicetarget);
    let background_bytes = Vec::from(slicebackground);

    let res = ddddocr::slide_match(target_bytes, background_bytes).unwrap();

    // res.as_ptr()
    let json = serde_json::to_string(&res).unwrap();
    match CString::new(json) {
        Ok(s) => {

            s.into_raw()
        },
        Err(_) => {
            // 处理转换错误，例如返回空字符串或NULL指针
            return std::ptr::null();
        }
    }
}
// 滑块算法一
#[no_mangle]
pub extern "stdcall" fn slideral_gorithm_one_simple_slide_match(target: *const u8, len: usize,background: *const u8, len2: usize) -> *const c_char {
    let slicetarget = unsafe { std::slice::from_raw_parts(target, len) };
    let slicebackground = unsafe { std::slice::from_raw_parts(background, len2) };
    let target_bytes = Vec::from(slicetarget);
    let background_bytes = Vec::from(slicebackground);
    let res = ddddocr::simple_slide_match(target_bytes, background_bytes).unwrap();
    let json = serde_json::to_string(&res).unwrap();
    match CString::new(json) {
        Ok(s) => {

            s.into_raw()
        },
        Err(_) => {
            // 处理转换错误，例如返回空字符串或NULL指针
            return std::ptr::null();
        }
    }
}
//滑块算法二
#[no_mangle]
pub extern "stdcall" fn slideral_gorithm_two_slide_comparison(target: *const u8, len: usize,background: *const u8, len2: usize) -> *const c_char {
    let slicetarget = unsafe { std::slice::from_raw_parts(target, len) };
    let slicebackground = unsafe { std::slice::from_raw_parts(background, len2) };
    let target_bytes = Vec::from(slicetarget);
    let background_bytes = Vec::from(slicebackground);
    let res = ddddocr::slide_comparison(target_bytes, background_bytes).unwrap();
    let json = serde_json::to_string(&res).unwrap();
    match CString::new(json) {
        Ok(s) => {

            s.into_raw()
        },

        Err(_) => {
            // 处理转换错误，例如返回空字符串或NULL指针
            return std::ptr::null();
        }
    }
}
#[no_mangle]
pub extern "stdcall" fn freee(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(ptr as *mut Ddddocr);
    }
}
#[no_mangle]
pub extern "stdcall" fn bson_to_json(data: *const u8, len: usize) -> *const c_char {
    let slice = unsafe { std::slice::from_raw_parts(data, len) };

    // 解析BSON并转换为JSON字符串
    let json_string = match bson_to_json_string(slice) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Error in my_function3: {}", e);
            // 在错误情况下，可以返回一个表示错误的指针
            return std::ptr::null();
        }
    };

    // 将JSON字符串转换为CString
    match CString::new(json_string) {
        Ok(cstring) => cstring.into_raw(),
        Err(_) => {
            eprintln!("Error: Unable to create CString.");
            std::ptr::null()
        }
    }
}
fn bson_to_json_string(data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    // 解析BSON并转换为JSON字符串
    let bson_value: bson::Bson = bson::from_slice(data)?;
    let document = match bson_value {
        bson::Bson::Document(doc) => doc,
        _ => return Err("BSON value is not a document".into()),
    };
    let json_string = serde_json::to_string_pretty(&document)?;

    Ok(json_string)
}



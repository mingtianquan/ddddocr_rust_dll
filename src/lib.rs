use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::Read;
use std::os::raw::{c_char, c_int, c_uchar, c_void};
use std::result::Result;
use std::vec::Vec;
use bson;
use ddddocr::*;
use serde_json;
use std::sync::{Arc, Mutex};
use encoding::{all::GBK, Encoding, EncoderTrap, DecoderTrap};
//初始化ocr识别
#[no_mangle]
pub extern "stdcall" fn initialize_OCR() -> *mut c_void {
    let c = ddddocr_classification().unwrap();
    let ocr : *mut Ddddocr = Box::into_raw(Box::new(c));
    ocr.cast()
}
//初始化目标检测
#[no_mangle]
pub extern "stdcall" fn initialize_detection() -> *mut c_void {
    let c = ddddocr_detection().unwrap();
    let ocr : *mut Ddddocr = Box::into_raw(Box::new(c));
    ocr.cast()
}
//载入自定义模型，初始化，识别请用classification_byte_slice
#[no_mangle]
pub extern "stdcall" fn load_model_charset(data: *const u8, len: usize, charsetdata: *const u8, charsetlen: usize) -> *mut c_void {
    // Convert raw pointers to slices
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    let model_bytes = Vec::from(slice);
    let charset = unsafe { std::slice::from_raw_parts(charsetdata, charsetlen) };
    // Convert charset bytes to UTF-8 string
    let charset_str_result = std::str::from_utf8(charset);
    let charset_str = match charset_str_result {
        Ok(s) => s,
        Err(_) => {
            // Handle invalid UTF-8 charset data
            return std::ptr::null_mut();
        }
    };
    // Deserialize JSON string to your charset type using serde_json
    let charset_result = serde_json::from_str(charset_str);
    let charset = match charset_result {
        Ok(c) => c,
        Err(_) => {
            // Handle JSON deserialization error
            return std::ptr::null_mut();
        }
    };
    // Create Ddddocr instance
    let ocr_result = Ddddocr::new(model_bytes, charset);
    let c = match ocr_result {
        Ok(c) => c,
        Err(_) => {
            // Handle Ddddocr creation error
            return std::ptr::null_mut();
        }
    };
    // Box the instance to manage memory properly
    let ocr_box = Box::new(c);
    // Return a raw pointer to the boxed instance
    Box::into_raw(ocr_box) as *mut c_void
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
    match GBK.encode(&res, EncoderTrap::Replace) {
        Ok(encoded) => {
            match CString::new(encoded) {
                Ok(s) => s.into_raw(),
                Err(_) => std::ptr::null(),
            }
        },
        Err(_) => std::ptr::null(),
    }
}

//ocr识别概率识别
#[no_mangle]
pub extern "stdcall" fn classification_probability_byte_slice(c: *mut c_void,data: *const u8, len: usize,set_rangesi_32: i32,set_ranges: *const c_char) -> *const c_char {
    // Convert raw pointers to slices
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    let image_bytes = Vec::from(slice);

    // Handle OCR instance
    let mut ocr: Box<Ddddocr> = unsafe { Box::from_raw(c.cast()) };

    // Convert set_ranges from C string to Rust string, considering it may be ANSI
    let set_ranges = if !set_ranges.is_null() {
        let c_str = unsafe { CStr::from_ptr(set_ranges) };
        match c_str.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                // Handle invalid UTF-8 string
                return std::ptr::null();
            }
        }
    } else {
        String::new()
    };

    // If set_ranges is not empty, set it; otherwise, use set_rangesi32
    if !set_ranges.is_empty() {
        ocr.set_ranges(set_ranges);
    } else {
        ocr.set_ranges(set_rangesi_32);
    }

    // Perform OCR classification
    let mut result = match ocr.classification_probability(image_bytes, false) {
        Ok(r) => r,
        Err(_) => {
            // Handle OCR classification error
            return std::ptr::null();
        }
    };

    println!("概率: {}", result.json());
    println!("识别结果: {}", result.get_text().to_string());

    // Convert result to JSON string
    let res = result.json().to_string();

    // Ensure OCR instance is not freed
    Box::into_raw(ocr);

    // Encode the result as GBK and return it as a C string
    match GBK.encode(&res, EncoderTrap::Replace) {
        Ok(encoded) => match CString::new(encoded) {
            Ok(s) => s.into_raw(),
            Err(_) => std::ptr::null(),
        },
        Err(_) => std::ptr::null(),
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

//目标检测二
#[no_mangle]
pub extern "stdcall" fn detection_byte_slice_er(c: *mut c_void, o: *mut c_void, data: *const u8, len: usize) -> *const c_char {
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    let image_bytes = Vec::from(slice);
    let mut detection_ocr: Box<Ddddocr> = unsafe {
        Box::from_raw(c.cast())
    };

    let mut ocr: Box<Ddddocr> = unsafe {
        Box::from_raw(o.cast())
    };

    let image_slice: &[u8] = &image_bytes;
    let result1 = detection_ocr.detection(image_slice).unwrap();
    let result = ocr.classification_bbox(image_slice, &result1).unwrap();

    // Prevent c from being released
    Box::into_raw(ocr);
    Box::into_raw(detection_ocr);

    // Convert result to JSON string
    let json = serde_json::to_string(&result).unwrap();
    // let json =  ddddocr::MapJson::json(&result);
    // Convert JSON string from UTF-8 to GBK
    match GBK.encode(&json, EncoderTrap::Replace) {
        Ok(encoded) => {
            match CString::new(encoded) {
                Ok(s) => s.into_raw(),
                Err(_) => std::ptr::null(),
            }
        },
        Err(_) => std::ptr::null(),
    }
}
// 滑块算法一
#[no_mangle]
pub extern "stdcall" fn slideral_gorithm_one_slide_match(target: *const u8, len: usize,background: *const u8, len2: usize) -> *const c_char {
    let slicetarget = unsafe { std::slice::from_raw_parts(target, len) };
    let slicebackground = unsafe { std::slice::from_raw_parts(background, len2) };
    let target_bytes = Vec::from(slicetarget);
    let background_bytes = Vec::from(slicebackground);

    let res = slide_match(target_bytes, background_bytes).unwrap();

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
    let res = simple_slide_match(target_bytes, background_bytes).unwrap();
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
    let res = slide_comparison(target_bytes, background_bytes).unwrap();
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
pub extern "stdcall" fn rust_free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(ptr as *mut Ddddocr);
    }
}

#[no_mangle]
pub extern "stdcall" fn free_string(ptr: *const c_char) {
    if ptr.is_null() {
        return;
    }
    // Convert the raw pointer back to a CString to deallocate the memory
    unsafe {
        let _ =  CString::from_raw(ptr as *mut c_char);
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

//回调函数示例！
type CallbackFn = extern "stdcall" fn(int_param: c_int, str_param: *const c_char);
//callback传递易语言函数地址指针 到整数 (&回调) 回调为子程序  这个子程序2个参数一个是整数型一个是文本型
#[no_mangle]
pub extern "stdcall" fn set_callback_and_call(callback: CallbackFn) {
    let int_param: c_int = 42; // 示例整数参数
    let str_param = CString::new("Hello from Rust").expect("CString::new failed");
    // 调用传递过来的回调函数
    callback(int_param, str_param.as_ptr());
}

//callback2传递易语言函数地址指针 到整数 (&回调) 回调为子程序  这个子程序2个参数都是整数型  指针到字节集 (bin, len)取回字节集
//这里是循环回调数据用loop循环往易语言传回字节集和字节集长度
type CallbackFn2 = extern "stdcall" fn(byte_ptr: *const std::ffi::c_uchar, byte_len: std::ffi::c_int);
#[no_mangle]
pub extern "stdcall" fn set_callback_and_call2(callback: CallbackFn2) {
    loop{
        // 示例文本字符串
        let text = "Hello from Rust";
        // 将字符串转换为字节数组
        let byte_array = text.as_bytes();
        let byte_len: std::ffi::c_int = byte_array.len() as std::ffi::c_int;
        // 调用传递过来的回调函数
        callback(byte_array.as_ptr(), byte_len);

    }

}
//callback3传递易语言函数地址指针 到整数 (&回调) 回调为子程序  这个子程序2个参数都是整数型  指针到字节集 (bin, len)取回字节集
type CallbackFn3 = extern "stdcall" fn(byte_ptr: *const c_uchar, byte_len: c_int);

lazy_static::lazy_static! {
    static ref BUFFER: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
}
//
#[no_mangle]
pub extern "stdcall" fn set_callback_and_call3(callback: CallbackFn3, file_path: *const c_char) {
    loop{

        // 将传入的 C 字符串转换为 Rust 字符串
        let c_str = unsafe { CStr::from_ptr(file_path) };
        let file_path = c_str.to_str().expect("Invalid UTF-8 string");
        // 打开并读取图片文件
        let mut file = File::open(file_path).expect("Failed to open file");
        let buffer = BUFFER.clone();
        let mut buffer_guard = buffer.lock().unwrap();
        buffer_guard.clear();
        file.read_to_end(&mut buffer_guard).expect("Failed to read file");
        let byte_len: c_int = buffer_guard.len() as c_int;
        // 使用 Box 包装回调函数，确保其生命周期长于调用
        let boxed_callback = Box::new(callback);
        // 调用传递过来的回调函数
        boxed_callback(buffer_guard.as_ptr(), byte_len);
        // 回调函数退出作用域后会自动释放内存

    }

}
//这个示例是易语言传递一个空白文本例如：str ＝ 取空白文本 (300) 调用e_redirect (str)  调试输出 (str) 可以取回结果11111
#[no_mangle]
pub extern "stdcall" fn e_redirect(set_ranges: *const c_char, buffer: *mut c_char) {
    // 确保从 C 传递的字符串是 ANSI 编码的，并将其转换为 Rust 字符串
    let c_str = unsafe { CStr::from_ptr(set_ranges) };
    let ansi_bytes = c_str.to_bytes();

    // 将 ANSI 编码的字节转换为 UTF-8 编码的字符串
    let utf8_string = match GBK.decode(ansi_bytes, DecoderTrap::Strict) {
        Ok(s) => s,
        Err(_) => {
            // 处理解码错误
            return;
        }
    };

    // 创建一个 Rust 字符串
    let rust_string = "11111";
    let result = format!("{}{}", utf8_string, rust_string);

    // 将结果字符串再编码为 GBK
    let gbk_result = match GBK.encode(&result, EncoderTrap::Replace) {
        Ok(encoded) => encoded,
        Err(_) => {
            // 处理编码错误
            return;
        }
    };

    // 将 GBK 编码的字节转换为 C 风格的字符串（包括空终止符）
    let c_string = match CString::new(gbk_result) {
        Ok(s) => s,
        Err(_) => {
            // 处理无效的 C 字符串（包含内嵌空字符）
            return;
        }
    };

    // 获取 C 风格字符串的字节切片（包括空终止符）
    let bytes = c_string.as_bytes_with_nul();

    // 安全地将字节复制到易语言提供的缓冲区中
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr() as *const _, buffer as *mut _, bytes.len());
    }
}

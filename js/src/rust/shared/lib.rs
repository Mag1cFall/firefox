/* Copyright 2018 Mozilla Foundation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
extern crate encoding_c;
extern crate encoding_c_mem;
extern crate mozglue_static;
extern crate normalizer_glue;
extern crate zlib_rs;

#[cfg(feature = "gluesmith")]
extern crate gluesmith;

#[cfg(feature = "icu4x")]
extern crate icu_capi;

#[cfg(feature = "icu4x")]
extern crate properties_glue;

#[cfg(feature = "icu4x")]
extern crate unicode_bidi_ffi;

#[cfg(feature = "icu4x")]
extern crate collator_glue;

#[cfg(feature = "icu4x")]
extern crate locale_glue;

/// mz_gzip_bound 返回 mz_gzip 压缩 len 字节时输出缓冲区所需的字节数上限
#[no_mangle]
pub extern "C" fn mz_gzip_bound(len: usize) -> usize {
    zlib_rs::compress_bound(len) + 32
}

/// mz_gzip 按 Firefox CompressionStream("gzip") 的参数与调用顺序以 zlib-rs 压缩 input，返回写入 out 的字节数，失败时为 0
#[no_mangle]
pub unsafe extern "C" fn mz_gzip(input: *const u8, len: usize, out: *mut u8, cap: usize) -> usize {
    use zlib_rs::c_api::z_stream;
    use zlib_rs::deflate::{self, DeflateConfig, DeflateStream, Method, Strategy};
    use zlib_rs::{DeflateFlush, ReturnCode};
    let mut stream: z_stream = std::mem::zeroed();
    let config = DeflateConfig { level: -1, method: Method::Deflated, window_bits: 31, mem_level: 8, strategy: Strategy::Default };
    if deflate::init(&mut stream, config) != ReturnCode::Ok {
        return 0;
    }
    stream.next_in = input as *mut u8;
    stream.avail_in = len as _;
    stream.next_out = out;
    stream.avail_out = cap as _;
    let code = match DeflateStream::from_stream_mut(&mut stream) {
        Some(state) => {
            deflate::deflate(state, DeflateFlush::NoFlush);
            deflate::deflate(state, DeflateFlush::Finish)
        }
        None => ReturnCode::StreamError,
    };
    let written = stream.total_out as usize;
    if let Some(state) = DeflateStream::from_stream_mut(&mut stream) {
        let _ = deflate::end(state);
    }
    if code == ReturnCode::StreamEnd { written } else { 0 }
}

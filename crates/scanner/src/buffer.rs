use std::mem::forget;

use serde::{Deserialize, Serialize};

//  Box<dyn DoubleEndedIterator<Item = &Support> + '_>

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Buffer {
    #[serde(default)]
    pub pointer: Option<usize>,
    pub width: usize,
    pub height: usize,
    pub size: usize,
}
impl Buffer {
    pub fn reserve(&mut self) {
        assert!(self.pointer.is_none());
        let mut buf: Vec<u8> = Vec::with_capacity(self.size);
        let ptr: *mut u8 = buf.as_mut_ptr();
        forget(buf);
        self.pointer = Some(ptr as usize);
    }

    pub fn dealloc(&mut self) {
        assert!(self.pointer.is_some());
        unsafe {
            let _ = Vec::from_raw_parts(self.pointer.unwrap() as *mut u8, self.size, self.size);
            // dropped here i think
            self.pointer = None;
            self.size = 0;
        }
    }

    #[inline(always)]
    pub fn pixel(&self, x: usize, y: usize) -> [u8; 4] {
        let ptr: *const u8 = self.pointer.expect("buffer not allocated") as *const u8;
        let idx: usize = y * self.width + x * 4;
        assert!(idx + 4 <= self.size, "pixel index out of buffer range");
        unsafe {
            let slice: &[u8] = std::slice::from_raw_parts(ptr, self.size);
            [slice[idx], slice[idx + 1], slice[idx + 2], slice[idx + 3]]
        }
    }
}

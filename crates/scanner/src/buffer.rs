use std::mem::forget;

use serde::{Deserialize, Serialize};

//  Box<dyn DoubleEndedIterator<Item = &Support> + '_>

#[derive(Debug, Serialize, Deserialize)]
pub struct Buffer {
    #[serde(default)]
    pub pointer: Option<usize>,
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
}

#![no_std]

use core::marker::PhantomData;
use core::ptr::NonNull;

// Trait representing a DMA-accessible memory region
pub trait DmaAccessible {
    const START_ADDR: usize;
    const END_ADDR: usize;
}

// Concrete memory region types
pub struct Sram1;
pub struct Dtcm;
pub struct Itcm;

impl DmaAccessible for Sram1 {
    const START_ADDR: usize = 0x3000_0000; // SRAM1 start address (RM0433, p. 131)
    const END_ADDR: usize = 0x3002_0000; // SRAM1 end address
}

impl DmaAccessible for Dtcm {
    const START_ADDR: usize = 0x2000_0000; // DTCM start address (RM0433, p. 131)
    const END_ADDR: usize = 0x2001_0000; // DTCM end address
}

impl DmaAccessible for Itcm {
    const START_ADDR: usize = 0x0000_0000; // ITCM start address (RM0433, p. 131)
    const END_ADDR: usize = 0x0001_0000; // ITCM end address
}

// DMA buffer wrapper type
pub struct DmaBuffer<T, Region: DmaAccessible> {
    ptr: NonNull<T>,
    len: usize,
    _region: PhantomData<Region>,
}

impl<T, Region: DmaAccessible> DmaBuffer<T, Region> {
    // Safe constructor: only accepts buffers placed in specific regions
    pub fn new(buffer: &'static mut [T]) -> Self {
        let addr = buffer.as_ptr() as usize;
        // Address range check at compile-time/runtime
        assert!(
            addr >= Region::START_ADDR
                && (addr + core::mem::size_of_val(buffer)) <= Region::END_ADDR,
            "Buffer not in DMA-accessible region"
        );

        let len = buffer.len();
        Self {
            ptr: NonNull::from(buffer).cast(),
            len,
            _region: PhantomData,
        }
    }

    // Provide borrowing for embassy DMA transfer (buffer cannot be modified during transfer)
    pub fn as_slice(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    // Get internal pointer (for passing to embassy DMA functions; unsafe but guaranteed by type)
    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[cfg(test)]
mod tests {
    use grounded::uninit::GroundedArrayCell;

    use crate::{DmaBuffer, Sram1};

    // Since there's no way to link to specific memory regions in a std environment,
    // the test is expected to panic, and I wanted to check if it builds and if it
    // can be combined with GroundedArrayCell.
    #[should_panic(expected = "Buffer not in DMA-accessible region")]
    #[test]
    fn test() {
        static BUFFER: GroundedArrayCell<u8, 128> = GroundedArrayCell::uninit();
        let tx_buffer: &mut [u8] = unsafe {
            BUFFER.initialize_all_copied(0);
            let (ptr, len) = BUFFER.get_ptr_len();
            core::slice::from_raw_parts_mut(ptr, len)
        };
        let _da = DmaBuffer::<u8, Sram1>::new(tx_buffer);
    }
}

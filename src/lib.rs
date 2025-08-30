//! # DMA Accessible Memory Regions for STM32H750
//!
//! This crate provides type-safe abstractions for DMA-accessible memory regions on STM32H750 microcontrollers.
//!
//! ## Important Safety Notice
//!
//! **STM32H750 DMA Limitations**: The DMA controller can only access memory regions that are accessible via the AXI bus.
//! This includes:
//! - SRAM1/2/3 (0x3000_0000 - 0x3004_0000)
//! - DTCM-RAM (0x2000_0000 - 0x2002_0000)
//! - ITCM-RAM (0x0000_0000 - 0x0002_0000)
//!
//! (see RM0433(datasheet), p. 130, p. 131)
//!
//! Attempting to access other memory regions will result in a bus error
//! and cause the microcontroller to enter a Halt state. This crate ensures safety by restricting DMA buffers
//! to these approved regions only.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use dma_accessible::{DmaBuffer, Sram1};
//! use grounded::uninit::GroundedArrayCell;
//!
//! // Buffer must be placed in a DMA-accessible region (e.g., SRAM1)
//! #[unsafe(link_section = ".sram1_bss")]
//! static BUFFER: GroundedArrayCell<u8, 1024> = GroundedArrayCell::uninit();
//!
//! let dma_buffer = DmaBuffer::<_, _, Sram1>::new(&BUFFER, 0);
//! ```
//!
//! ## Regions
//! see RM0433(datasheet), p. 130, p. 131
//!
//! - `Sram1`: SRAM1 region (0x3000_0000 - 0x3002_0000)
//! - `Dtcm`: DTCM-RAM region (0x2000_0000 - 0x2001_0000)
//! - `Itcm`: ITCM-RAM region (0x0000_0000 - 0x0001_0000)

#![no_std]

use core::marker::PhantomData;
use core::ptr::NonNull;

use grounded::uninit::GroundedArrayCell;

// Trait representing a DMA-accessible memory region
pub trait DmaAccessible {
    const START_ADDR: usize;
    const END_ADDR: usize;
}

/// SRAM1 memory region (0x3000_0000 - 0x3002_0000)
/// This region is accessible via AXI bus and safe for DMA operations.
pub struct Sram1;

/// DTCM-RAM memory region (0x2000_0000 - 0x2001_0000)
/// This region is accessible via AXI bus and safe for DMA operations.
pub struct Dtcm;

/// ITCM-RAM memory region (0x0000_0000 - 0x0001_0000)
/// This region is accessible via AXI bus and safe for DMA operations.
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

/// A type-safe wrapper for DMA buffers that ensures the buffer is located in a DMA-accessible memory region.
///
/// This struct is just a wrapper for a slice of a type, but guarantees that DMA operations will only target memory regions
/// that are accessible via the AXI bus on STM32H750, preventing bus errors.
///
/// # Type Parameters
/// - `T`: The type of elements in the buffer
/// - `Region`: A type implementing `DmaAccessible` that specifies the memory region
///
/// # Safety
/// The buffer address is validated at construction time to ensure it falls within the specified region.
pub struct DmaBuffer<T, const LEN: usize, Region> {
    ptr: NonNull<T>,
    _region: PhantomData<Region>,
}

impl<T: Copy, const LEN: usize, Region: DmaAccessible> DmaBuffer<T, LEN, Region> {
    pub const LENGTH: usize = LEN;
    /// Safe constructor: only accepts buffers placed in specific regions
    ///
    /// # Panics
    /// Panics if the buffer is not located within the specified DMA-accessible region.
    ///
    /// # Safety
    /// The caller must ensure that the buffer remains valid for the lifetime of this struct
    /// and that no other references to the buffer exist while DMA operations are in progress.
    /// Note: The `'static` lifetime enforces the reference's validity but does not guarantee the buffer is a `static` variable
    /// (e.g., it could be a leaked heap allocation). It is just for rejecting local variable’s simple reference.
    /// For DMA safety, ensure the buffer is placed in a `static` variable.
    /// like:
    /// ```rust,no-run
    /// use dma_accessible::{DmaBuffer, Sram1};
    /// use grounded::uninit::GroundedArrayCell;
    ///
    /// // Buffer must be placed in a DMA-accessible region (e.g., SRAM1)
    /// #[unsafe(link_section = ".sram1_bss")]
    /// static BUFFER: GroundedArrayCell<u8, 1024> = GroundedArrayCell::uninit();
    ///
    /// let dma_buffer = DmaBuffer::<_, _, Sram1>::new(&BUFFER, 0);
    /// ```
    pub fn new(buffer: &'static GroundedArrayCell<T, LEN>, initialize_value: T) -> Self {
        let buffer: &mut [T] = unsafe {
            buffer.initialize_all_copied(initialize_value);
            let (ptr, len) = buffer.get_ptr_len();
            core::slice::from_raw_parts_mut(ptr, len)
        };

        let addr = buffer.as_ptr() as usize;
        // Address range check at compile-time/runtime
        assert!(
            addr >= Region::START_ADDR && (addr + LEN) <= Region::END_ADDR,
            "Buffer not in DMA-accessible region"
        );
        assert_eq!(buffer.len(), LEN);
        Self {
            ptr: NonNull::from(buffer).cast(),
            _region: PhantomData,
        }
    }

    /// Provide borrowing for embassy DMA transfer (buffer cannot be modified during transfer)
    pub fn as_slice(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), LEN) }
    }

    /// Returns a mutable slice to the buffer.
    ///
    /// # Safety
    /// The caller must ensure that no DMA operations are currently using this buffer.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_ptr(), LEN) }
    }

    /// Get internal pointer (for passing to embassy DMA functions; unsafe but guaranteed by type)
    ///
    /// # Safety
    /// This pointer is guaranteed to point to a valid DMA-accessible memory region,
    /// but the caller must ensure proper synchronization with DMA operations.
    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    /// Returns a mutable pointer to the buffer.
    ///
    /// # Safety
    /// This pointer is guaranteed to point to a valid DMA-accessible memory region,
    /// but the caller must ensure proper synchronization with DMA operations.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
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
        let _da = DmaBuffer::<u8, 128, Sram1>::new(&BUFFER, 0);
    }
}

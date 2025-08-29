# DMA Accessible Memory Regions for STM32H750

[![Crates.io](https://img.shields.io/crates/v/dma_accessible.svg)](https://crates.io/crates/dma_accessible)
[![Documentation](https://docs.rs/dma_accessible/badge.svg)](https://docs.rs/dma_accessible)

A Rust crate providing type-safe abstractions for DMA-accessible memory regions on STM32H750 microcontrollers.

## ⚠️ Critical Safety Warning

**STM32H750 DMA Limitations**: The DMA controller can only access memory regions that are accessible via the AXI bus. This includes:

- **SRAM1/2/3** (0x3000_0000 - 0x3004_0000)
- **DTCM-RAM** (0x2000_0000 - 0x2002_0000)  
- **ITCM-RAM** (0x0000_0000 - 0x0002_0000)

**Attempting to access other memory regions (such as Flash memory or certain peripheral regions) will result in a bus error and cause the microcontroller to enter a Halt state.** This crate ensures compile-time safety by restricting DMA buffers to these approved regions only.

## Features

- **Type Safety**: Compile-time guarantees that DMA buffers are in valid regions
- **Zero Cost**: No runtime overhead for memory region validation
- **Embassy Compatible**: Designed to work seamlessly with Embassy's DMA abstractions
- **Memory Region Types**: Pre-defined types for common STM32H750 memory regions

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
dma_accessible = "0.1.0"
```

## Usage

### Basic Example

```rust
use dma_accessible::{DmaBuffer, Sram1};

// Buffer must be placed in a DMA-accessible region (e.g., SRAM1)
#[link_section = ".sram1"]
static mut BUFFER: [u8; 1024] = [0; 1024];

fn main() {
    let dma_buffer = DmaBuffer::<u8, Sram1>::new(unsafe { &mut BUFFER });
    
    // Use with Embassy DMA
    // dma_buffer.as_slice() or dma_buffer.as_ptr()
}
```

### With Embassy

```rust
use dma_accessible::{DmaBuffer, Dtcm};
use embassy_stm32::dma::Transfer;

// DTCM-RAM buffer for fast access
#[link_section = ".dtcm"]
static mut TX_BUFFER: [u8; 256] = [0; 256];

async fn dma_transfer() {
    let dma_buf = DmaBuffer::<u8, Dtcm>::new(unsafe { &mut TX_BUFFER });
    
    // Safe to use with Embassy DMA transfers
    let transfer = Transfer::new(/* ... */, dma_buf.as_slice());
    transfer.await;
}
```

## Memory Regions

| Region | Address Range | Description |
|--------|---------------|-------------|
| `Sram1` | 0x3000_0000 - 0x3002_0000 | SRAM1 region |
| `Dtcm` | 0x2000_0000 - 0x2001_0000 | Data Tightly Coupled Memory |
| `Itcm` | 0x0000_0000 - 0x0001_0000 | Instruction Tightly Coupled Memory |

## API Reference

### `DmaBuffer<T, Region>`

A type-safe wrapper for DMA buffers.

#### Methods

- `new(buffer: &'static mut [T]) -> Self`: Creates a new DMA buffer (panics if buffer is not in valid region)
- `as_slice(&self) -> &[T]`: Returns an immutable slice
- `as_mut_slice(&mut self) -> &mut [T]`: Returns a mutable slice  
- `as_ptr(&self) -> *const T`: Returns a raw pointer
- `as_mut_ptr(&mut self) -> *mut T`: Returns a mutable raw pointer
- `len(&self) -> usize`: Returns buffer length
- `is_empty(&self) -> bool`: Returns true if buffer is empty

## Safety Considerations

- Buffers must be placed in DMA-accessible memory regions using linker sections
- Ensure no other references exist to the buffer during DMA operations
- The crate performs runtime checks in debug builds to validate memory regions


## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## References

- [STM32H750 Reference Manual (RM0433)](https://www.st.com/resource/en/reference_manual/rm0433-stm32h742-stm32h743-753-and-stm32h750-value-line-advanced-armbased-32bit-mcus-stmicroelectronics.pdf)
- [Embassy STM32](https://github.com/embassy-rs/embassy)

# DMA Accessible Memory Regions for STM32H750

[![Crates.io](https://img.shields.io/crates/v/dma_accessible.svg)](https://crates.io/crates/dma_accessible)
[![Documentation](https://docs.rs/dma_accessible/badge.svg)](https://docs.rs/dma_accessible)

A Rust crate providing type-safe abstractions for DMA-accessible memory regions on STM32H750 microcontrollers.

## ⚠️ Critical Safety Warning

**STM32H750 DMA Limitations**: The DMA controller can only access memory regions that are accessible via the AXI bus. This includes:

- **SRAM1/2/3** (0x3000_0000 - 0x3004_0000)
- **DTCM-RAM** (0x2000_0000 - 0x2002_0000)  
- **ITCM-RAM** (0x0000_0000 - 0x0002_0000)

**Attempting to access other memory regions will result in a bus error and cause the microcontroller to enter a Halt state.** Attempting to access other memory regions will result in a bus error
and cause the microcontroller to enter a Halt state. This crate ensures safety by restricting DMA buffers to these approved regions only.

## Usage

### Basic Example

```rust
use dma_accessible::{DmaBuffer, Sram1};
use grounded::uninit::GroundedArrayCell;
// Buffer must be placed in a DMA-accessible region (e.g., SRAM1)
#[unsafe(link_section = ".sram1_bss")]
static BUFFER: GroundedArrayCell<u8, 1024> = GroundedArrayCell::uninit();
let raw_buffer = unsafe {
    BUFFER.initialize_all_copied(0);
    let (ptr, len) = BUFFER.get_ptr_len();
    core::slice::from_raw_parts_mut(ptr, len)
};
let dma_buffer = DmaBuffer::<u8, Sram1>::new(raw_buffer);
```

## Memory Regions

| Region | Address Range | Description |
|--------|---------------|-------------|
| `Sram1` | 0x3000_0000 - 0x3002_0000 | SRAM1 region |
| `Dtcm` | 0x2000_0000 - 0x2001_0000 | Data Tightly Coupled Memory |
| `Itcm` | 0x0000_0000 - 0x0001_0000 | Instruction Tightly Coupled Memory |

## Safety Considerations

- Buffers must be placed in DMA-accessible memory regions using linker sections
- Ensure no other references exist to the buffer during DMA operations
- The crate performs runtime checks only(not compile-time)


## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## References

- [STM32H750 Reference Manual (RM0433)](https://www.st.com/resource/en/reference_manual/rm0433-stm32h742-stm32h743-753-and-stm32h750-value-line-advanced-armbased-32bit-mcus-stmicroelectronics.pdf)
- [Embassy STM32](https://github.com/embassy-rs/embassy)

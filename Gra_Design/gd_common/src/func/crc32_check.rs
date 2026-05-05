
use crc::{Crc,CRC_32_ISO_HDLC};
use crate::{FrameData,FrameConnection,BMI270Samples};
use anyhow::{anyhow, Result};



pub fn verify_samples_crc(samples: &[BMI270Samples;10], expected_crc: u32) ->Result<()> {
    let bytes = bytemuck::cast_slice(samples);
    let result_crc = Crc::<u32>::new(&CRC_32_ISO_HDLC).checksum(bytes);
    if result_crc != expected_crc {
        Err(anyhow!("CRC did not verify"))
    } else {
        Ok(())
    }
}








// FITs file has
// - Primary header and data unit (HDU)
// - Conforming extentions (optional)
// - Other special records (optional, restricted)

// FITS blocks are 2880 bytes, integral number of blocks in a FITS file
// ASCII header, 80 byte keyword value pairs, bytes 0-7 are keyword, then "= "
// and then the value is bytes 10-79.
//
// END keyword indicates the end of the ASCII header, rest of the block is
// space filled (0x20).
//
// Mandatory keywords:
// SIMPLE = T (file conforms to FITS standard)
// BITPIX (bits / data pixel)
// NAXIS (number of data axes/dimensions in data)
// NAXISn, n=1, ..., NAXIS (length of data axes)
// ...
// (other keywords)
// ...
// END

pub mod texture_display;
use std::collections::HashMap;

fn parse_hdu_block(data: &[u8]) -> HashMap<String, String> {
    let kvpairs = 2880 / 80;
    let mut output = HashMap::new();
    for i in 0..kvpairs {
        let key = std::str::from_utf8(&data[i * 80..i * 80 + 8])
            .expect("failed to parse keyword as UTF8");
        let value = std::str::from_utf8(&data[i * 80 + 10..(i + 1) * 80])
            .expect("failed to parse value as UTF8");
        output.insert(key.trim().to_string(), value.trim().to_string());
    }
    output
}

pub fn get_image_dims(kv_pairs: &HashMap<String, String>) -> (u32, u32, u32) {
    let mut bytes_per_element = 0;
    let mut dx = 0;
    let mut dy = 0;
    let mut _naxis = 0;
    for (k, v) in kv_pairs {
        if k == "BITPIX" {
            bytes_per_element = v.split('/').collect::<Vec<_>>()[0]
                .trim()
                .parse::<u32>()
                .expect("failed to parse BITPIX");
            bytes_per_element /= 8;
        } else if k == "NAXIS" {
            _naxis = v.split('/').collect::<Vec<_>>()[0]
                .trim()
                .parse::<u32>()
                .expect("failed to parse NAXIS");
        } else if k == "NAXIS1" {
            dx = v.split('/').collect::<Vec<_>>()[0]
                .trim()
                .parse::<u32>()
                .expect("failed to parse NAXIS1");
        } else if k == "NAXIS2" {
            dy = v.split('/').collect::<Vec<_>>()[0]
                .trim()
                .parse::<u32>()
                .expect("failed to parse NAXIS2");
        }
    }
    (dx, dy, bytes_per_element)
}

pub fn parse_primary_hdu(data: &[u8]) -> (HashMap<String, String>, Vec<u8>) {
    let nblocks = data.len() / 2880;
    let mut last_block = false;

    let mut kv_pairs: HashMap<String, String> = HashMap::new();
    let mut n_header_blocks = 0;
    while !last_block {
        let block = &data[2880 * n_header_blocks..2880 * (n_header_blocks + 1)];
        kv_pairs.extend(parse_hdu_block(block));
        kv_pairs.retain(|k, _v| !k.is_empty());
        if kv_pairs.keys().any(|k| k == "END") {
            last_block = true;
        }
        n_header_blocks += 1;
    }

    // Data
    let (dx, dy, bytes_per_element) = get_image_dims(&kv_pairs);
    let nbytes = dx as usize * dy as usize * bytes_per_element as usize;
    let mut n_data_blocks = nbytes / 2880;
    if nbytes % 2880 > 0 {
        n_data_blocks += 1;
    }
    let start_idx = 2880 * n_header_blocks;
    let end_idx = start_idx + 2880 * n_data_blocks;
    let _hdu_data = &data[start_idx..end_idx];
    let truncated_hdu_data = &data[start_idx..start_idx + nbytes];
    (kv_pairs, truncated_hdu_data.to_vec())
}

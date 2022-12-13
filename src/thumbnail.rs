use crate::error::Error;
use crate::Result;

use std::fs;
use std::io::Seek;
use std::io::{Read, SeekFrom};
use std::path::Path;
use std::time::Instant;

const BUFFER_LEN: usize = 2_usize.pow(20);

const FIRST: [u8; 4] = [0xFF, 0xD8, 0xFF, 0xDB];
const END: [u8; 2] = [0xFF, 0xD9];
const FOURTH: [u8; 5] = [0xd8, 0xe0, 0xe1, 0xe2, 0xdb];

const MIN_SIZE: u64 = 56 * 2_u64.pow(10); // 56KiB

pub fn extract_images(filename: impl AsRef<Path>, mut skip: u8) -> Result<Vec<u8>> {
    if skip > 3 {
        return Err(Error::capture("Skip shouldn't be greater than 3"))?;
    }
    let filename = filename.as_ref();

    let mut src_f = fs::File::open(filename)?;
    let mut hit_index: Vec<(u64, u64)> = Vec::new();

    let mut search_strings_idx = 0;
    let search_strings_iter = &mut [FIRST.iter(), END.iter()];
    let mut search_strings_iter_cloned = search_strings_iter.clone();
    let mut search_strings = search_strings_iter_cloned.iter_mut();
    let mut search_str_bytes = search_strings.next();
    let mut search_byte = search_str_bytes.as_mut().unwrap().next();

    let buffer = &mut [0; BUFFER_LEN];
    // let mut hits = 0;
    let mut start_addr = 0;
    let mut searching = false;
    let mut middle_searching = false;
    let mut reset_search_byte = false;
    let mut cur_addr: u64 = 0;
    let start_time = Instant::now();

    loop {
        let n = src_f.read(buffer)?;
        if n == 0 {
            break;
        }

        for (cur_byte, &byte) in (0_u64..).zip(buffer.iter()) {
            if search_byte.is_none() {
                reset_search_byte = true;
            }

            if search_byte.is_some()
                && if *search_byte.unwrap() == 0xDB {
                    FOURTH.contains(&byte)
                } else if *search_byte.unwrap() == 0xd9 {
                    if byte == 0xd9 {
                        let buf = make_buffer(&src_f, start_addr, cur_addr)?;
                        img_parts::jpeg::Jpeg::from_bytes(buf.into()).is_ok()
                    } else {
                        byte == *search_byte.unwrap()
                    }
                } else {
                    byte == *search_byte.unwrap()
                }
            {
                if !searching {
                    start_addr = cur_addr;
                    searching = true;
                }
                middle_searching = true;
                search_byte = search_str_bytes.as_mut().unwrap().next(); // set to the next search byte
                if search_byte.is_none() {
                    // use next search string when last one is finished
                    search_str_bytes = search_strings.next();
                    search_strings_idx += 1;
                    middle_searching = false;
                    search_byte = match search_str_bytes {
                        Some(ref mut search_str_bytes) => search_str_bytes.next(),
                        None => {
                            // write_image(filename, &src_f, start_addr, cur_addr, hits);
                            // let buf = make_buffer(&src_f, start_addr, cur_addr, hits);
                            // if img_parts::jpeg::Jpeg::from_bytes(buf.into()).is_ok() {
                            //     continue
                            // }
                            // if verbose {
                            //     println!("Found {}. image at {}", hits, start_addr);
                            // }
                            // error!("size is smaller than {}", MIN_SIZE);
                            hit_index.push((start_addr, cur_addr));
                            src_f
                                .seek(SeekFrom::Start(cur_addr + (BUFFER_LEN as u64 - cur_byte)))?;

                            if skip == 0 {
                                if (cur_addr - start_addr) < MIN_SIZE {
                                    trace!("size is smaller than {}", MIN_SIZE);
                                    skip += 1
                                } else {
                                    let buffer = make_buffer(&src_f, start_addr, cur_addr)?;
                                    // crate::helpers::remove_exif(&mut buffer)?;
                                    return Ok(buffer);
                                }
                            }

                            skip -= 1;
                            // hits += 1;
                            search_strings_idx = 0;
                            reset_search_byte = true;
                            searching = false;
                            None
                        }
                    }
                }
            } else if middle_searching {
                reset_search_byte = true;
                middle_searching = false;

                if search_strings_idx == 0 {
                    searching = false;
                }
            }

            if reset_search_byte {
                search_strings_iter_cloned = search_strings_iter.clone();
                search_strings = search_strings_iter_cloned.iter_mut();
                search_str_bytes = search_strings.nth(search_strings_idx);
                // search_byte = search_str_bytes.as_mut().unwrap().nth(0);
                search_byte = search_str_bytes.as_mut().unwrap().next();
                reset_search_byte = false;
            }

            cur_addr += 1;
            // cur_byte += 1;
        }
    }

    let time = Instant::now() - start_time;
    if skip as usize + 1 >= hit_index.len() {
        warn!("Unable to get thumbnail using extract {:?}", filename);
        return get_thumbnail_from_raw(filename);
    }
    trace!(
        "Generating thumbnail for {} took {:?}",
        filename.display(),
        time
    );
    let start = hit_index[skip as usize].0;
    let end = hit_index[skip as usize].1;

    let buffer = make_buffer(&src_f, start, end)?;
    // crate::helpers::remove_exif(&mut buffer)?;
    Ok(buffer)
}

pub fn make_buffer(mut src_f: &std::fs::File, start_addr: u64, end_addr: u64) -> Result<Vec<u8>> {
    let mut jpeg_buffer: Vec<u8> = vec![0; (end_addr - start_addr) as usize];
    src_f.seek(SeekFrom::Start(start_addr))?;
    let n = src_f.read(&mut jpeg_buffer)?;
    if n != jpeg_buffer.len() {
        warn!("Thumbnail buffer was not filled")
    }
    Ok(jpeg_buffer)
}
pub fn get_thumbnail_from_raw(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    let mut p = libraw_r::Processor::default();
    p.open(path)?;
    Ok(p.jpeg(75)?)
}

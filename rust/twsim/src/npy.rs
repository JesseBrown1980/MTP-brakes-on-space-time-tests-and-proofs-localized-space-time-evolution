//! Minimal reader for the 1-D little-endian 8-byte integer .npy files the tables use.

use std::fs;

fn parse(path: &str) -> Result<(Vec<u8>, usize, String), String> {
    let b = fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    if b.len() < 10 || &b[..6] != b"\x93NUMPY" {
        return Err(format!("{path}: not npy"));
    }
    let (hlen, start) = if b[6] == 1 {
        (u16::from_le_bytes([b[8], b[9]]) as usize, 10)
    } else {
        (u32::from_le_bytes([b[8], b[9], b[10], b[11]]) as usize, 12)
    };
    let header = String::from_utf8_lossy(&b[start..start + hlen]).to_string();
    if !header.contains("'fortran_order': False") {
        return Err(format!("{path}: fortran order"));
    }
    let shape = header.split("'shape': (").nth(1).ok_or("shape")?;
    let n: usize = shape.split(',').next().ok_or("shape")?.trim().trim_end_matches(')').parse().map_err(|_| "shape")?;
    let descr = header.split("'descr': '").nth(1).ok_or("descr")?.split('\'').next().ok_or("descr")?.to_string();
    let data = b[start + hlen..].to_vec();
    if data.len() != n * 8 {
        return Err(format!("{path}: size {} != {}", data.len(), n * 8));
    }
    Ok((data, n, descr))
}

pub fn read_u64(path: &str) -> Result<Vec<u64>, String> {
    let (d, n, descr) = parse(path)?;
    if descr != "<u8" {
        return Err(format!("{path}: dtype {descr}"));
    }
    Ok((0..n).map(|i| u64::from_le_bytes(d[8 * i..8 * i + 8].try_into().unwrap())).collect())
}

pub fn read_i64(path: &str) -> Result<Vec<i64>, String> {
    let (d, n, descr) = parse(path)?;
    if descr != "<i8" {
        return Err(format!("{path}: dtype {descr}"));
    }
    Ok((0..n).map(|i| i64::from_le_bytes(d[8 * i..8 * i + 8].try_into().unwrap())).collect())
}

use std::num::Wrapping;

/**
 * encodes u64 as an array of u32 elements, each 4 ascii digits
 * like:
 * [u32, u32, u32, u32]
 * where
 * u32: 00110001 00110000 00110000 00110000
 * ascii:  1        0         0        0
 */
pub fn nonce_to_u32arr(nonce: u64) -> [u32; 5] {
    let mut digit;
    let mut arr: [u32; 5] = [0; 5];

    digit = (nonce / 1) % 10;
    arr[4] += (digit | 0x30) as u32;
    digit = (nonce / 10) % 10;
    arr[4] += ((digit | 0x30) << 8) as u32;
    digit = (nonce / 100) % 10;
    arr[4] += ((digit | 0x30) << 16) as u32;
    digit = (nonce / 1000) % 10;
    arr[4] += ((digit | 0x30) << 24) as u32;

    digit = (nonce / 10000) % 10;
    arr[3] += (digit | 0x30) as u32;
    digit = (nonce / 100000) % 10;
    arr[3] += ((digit | 0x30) << 8) as u32;
    digit = (nonce / 1000000) % 10;
    arr[3] += ((digit | 0x30) << 16) as u32;
    digit = (nonce / 10000000) % 10;
    arr[3] += ((digit | 0x30) << 24) as u32;

    digit = (nonce / 100000000) % 10;
    arr[2] += ((digit | 0x30)) as u32;
    digit = (nonce / 1000000000) % 10;
    arr[2] += ((digit | 0x30) << 8) as u32;
    digit = (nonce / 10000000000) % 10;
    arr[2] += ((digit | 0x30) << 16) as u32;
    digit = (nonce / 100000000000) % 10;
    arr[2] += ((digit | 0x30) << 24) as u32;

    digit = (nonce / 1000000000000) % 10;
    arr[1] += ((digit | 0x30)) as u32;
    digit = (nonce / 10000000000000) % 10;
    arr[1] += ((digit | 0x30) << 8) as u32;
    digit = (nonce / 100000000000000) % 10;
    arr[1] += ((digit | 0x30) << 16) as u32;
    digit = (nonce / 1000000000000000) % 10;
    arr[1] += ((digit | 0x30) << 24) as u32;

    digit = (nonce / 10000000000000000) % 10;
    arr[0] += ((digit | 0x30)) as u32;
    digit = (nonce / 100000000000000000) % 10;
    arr[0] += ((digit | 0x30) << 8) as u32;
    digit = (nonce / 1000000000000000000) % 10;
    arr[0] += ((digit | 0x30) << 16) as u32;
    digit = (nonce / 10000000000000000000) % 10;
    arr[0] += ((digit | 0x30) << 24) as u32;

    return arr;
}

pub fn to_u32(data: &str) -> u32 {
    let mut res: u32 = 0;

    res += data.chars().nth(0).unwrap() as u32;
    res = res << 8;
    res += data.chars().nth(1).unwrap() as u32;
    res = res << 8;
    res += data.chars().nth(2).unwrap() as u32;
    res = res << 8;
    res += data.chars().nth(3).unwrap() as u32;

    return res;
}

pub fn sha1_prehash(hash: &str) -> [u32; 5] {
    let mut words: [u32; 80] = [0; 80];

    for i in 0..16 {
        words[i] = crate::modules::helpers::to_u32(&hash[(i*4)..(i*4) + 4])
    }

    for i in 16..80 {
        words[i] = ROTL(words[i-3] ^ words[i-8] ^ words[i-14] ^ words[i-16], 1);
    }

    // allow wrapping additions on russy ahhhh
    let h0: Wrapping<u32> = Wrapping(0x67452301);
    let h1: Wrapping<u32> = Wrapping(0xEFCDAB89);
    let h2: Wrapping<u32> = Wrapping(0x98BADCFE);
    let h3: Wrapping<u32> = Wrapping(0x10325476);
    let h4: Wrapping<u32> = Wrapping(0xC3D2E1F0);

    let k0: Wrapping<u32> = Wrapping(0x5A827999);
    let k1: Wrapping<u32> = Wrapping(0x6ED9EBA1);
    let k2: Wrapping<u32> = Wrapping(0x8F1BBCDC);
    let k3: Wrapping<u32> = Wrapping(0xCA62C1D6);

    let mut a: Wrapping<u32> = h0;
    let mut b: Wrapping<u32> = h1;
    let mut c: Wrapping<u32> = h2;
    let mut d: Wrapping<u32> = h3;
    let mut e: Wrapping<u32> = h4;
    let mut f: Wrapping<u32>;
    let mut k: Wrapping<u32>;
    let mut t: Wrapping<u32>;

    // main loop
    for i in 0..80 {
        if i < 20 {
            f = (b & c) | ((!b) & d);
            k = k0;
        } else if i < 40 {
            f = b ^ c ^ d;
            k = k1;
        } else if i < 60 {
            f = (b & c) | (b & d) | (c & d);
            k = k2;
        } else {
            f = b ^ c ^ d;
            k = k3;
        }

        t = Wrapping(ROTL(a.0, 5)) + f + e + k + Wrapping(words[i]);
        e = d;
        d = c;
        c = Wrapping(ROTL(b.0, 30));
        b = a;
        a = t;
    }

    return [(h0 + a).0, (h1 + b).0, (h2 + c).0, (h3 + d).0, (h4 + e).0];
}

#[allow(non_snake_case)]
pub fn ROTL(x: u32, n: u32) -> u32 {(x << n) | (x >> (32 - n))}
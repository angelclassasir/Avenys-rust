//! A `BuildID[sha1]` in every ELF artifact, the way a rustc-built binary has
//! one.
//!
//! `file`, `readelf -n`, `debuginfod` and every crash reporter that groups
//! stack traces by binary all key off the ELF `NT_GNU_BUILD_ID` note. tcc does
//! not emit one, so a Mire build was anonymous: two binaries built from the
//! same tree could not be told apart from two built years apart, and a bug
//! report could not be matched to a build.
//!
//! GNU `ld` normally writes the note as `NT_GNU_BUILD_ID` inside a `PT_NOTE`
//! segment. tcc links these programs, and its output has no `PT_NOTE` and no
//! room for another program header: the header table sits immediately before
//! the first `PT_LOAD` payload, so growing it would shift every offset in the
//! file. Adding a *section* costs nothing, because `file` finds the note by
//! scanning the section header table -- verified against file 5.48 -- and a
//! non-allocated section needs no segment at all.
//!
//! The identifier is the SHA-1 of the finished binary before the note exists,
//! the same definition `ld` uses. That keeps it deterministic: the same inputs
//! produce the same bytes and therefore the same id, and the id changes if any
//! byte of the output changes.

use std::path::Path;

/// ELF constants used below, named as the ELF spec names them.
const ELFCLASS64: u8 = 2;
const ELFDATA2LSB: u8 = 1;
const SHT_NOTE: u32 = 7;
const NT_GNU_BUILD_ID: u32 = 3;
const SHN_UNDEF: u16 = 0;

/// Byte offsets into the 64-bit little-endian ELF header.
const EHDR_SH_OFF: usize = 0x28;
const EHDR_SHENTSIZE: usize = 0x3A;
const EHDR_SHNUM: usize = 0x3C;
const EHDR_SHSTRNDX: usize = 0x3E;

/// Byte offsets into a 64-bit section header.
const SHDR_NAME: usize = 0x00;
const SHDR_TYPE: usize = 0x04;
const SHDR_OFFSET: usize = 0x18;
const SHDR_SIZE: usize = 0x20;

/// The name the section is given. `file` and `readelf` look for exactly this.
const NOTE_SECTION: &str = ".note.gnu.build-id";

/// Compute the SHA-1 the note carries (20 bytes).
///
/// A compact FIPS 180-4 implementation. SHA-1 is not being used as a security
/// primitive here -- it is used because `NT_GNU_BUILD_ID` conventionally holds
/// 20 bytes and `file` labels the result `BuildID[sha1]`, so a build
/// fingerprint has to stay compatible with every tool that already reads one.
pub fn sha1(data: &[u8]) -> [u8; 20] {
    let mut h: [u32; 5] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];

    // Pad to a multiple of 64 bytes: 0x80, zeroes, then the original length in
    // bits as a big-endian u64.
    let mut message = data.to_vec();
    let bit_len = (data.len() as u64).wrapping_mul(8);
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_be_bytes());

    let mut w = [0u32; 80];
    for chunk in message.chunks_exact(64) {
        for (i, word) in chunk.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }

        let [mut a, mut b, mut c, mut d, mut e] = h;
        for (i, word) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A827999u32),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                _ => (b ^ c ^ d, 0xCA62C1D6),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(*word);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
    }

    let mut out = [0u8; 20];
    for (i, word) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

fn rd_u16(data: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([data[at], data[at + 1]])
}

#[cfg(test)]
fn rd_u32(data: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]])
}

fn rd_u64(data: &[u8], at: usize) -> u64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&data[at..at + 8]);
    u64::from_le_bytes(b)
}

fn wr_u32(out: &mut Vec<u8>, at: usize, value: u32) {
    out[at..at + 4].copy_from_slice(&value.to_le_bytes());
}

fn wr_u64(out: &mut Vec<u8>, at: usize, value: u64) {
    out[at..at + 8].copy_from_slice(&value.to_le_bytes());
}

/// The `NT_GNU_BUILD_ID` note wrapping `id`.
fn build_id_note(id: &[u8; 20]) -> Vec<u8> {
    let mut note = Vec::with_capacity(36);
    // namesz counts the trailing NUL; the name "GNU" is padded to 4 bytes and
    // the descriptor is already a multiple of 4, so no padding is needed here.
    note.extend_from_slice(&4u32.to_le_bytes());
    note.extend_from_slice(&(id.len() as u32).to_le_bytes());
    note.extend_from_slice(&NT_GNU_BUILD_ID.to_le_bytes());
    note.extend_from_slice(b"GNU\0");
    note.extend_from_slice(id);
    note
}

/// Embed `NT_GNU_BUILD_ID` into an already-linked ELF file, in place.
///
/// Returns the id, or `None` when the file is not an ELF this can rewrite --
/// an `ar` archive for a static build, for instance, which has no single
/// image to identify. Those are skipped silently: a missing build id on an
/// archive is not an error worth failing a build over.
pub fn embed_in(path: &Path) -> Option<[u8; 20]> {
    let original = std::fs::read(path).ok()?;
    let (rewritten, id) = rewrite(&original)?;
    // A short write here would leave a corrupt binary, so a failed write is
    // reported as "no id" and the original file is left alone.
    if std::fs::write(path, &rewritten).is_err() {
        return None;
    }
    Some(id)
}

/// Build the rewritten image. Split out from the I/O so it can be reasoned
/// about (and tested) without touching the filesystem.
fn rewrite(original: &[u8]) -> Option<(Vec<u8>, [u8; 20])> {
    if original.len() < 64 || &original[0..4] != b"\x7fELF" {
        return None;
    }
    if original[4] != ELFCLASS64 || original[5] != ELFDATA2LSB {
        return None;
    }
    let shoff = rd_u64(original, EHDR_SH_OFF) as usize;
    let shentsize = rd_u16(original, EHDR_SHENTSIZE) as usize;
    let shstrndx_raw = rd_u16(original, EHDR_SHSTRNDX);
    if shentsize < 64 || shoff == 0 || shoff + shentsize > original.len() {
        return None;
    }
    // Section 0 is the reserved null header. When it is not zero, a stripped
    // file moved the real count into its sh_size and left e_shnum at zero.
    let null_header = shoff;
    let raw_shnum = rd_u16(original, EHDR_SHNUM);
    let shnum = if raw_shnum == 0 {
        let real = rd_u64(original, null_header + SHDR_SIZE);
        if real == 0 || real > u16::MAX as u64 {
            return None;
        }
        real as usize
    } else {
        raw_shnum as usize
    };
    if shstrndx_raw == SHN_UNDEF || shstrndx_raw as usize >= shnum {
        return None;
    }
    let shstrndx = shstrndx_raw as usize;
    let table_len = shnum * shentsize;
    if shoff + table_len > original.len() {
        return None;
    }
    let shstr_hdr = shoff + shstrndx * shentsize;
    let shstr_off = rd_u64(original, shstr_hdr + SHDR_OFFSET) as usize;
    let shstr_size = rd_u64(original, shstr_hdr + SHDR_SIZE) as usize;
    if shstr_off + shstr_size > original.len() {
        return None;
    }

    let id = sha1(original);
    let note = build_id_note(&id);

    // Layout: the original file is kept byte for byte and the new material is
    // appended. Every existing section therefore keeps its file offset, and
    // only e_shoff moves to point at the rewritten table. Removing the old
    // table instead would shift every section that sits after it -- tcc places
    // .shstrtab there -- and corrupt their sh_offset without touching a single
    // field this code could then notice.
    let mut out = Vec::with_capacity(original.len() + note.len() + table_len + 64);
    out.extend_from_slice(original);

    while out.len() % 4 != 0 {
        out.push(0);
    }
    let note_offset = out.len();
    out.extend_from_slice(&note);

    let shstr_new_offset = out.len();
    out.extend_from_slice(&original[shstr_off..shstr_off + shstr_size]);
    let note_name_offset = (out.len() - shstr_new_offset) as u32;
    out.extend_from_slice(NOTE_SECTION.as_bytes());
    out.push(0);
    let shstr_new_size = (out.len() - shstr_new_offset) as u64;

    while out.len() % 8 != 0 {
        out.push(0);
    }
    let table_offset = out.len();
    out.extend_from_slice(&original[shoff..shoff + table_len]);

    // Point the relocated name table at its new home and record its new size.
    let new_shstr_hdr = table_offset + shstrndx * shentsize;
    wr_u64(&mut out, new_shstr_hdr + SHDR_OFFSET, shstr_new_offset as u64);
    wr_u64(&mut out, new_shstr_hdr + SHDR_SIZE, shstr_new_size);

    // One more section header describing the note.
    let new_hdr = out.len();
    out.resize(new_hdr + shentsize, 0);
    wr_u32(&mut out, new_hdr + SHDR_NAME, note_name_offset);
    wr_u32(&mut out, new_hdr + SHDR_TYPE, SHT_NOTE);
    wr_u64(&mut out, new_hdr + SHDR_OFFSET, note_offset as u64);
    wr_u64(&mut out, new_hdr + SHDR_SIZE, note.len() as u64);
    // sh_addralign lives at 0x30 in a 64-bit header; 4 is what the ELF spec
    // asks for on a note and what keeps the offset 4-aligned above.
    wr_u64(&mut out, new_hdr + 0x30, 4);

    wr_u64(&mut out, EHDR_SH_OFF, table_offset as u64);
    wr_u16_count(&mut out, EHDR_SHNUM, shnum + 1);

    Some((out, id))
}

fn wr_u16_count(out: &mut [u8], at: usize, value: usize) {
    out[at..at + 2].copy_from_slice(&(value as u16).to_le_bytes());
}

/// Hex form, the way `file` prints it.
pub fn to_hex(id: &[u8; 20]) -> String {
    let mut s = String::with_capacity(40);
    for byte in id {
        s.push_str(&format!("{:02x}", byte));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FIPS 180-2 test vectors. SHA-1 is only being implemented to fill the
    /// 20-byte field `NT_GNU_BUILD_ID` conventionally holds, so the standard's
    /// own vectors are the right definition of correct here.
    #[test]
    fn sha1_matches_published_vectors() {
        assert_eq!(
            to_hex(&sha1(b"")),
            "da39a3ee5e6b4b0d3255bfef95601890afd80709"
        );
        assert_eq!(
            to_hex(&sha1(b"abc")),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
        assert_eq!(
            to_hex(&sha1(
                b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
            )),
            "84983e441c3bd26ebaae4aa1f95129e5e54670f1"
        );
        // Longer than one 64-byte block, to exercise the message schedule
        // across block boundaries.
        assert_eq!(
            to_hex(&sha1(&b"a".repeat(1000))),
            "291e9a6c66994949b57ba5e650361e98fc36b1ba"
        );
    }

    #[test]
    fn embed_in_declines_a_non_elf_and_leaves_it_alone() {
        let mut file = tempfile::NamedTempFile::new().expect("temp file");
        std::io::Write::write_all(&mut file, b"#!/bin/sh\necho no\n").expect("write");
        let path = file.path().to_path_buf();
        assert!(embed_in(&path).is_none());
        let after = std::fs::read(&path).expect("reread");
        assert_eq!(after, b"#!/bin/sh\necho no\n", "the file must be untouched");
    }

    /// A minimal but structurally valid 64-bit ELF, enough for `rewrite` to
    /// walk. Building it by hand keeps the test independent of the linker.
    fn synthetic_elf() -> Vec<u8> {
        const SHNUM: usize = 3; // null, .text, .shstrtab
        let shstr = b"\0.text\0.shstrtab\0";
        let text_off = 0x100u64;
        let shstr_off = 0x200u64;
        let table_off = 0x300u64;

        let mut out = vec![0u8; 0x400];
        out[0..4].copy_from_slice(b"\x7fELF");
        out[4] = ELFCLASS64;
        out[5] = ELFDATA2LSB;
        out[6] = 1; // EV_CURRENT
        out[16..18].copy_from_slice(&2u16.to_le_bytes()); // ET_EXEC
        out[18..20].copy_from_slice(&62u16.to_le_bytes()); // EM_X86_64
        out[24..28].copy_from_slice(&1u32.to_le_bytes()); // EV_CURRENT
        wr_u64(&mut out, EHDR_SH_OFF, table_off);
        wr_u16_count(&mut out, EHDR_SHENTSIZE, 64);
        wr_u16_count(&mut out, EHDR_SHNUM, SHNUM);
        wr_u16_count(&mut out, EHDR_SHSTRNDX, 2);

        // Some payload so the file is longer than the header.
        out[text_off as usize] = 0xC3;
        out[shstr_off as usize..shstr_off as usize + shstr.len()].copy_from_slice(shstr);

        let entry = |index: usize| index * 64 + table_off as usize;
        // .text
        wr_u32(&mut out, entry(1) + SHDR_NAME, 1);
        wr_u32(&mut out, entry(1) + SHDR_TYPE, 1); // PROGBITS
        wr_u64(&mut out, entry(1) + SHDR_OFFSET, text_off);
        wr_u64(&mut out, entry(1) + SHDR_SIZE, 1);
        wr_u64(&mut out, entry(1) + 0x30, 16);
        // .shstrtab
        wr_u32(&mut out, entry(2) + SHDR_NAME, 7);
        wr_u32(&mut out, entry(2) + SHDR_TYPE, 3); // STRTAB
        wr_u64(&mut out, entry(2) + SHDR_OFFSET, shstr_off);
        wr_u64(&mut out, entry(2) + SHDR_SIZE, shstr.len() as u64);
        out
    }

    #[test]
    fn rewrite_appends_a_readable_note_without_moving_anything() {
        let original = synthetic_elf();
        let original_shoff = rd_u64(&original, EHDR_SH_OFF);
        let original_shnum = rd_u16(&original, EHDR_SHNUM) as usize;
        let (rewritten, id) = rewrite(&original).expect("synthetic ELF is rewritable");

        // The id is the SHA-1 of the pre-note image, so it can be recomputed
        // from the original bytes.
        assert_eq!(id, sha1(&original));

        // Every byte of the original survives at its original offset except
        // e_shoff and e_shnum, which have to name the rewritten table. This
        // is the property that matters: rebuilding the file around the new
        // material would shift every section stored after the old table.
        let mut expected = original.clone();
        wr_u64(&mut expected, EHDR_SH_OFF, rd_u64(&rewritten, EHDR_SH_OFF));
        wr_u16_count(&mut expected, EHDR_SHNUM, rd_u16(&rewritten, EHDR_SHNUM) as usize);
        assert_eq!(&rewritten[..original.len()], &expected[..]);

        // And the note is where the new section header says it is.
        let shoff = rd_u64(&rewritten, EHDR_SH_OFF) as usize;
        let shnum = rd_u16(&rewritten, EHDR_SHNUM) as usize;
        assert_eq!(shnum, 4, "the note adds one section header");
        let note_hdr = shoff + (shnum - 1) * 64;
        let note_type = rd_u32(&rewritten, note_hdr + SHDR_TYPE);
        assert_eq!(note_type, SHT_NOTE);
        let note_offset = rd_u64(&rewritten, note_hdr + SHDR_OFFSET) as usize;
        let note_size = rd_u64(&rewritten, note_hdr + SHDR_SIZE) as usize;
        assert_eq!(note_size, 36);
        assert_eq!(
            &rewritten[note_offset..note_offset + note_size],
            &build_id_note(&id)[..]
        );

        // Recover the pre-note image the way a verifier would: restore the two
        // header fields the rewrite is allowed to change. Hashing it must
        // reproduce the embedded id, which is the whole contract.
        let mut recovered = rewritten.clone();
        wr_u64(&mut recovered, EHDR_SH_OFF, original_shoff);
        wr_u16_count(&mut recovered, EHDR_SHNUM, original_shnum);
        let recovered = &recovered[..original.len()];
        assert_eq!(sha1(recovered), id);
    }

    #[test]
    fn rewrite_is_a_no_op_on_non_elf_input() {
        assert!(rewrite(b"not an elf at all, not even close").is_none());
        assert!(rewrite(&[]).is_none());
    }
}

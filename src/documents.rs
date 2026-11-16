//! Bounded local TXT/DOCX extraction. Never extracts ZIP paths to disk.
use quick_xml::{events::Event, Reader};
use std::io::{Cursor, Read};
pub const MAX_FILE_BYTES: usize = 10 * 1024 * 1024;
const MAX_XML: u64 = 4 * 1024 * 1024;
fn entry(
    archive: &mut zip::ZipArchive<Cursor<&[u8]>>,
    name: &str,
) -> Result<Vec<u8>, &'static str> {
    let f = archive
        .by_name(name)
        .map_err(|_| "Struktur DOCX tidak lengkap.")?;
    if f.size() > MAX_XML {
        return Err("Isi XML DOCX terlalu besar.");
    }
    let mut bytes = Vec::new();
    f.take(MAX_XML + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "DOCX rusak atau terenkripsi.")?;
    if bytes.len() as u64 > MAX_XML {
        return Err("Isi XML DOCX terlalu besar.");
    }
    Ok(bytes)
}
pub fn extract(name: &str, bytes: &[u8]) -> Result<String, &'static str> {
    if bytes.len() > MAX_FILE_BYTES {
        return Err("File terlalu besar. Maksimal 10 MB (10.485.760 byte).");
    }
    let lower = name.to_lowercase();
    let text = if lower.ends_with(".txt") {
        let s = std::str::from_utf8(bytes).map_err(|_| "File bukan teks UTF-8 yang valid.")?;
        if s.contains('\0') {
            return Err("File bukan teks UTF-8 yang valid.");
        }
        s.trim_start_matches('\u{feff}').to_string()
    } else if lower.ends_with(".docx") {
        docx(bytes)?
    } else {
        return Err("Gunakan file DOCX atau TXT. PDF dan DOC belum didukung.");
    };
    if text.chars().count() > crate::MAX_CHARS {
        return Err("Teks melebihi 100.000 karakter. Pilih bagian laporan yang lebih pendek.");
    }
    if text.trim().is_empty() {
        return Err("Dokumen tidak berisi teks yang dapat dibaca. OCR belum tersedia.");
    }
    Ok(text)
}
fn docx(bytes: &[u8]) -> Result<String, &'static str> {
    if !bytes.starts_with(b"PK\x03\x04") {
        return Err("File bukan DOCX yang valid.");
    }
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| "File DOCX rusak.")?;
    if archive.len() > 2048 {
        return Err("Terlalu banyak komponen dalam DOCX.");
    }
    let mut total = 0u64;
    let mut names = std::collections::HashSet::new();
    for i in 0..archive.len() {
        let f = archive
            .by_index(i)
            .map_err(|_| "DOCX rusak atau terenkripsi.")?;
        total = total
            .checked_add(f.size())
            .ok_or("Ukuran DOCX tidak valid.")?;
        if total > 32 * 1024 * 1024 {
            return Err("DOCX terlalu besar setelah dekompresi (maks. 32 MB).");
        }
        if !names.insert(f.name().to_owned()) {
            return Err("DOCX memiliki komponen duplikat.");
        }
        if f.name().ends_with("vbaProject.bin") {
            return Err("Dokumen dengan macro tidak didukung.");
        }
    }
    let types = entry(&mut archive, "[Content_Types].xml")?;
    if !std::str::from_utf8(&types)
        .map_err(|_| "Metadata DOCX tidak valid.")?
        .contains(
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml",
        )
    {
        return Err("Jenis dokumen bukan DOCX standar.");
    }
    let xml = entry(&mut archive, "word/document.xml")?;
    let mut reader = Reader::from_reader(xml.as_slice());
    let mut output = String::new();
    let mut in_text = false;
    let mut root = false;
    let mut depth = 0usize;
    loop {
        match reader.read_event().map_err(|_| "XML DOCX rusak.")? {
            Event::DocType(_) => return Err("DOCTYPE tidak diizinkan dalam DOCX."),
            Event::Start(e) => {
                depth += 1;
                if depth > 128 {
                    return Err("Struktur DOCX terlalu dalam.");
                }
                if depth == 1 {
                    if e.local_name().as_ref() != b"document" {
                        return Err("Root dokumen tidak valid.");
                    }
                    root = true;
                }
                if e.local_name().as_ref() == b"t" {
                    in_text = true;
                }
            }
            Event::Text(e) if in_text => {
                output.push_str(&e.unescape().map_err(|_| "Teks XML tidak valid.")?)
            }
            Event::End(e) => {
                if e.local_name().as_ref() == b"t" {
                    in_text = false;
                }
                if e.local_name().as_ref() == b"p" {
                    output.push('\n');
                }
                depth = depth.checked_sub(1).ok_or("XML DOCX tidak seimbang.")?;
            }
            Event::Empty(e) => {
                if matches!(e.local_name().as_ref(), b"tab" | b"br" | b"cr") {
                    output.push(' ');
                }
            }
            Event::Eof => break,
            _ => {}
        };
        if output.len() > crate::MAX_CHARS * 4 {
            return Err("Teks DOCX terlalu panjang.");
        }
    }
    if !root || depth != 0 {
        return Err("XML DOCX tidak lengkap.");
    }
    Ok(output)
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    fn fixture(xml: &str) -> Vec<u8> {
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let opt =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("[Content_Types].xml", opt).unwrap();
        zip.write_all(b"<Types>application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml</Types>").unwrap();
        zip.start_file("word/document.xml", opt).unwrap();
        zip.write_all(xml.as_bytes()).unwrap();
        zip.finish().unwrap().into_inner()
    }
    #[test]
    fn paragraphs_and_entities() {
        let b=fixture("<w:document xmlns:w='urn:test'><w:body><w:p><w:r><w:t>Tim &amp; </w:t></w:r><w:r><w:t>organisasi</w:t></w:r></w:p><w:tbl><w:tr><w:tc><w:p><w:r><w:t>Produktivitas</w:t></w:r></w:p></w:tc></w:tr></w:tbl></w:body></w:document>");
        assert_eq!(
            extract("a.docx", &b).unwrap(),
            "Tim & organisasi\nProduktivitas\n"
        );
    }
    #[test]
    fn renamed_file() {
        assert!(extract("a.docx", b"not a zip").is_err());
    }
    #[test]
    fn corrupt_xml() {
        assert!(extract("a.docx", &fixture("<document><p>")).is_err());
    }
    #[test]
    fn no_entities() {
        assert!(extract("a.docx", &fixture("<!DOCTYPE x><document/>")).is_err());
    }
    #[test]
    fn zip_bomb() {
        assert!(extract("a.docx", &fixture(&"a".repeat(MAX_XML as usize + 1))).is_err());
    }
    #[test]
    fn limits() {
        assert!(extract("a.txt", &vec![b'a'; MAX_FILE_BYTES + 1]).is_err());
    }
    #[test]
    fn utf8() {
        assert_eq!(extract("a.txt", "équipe".as_bytes()).unwrap(), "équipe");
        assert!(extract("a.txt", &[255]).is_err());
    }
}

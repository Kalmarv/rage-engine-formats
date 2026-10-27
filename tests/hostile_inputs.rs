//! Inputs built by hand to be wrong: references the install does not hold, mount data that is not what the game's
//! files are, and archives that do not exist. Each must come back as an [`Error`] that names where the bad bytes are
//! and what was expected, never a panic.
use rage_engine_formats::archive::archive_set::{ArchiveSet, FileRef, Install, MapState};
use rage_engine_formats::archive::keys::Keys;
use rage_engine_formats::{Error, mount, xml};

/// The offset and detail of an [`Error::Invalid`].
fn invalid(e: Error) -> (&'static str, u64, String) {
    match e {
        Error::Invalid {
            format,
            offset,
            detail,
        } => (format, offset, detail),
        other => panic!("not an Invalid error: {other}"),
    }
}

fn one_file_install() -> Install {
    ArchiveSet::synthetic(&["a.ydr"], &[]).install
}

#[test]
fn a_file_reference_from_another_install_is_an_error() {
    let install = one_file_install();
    let keys = Keys::none();
    let foreign = FileRef { archive: 7, entry: 0 };
    for e in [
        install.binary(foreign, &keys).unwrap_err(),
        install.stored(foreign, &keys).map(|_| ()).unwrap_err(),
        install.resource(foreign, &keys).map(|_| ()).unwrap_err(),
    ] {
        let (format, _, detail) = invalid(e);
        assert_eq!(format, "install");
        assert!(
            detail.contains("archive 7") && detail.contains("1 archives"),
            "{detail}"
        );
    }
    assert!(ArchiveSet::synthetic(&["a.ydr"], &[]).path(foreign).is_none());
    // an entry the archive does not hold is the archive's error, not a panic
    let missing = FileRef { archive: 0, entry: 9 };
    assert!(install.binary(missing, &Keys::none()).is_err());
}

#[test]
fn an_install_without_its_mount_data_names_the_file_and_what_it_is_for() {
    let e = ArchiveSet::build(one_file_install(), &Keys::none(), MapState::Story).unwrap_err();
    assert!(matches!(e, Error::NotFound { .. }), "{e:?}");
    let text = e.to_string();
    assert!(text.contains("extratitleupdatedata.meta"), "{text}");
    assert!(text.contains("title-update mounts"), "{text}");
}

#[test]
fn xml_parse_errors_name_the_bad_bytes() {
    // the mismatched end tag starts at byte 5
    let (_, offset, detail) = invalid(xml::parse(b"<a>\n<b></c></a>", "t").unwrap_err());
    assert_eq!(offset, 7, "{detail}");
    assert!(detail.contains("</b>") && detail.contains("</c>"), "{detail}");
    // the byte order mark counts: the second root starts after 3 + 4 bytes
    let (_, offset, _) = invalid(xml::parse(b"\xEF\xBB\xBF<a/><b/>", "t").unwrap_err());
    assert_eq!(offset, 7);
    // invalid UTF-8 at byte 4
    let (_, offset, _) = invalid(xml::parse(b"<a>\xFF</a>", "t").unwrap_err());
    assert_eq!(offset, 3);
    // an unknown entity is at its `&`
    let (_, offset, detail) = invalid(xml::parse(b"<a>x&nope;</a>", "t").unwrap_err());
    assert_eq!(offset, 4, "{detail}");
    // an end tag that closes another element is at the end tag, and says which end tag it expected
    let (_, offset, detail) = invalid(xml::parse(b"<a><b></a>", "t").unwrap_err());
    assert_eq!(offset, 6, "{detail}");
    assert!(detail.contains("expected </b>"), "{detail}");
    // an element left open is at its start tag
    let (_, offset, detail) = invalid(
        xml::parse(
            b"<a>
<b>", "t",
        )
        .unwrap_err(),
    );
    assert_eq!(offset, 4, "{detail}");
    assert!(detail.contains("expected </b>"), "{detail}");
}

#[test]
fn mount_errors_name_the_element_that_is_wrong() {
    let doc =
        br#"<SMandatoryPacksData><Paths><Item>dlcpacks:\a\</Item><Bogus/></Paths></SMandatoryPacksData>"#;
    let at = doc.windows(6).position(|w| w == b"<Bogus").expect("fixture") as u64;
    let (format, offset, detail) = invalid(mount::pack_list(doc).unwrap_err());
    assert_eq!((format, offset), ("XML", at), "{detail}");
    assert!(detail.contains("expected <Item>"), "{detail}");

    let doc = b"<SSetupData>\n<order value=\"x\"/></SSetupData>";
    let at = doc.windows(6).position(|w| w == b"<order").expect("fixture") as u64;
    let (_, offset, detail) = invalid(mount::setup(doc, "setup2.xml").unwrap_err());
    assert_eq!(offset, at, "{detail}");
    assert!(detail.contains("expected an integer"), "{detail}");
}

#[test]
fn a_pack_path_of_another_kind_is_refused_at_its_item() {
    let doc = b"<SMandatoryPacksData><Paths><Item>dlcpacks:/a/</Item><Item>C:/elsewhere/</Item></Paths></SMandatoryPacksData>";
    let at = doc.windows(5).rposition(|w| w == b"<Item").expect("fixture") as u64;
    let (format, offset, detail) = invalid(mount::pack_list(doc).unwrap_err());
    assert_eq!((format, offset), ("XML", at), "{detail}");
    assert!(detail.contains("expected dlcpacks:/<name>/"), "{detail}");
}

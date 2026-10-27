//! Readers for the file formats of Rockstar's RAGE engine, as Grand Theft Auto V stores them: bytes (or, for
//! archives, paths) in, plain records out.
//!
//! It was written against GTA V Enhanced; the Legacy edition is read where a module says so ([`fxc`] reads the
//! Legacy edition's effect files, and [`archive::keys::Keys::from_legacy_executable`] finds key material in its
//! executable). Every read is bounded by the bytes it was given, and a malformed file is an [`Error`] naming the
//! format, the offset (0 for the XML files, which do not track one) and what was wrong, never a panic.
//!
//! - Archives ([`archive`]): RPF7 with its NG and AES ciphers, the mount data that says which archives are live
//!   ([`mount`]), and the install's listing. The cipher keys are never part of this crate: the caller supplies the key
//!   material ([`archive::keys::Keys`]) and the digests it is checked against ([`archive::keys::KeyDigests`]).
//! - Resources (RSC7, [`resource`]) and what they hold: drawables ([`drawable`]), texture dictionaries ([`texture`],
//!   buffer formats in [`sga`]), collision bounds ([`bounds`]), clip dictionaries ([`ycd`], [`ycd_channels`]).
//! - META documents ([`meta`]) and the map data ([`map`]) and ped metadata ([`ped`]) read from them.
//! - Shader containers: compiled effects ([`fxc`]) and shader databases ([`sgd2`], [`sgd2_effect`], [`sgd2_table`]).
//! - The game's XML data files ([`xml`]), their binary form RBF ([`rbf`]) and the texture-dictionary parent list
//!   ([`gtxd`]).
//!
//! # Example
//!
//! Parsing needs no game files. This reads a DLC pack list and shows a refusal:
//!
//! ```
//! use rage_engine_formats::{Error, mount, xml};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // A DLC pack list (`dlclist.xml`): the packs the game looks for, in list order.
//!     let dlclist =
//!         br"<SMandatoryPacksData><Paths><Item>dlcpacks:\mpbeach\</Item></Paths></SMandatoryPacksData>";
//!     println!("{:?}", mount::pack_list(dlclist)?);
//!
//!     // Malformed input is an error that says what was wrong, never a panic.
//!     let Err(refused) = xml::parse(b"<a><b></a>", "example.xml") else {
//!         unreachable!("the closing tag does not match")
//!     };
//!     assert!(matches!(refused, Error::Invalid { format: "XML", .. }));
//!     println!("{refused}");
//!     Ok(())
//! }
//! ```
//!
//! Opening an install (`archive::archive_set::Install::open`) takes the folder and the caller's key material; see
//! `examples/list_install.rs`.
#![deny(clippy::disallowed_methods, clippy::disallowed_types)]

pub mod archive;
pub mod bounds;
pub mod drawable;
pub mod fxc;
pub mod gtxd;
pub mod le;
pub mod map;
pub mod meta;
pub mod mount;
pub mod ped;
pub mod rbf;
pub mod resource;
pub mod sga;
pub mod sgd2;
pub mod sgd2_effect;
pub mod sgd2_table;
pub mod texture;
pub mod xml;
pub mod ycd;
pub mod ycd_channels;

/// Why a file could not be read.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The bytes do not hold what the format says: the format, the offset and what is wrong. The XML files ([`xml`],
    /// [`mount`]) report offset 0.
    #[error("{format} at {offset:#x}: {detail}")]
    Invalid {
        format: &'static str,
        offset: u64,
        detail: String,
    },
    /// The key material is missing, the wrong size or does not match its digest.
    #[error("key material: {0}")]
    Keys(String),
    /// A file could not be read.
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
    /// A compressed block did not inflate.
    #[error("compression: {0}")]
    Compression(#[from] primase_foundations::compress::Error),
}

/// `Result` with this crate's [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// A read past the bytes there are ([`primase_foundations::bytes::View`], [`primase_foundations::bytes::Cursor`]): the format's error at its
/// offset.
impl From<primase_foundations::bytes::BoundsError> for Error {
    fn from(e: primase_foundations::bytes::BoundsError) -> Self {
        Error::from(primase_foundations::bytes::FieldError {
            what: "bytes",
            bounds: e,
        })
    }
}

/// A read past the bytes there are, of a named format (`c.u32(LE).field("RBF")`).
impl From<primase_foundations::bytes::FieldError> for Error {
    fn from(e: primase_foundations::bytes::FieldError) -> Self {
        let offset = u64::try_from(e.bounds.absolute()).unwrap_or(u64::MAX);
        let detail = match e.bounds {
            primase_foundations::bytes::BoundsError::OutOfBounds { length, total, .. } => {
                format!("{length} bytes past the {total} there are")
            }
            primase_foundations::bytes::BoundsError::LengthOverflow { count, stride, .. } => {
                format!("{count} entries of {stride} bytes overflow")
            }
        };
        invalid(e.what, offset, detail)
    }
}

pub(crate) fn invalid(format: &'static str, offset: u64, detail: impl ToString) -> Error {
    Error::Invalid {
        format,
        offset,
        detail: detail.to_string(),
    }
}

//! File classes named by GTA V archive-member extensions. Resource parser modules document supported layouts;
//! classification alone does not claim that a file can be decoded. Unknown extensions remain unclassified.

/// The archive member's file class, independently of the consumer's opening or display policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileClass {
    /// Models (`ydr`).
    Ydr,
    /// Fragment models (`yft`).
    Yft,
    /// Model dictionaries (`ydd`).
    Ydd,
    /// Texture dictionaries (`ytd`).
    Ytd,
    /// Collision (`ybn`).
    Ybn,
    /// Map placements (`ymap`).
    Ymap,
    /// Archetype definitions (`ytyp`).
    Ytyp,
    /// Clip dictionaries (`ycd`).
    Ycd,
    /// Expression dictionaries (`yed`).
    Yed,
    /// Cloth (`yld`).
    Yld,
    /// Particle effects (`ypt`).
    Ypt,
    /// Navigation meshes (`ynv`).
    Ynv,
    /// Path nodes (`ynd`).
    Ynd,
    /// Vehicle recordings (`yvr`).
    Yvr,
    /// Waypoint recordings (`ywr`).
    Ywr,
    /// Pose matchers (`ypdb`).
    Ypdb,
    /// Metadata tables (`ymt`).
    Ymt,
    /// Manifests (`ymf`).
    Ymf,
    /// Metadata (`meta`).
    Meta,
    /// XML files (`xml`).
    Xml,
    /// Data files (`dat`).
    Dat,
    /// Audio streams (`awc`).
    Awc,
    /// Audio data (`rel`).
    Rel,
    /// Text tables (`gxt2`).
    Gxt2,
    /// Scripts (`ysc`).
    Ysc,
    /// Cutscenes (`cut`).
    Cut,
    /// Interface movies (`gfx`).
    Gfx,
    /// Videos (`bik`).
    Bik,
    /// Shaders (`fxc`).
    Fxc,
    /// Fonts (`ttf`).
    Ttf,
    /// Archives (`rpf`).
    Rpf,
    /// Name tables (`nametable`).
    Nametable,
    /// Images (`dds, png, jpg, bmp`).
    Dds,
    /// Text files (`txt`).
    Txt,
}
impl FileClass {
    /// Plural name of this file class.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Ydr => "Models",
            Self::Yft => "Fragment models",
            Self::Ydd => "Model dictionaries",
            Self::Ytd => "Texture dictionaries",
            Self::Ybn => "Collision",
            Self::Ymap => "Map placements",
            Self::Ytyp => "Archetype definitions",
            Self::Ycd => "Clip dictionaries",
            Self::Yed => "Expression dictionaries",
            Self::Yld => "Cloth",
            Self::Ypt => "Particle effects",
            Self::Ynv => "Navigation meshes",
            Self::Ynd => "Path nodes",
            Self::Yvr => "Vehicle recordings",
            Self::Ywr => "Waypoint recordings",
            Self::Ypdb => "Pose matchers",
            Self::Ymt => "Metadata tables",
            Self::Ymf => "Manifests",
            Self::Meta => "Metadata",
            Self::Xml => "XML files",
            Self::Dat => "Data files",
            Self::Awc => "Audio streams",
            Self::Rel => "Audio data",
            Self::Gxt2 => "Text tables",
            Self::Ysc => "Scripts",
            Self::Cut => "Cutscenes",
            Self::Gfx => "Interface movies",
            Self::Bik => "Videos",
            Self::Fxc => "Shaders",
            Self::Ttf => "Fonts",
            Self::Rpf => "Archives",
            Self::Nametable => "Name tables",
            Self::Dds => "Images",
            Self::Txt => "Text files",
        }
    }
    /// Broad content role, useful to a consumer without interpreting extension names.
    pub const fn role(self) -> &'static str {
        match self {
            Self::Ydr => "mesh",
            Self::Yft => "mesh",
            Self::Ydd => "mesh",
            Self::Ytd => "image",
            Self::Ybn => "collision",
            Self::Ymap => "map",
            Self::Ytyp => "map",
            Self::Ycd => "animation",
            Self::Yed => "other",
            Self::Yld => "other",
            Self::Ypt => "other",
            Self::Ynv => "map",
            Self::Ynd => "map",
            Self::Yvr => "animation",
            Self::Ywr => "animation",
            Self::Ypdb => "animation",
            Self::Ymt => "other",
            Self::Ymf => "other",
            Self::Meta => "other",
            Self::Xml => "other",
            Self::Dat => "other",
            Self::Awc => "other",
            Self::Rel => "other",
            Self::Gxt2 => "other",
            Self::Ysc => "other",
            Self::Cut => "animation",
            Self::Gfx => "other",
            Self::Bik => "other",
            Self::Fxc => "material",
            Self::Ttf => "other",
            Self::Rpf => "other",
            Self::Nametable => "other",
            Self::Dds => "image",
            Self::Txt => "other",
        }
    }
}
/// Classify a lowercase extension without its dot, as the archive listing provides it.
pub fn classify(extension: &str) -> Option<FileClass> {
    Some(match extension {
        "ydr" => FileClass::Ydr,
        "yft" => FileClass::Yft,
        "ydd" => FileClass::Ydd,
        "ytd" => FileClass::Ytd,
        "ybn" => FileClass::Ybn,
        "ymap" => FileClass::Ymap,
        "ytyp" => FileClass::Ytyp,
        "ycd" => FileClass::Ycd,
        "yed" => FileClass::Yed,
        "yld" => FileClass::Yld,
        "ypt" => FileClass::Ypt,
        "ynv" => FileClass::Ynv,
        "ynd" => FileClass::Ynd,
        "yvr" => FileClass::Yvr,
        "ywr" => FileClass::Ywr,
        "ypdb" => FileClass::Ypdb,
        "ymt" => FileClass::Ymt,
        "ymf" => FileClass::Ymf,
        "meta" => FileClass::Meta,
        "xml" => FileClass::Xml,
        "dat" => FileClass::Dat,
        "awc" => FileClass::Awc,
        "rel" => FileClass::Rel,
        "gxt2" => FileClass::Gxt2,
        "ysc" => FileClass::Ysc,
        "cut" => FileClass::Cut,
        "gfx" => FileClass::Gfx,
        "bik" => FileClass::Bik,
        "fxc" => FileClass::Fxc,
        "ttf" => FileClass::Ttf,
        "rpf" => FileClass::Rpf,
        "nametable" => FileClass::Nametable,
        "dds" | "png" | "jpg" | "bmp" => FileClass::Dds,
        "txt" => FileClass::Txt,
        _ => return None,
    })
}

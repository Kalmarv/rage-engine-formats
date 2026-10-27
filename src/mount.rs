//! The game's mount data: the pack list (`dlclist.xml`), each pack's `setup2.xml` and content file, extra folder
//! mounts and title-update mounts. Read strictly (every element must be one the game's files use); items tagged
//! for other platforms are left out (`platform="ps5|xbsx"`: the files they name are absent from PC installs).
use crate::xml::{Element, parse};
use crate::{Result, invalid};

/// The platform name items are matched against (`x64`, as `%PLATFORM%` expands on PC).
pub const PLATFORM: &str = "x64";

fn for_this_platform(e: &Element) -> bool {
    e.attr("platform")
        .is_none_or(|p| p.split('|').any(|x| x.trim().eq_ignore_ascii_case(PLATFORM)))
}

fn items(e: Option<&Element>) -> impl Iterator<Item = &Element> {
    e.into_iter()
        .flat_map(|e| e.items())
        .filter(|i| for_this_platform(i))
}

fn texts(e: Option<&Element>) -> Vec<String> {
    items(e)
        .map(|i| i.text.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect()
}

fn flag(e: &Element, name: &str, what: &str) -> Result<bool> {
    let Some(c) = e.child(name) else { return Ok(false) };
    match c.attr("value") {
        None => Ok(false),
        Some(v) if v.eq_ignore_ascii_case("true") => Ok(true),
        Some(v) if v.eq_ignore_ascii_case("false") => Ok(false),
        Some(v) => Err(invalid(
            "XML",
            c.offset,
            format!("{what}: <{name} value=\"{v}\">: expected true or false"),
        )),
    }
}

fn int(e: &Element, name: &str, what: &str) -> Result<i64> {
    let Some(c) = e.child(name) else { return Ok(0) };
    match c.attr("value") {
        None => Ok(0),
        Some(v) => v.trim().parse().map_err(|_| {
            invalid(
                "XML",
                c.offset,
                format!("{what}: <{name} value=\"{v}\">: expected an integer"),
            )
        }),
    }
}

/// `dlclist.xml`: the packs the game looks for, in list order, as `dlcpacks:/<name>/` paths (or `platform:/<folder>/`
/// for packs inside the base images), `/`-separated and lowercase. Any other kind of path is refused at its item.
pub fn pack_list(bytes: &[u8]) -> Result<Vec<String>> {
    let what = "dlclist.xml";
    let r = parse(bytes, what)?;
    r.only(what, &["Paths"])?;
    let paths = r.child("Paths");
    if let Some(p) = paths {
        p.only(what, &["Item"])?;
    }
    let mut out = Vec::new();
    for item in items(paths) {
        let written = item.text.trim();
        if written.is_empty() {
            continue;
        }
        let path = written.replace('\\', "/").to_lowercase();
        if !path.starts_with("dlcpacks:/") && !path.starts_with("platform:/") {
            return Err(invalid(
                "XML",
                item.offset,
                format!("{what}: pack path '{written}', expected dlcpacks:/<name>/ or platform:/<folder>/"),
            ));
        }
        out.push(path);
    }
    Ok(out)
}

/// A pack's `setup2.xml`: how the pack is mounted and which change-set groups it offers.
#[derive(Clone, Debug, Default)]
pub struct Setup {
    /// The device name the pack's files are mounted under (`dlcMPBeach`).
    pub device_name: String,
    /// The name of the pack's content file.
    pub dat_file: String,
    /// The pack's name as written (a hash name or a plain one).
    pub name_hash: String,
    /// The `type` element's text as written.
    pub kind: String,
    /// The pack's mount order; lower mounts first.
    pub order: i64,
    /// Breaks ties between packs of one `order`.
    pub minor_order: i64,
    /// How many subpack archives (`dlc1.rpf`, ...) the pack may have.
    pub sub_pack_count: i64,
    /// Whether the pack is a level pack.
    pub level_pack: bool,
    /// Change-set groups in order: group name, change set names.
    pub groups: Vec<(String, Vec<String>)>,
}

/// Reads a pack's `setup2.xml`; `what` names the file in error messages.
pub fn setup(bytes: &[u8], what: &str) -> Result<Setup> {
    let r = parse(bytes, what)?;
    r.only(
        what,
        &[
            "deviceName",
            "datFile",
            "timeStamp",
            "nameHash",
            "contentChangeSetGroups",
            "contentChangeSets",
            "type",
            "order",
            "minorOrder",
            "subPackCount",
            "isLevelPack",
            "dependencyPackHash",
            "requiredVersion",
            "scriptCallstackSize",
            "startupScript",
            "patchPackPath",
        ],
    )?;
    let mut groups = Vec::new();
    if let Some(g) = r.child("contentChangeSetGroups") {
        g.only(what, &["Item"])?;
        for item in items(Some(g)) {
            item.only(what, &["NameHash", "ContentChangeSets"])?;
            groups.push((
                item.child_text("NameHash").unwrap_or_default().to_string(),
                texts(item.child("ContentChangeSets")),
            ));
        }
    }
    Ok(Setup {
        device_name: r.child_text("deviceName").unwrap_or_default().to_string(),
        dat_file: r.child_text("datFile").unwrap_or_default().to_string(),
        name_hash: r.child_text("nameHash").unwrap_or_default().to_string(),
        kind: r.child_text("type").unwrap_or_default().to_string(),
        order: int(&r, "order", what)?,
        minor_order: int(&r, "minorOrder", what)?,
        sub_pack_count: int(&r, "subPackCount", what)?,
        level_pack: flag(&r, "isLevelPack", what)?,
        groups,
    })
}

/// One data file of a content file.
#[derive(Clone, Debug, Default)]
pub struct DataFile {
    /// The `filename` element as written, with its device prefix and `%PLATFORM%` placeholder unexpanded
    /// (`dlcMPHeist:/%PLATFORM%/levels/...rpf`).
    pub filename: String,
    /// The data file's type (`RPF_FILE`, `GTXD_PARENTING_DATA`, ...).
    pub file_type: String,
    /// Whether the file is an overlay: its files replace files of the same name whenever it is mounted.
    pub overlay: bool,
    /// Whether the file starts disabled (a change set enables it).
    pub disabled: bool,
}

/// A change to the map data, inside a change set.
#[derive(Clone, Debug, Default)]
pub struct MapChange {
    /// The `associatedMap` element: the name of the map the change applies to, as written.
    pub associated_map: String,
    /// The `filesToEnable` entries: data files the change enables, as written.
    pub files_to_enable: Vec<String>,
    /// The `filesToDisable` entries: data files the change disables, as written.
    pub files_to_disable: Vec<String>,
    /// Archives the change invalidates, with those overlaying them.
    pub files_to_invalidate: Vec<String>,
}

/// A content change set.
#[derive(Clone, Debug, Default)]
pub struct ChangeSet {
    /// The change set's name; several may share one, each for its own level.
    pub name: String,
    /// The `filesToEnable` entries: data files the change set enables, as written.
    pub files_to_enable: Vec<String>,
    /// The `filesToDisable` entries: data files the change set disables, as written.
    pub files_to_disable: Vec<String>,
    /// The `filesToInvalidate` entries: archives the change set invalidates, as written.
    pub files_to_invalidate: Vec<String>,
    /// The changes to the map data the change set makes.
    pub map_changes: Vec<MapChange>,
    /// `$level=<name>` from its execution conditions: the change set applies to that level only.
    pub level: Option<String>,
}

/// A pack's content file.
#[derive(Clone, Debug, Default)]
pub struct Content {
    /// The data files the pack defines, in document order.
    pub data_files: Vec<DataFile>,
    /// The change sets the pack defines, in document order.
    pub change_sets: Vec<ChangeSet>,
}

/// Reads a pack's content file (its data files and change sets); `what` names the file in error messages.
pub fn content(bytes: &[u8], what: &str) -> Result<Content> {
    let r = parse(bytes, what)?;
    r.only(
        what,
        &[
            "disabledFiles",
            "includedXmlFiles",
            "includedDataFiles",
            "dataFiles",
            "contentChangeSets",
            "patchFiles",
        ],
    )?;
    let mut data_files = Vec::new();
    if let Some(d) = r.child("dataFiles") {
        d.only(what, &["Item"])?;
        for item in items(Some(d)) {
            item.only(
                what,
                &[
                    "filename",
                    "fileType",
                    "overlay",
                    "disabled",
                    "persistent",
                    "locked",
                    "loadCompletely",
                    "contents",
                    "patchFile",
                ],
            )?;
            data_files.push(DataFile {
                filename: item.child_text("filename").unwrap_or_default().to_string(),
                file_type: item.child_text("fileType").unwrap_or_default().to_string(),
                overlay: flag(item, "overlay", what)?,
                disabled: flag(item, "disabled", what)?,
            });
        }
    }
    let mut change_sets = Vec::new();
    if let Some(c) = r.child("contentChangeSets") {
        c.only(what, &["Item"])?;
        for item in items(Some(c)) {
            item.only(
                what,
                &[
                    "changeSetName",
                    "associatedMap",
                    "executionConditions",
                    "filesToDisable",
                    "filesToEnable",
                    "filesToInvalidate",
                    "mapChangeSetData",
                    "residentResources",
                    "txdToLoad",
                    "txdToUnload",
                    "unregisterResources",
                    "loadingScreenContext",
                    "requiresLoadingScreen",
                    "useCacheLoader",
                ],
            )?;
            let mut level = None;
            if let Some(ec) = item.child("executionConditions") {
                ec.only(what, &["activeChangesetConditions", "genericConditions"])?;
                if ec
                    .child("activeChangesetConditions")
                    .is_some_and(|a| !a.children.is_empty() || !a.text.trim().is_empty())
                {
                    return Err(invalid(
                        "XML",
                        ec.offset,
                        format!(
                            "{what}: <activeChangesetConditions> holds conditions, expected it empty (they are not read)"
                        ),
                    ));
                }
                let g = ec.child_text("genericConditions").unwrap_or_default();
                if !g.is_empty() {
                    let at = ec.child("genericConditions").map_or(ec.offset, |c| c.offset);
                    level = Some(
                        g.strip_prefix("$level=")
                            .ok_or_else(|| {
                                invalid(
                                    "XML",
                                    at,
                                    format!("{what}: generic condition '{g}', expected '$level=<name>'"),
                                )
                            })?
                            .to_string(),
                    );
                }
            }
            let mut map_changes = Vec::new();
            if let Some(m) = item.child("mapChangeSetData") {
                m.only(what, &["Item"])?;
                for mi in items(Some(m)) {
                    mi.only(
                        what,
                        &[
                            "associatedMap",
                            "filesToDisable",
                            "filesToEnable",
                            "filesToInvalidate",
                            "residentResources",
                            "txdToLoad",
                            "txdToUnload",
                            "unregisterResources",
                        ],
                    )?;
                    map_changes.push(MapChange {
                        associated_map: mi.child_text("associatedMap").unwrap_or_default().to_string(),
                        files_to_enable: texts(mi.child("filesToEnable")),
                        files_to_disable: texts(mi.child("filesToDisable")),
                        files_to_invalidate: texts(mi.child("filesToInvalidate")),
                    });
                }
            }
            change_sets.push(ChangeSet {
                name: item.child_text("changeSetName").unwrap_or_default().to_string(),
                files_to_enable: texts(item.child("filesToEnable")),
                files_to_disable: texts(item.child("filesToDisable")),
                files_to_invalidate: texts(item.child("filesToInvalidate")),
                map_changes,
                level,
            });
        }
    }
    Ok(Content {
        data_files,
        change_sets,
    })
}

/// `extrafoldermount.xml`: folder mounts as (path, mounted as).
pub fn folder_mounts(bytes: &[u8], what: &str) -> Result<Vec<(String, String)>> {
    let r = parse(bytes, what)?;
    r.only(what, &["FolderMounts"])?;
    let mut out = Vec::new();
    if let Some(f) = r.child("FolderMounts") {
        f.only(what, &["Item"])?;
        for item in items(Some(f)) {
            item.only(what, &["path", "mountAs"])?;
            out.push((
                item.child_text("path").unwrap_or_default().to_string(),
                item.child_text("mountAs").unwrap_or_default().to_string(),
            ));
        }
    }
    Ok(out)
}

/// `extratitleupdatedata.meta`: title-update mounts as (device name, path).
pub fn title_update_mounts(bytes: &[u8], what: &str) -> Result<Vec<(String, String)>> {
    let r = parse(bytes, what)?;
    r.only(what, &["Mounts"])?;
    let mut out = Vec::new();
    if let Some(m) = r.child("Mounts") {
        m.only(what, &["Item"])?;
        for item in items(Some(m)) {
            item.only(what, &["deviceName", "path"])?;
            out.push((
                item.child_text("deviceName").unwrap_or_default().to_string(),
                item.child_text("path").unwrap_or_default().to_string(),
            ));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_content_platforms_and_conditions() {
        let s = br#"<?xml version="1.0"?><SSetupData><deviceName>dlcX</deviceName><datFile>content.xml</datFile>
            <order value="7"/><subPackCount value="2"/><contentChangeSetGroups><Item><NameHash>GROUP_MAP</NameHash>
            <ContentChangeSets><Item>CS_A</Item><Item platform="ps5|xbsx">CS_CONSOLE</Item><item platform="x64">CS_PC</item>
            </ContentChangeSets></Item></contentChangeSetGroups></SSetupData>"#;
        let su = setup(s, "setup2.xml").expect("setup");
        assert_eq!(
            (su.device_name.as_str(), su.order, su.sub_pack_count),
            ("dlcX", 7, 2)
        );
        assert_eq!(
            su.groups,
            vec![(
                "GROUP_MAP".to_string(),
                vec!["CS_A".to_string(), "CS_PC".to_string()]
            )]
        );
        let c = br#"<CDataFileMgr__ContentsOfDataFileXml><dataFiles><Item><filename>dlcX:/%PLATFORM%/a.rpf</filename>
            <fileType>RPF_FILE</fileType><overlay value="true"/><disabled value="true"/></Item></dataFiles>
            <contentChangeSets><Item><changeSetName>CS_A</changeSetName><filesToEnable><Item>dlcX:/%PLATFORM%/a.rpf</Item>
            </filesToEnable><executionConditions><activeChangesetConditions/><genericConditions>$level=MO_JIM_L11</genericConditions>
            </executionConditions></Item><Item><changeSetName>CS_A</changeSetName><mapChangeSetData><Item><associatedMap>m</associatedMap>
            <filesToInvalidate><Item>platform:/levels/gta5/b.rpf</Item></filesToInvalidate></Item></mapChangeSetData></Item>
            </contentChangeSets></CDataFileMgr__ContentsOfDataFileXml>"#;
        let co = content(c, "content.xml").expect("content");
        assert!(co.data_files[0].overlay && co.data_files[0].disabled);
        assert_eq!(co.change_sets.len(), 2);
        assert_eq!(co.change_sets[0].level.as_deref(), Some("MO_JIM_L11"));
        assert_eq!(
            co.change_sets[1].map_changes[0].files_to_invalidate,
            vec!["platform:/levels/gta5/b.rpf"]
        );
        // unknown elements are refused
        assert!(setup(b"<SSetupData><mystery/></SSetupData>", "s").is_err());
        assert!(content(b"<CDataFileMgr__ContentsOfDataFileXml><dataFiles><Item><what/></Item></dataFiles></CDataFileMgr__ContentsOfDataFileXml>", "c").is_err());
    }
}

use rage_engine_formats::{Error, mount, xml};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A DLC pack list (`dlclist.xml`): the packs the game looks for, in list order.
    let dlclist =
        br"<SMandatoryPacksData><Paths><Item>dlcpacks:\mpbeach\</Item></Paths></SMandatoryPacksData>";
    println!("{:?}", mount::pack_list(dlclist)?);

    // Malformed input is an error that says what was wrong, never a panic.
    let Err(refused) = xml::parse(b"<a><b></a>", "example.xml") else {
        unreachable!("the closing tag does not match")
    };
    assert!(matches!(refused, Error::Invalid { format: "XML", .. }));
    println!("{refused}");
    Ok(())
}

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use xml::reader::{EventReader, XmlEvent};
use xml::attribute::OwnedAttribute;

#[derive(Debug)]
pub struct Tile {
    pub name: String,
}

pub struct AutoLayout {

}

pub struct FixedLayout {

}

pub enum Layout {
    AutoLayout(AutoLayout),
    FixedLayout(FixedLayout),
}

pub struct DeviceInfo {

}

pub struct Switch {

}

pub struct Segment {

}

pub struct PBType {
    pub name: String,
}

pub struct FPGAArch {
    pub tiles: Vec<Tile>,
    pub layouts: Vec<Layout>,
    pub device: DeviceInfo,
    pub switch_list: Vec<Switch>,
    pub segment_list: Vec<Segment>,
    pub complex_block_list: Vec<PBType>,
}

fn parse_tile(_name: &str,
              attributes: &Vec<OwnedAttribute>,
              parser: &mut EventReader<BufReader<File>>) -> Tile {

    // TODO: Verify the name and attributes are expected.

    let mut tile_name = String::new();
    for a in attributes {
        match a.name.to_string().as_str() {
            "name" => {
                tile_name = a.value.clone();
            },
            _ => {},
        };
    }

    let new_tile = Tile {
        name: tile_name,
    };

    // Skip the contents of the tile for now.
    // TODO: Add error check here.
    let _ = parser.skip();

    return new_tile;
}

fn parse_tiles(_name: &str,
               _attributes: &Vec<OwnedAttribute>,
               parser: &mut EventReader<BufReader<File>>) -> Vec<Tile> {
    // TODO: Error check the name and attributes to ensure that they are corrrect.

    // Iterate over the parser until we reach the EndElement for tile.
    let mut tiles: Vec<Tile> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "tile" => {
                        tiles.push(parse_tile(&name.to_string(), &attributes, parser));
                    },
                    _ => {},
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                if name.to_string() == "tiles" {
                    break;
                }
            },
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            },
            // TODO: Handle the other cases.
            _ => {},
        }
    };

    return tiles;
}

pub fn parse(arch_file: &Path) -> std::io::Result<FPGAArch> {
    let file = File::open(arch_file)?;
    // Buffering is used for performance.
    let file = BufReader::new(file);

    let mut tiles: Vec<Tile> = Vec::new();

    // TODO: We should ignore comments and maybe whitespace.
    let mut parser = EventReader::new(file);

    // TODO: We should check that the first tag is the architecture tag.

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "models" => {
                        // TODO: Implement.
                        let _ = parser.skip();
                    },
                    "tiles" => {
                        // TODO: Need to check that we do not see multiple tiles tags.
                        tiles = parse_tiles(&name.to_string(), &attributes, &mut parser);
                    },
                    "layout" => {
                        // TODO: Implement.
                        let _ = parser.skip();
                    },
                    "device" => {
                        // TODO: Implement.
                        let _ = parser.skip();
                    },
                    "switchlist" => {
                        // TODO: Implement.
                        let _ = parser.skip();
                    },
                    "segmentlist" => {
                        // TODO: Implement.
                        let _ = parser.skip();
                    },
                    "complexblocklist" => {
                        // TODO: Implement.
                        let _ = parser.skip();
                    },
                    _ => {
                        // TODO: Raise an error here if a tag is found that is
                        //       not of the above types.
                    },
                };
            },
            Ok(XmlEvent::EndElement { name: _ }) => {
                // TODO: We should never see an end element if the sub-parsers
                //       are doing their job. This would imply that there is a
                //       problem.
                //       The only end element we should see is the architecture
                //       tag.
            },
            Ok(XmlEvent::EndDocument) => {
                break;
            },
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            },
            // There's more: https://docs.rs/xml/latest/xml/reader/enum.XmlEvent.html
            _ => {},
        };
    }

    println!("{:?}", tiles);

    return Ok(FPGAArch {
        tiles: tiles,
        layouts: Vec::new(),
        device: DeviceInfo {},
        switch_list: Vec::new(),
        segment_list: Vec::new(),
        complex_block_list: Vec::new(),
    });
}

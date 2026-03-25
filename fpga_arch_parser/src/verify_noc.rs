use xml::common::TextPosition;

use crate::{FPGAArchParseError, NoCInfo, Tile};

pub fn verify_noc(
    noc: &NoCInfo,
    tiles: &[Tile],
    position: TextPosition,
) -> Result<(), FPGAArchParseError> {
    // Verify that the noc router tile name exists.
    if !tiles
        .iter()
        .any(|tile| tile.name == noc.noc_router_tile_name)
    {
        return Err(FPGAArchParseError::InvalidTag(
            format!("Unknown NoC router tile: {}", noc.noc_router_tile_name),
            position,
        ));
    }

    Ok(())
}

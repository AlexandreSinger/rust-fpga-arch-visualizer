use std::{collections::HashMap, ops::RangeInclusive};

use crate::{
    FPGAArchParseError,
    arch::{PinLoc, PinSide, Port, SubTile, SubTilePinLocations},
};

type TilePinIndexMap = HashMap<String, Vec<HashMap<String, Vec<usize>>>>;

// TODO: This should be consolidated with PinLoc.
#[derive(Clone)]
pub struct PhysicalPinLoc {
    pub side: PinSide,
    pub xoffset: usize,
    pub yoffset: usize,
}

impl Default for PhysicalPinLoc {
    fn default() -> Self {
        Self {
            side: PinSide::Left,
            xoffset: 0,
            yoffset: 0,
        }
    }
}

pub struct TilePinMapper {
    pub num_pins_in_tile: usize,
    // [sub_tile_name][sub_tile_cap_index][port_bus_name][port_index] -> pin_index
    pub pin_index_lookup: TilePinIndexMap,

    pub pin_name_lookup: Vec<String>,

    // Note: A pin may be on multiple sides.
    pub pin_locs: Vec<Vec<PhysicalPinLoc>>,
}

impl TilePinMapper {
    pub fn parse_pin_name(&self, pin_name: &str) -> Result<Vec<usize>, String> {
        let split_pin_string: Vec<&str> = pin_name.split(".").collect();
        // Expect there to only be 2.
        // <sub_tile_name>([{bus}])?.<sub_tile_port>([{bus}])?
        if split_pin_string.len() != 2 {
            return Err(
                "Invalid pin string, expected to be of the form '<sub_tile_name>.<sub_tile_port>'."
                    .to_string(),
            );
        }
        let sub_tile_portion = split_pin_string[0];
        let port_portion = split_pin_string[1];

        // Split the sub-tile name and the bus.
        let (sub_tile, sub_tile_bus_slice) =
            split_bus_name(sub_tile_portion).map_err(|e| format!("{:?}", e))?;
        // Get the sub-tile lookup. We will use this to get the capacity of the sub-tile.
        let sub_tile_lookup = match self.pin_index_lookup.get(sub_tile) {
            Some(l) => l,
            None => return Err(format!("Could not find sub-tile for pin: {}", sub_tile)),
        };
        let sub_tile_capacity = sub_tile_lookup.len() as i32;
        // Parse the bus.
        let sub_tile_bus = match sub_tile_bus_slice {
            Some(bus_slice) => parse_bus(bus_slice).map_err(|e| format!("{:?}", e))?,
            None => 0..=(sub_tile_capacity - 1),
        };

        // Split the port name from the bus
        let (port_name, port_bus_slice) =
            split_bus_name(port_portion).map_err(|e| format!("{:?}", e))?;
        // Get the number of pins in the port.
        // Note: Here we assume that each sub-tile with the same name has the same ports.
        //       This is currently guaranteed by the architecture description.
        let num_port_pins = match sub_tile_lookup[*sub_tile_bus.start() as usize].get(port_name) {
            Some(lookup) => lookup.len() as i32,
            None => return Err(format!("Unable to find port with name: {}", port_name)),
        };
        // Parse the bus.
        let port_bus = match port_bus_slice {
            Some(bus_slice) => parse_bus(bus_slice).map_err(|e| format!("{:?}", e))?,
            None => 0..=(num_port_pins - 1),
        };

        // Get the pins.
        let mut pins: Vec<usize> = Vec::new();
        for sub_tile_cap_index in sub_tile_bus {
            if sub_tile_cap_index < 0 || sub_tile_cap_index >= sub_tile_capacity {
                return Err("Invalid sub tile index.".to_string());
            }
            let sub_tile_pin_index_lookup =
                &sub_tile_lookup[sub_tile_cap_index as usize][port_name];
            for bit in port_bus.clone() {
                if bit < 0 || bit >= num_port_pins {
                    return Err("Invalid port bit position.".to_string());
                }
                pins.push(sub_tile_pin_index_lookup[bit as usize]);
            }
        }

        Ok(pins)
    }
}

pub fn build_tile_pin_mapper(
    sub_tiles: &Vec<SubTile>,
    tile_width: usize,
    tile_height: usize,
) -> Result<TilePinMapper, FPGAArchParseError> {
    let mut num_pins_in_tile: usize = 0;
    let mut pin_index_lookup: TilePinIndexMap = HashMap::new();
    let mut pin_name_lookup: Vec<String> = Vec::new();
    for sub_tile in sub_tiles {
        let mut sub_tile_pin_lookup = Vec::new();
        for sub_tile_cap_index in 0..sub_tile.capacity {
            let mut port_name_pin_lookup = HashMap::new();
            for port in &sub_tile.ports {
                let (port_name, num_pins) = match port {
                    Port::Input(input_port) => (&input_port.name, input_port.num_pins),
                    Port::Output(output_port) => (&output_port.name, output_port.num_pins),
                    Port::Clock(clock_port) => (&clock_port.name, clock_port.num_pins),
                };
                let num_pins = num_pins as usize;
                let mut pin_indices = Vec::new();
                for pin_index in num_pins_in_tile..num_pins_in_tile + num_pins {
                    pin_indices.push(pin_index);
                    let sub_tile_name = if sub_tile.capacity > 1 {
                        format!("{}[{}]", sub_tile.name, sub_tile_cap_index)
                    } else {
                        sub_tile.name.clone()
                    };
                    let pin_port_name = if num_pins > 1 {
                        format!("{}[{}]", port_name, pin_index - num_pins_in_tile)
                    } else {
                        port_name.clone()
                    };
                    pin_name_lookup.push(format!("{}.{}", sub_tile_name, pin_port_name));
                }
                num_pins_in_tile += num_pins;
                if port_name_pin_lookup.contains_key(port_name) {
                    return Err(FPGAArchParseError::PinParsingError(format!(
                        "Found duplicate port name: {}",
                        port_name
                    )));
                }
                port_name_pin_lookup.insert(port_name.clone(), pin_indices);
            }
            sub_tile_pin_lookup.push(port_name_pin_lookup);
        }
        if pin_index_lookup.contains_key(&sub_tile.name) {
            return Err(FPGAArchParseError::PinParsingError(format!(
                "Found duplicate port name: {}",
                sub_tile.name
            )));
        }
        pin_index_lookup.insert(sub_tile.name.clone(), sub_tile_pin_lookup);
    }

    let mut pin_locs: Vec<Vec<PhysicalPinLoc>> = vec![Vec::new(); num_pins_in_tile];
    for sub_tile in sub_tiles {
        match &sub_tile.pin_locations {
            SubTilePinLocations::Custom(custom_pin_locations) => {
                for loc in &custom_pin_locations.pin_locations {
                    let pins = get_pins_in_pin_loc(loc, sub_tile, &pin_index_lookup)?;
                    let pin_loc = PhysicalPinLoc {
                        side: loc.side.clone(),
                        xoffset: loc.xoffset,
                        yoffset: loc.yoffset,
                    };
                    for pin in pins {
                        pin_locs[pin].push(pin_loc.clone());
                    }
                }
            }
            SubTilePinLocations::Spread => {
                let slots = spread_slots(tile_width, tile_height);
                assign_pins_round_robin(sub_tile, &pin_index_lookup, &slots, &mut pin_locs);
            }
            SubTilePinLocations::Perimeter => {
                let slots = perimeter_slots(tile_width, tile_height);
                assign_pins_round_robin(sub_tile, &pin_index_lookup, &slots, &mut pin_locs);
            }
            SubTilePinLocations::SpreadInputsPerimeterOutputs => {
                let spread = spread_slots(tile_width, tile_height);
                let perimeter = perimeter_slots(tile_width, tile_height);
                assign_pins_spread_inputs_perimeter_outputs(
                    sub_tile,
                    &pin_index_lookup,
                    &spread,
                    &perimeter,
                    &mut pin_locs,
                );
            }
        }
    }

    Ok(TilePinMapper {
        num_pins_in_tile,
        pin_index_lookup,
        pin_name_lookup,
        pin_locs,
    })
}

fn spread_slots(tile_width: usize, tile_height: usize) -> Vec<PhysicalPinLoc> {
    let mut slots = Vec::new();
    for side in [PinSide::Left, PinSide::Right, PinSide::Bottom, PinSide::Top] {
        for yoffset in 0..tile_height {
            for xoffset in 0..tile_width {
                slots.push(PhysicalPinLoc { side: side.clone(), xoffset, yoffset });
            }
        }
    }
    slots
}

fn perimeter_slots(tile_width: usize, tile_height: usize) -> Vec<PhysicalPinLoc> {
    let mut slots = Vec::new();
    for side in [PinSide::Left, PinSide::Right, PinSide::Bottom, PinSide::Top] {
        for yoffset in 0..tile_height {
            for xoffset in 0..tile_width {
                let on_perimeter = match side {
                    PinSide::Left => xoffset == 0,
                    PinSide::Right => xoffset == tile_width - 1,
                    PinSide::Bottom => yoffset == 0,
                    PinSide::Top => yoffset == tile_height - 1,
                };
                if on_perimeter {
                    slots.push(PhysicalPinLoc { side: side.clone(), xoffset, yoffset });
                }
            }
        }
    }
    slots
}

fn assign_pins_round_robin(
    sub_tile: &SubTile,
    pin_index_lookup: &TilePinIndexMap,
    slots: &[PhysicalPinLoc],
    pin_locs: &mut Vec<Vec<PhysicalPinLoc>>,
) {
    if slots.is_empty() {
        return;
    }
    let sub_tile_lookup = &pin_index_lookup[&sub_tile.name];
    for cap_lookup in sub_tile_lookup {
        for port in &sub_tile.ports {
            let port_name = match port {
                Port::Input(p) => &p.name,
                Port::Output(p) => &p.name,
                Port::Clock(p) => &p.name,
            };
            for &pin_idx in &cap_lookup[port_name] {
                pin_locs[pin_idx].push(slots[pin_idx % slots.len()].clone());
            }
        }
    }
}

fn assign_pins_spread_inputs_perimeter_outputs(
    sub_tile: &SubTile,
    pin_index_lookup: &TilePinIndexMap,
    spread: &[PhysicalPinLoc],
    perimeter: &[PhysicalPinLoc],
    pin_locs: &mut Vec<Vec<PhysicalPinLoc>>,
) {
    let sub_tile_lookup = &pin_index_lookup[&sub_tile.name];
    for cap_lookup in sub_tile_lookup {
        for port in &sub_tile.ports {
            let (port_name, slots): (&String, &[PhysicalPinLoc]) = match port {
                Port::Input(p) => (&p.name, spread),
                Port::Output(p) => (&p.name, perimeter),
                Port::Clock(p) => (&p.name, spread),
            };
            if slots.is_empty() {
                continue;
            }
            for &pin_idx in &cap_lookup[port_name] {
                pin_locs[pin_idx].push(slots[pin_idx % slots.len()].clone());
            }
        }
    }
}

fn get_pins_in_pin_loc(
    loc: &PinLoc,
    sub_tile: &SubTile,
    pin_index_lookup: &TilePinIndexMap,
) -> Result<Vec<usize>, FPGAArchParseError> {
    let mut pins: Vec<usize> = Vec::new();
    for pin_string in &loc.pin_strings {
        let split_pin_string: Vec<&str> = pin_string.split(".").collect();
        // Expect there to only be 2.
        // <sub_tile_name>([{bus}])?.<sub_tile_port>([{bus}])?
        if split_pin_string.len() != 2 {
            return Err(FPGAArchParseError::PinParsingError(
                "Invalid pin string, expected to be of the form '<sub_tile_name>.<sub_tile_port>'."
                    .to_string(),
            ));
        }
        let sub_tile_portion = split_pin_string[0];
        let port_portion = split_pin_string[1];

        // Parse the sub-tile portion.
        let (sub_tile_name, sub_tile_bus_slice) = split_bus_name(sub_tile_portion)?;
        if sub_tile_name != sub_tile.name {
            return Err(FPGAArchParseError::PinParsingError(
                "Invalid pin string, does not start with the correct sub-tile.".to_string(),
            ));
        }
        let sub_tile_bus = match sub_tile_bus_slice {
            Some(bus_slice) => parse_bus(bus_slice)?,
            None => 0..=(sub_tile.capacity - 1),
        };

        // Parse port portion.
        let (port_name, port_bus_slice) = split_bus_name(port_portion)?;
        // TODO: We can make this lookup much faster by having a lookup between [sub-tile][port] -> num_pins
        let port = sub_tile.ports.iter().find(|&port| {
            let other_port_name = match &port {
                Port::Input(input_port) => &input_port.name,
                Port::Output(output_port) => &output_port.name,
                Port::Clock(clock_port) => &clock_port.name,
            };
            other_port_name == port_name
        });
        let port = match port {
            Some(p) => p,
            None => {
                return Err(FPGAArchParseError::PinParsingError(
                    "Cannot find port in pin string".to_string(),
                ));
            }
        };
        let num_port_pins = match &port {
            Port::Input(input_port) => input_port.num_pins,
            Port::Output(output_port) => output_port.num_pins,
            Port::Clock(clock_port) => clock_port.num_pins,
        };
        let port_bus = match port_bus_slice {
            Some(bus_slice) => parse_bus(bus_slice)?,
            None => 0..=(num_port_pins - 1),
        };

        // Get the pins.
        for sub_tile_cap_index in sub_tile_bus {
            if sub_tile_cap_index < 0 || sub_tile_cap_index >= sub_tile.capacity {
                return Err(FPGAArchParseError::PinParsingError(
                    "Invalid sub tile index.".to_string(),
                ));
            }
            let sub_tile_pin_index_lookup =
                &pin_index_lookup[sub_tile_name][sub_tile_cap_index as usize][port_name];
            for bit in port_bus.clone() {
                if bit < 0 || bit >= num_port_pins {
                    return Err(FPGAArchParseError::PinParsingError(
                        "Invalid port bit position.".to_string(),
                    ));
                }
                pins.push(sub_tile_pin_index_lookup[bit as usize]);
            }
        }
    }

    Ok(pins)
}

fn split_bus_name(s: &str) -> Result<(&str, Option<&str>), FPGAArchParseError> {
    if let Some(idx) = s.find('[') {
        let (name, slice) = s.split_at(idx);

        if !slice.ends_with(']') {
            return Err(FPGAArchParseError::PinParsingError(format!(
                "Invalid bus slice: {}",
                s
            )));
        }

        Ok((name, Some(slice)))
    } else {
        Ok((s, None))
    }
}

fn parse_bus(bus: &str) -> Result<RangeInclusive<i32>, FPGAArchParseError> {
    if !bus.starts_with('[') || !bus.ends_with(']') {
        return Err(FPGAArchParseError::PinParsingError(format!(
            "Invalid bus format: {}",
            bus
        )));
    }

    let inner = &bus[1..bus.len() - 1];

    if let Some((a, b)) = inner.split_once(':') {
        let msb: i32 = a
            .trim()
            .parse()
            .map_err(|_| FPGAArchParseError::PinParsingError("Invalid number".to_string()))?;
        let lsb: i32 = b
            .trim()
            .parse()
            .map_err(|_| FPGAArchParseError::PinParsingError("Invalid number".to_string()))?;
        Ok(msb.min(lsb)..=msb.max(lsb))
    } else {
        let bit: i32 = inner
            .trim()
            .parse()
            .map_err(|_| FPGAArchParseError::PinParsingError("Invalid number".to_string()))?;
        Ok(bit..=bit)
    }
}

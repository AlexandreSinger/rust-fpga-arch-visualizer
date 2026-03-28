use std::collections::HashSet;

#[cfg(not(target_arch = "wasm32"))]
use yaml_rust2::YamlLoader;

pub struct SBMaps {
    patterns: Vec<SBMapPattern>,

    unique_file_names: HashSet<String>,
}

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
#[derive(Debug, PartialEq)]
pub enum SBPatternVal {
    Constant {
        val: usize,
    },
    Wildcard,
    List {
        vals: Vec<usize>,
    },
    Range {
        start: usize,
        end: usize,
        step: usize,
    },
}

pub struct SBPattern {
    x_pattern: SBPatternVal,
    y_pattern: SBPatternVal,
}

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
#[derive(Debug, PartialEq)]
pub enum SBMapTemplate {
    File { file_name: String },
    Null,
}

pub struct SBMapPattern {
    pub pattern: SBPattern,
    pub template: SBMapTemplate,
}

impl SBMaps {
    pub fn get_sb_template(&self, x: usize, y: usize) -> Option<&SBMapTemplate> {
        // Return the first pattern match.
        for pattern in &self.patterns {
            if check_for_pattern_match(&pattern.pattern, x, y) {
                return Some(&pattern.template);
            }
        }

        // If none are matched, return none.
        None
    }

    pub fn get_unique_file_names(&self) -> &HashSet<String> {
        &self.unique_file_names
    }
}

fn check_for_pattern_match(pattern: &SBPattern, x: usize, y: usize) -> bool {
    check_for_val_pattern_match(&pattern.x_pattern, x)
        && check_for_val_pattern_match(&pattern.y_pattern, y)
}

fn check_for_val_pattern_match(pattern: &SBPatternVal, v: usize) -> bool {
    match pattern {
        // If wildcard, always return true.
        SBPatternVal::Wildcard => true,
        // If constant, return true if the value matches.
        SBPatternVal::Constant { val } => v == *val,
        // If list, return true if the list contains the value.
        SBPatternVal::List { vals } => vals.contains(&v),
        // For range, return true if the value is within the range.
        SBPatternVal::Range { start, end, step } => {
            if v > *end || v < *start {
                // Return false if out of range.
                false
            } else {
                // Return true if hits one of the step points.
                (v - *start).is_multiple_of(*step)
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn parse_sb_maps_yaml_from_string(sb_maps_str: &str) -> Result<SBMaps, String> {
    let docs = YamlLoader::load_from_str(sb_maps_str);
    let docs = match docs {
        Ok(d) => d,
        Err(e) => return Err(format!("{e}")),
    };

    if docs.len() != 1 {
        return Err("YAML file has more than one document. Something went wrong.".to_string());
    }

    let doc = match docs.first() {
        Some(v) => v,
        None => return Err("YAML Docs are empty. Something went wrong.".to_string()),
    };

    // Verify the top-level hash.
    if let Some(top_level_hash) = doc.as_hash() {
        if top_level_hash.len() != 1 {
            return Err("Top-level hash expected to only contain the key 'SB_MAPS'".to_string());
        }
    } else {
        return Err("Top-level YAML object expected to be a hash.".to_string());
    }

    let sb_patterns_yaml = &doc["SB_MAPS"];
    if sb_patterns_yaml.is_badvalue() {
        return Err("Missing top-level key 'SB_MAPS'.".to_string());
    }

    let mut patterns: Vec<SBMapPattern> = Vec::new();
    let mut unique_file_names: HashSet<String> = HashSet::new();
    if let Some(sb_patterns_hash) = sb_patterns_yaml.as_hash() {
        for (key, value) in sb_patterns_hash.iter() {
            let sb_pattern_string = match key.as_str() {
                Some(v) => v,
                None => return Err("SB pattern expected to be a string.".to_string()),
            };
            let pattern = parse_sb_pattern(sb_pattern_string)?;
            let sb_template = match value {
                yaml_rust2::Yaml::String(str) => {
                    unique_file_names.insert(str.clone());
                    SBMapTemplate::File {
                        file_name: str.clone(),
                    }
                }
                yaml_rust2::Yaml::Null => SBMapTemplate::Null,
                _ => return Err("SB template expected to be a string or null.".to_string()),
            };
            patterns.push(SBMapPattern {
                pattern,
                template: sb_template,
            });
        }
    } else {
        return Err("SB_MAPS YAML object expected to be a hash.".to_string());
    }

    Ok(SBMaps {
        patterns,
        unique_file_names,
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_sb_pattern(sb_pattern_string: &str) -> Result<SBPattern, String> {
    if !sb_pattern_string.starts_with("SB_") {
        return Err(format!(
            "SB pattern should start with 'SB_'. Pattern found: {}",
            sb_pattern_string
        ));
    }
    if !sb_pattern_string.ends_with("_") {
        return Err(format!(
            "SB pattern should end with '_'. Pattern found: {}",
            sb_pattern_string
        ));
    }
    let sb_pattern_string = &sb_pattern_string[3..(sb_pattern_string.len() - 1)];

    let split_pattern: Vec<&str> = sb_pattern_string.split("__").collect();

    if split_pattern.len() != 2 {
        return Err(format!(
            "SB pattern should have two pattern dimensions. Pattern found: {}",
            sb_pattern_string
        ));
    }

    let x_pattern = parse_sb_pattern_val(split_pattern[0])?;
    let y_pattern = parse_sb_pattern_val(split_pattern[1])?;

    Ok(SBPattern {
        x_pattern,
        y_pattern,
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_sb_pattern_val(pattern_str: &str) -> Result<SBPatternVal, String> {
    // If this looks like a wildcard, it is a wildcard.
    // TODO: Talk to Amin about the raw wildcard being supported.
    if pattern_str == "\\*" || pattern_str == "*" {
        return Ok(SBPatternVal::Wildcard);
    }

    // If it can be parsed as an integer, it is a constant.
    if let Ok(val) = pattern_str.parse::<usize>() {
        return Ok(SBPatternVal::Constant { val });
    }

    // If it has a comma in it, we assume it is a list.
    if pattern_str.contains(",") {
        let list_vals: Vec<&str> = pattern_str.split(",").collect();
        let mut vals: Vec<usize> = Vec::new();
        for list_val in list_vals {
            let val = match list_val.parse::<usize>() {
                Ok(v) => v,
                Err(e) => return Err(format!("Error parsing list element: {e}")),
            };
            vals.push(val);
        }

        return Ok(SBPatternVal::List { vals });
    }

    // If it has colons in it and starts and ends with brackets, we assume it is a range.
    if pattern_str.contains(":") && pattern_str.starts_with("[") && pattern_str.ends_with("]") {
        // Remove the brackets.
        let no_brackets_str = &pattern_str[1..(pattern_str.len() - 1)];
        // Split the colons.
        let range_vals: Vec<&str> = no_brackets_str.split(":").collect();
        if range_vals.len() != 3 {
            return Err(format!(
                "Range expected to have 3 elements: {}",
                pattern_str
            ));
        }
        // Parse the start, step, and end.
        let start = match range_vals[0].parse::<usize>() {
            Ok(v) => v,
            Err(e) => {
                return Err(format!(
                    "Error parsing range start element in pattern {pattern_str}: {e}"
                ));
            }
        };
        let end = match range_vals[1].parse::<usize>() {
            Ok(v) => v,
            Err(e) => {
                return Err(format!(
                    "Error parsing range end element in pattern {pattern_str}: {e}"
                ));
            }
        };
        let step = match range_vals[2].parse::<usize>() {
            Ok(v) => v,
            Err(e) => {
                return Err(format!(
                    "Error parsing range step element in pattern {pattern_str}: {e}"
                ));
            }
        };
        if step == 0 {
            return Err(format!(
                "Range step must be greater than 0 in pattern {pattern_str}"
            ));
        }
        if start > end {
            return Err(format!(
                "Range start must be less than or equal to end in pattern {pattern_str}"
            ));
        }

        return Ok(SBPatternVal::Range { start, end, step });
    }

    // If none of the above matched, return an error.
    Err(format!("Could not parse SB pattern: {}", pattern_str))
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn doc_example() -> Result<(), String> {
        let s = "
SB_MAPS:
   SB_0__*_: null
   SB_*__0_: null
   SB_1__*_: sb_perimeter.csv
   SB_*__1_: sb_perimeter.csv
   SB_[2:10:2]__*_: sb_dsp.csv
   SB_*__*_: sb_main.csv
";
        let sb_maps = parse_sb_maps_yaml_from_string(s)?;

        assert_eq!(sb_maps.patterns.len(), 6);

        assert_eq!(
            sb_maps.patterns[0].pattern.x_pattern,
            SBPatternVal::Constant { val: 0 }
        );
        assert_eq!(
            sb_maps.patterns[0].pattern.y_pattern,
            SBPatternVal::Wildcard
        );
        assert_eq!(sb_maps.patterns[0].template, SBMapTemplate::Null);

        assert_eq!(
            sb_maps.patterns[1].pattern.x_pattern,
            SBPatternVal::Wildcard
        );
        assert_eq!(
            sb_maps.patterns[1].pattern.y_pattern,
            SBPatternVal::Constant { val: 0 }
        );
        assert_eq!(sb_maps.patterns[1].template, SBMapTemplate::Null);

        assert_eq!(
            sb_maps.patterns[2].pattern.x_pattern,
            SBPatternVal::Constant { val: 1 }
        );
        assert_eq!(
            sb_maps.patterns[2].pattern.y_pattern,
            SBPatternVal::Wildcard
        );
        assert_eq!(
            sb_maps.patterns[2].template,
            SBMapTemplate::File {
                file_name: "sb_perimeter.csv".to_string()
            }
        );

        assert_eq!(
            sb_maps.patterns[3].pattern.x_pattern,
            SBPatternVal::Wildcard
        );
        assert_eq!(
            sb_maps.patterns[3].pattern.y_pattern,
            SBPatternVal::Constant { val: 1 }
        );
        assert_eq!(
            sb_maps.patterns[3].template,
            SBMapTemplate::File {
                file_name: "sb_perimeter.csv".to_string()
            }
        );

        assert_eq!(
            sb_maps.patterns[4].pattern.x_pattern,
            SBPatternVal::Range {
                start: 2,
                end: 10,
                step: 2
            }
        );
        assert_eq!(
            sb_maps.patterns[4].pattern.y_pattern,
            SBPatternVal::Wildcard
        );
        assert_eq!(
            sb_maps.patterns[4].template,
            SBMapTemplate::File {
                file_name: "sb_dsp.csv".to_string()
            }
        );

        assert_eq!(
            sb_maps.patterns[5].pattern.x_pattern,
            SBPatternVal::Wildcard
        );
        assert_eq!(
            sb_maps.patterns[5].pattern.y_pattern,
            SBPatternVal::Wildcard
        );
        assert_eq!(
            sb_maps.patterns[5].template,
            SBMapTemplate::File {
                file_name: "sb_main.csv".to_string()
            }
        );

        // From the first 2 templates, these rows should be null.
        for i in 0..10 {
            assert_eq!(
                *sb_maps
                    .get_sb_template(i, 0)
                    .expect("template should match"),
                SBMapTemplate::Null
            );
            assert_eq!(
                *sb_maps
                    .get_sb_template(0, i)
                    .expect("template should match"),
                SBMapTemplate::Null
            );
        }
        // From the following 2 templates, these should be perimeter.
        for i in 1..10 {
            assert_eq!(
                *sb_maps
                    .get_sb_template(i, 1)
                    .expect("template should match"),
                SBMapTemplate::File {
                    file_name: "sb_perimeter.csv".to_string()
                }
            );
            assert_eq!(
                *sb_maps
                    .get_sb_template(1, i)
                    .expect("template should match"),
                SBMapTemplate::File {
                    file_name: "sb_perimeter.csv".to_string()
                }
            );
        }

        for i in 2..10 {
            for j in (2..=10).step_by(2) {
                // From the dsp pattern.
                assert_eq!(
                    *sb_maps
                        .get_sb_template(j, i)
                        .expect("template should match"),
                    SBMapTemplate::File {
                        file_name: "sb_dsp.csv".to_string()
                    }
                );
                // Everywhere else.
                assert_eq!(
                    *sb_maps
                        .get_sb_template(j + 1, i)
                        .expect("template should match"),
                    SBMapTemplate::File {
                        file_name: "sb_main.csv".to_string()
                    }
                );
            }
        }

        // Check that the end step is working for the dsp.
        for i in 2..10 {
            assert_eq!(
                *sb_maps
                    .get_sb_template(12, i)
                    .expect("template should match"),
                SBMapTemplate::File {
                    file_name: "sb_main.csv".to_string()
                }
            );
        }

        // Check the unique file names all exist.
        let unique_file_names = sb_maps.get_unique_file_names();
        assert_eq!(unique_file_names.len(), 3);
        assert!(unique_file_names.contains("sb_perimeter.csv"));
        assert!(unique_file_names.contains("sb_dsp.csv"));
        assert!(unique_file_names.contains("sb_main.csv"));

        Ok(())
    }

    #[test]
    fn sample_example() -> Result<(), String> {
        let s = "
SB_MAPS:
  # ==================================================
  # Corners
  # ==================================================
  SB_0__0_: null
  SB_0__41_: null
  SB_41__0_: null
  SB_41__41_: null
  # ==================================================
  # IO Switchboxes
  # ==================================================
  SB_\\*__0_: sb_io.csv
  SB_0__\\*_: sb_io.csv
  SB_41__\\*_: sb_io.csv
  SB_\\*__41_: sb_io.csv
  # ==================================================
  # DSP Related Column 1
  # ==================================================
  SB_[6:41:8]__[1:41:4]_: sb_mult_36_0.csv
  SB_[6:41:8]__[2:41:4]_: sb_mult_36_1.csv
  SB_[6:41:8]__[3:41:4]_: sb_mult_36_2.csv
  SB_[6:41:8]__[4:41:4]_: sb_mult_36_3.csv
  # ==================================================
  # BRAM Related
  # ==================================================
  SB_[2:41:8]__[1:41:6]_: sb_memory_0.csv
  SB_[2:41:8]__[2:41:6]_: sb_memory_1.csv
  SB_[2:41:8]__[3:41:6]_: sb_memory_2.csv
  SB_[2:41:8]__[4:41:6]_: sb_memory_3.csv
  SB_[2:41:8]__[5:41:6]_: sb_memory_4.csv
  SB_[2:41:8]__[6:41:6]_: sb_memory_5.csv
  # ==================================================
  SB_\\*__\\*_: sb_main.csv
";
        let sb_maps = parse_sb_maps_yaml_from_string(s)?;

        assert_eq!(sb_maps.patterns.len(), 19);

        // Validate corner patterns
        assert_eq!(
            sb_maps.patterns[0].pattern.x_pattern,
            SBPatternVal::Constant { val: 0 }
        );
        assert_eq!(
            sb_maps.patterns[0].pattern.y_pattern,
            SBPatternVal::Constant { val: 0 }
        );
        assert_eq!(sb_maps.patterns[0].template, SBMapTemplate::Null);

        assert_eq!(
            sb_maps.patterns[1].pattern.x_pattern,
            SBPatternVal::Constant { val: 0 }
        );
        assert_eq!(
            sb_maps.patterns[1].pattern.y_pattern,
            SBPatternVal::Constant { val: 41 }
        );
        assert_eq!(sb_maps.patterns[1].template, SBMapTemplate::Null);

        assert_eq!(
            sb_maps.patterns[2].pattern.x_pattern,
            SBPatternVal::Constant { val: 41 }
        );
        assert_eq!(
            sb_maps.patterns[2].pattern.y_pattern,
            SBPatternVal::Constant { val: 0 }
        );
        assert_eq!(sb_maps.patterns[2].template, SBMapTemplate::Null);

        assert_eq!(
            sb_maps.patterns[3].pattern.x_pattern,
            SBPatternVal::Constant { val: 41 }
        );
        assert_eq!(
            sb_maps.patterns[3].pattern.y_pattern,
            SBPatternVal::Constant { val: 41 }
        );
        assert_eq!(sb_maps.patterns[3].template, SBMapTemplate::Null);

        // Validate IO edge patterns
        assert_eq!(
            sb_maps.patterns[4].pattern.x_pattern,
            SBPatternVal::Wildcard
        );
        assert_eq!(
            sb_maps.patterns[4].pattern.y_pattern,
            SBPatternVal::Constant { val: 0 }
        );
        assert_eq!(
            sb_maps.patterns[4].template,
            SBMapTemplate::File {
                file_name: "sb_io.csv".to_string()
            }
        );

        assert_eq!(
            sb_maps.patterns[5].pattern.x_pattern,
            SBPatternVal::Constant { val: 0 }
        );
        assert_eq!(
            sb_maps.patterns[5].pattern.y_pattern,
            SBPatternVal::Wildcard
        );
        assert_eq!(
            sb_maps.patterns[5].template,
            SBMapTemplate::File {
                file_name: "sb_io.csv".to_string()
            }
        );

        assert_eq!(
            sb_maps.patterns[6].pattern.x_pattern,
            SBPatternVal::Constant { val: 41 }
        );
        assert_eq!(
            sb_maps.patterns[6].pattern.y_pattern,
            SBPatternVal::Wildcard
        );
        assert_eq!(
            sb_maps.patterns[6].template,
            SBMapTemplate::File {
                file_name: "sb_io.csv".to_string()
            }
        );

        assert_eq!(
            sb_maps.patterns[7].pattern.x_pattern,
            SBPatternVal::Wildcard
        );
        assert_eq!(
            sb_maps.patterns[7].pattern.y_pattern,
            SBPatternVal::Constant { val: 41 }
        );
        assert_eq!(
            sb_maps.patterns[7].template,
            SBMapTemplate::File {
                file_name: "sb_io.csv".to_string()
            }
        );

        // Validate DSP patterns
        for i in 0..4 {
            assert_eq!(
                sb_maps.patterns[8 + i].pattern.x_pattern,
                SBPatternVal::Range {
                    start: 6,
                    end: 41,
                    step: 8
                }
            );
            assert_eq!(
                sb_maps.patterns[8 + i].pattern.y_pattern,
                SBPatternVal::Range {
                    start: (1 + i),
                    end: 41,
                    step: 4
                }
            );
            assert_eq!(
                sb_maps.patterns[8 + i].template,
                SBMapTemplate::File {
                    file_name: format!("sb_mult_36_{}.csv", i)
                }
            );
        }

        // Validate BRAM patterns
        for i in 0..6 {
            assert_eq!(
                sb_maps.patterns[12 + i].pattern.x_pattern,
                SBPatternVal::Range {
                    start: 2,
                    end: 41,
                    step: 8
                }
            );
            assert_eq!(
                sb_maps.patterns[12 + i].pattern.y_pattern,
                SBPatternVal::Range {
                    start: (1 + i),
                    end: 41,
                    step: 6
                }
            );
            assert_eq!(
                sb_maps.patterns[12 + i].template,
                SBMapTemplate::File {
                    file_name: format!("sb_memory_{}.csv", i)
                }
            );
        }

        // Validate catch-all pattern
        assert_eq!(
            sb_maps.patterns[18].pattern.x_pattern,
            SBPatternVal::Wildcard
        );
        assert_eq!(
            sb_maps.patterns[18].pattern.y_pattern,
            SBPatternVal::Wildcard
        );
        assert_eq!(
            sb_maps.patterns[18].template,
            SBMapTemplate::File {
                file_name: "sb_main.csv".to_string()
            }
        );

        // Test get_sb_template for a 41x41 device
        // Corners should return null
        assert_eq!(
            *sb_maps
                .get_sb_template(0, 0)
                .expect("template should match"),
            SBMapTemplate::Null
        );
        assert_eq!(
            *sb_maps
                .get_sb_template(0, 41)
                .expect("template should match"),
            SBMapTemplate::Null
        );
        assert_eq!(
            *sb_maps
                .get_sb_template(41, 0)
                .expect("template should match"),
            SBMapTemplate::Null
        );
        assert_eq!(
            *sb_maps
                .get_sb_template(41, 41)
                .expect("template should match"),
            SBMapTemplate::Null
        );

        // IO edges should return sb_io.csv
        assert_eq!(
            *sb_maps
                .get_sb_template(5, 0)
                .expect("template should match"),
            SBMapTemplate::File {
                file_name: "sb_io.csv".to_string()
            }
        );
        assert_eq!(
            *sb_maps
                .get_sb_template(0, 5)
                .expect("template should match"),
            SBMapTemplate::File {
                file_name: "sb_io.csv".to_string()
            }
        );
        assert_eq!(
            *sb_maps
                .get_sb_template(41, 20)
                .expect("template should match"),
            SBMapTemplate::File {
                file_name: "sb_io.csv".to_string()
            }
        );
        assert_eq!(
            *sb_maps
                .get_sb_template(15, 41)
                .expect("template should match"),
            SBMapTemplate::File {
                file_name: "sb_io.csv".to_string()
            }
        );

        // DSP positions should return sb_mult files
        for x in (6..=40).step_by(8) {
            for y in (1..=40).step_by(4) {
                assert_eq!(
                    *sb_maps
                        .get_sb_template(x, y)
                        .expect("template should match"),
                    SBMapTemplate::File {
                        file_name: "sb_mult_36_0.csv".to_string()
                    }
                );
                assert_eq!(
                    *sb_maps
                        .get_sb_template(x, y + 1)
                        .expect("template should match"),
                    SBMapTemplate::File {
                        file_name: "sb_mult_36_1.csv".to_string()
                    }
                );
                assert_eq!(
                    *sb_maps
                        .get_sb_template(x, y + 2)
                        .expect("template should match"),
                    SBMapTemplate::File {
                        file_name: "sb_mult_36_2.csv".to_string()
                    }
                );
                assert_eq!(
                    *sb_maps
                        .get_sb_template(x, y + 3)
                        .expect("template should match"),
                    SBMapTemplate::File {
                        file_name: "sb_mult_36_3.csv".to_string()
                    }
                );
            }
        }

        // BRAM positions should return sb_memory files
        for x in (2..=40).step_by(8) {
            for y in (1..=31).step_by(6) {
                assert_eq!(
                    *sb_maps
                        .get_sb_template(x, y)
                        .expect("template should match"),
                    SBMapTemplate::File {
                        file_name: "sb_memory_0.csv".to_string()
                    }
                );
                assert_eq!(
                    *sb_maps
                        .get_sb_template(x, y + 1)
                        .expect("template should match"),
                    SBMapTemplate::File {
                        file_name: "sb_memory_1.csv".to_string()
                    }
                );
                assert_eq!(
                    *sb_maps
                        .get_sb_template(x, y + 2)
                        .expect("template should match"),
                    SBMapTemplate::File {
                        file_name: "sb_memory_2.csv".to_string()
                    }
                );
                assert_eq!(
                    *sb_maps
                        .get_sb_template(x, y + 3)
                        .expect("template should match"),
                    SBMapTemplate::File {
                        file_name: "sb_memory_3.csv".to_string()
                    }
                );
                assert_eq!(
                    *sb_maps
                        .get_sb_template(x, y + 4)
                        .expect("template should match"),
                    SBMapTemplate::File {
                        file_name: "sb_memory_4.csv".to_string()
                    }
                );
                assert_eq!(
                    *sb_maps
                        .get_sb_template(x, y + 5)
                        .expect("template should match"),
                    SBMapTemplate::File {
                        file_name: "sb_memory_5.csv".to_string()
                    }
                );
            }
        }

        // Regular interior positions should return sb_main.csv (catch-all)
        assert_eq!(
            *sb_maps
                .get_sb_template(5, 5)
                .expect("template should match"),
            SBMapTemplate::File {
                file_name: "sb_main.csv".to_string()
            }
        );
        assert_eq!(
            *sb_maps
                .get_sb_template(15, 20)
                .expect("template should match"),
            SBMapTemplate::File {
                file_name: "sb_main.csv".to_string()
            }
        );
        assert_eq!(
            *sb_maps
                .get_sb_template(35, 35)
                .expect("template should match"),
            SBMapTemplate::File {
                file_name: "sb_main.csv".to_string()
            }
        );

        // Check the unique file names all exist.
        let unique_file_names = sb_maps.get_unique_file_names();
        assert_eq!(unique_file_names.len(), 12);
        assert!(unique_file_names.contains("sb_io.csv"));
        assert!(unique_file_names.contains("sb_mult_36_0.csv"));
        assert!(unique_file_names.contains("sb_mult_36_1.csv"));
        assert!(unique_file_names.contains("sb_mult_36_2.csv"));
        assert!(unique_file_names.contains("sb_mult_36_3.csv"));
        assert!(unique_file_names.contains("sb_memory_0.csv"));
        assert!(unique_file_names.contains("sb_memory_1.csv"));
        assert!(unique_file_names.contains("sb_memory_2.csv"));
        assert!(unique_file_names.contains("sb_memory_3.csv"));
        assert!(unique_file_names.contains("sb_memory_4.csv"));
        assert!(unique_file_names.contains("sb_memory_5.csv"));
        assert!(unique_file_names.contains("sb_main.csv"));

        Ok(())
    }
}

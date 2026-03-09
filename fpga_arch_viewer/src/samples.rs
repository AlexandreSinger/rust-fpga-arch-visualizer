//! Sample architectures embedded in the binary for users to explore without loading files

pub struct SampleArchitecture {
    pub name: &'static str,
    pub data: &'static [u8],
}

impl SampleArchitecture {
    pub fn all() -> &'static [Self] {
        static SAMPLES: &[SampleArchitecture] = &[
            SampleArchitecture {
                name: "MCNC Architecture",
                data: include_bytes!("../../fpga_arch_parser/tests/k6_frac_N10_40nm.xml"),
            },
            SampleArchitecture {
                name: "VTR Flagship Architecture",
                data: include_bytes!(
                    "../../fpga_arch_parser/tests/k6_frac_N10_frac_chain_mem32K_40nm.xml"
                ),
            },
            SampleArchitecture {
                name: "Koios Architecture",
                data: include_bytes!(
                    "../../fpga_arch_parser/tests/k6FracN10LB_mem20K_complexDSP_customSB_22nm.xml"
                ),
            },
        ];
        SAMPLES
    }
}

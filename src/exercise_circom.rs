use anyhow::{Ok, Result};

use std::path::Path;
use std::{ffi::OsStr, io::Write};

use crossterm::style::Stylize;

use crate::path::change_extension_with_suffix;
use crate::{
    cmd::WasmWitnessCmd,
    cmd_snarkjs::SnarkjsCmd,
    path::{append_compiled_folder, change_extension},
};

pub struct CircomExercise<'a> {
    pub circuit_dir: &'a Path,
    pub circuit_file: &'a OsStr,
    pub ptau: &'a str,
}

impl<'a> CircomExercise<'a> {
    pub fn generate_witness(&self, output: &mut Vec<u8>) -> Result<bool> {
        writeln!(output, "{}", "Computing witness...".underlined())?;

        let input_file = change_extension(self.circuit_file, "json")
            .display()
            .to_string();
        let input_file_dir = format!("{:?}/{}", "../", input_file).replace('"', "");
        let compiled_folder: String = self
            .circuit_file
            .to_str()
            .unwrap()
            .replace(".circom", "_js");
        let compiled_dir = append_compiled_folder(self.circuit_dir, &compiled_folder);
        let wasm_file = change_extension(self.circuit_file, "wasm")
            .display()
            .to_string();

        let mut generate_witness_cmd = WasmWitnessCmd {
            args: &[&wasm_file, &input_file_dir, "witness.wtns"],
            description: "Computing witness",
            output,
            compiled_dir: &compiled_dir,
        };

        generate_witness_cmd.run()
    }

    // "powersoftau new bn128 9 pot9_0000.ptau -v"
    pub fn start_ceremony(&self, output: &mut Vec<u8>) -> Result<bool> {
        writeln!(output, "{}", "Start ceremony...")?;

        let mut start_ceremony_cmd = SnarkjsCmd {
            pot_dir: &self.circuit_dir,
            args: &[
                "powersoftau",
                "new",
                "bn128",
                "9",
                &self.contribution_file_0(),
                "-v",
            ],

            description: "Start power of tau ceremony",
            output,
        };

        start_ceremony_cmd.run()
    }

    pub fn contribute_ceremony(
        &self,
        output: &mut Vec<u8>,
        // contribute_in_file_name: &Path,
        // contribute_out_file_name: &Path,
    ) -> Result<bool> {
        writeln!(output, "{}", "Contribute to the ceremony...")?;

        // Create ceremony with skipping entropy input
        let mut contribute_ceremony_cmd = SnarkjsCmd {
            pot_dir: &self.circuit_dir,
            args: &[
                "ptc",
                &self.contribution_file_0(),
                &self.contribution_file_1(),
                "-v",
                "-e",
            ],
            description: "First contribution",
            output,
        };

        contribute_ceremony_cmd.run()
    }

    pub fn prepare_circuit_proof(&self, output: &mut Vec<u8>) -> Result<bool> {
        writeln!(output, "{}", "Prepare circuit...")?;

        let mut prepare_circuit_proof_cmd = SnarkjsCmd {
            pot_dir: &self.circuit_dir,
            args: &[
                "pt2",
                &self.contribution_file_1(),
                &self.contribution_file_final(),
                "-v",
                "-e",
            ],
            description: "Prepare circuit-specific",
            output,
        };

        prepare_circuit_proof_cmd.run()
    }

    pub fn create_z_key(&self, output: &mut Vec<u8>) -> Result<bool> {
        writeln!(output, "{}", "Create .zkey ...")?;

        let r1cs_file = change_extension(&self.circuit_file, "r1cs")
            .display()
            .to_string();
        let z_key_file_name: String =
            change_extension_with_suffix(&self.circuit_file, "_0000", "zkey")
                .display()
                .to_string();

        let mut create_z_key_cmd = SnarkjsCmd {
            pot_dir: &self.circuit_dir,
            args: &[
                "groth16",
                "setup",
                &r1cs_file,
                "pot9_final.ptau",
                &z_key_file_name,
            ],
            description: "Create .zkey",
            output,
        };

        create_z_key_cmd.run()
    }

    pub fn contribute_z_key(&self, output: &mut Vec<u8>) -> Result<bool> {
        writeln!(output, "{}", "Contribute .zkey ...")?;

        let z_key_in_file_name = change_extension_with_suffix(&self.circuit_file, "_0000", "zkey");
        let z_key_out_file_name = change_extension_with_suffix(&self.circuit_file, "_0001", "zkey");

        let mut contribute_z_key_cmd = SnarkjsCmd {
            pot_dir: &self.circuit_dir,
            args: &[
                "zkc",
                &z_key_in_file_name.as_path().display().to_string(),
                &z_key_out_file_name.as_path().display().to_string(),
                "-v",
                "-e",
            ],
            description: "Create .zkey",
            output,
        };

        let contribute_z_key_cmd_success = contribute_z_key_cmd.run()?;

        if !contribute_z_key_cmd_success {
            return Ok(false);
        }

        Ok(true)
    }

    pub fn export_verification_key(&self, output: &mut Vec<u8>) -> Result<bool> {
        writeln!(output, "{}", "Export verification key...")?;

        let z_key_in_file_name = change_extension_with_suffix(&self.circuit_file, "_0001", "zkey");
        let json_key_file_name =
            change_extension_with_suffix(&self.circuit_file, "_verification", "json");

        let mut contribute_z_key_cmd = SnarkjsCmd {
            pot_dir: &self.circuit_dir,
            args: &[
                "zkev",
                &z_key_in_file_name.as_path().display().to_string(),
                &json_key_file_name.as_path().display().to_string(),
            ],
            description: "Export json verification key",
            output,
        };

        contribute_z_key_cmd.run()
    }

    pub fn generate_proof(&self, output: &mut Vec<u8>) -> Result<bool> {
        writeln!(output, "{}", "Generating proof...")?;

        let compiled_folder = &self
            .circuit_file
            .to_str()
            .unwrap()
            .replace(".circom", "_js");
        let witness_folder_with_name = format!("{}/{}", compiled_folder, "witness.wtns");

        let z_key_in_file_name = change_extension_with_suffix(&self.circuit_file, "_0001", "zkey");
        let proof_file_name = change_extension_with_suffix(&self.circuit_file, "_prove", "json");
        let json_key_file_name = change_extension_with_suffix(&self.circuit_file, "_out", "json");

        let mut generate_proof_cmd = SnarkjsCmd {
            pot_dir: &self.circuit_dir,
            args: &[
                "g16p",
                &z_key_in_file_name.as_path().display().to_string(),
                &witness_folder_with_name,
                &proof_file_name.as_path().display().to_string(),
                &json_key_file_name.as_path().display().to_string(),
            ],
            description: "Generate proof file",
            output,
        };

        generate_proof_cmd.run()
    }

    pub fn verify_proof(&self, output: &mut Vec<u8>) -> Result<bool> {
        writeln!(output, "{}", "Verifying proof...")?;

        let verification_key_file_name =
            change_extension_with_suffix(&self.circuit_file, "_verification", "json");
        let public_key_file_name = change_extension_with_suffix(&self.circuit_file, "_out", "json");
        let proof_file_name = change_extension_with_suffix(&self.circuit_file, "_prove", "json");

        let mut verify_proof_cmd = SnarkjsCmd {
            pot_dir: &self.circuit_dir,
            args: &[
                "g16v",
                &verification_key_file_name.display().to_string(),
                &public_key_file_name.display().to_string(),
                &proof_file_name.display().to_string(),
            ],
            description: "Verify proof",
            output,
        };

        verify_proof_cmd.run()
    }

    fn contribution_file_0(&self) -> String {
        self.contribution_file_name("_0000")
    }

    fn contribution_file_1(&self) -> String {
        self.contribution_file_name("_0001")
    }

    fn contribution_file_final(&self) -> String {
        self.contribution_file_name("_final")
    }

    fn contribution_file_name(&self, suffix: &str) -> String {
        // "pot" + &self.ptau.to_owned() + suffix + ".ptau"
        format!("pot{}{}.ptau", &self.ptau, &suffix)
    }
}

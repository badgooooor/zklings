use anyhow::Result;

use std::path::Path;
use std::{ffi::OsStr, io::Write};

use crossterm::style::Stylize;

use crate::{
    cmd::WasmWitnessCmd,
    cmd_snarkjs::SnarkjsCmd,
    path::{append_compiled_folder, change_extension},
};

pub fn generate_witness(
    output: &mut Vec<u8>,
    circuit_dir: &Path,
    circuit_file: &OsStr,
) -> Result<bool> {
    writeln!(output, "{}", "Computing witness...".underlined())?;

    let input_file = change_extension(circuit_file, "json").display().to_string();
    let input_file_dir = format!("{:?}/{}", "../", input_file).replace('"', "");
    let compiled_folder = circuit_file.to_str().unwrap().replace(".circom", "_js");
    let compiled_dir = append_compiled_folder(circuit_dir, &compiled_folder);
    let wasm_file = change_extension(circuit_file, "wasm").display().to_string();

    let mut generate_witness_cmd = WasmWitnessCmd {
        args: &[&wasm_file, &input_file_dir, "witness.wtns"],
        description: "Computing witness",
        output,
        compiled_dir: &compiled_dir,
    };

    let generate_witness_success = generate_witness_cmd.run()?;

    if !generate_witness_success {
        return Ok(false);
    }

    Ok(generate_witness_success)
}

// "powersoftau new bn128 12 pot12_0000.ptau -v"
pub fn start_ceremony(output: &mut Vec<u8>, pot_dir: &Path) -> Result<bool> {
    writeln!(output, "{}", "Start ceremony...")?;

    let mut start_ceremony_cmd = SnarkjsCmd {
        pot_dir,
        args: &["powersoftau", "new", "bn128", "12", "pot12_0000.ptau", "-v"],
        description: "Start power of tau ceremony",
        output,
    };

    let start_ceremony_success = start_ceremony_cmd.run()?;

    if !start_ceremony_success {
        return Ok(false);
    }

    Ok(start_ceremony_success)
}

pub fn contribute_ceremony(output: &mut Vec<u8>, pot_dir: &Path) -> Result<bool> {
    writeln!(output, "{}", "Contribute to the ceremony")?;

    // Create ceremony with skipping entropy input
    let mut contribute_ceremony_cmd = SnarkjsCmd {
        pot_dir,
        args: &[
            "poweroftau",
            "contribute",
            "pot12_0000.ptau",
            "pot12_0001.ptau",
            "--name=\"First contribution\"",
            "-v",
            "-e",
        ],
        description: "First contribution",
        output,
    };

    let contribute_ceremony_success = contribute_ceremony_cmd.run()?;

    if !contribute_ceremony_success {
        return Ok(false);
    }

    Ok(true)
}

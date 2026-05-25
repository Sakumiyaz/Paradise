#![allow(dead_code)]
#![allow(non_snake_case)]
// ============================================================================
// EDEN Seed Generator - Deterministic Seed from Intent
// ============================================================================
//
// This binary generates a deterministic 128-byte seed from a user-provided
// string (phrase, name, intention). It uses a slow hash function for
// key derivation and analyzes the input string to adjust cosmic constants.
//
// Usage:
//   eden_seed_generator "Mi universo personal" [--output ~/.eden/config.toml]
//
// Output: A 128-byte seed file and a configuration file with adjusted
// constants that can be read by the main EDEN binary.
//
// ============================================================================

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// CONSTANTS
// ============================================================================

/// Default output directory
const DEFAULT_EDEN_DIR: &str = ".eden";
/// Default seed filename
const DEFAULT_SEED_FILE: &str = "seed.toml";
/// Default config filename
const DEFAULT_CONFIG_FILE: &str = "config.toml";
/// Number of hash iterations (slow derivation)
const HASH_ITERATIONS: u32 = 100_000;

/// Trait keywords and their corresponding adjustments
const CAOTICO_KEYWORDS: &[&str] = &["caotico", "caótico", "chaos", "random", "turbio"];
const PACIFICO_KEYWORDS: &[&str] = &[
    "pacifico",
    "pacífico",
    "peace",
    "calm",
    "tranquilo",
    "sereno",
];
const DIVERSO_KEYWORDS: &[&str] = &["diverso", "diversity", "variado", "muchos", "abundante"];
const OSCURO_KEYWORDS: &[&str] = &["oscuro", "dark", "siniestro", "shadow", "noche"];
const LUMINOSO_KEYWORDS: &[&str] = &["luminoso", "light", "brillante", "claro", "dia", "sol"];
const VITAL_KEYWORDS: &[&str] = &["vital", "alive", "vivo", "vida", "energia", "fuerte"];
const FRAGIL_KEYWORDS: &[&str] = &["fragil", "frágil", "weak", "delicado"];

// ============================================================================
// TRAIT ANALYSIS
// ============================================================================

/// Result of analyzing the input string for traits
#[derive(Debug, Clone, Default)]
struct AnalisisTraits {
    caotico: f64,
    pacifico: f64,
    diverso: f64,
    oscuro: f64,
    luminoso: f64,
    vital: f64,
    fragil: f64,
}

impl AnalisisTraits {
    fn analizar(input: &str) -> Self {
        let input_lower = input.to_lowercase();
        let mut traits = AnalisisTraits::default();

        for kw in CAOTICO_KEYWORDS {
            if input_lower.contains(kw) {
                traits.caotico += 0.25;
            }
        }
        for kw in PACIFICO_KEYWORDS {
            if input_lower.contains(kw) {
                traits.pacifico += 0.25;
            }
        }
        for kw in DIVERSO_KEYWORDS {
            if input_lower.contains(kw) {
                traits.diverso += 0.25;
            }
        }
        for kw in OSCURO_KEYWORDS {
            if input_lower.contains(kw) {
                traits.oscuro += 0.25;
            }
        }
        for kw in LUMINOSO_KEYWORDS {
            if input_lower.contains(kw) {
                traits.luminoso += 0.25;
            }
        }
        for kw in VITAL_KEYWORDS {
            if input_lower.contains(kw) {
                traits.vital += 0.25;
            }
        }
        for kw in FRAGIL_KEYWORDS {
            if input_lower.contains(kw) {
                traits.fragil += 0.25;
            }
        }

        traits.caotico = traits.caotico.min(1.0);
        traits.pacifico = traits.pacifico.min(1.0);
        traits.diverso = traits.diverso.min(1.0);
        traits.oscuro = traits.oscuro.min(1.0);
        traits.luminoso = traits.luminoso.min(1.0);
        traits.vital = traits.vital.min(1.0);
        traits.fragil = traits.fragil.min(1.0);

        traits
    }

    fn resumen(&self) -> String {
        let mut parts = Vec::new();
        if self.caotico > 0.5 {
            parts.push(format!("caotic({:.0}%)", self.caotico * 100.0));
        }
        if self.pacifico > 0.5 {
            parts.push(format!("pacific({:.0}%)", self.pacifico * 100.0));
        }
        if self.diverso > 0.5 {
            parts.push(format!("diverse({:.0}%)", self.diverso * 100.0));
        }
        if self.oscuro > 0.5 {
            parts.push(format!("dark({:.0}%)", self.oscuro * 100.0));
        }
        if self.luminoso > 0.5 {
            parts.push(format!("luminous({:.0}%)", self.luminoso * 100.0));
        }
        if self.vital > 0.5 {
            parts.push(format!("vital({:.0}%)", self.vital * 100.0));
        }
        if self.fragil > 0.5 {
            parts.push(format!("fragile({:.0}%)", self.fragil * 100.0));
        }
        if parts.is_empty() {
            "balanced".to_string()
        } else {
            parts.join(", ")
        }
    }
}

// ============================================================================
// SEED GENERATION (Slow Hash)
// ============================================================================

/// Simple slow hash function using multiple rounds of mixing
fn slow_hash(input: &[u8], salt: &[u8], iterations: u32) -> [u8; 128] {
    let mut result = [0u8; 128];

    // Combine input and salt
    let mut combined = Vec::with_capacity(input.len() + salt.len());
    combined.extend_from_slice(input);
    combined.extend_from_slice(salt);

    // Initial state based on golden ratio
    let mut state: [u64; 4] = [
        0x9e3779b97f4a7c15u64,
        0xbf09f4a3c7e3b8f0u64,
        0x1f9c0b7c3e7d2a1fu64,
        0x8c4f5d3e7b9a2c1fu64,
    ];

    // Mix input into state
    for (i, &byte) in input.iter().enumerate() {
        state[i % 4] = state[i % 4]
            .wrapping_add(byte as u64)
            .wrapping_mul(0x9e3779b97f4a7c15u64);
    }

    // Multiple rounds of mixing
    for iter in 0..iterations {
        state[0] = state[0].wrapping_add(iter as u64);

        let mix: u64 = state[0] ^ state[1] ^ state[2] ^ state[3];

        for i in 0..4 {
            state[i] = state[i]
                .wrapping_add(mix)
                .wrapping_mul(0x9e3779b97f4a7c15u64);
            state[i] ^= state[i] >> 41;
        }

        // Mix in the combined input periodically
        if iter % 1000 == 0 {
            for (j, &byte) in combined.iter().enumerate() {
                state[(j + iter as usize) % 4] = state[(j + iter as usize) % 4]
                    .wrapping_add(byte as u64)
                    .wrapping_mul(0x9e3779b97f4a7c15u64);
            }
        }
    }

    // Convert state to bytes (128 bytes output)
    for i in 0..16 {
        let val = state[i % 4];
        result[i * 8] = val as u8;
        result[i * 8 + 1] = (val >> 8) as u8;
        result[i * 8 + 2] = (val >> 16) as u8;
        result[i * 8 + 3] = (val >> 24) as u8;
        result[i * 8 + 4] = (val >> 32) as u8;
        result[i * 8 + 5] = (val >> 40) as u8;
        result[i * 8 + 6] = (val >> 48) as u8;
        result[i * 8 + 7] = (val >> 56) as u8;
    }

    // Additional mixing pass
    for i in 0..128 {
        let idx = (state[(i / 32) % 4] % 128) as usize;
        result[i] = result[i] ^ result[idx];
        result[i] = result[i]
            .wrapping_add(0x9e as u8)
            .wrapping_mul((i as u8).wrapping_add(1));
    }

    result
}

/// Generate a 128-byte seed from input string using slow hash
fn generar_seed(input: &str, sal: &[u8]) -> [u8; 128] {
    slow_hash(input.as_bytes(), sal, HASH_ITERATIONS)
}

/// Convert 128-byte seed to hexadecimal string (256 hex chars)
fn seed_a_hex(seed: &[u8; 128]) -> String {
    seed.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Parse hex string back to seed
fn hex_a_seed(hex: &str) -> Result<[u8; 128], Box<dyn std::error::Error>> {
    if hex.len() != 256 {
        return Err("Hex string must be 256 characters".into());
    }
    let mut seed = [0u8; 128];
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        let s = std::str::from_utf8(chunk)?;
        seed[i] = u8::from_str_radix(s, 16)?;
    }
    Ok(seed)
}

// ============================================================================
// CONSTANT ADJUSTMENTS
// ============================================================================

#[derive(Debug, Clone)]
struct ConstantesAjustadas {
    tau1_mult: [f64; 3],
    tau2_mult: [f64; 3],
    tau3_mult: [f64; 3],
    difusion_base: f64,
    decaimiento: f64,
}

impl Default for ConstantesAjustadas {
    fn default() -> Self {
        Self {
            tau1_mult: [1.0, 1.0, 1.0],
            tau2_mult: [1.0, 1.0, 1.0],
            tau3_mult: [1.0, 1.0, 1.0],
            difusion_base: 0.5,
            decaimiento: 0.1,
        }
    }
}

impl ConstantesAjustadas {
    fn desde_traits(traits: &AnalisisTraits) -> Self {
        let mut c = ConstantesAjustadas::default();

        if traits.caotico > 0.0 {
            let factor = 1.0 + traits.caotico * 0.5;
            c.tau1_mult = [factor; 3];
            c.difusion_base *= 1.0 + traits.caotico * 0.3;
        }

        if traits.pacifico > 0.0 {
            let factor = 1.0 - traits.pacifico * 0.3;
            c.tau1_mult = [factor; 3];
            c.difusion_base *= 1.0 - traits.pacifico * 0.2;
        }

        if traits.diverso > 0.0 {
            let factor = 1.0 + traits.diverso * 0.2;
            c.tau2_mult = [factor; 3];
        }

        if traits.oscuro > 0.0 {
            c.decaimiento *= 1.0 + traits.oscuro * 0.4;
        }

        if traits.luminoso > 0.0 {
            c.decaimiento *= 1.0 - traits.luminoso * 0.3;
        }

        if traits.vital > 0.0 {
            for i in 0..3 {
                c.tau1_mult[i] *= 1.0 + traits.vital * 0.2;
                c.tau3_mult[i] *= 1.0 + traits.vital * 0.15;
            }
        }

        if traits.fragil > 0.0 {
            for i in 0..3 {
                c.tau1_mult[i] *= 1.0 - traits.fragil * 0.25;
                c.tau3_mult[i] *= 1.0 - traits.fragil * 0.2;
            }
        }

        for val in &mut c.tau1_mult {
            *val = (*val).max(0.1).min(3.0);
        }
        for val in &mut c.tau2_mult {
            *val = (*val).max(0.1).min(3.0);
        }
        for val in &mut c.tau3_mult {
            *val = (*val).max(0.1).min(3.0);
        }
        c.difusion_base = c.difusion_base.max(0.1).min(2.0);
        c.decaimiento = c.decaimiento.max(0.01).min(0.9);

        c
    }
}

// ============================================================================
// OUTPUT GENERATION
// ============================================================================

fn generar_seed_toml(seed: &[u8; 128], nombre: &str, traits: &AnalisisTraits) -> String {
    let hex = seed_a_hex(seed);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    format!(
        r#"# EDEN Seed File
# Generated: {}
# Name: "{}"
# Traits: {}

[seed]
hex = "{}"

[metadata]
nombre = "{}"
timestamp = {}
traits.caotico = {:.2}
traits.pacifico = {:.2}
traits.diverso = {:.2}
traits.oscuro = {:.2}
traits.luminoso = {:.2}
traits.vital = {:.2}
traits.fragil = {:.2}
"#,
        timestamp,
        nombre,
        traits.resumen(),
        hex,
        nombre,
        timestamp,
        traits.caotico,
        traits.pacifico,
        traits.diverso,
        traits.oscuro,
        traits.luminoso,
        traits.vital,
        traits.fragil,
    )
}

fn generar_config(
    seed: &[u8; 128],
    nombre: &str,
    traits: &AnalisisTraits,
    constantes: &ConstantesAjustadas,
) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let derive_u32 = |bytes: &[u8], start: usize| -> u32 {
        let mut val: u32 = 0;
        for i in 0..4 {
            val = val.wrapping_add(bytes[(start + i) % 128] as u32);
        }
        val
    };

    let tau1_base = derive_u32(seed, 0) as f64 / u32::MAX as f64;
    let tau2_base = derive_u32(seed, 16) as f64 / u32::MAX as f64;
    let tau3_base = derive_u32(seed, 32) as f64 / u32::MAX as f64;
    let dif_base = derive_u32(seed, 48) as f64 / u32::MAX as f64;
    let dec_base = derive_u32(seed, 64) as f64 / u32::MAX as f64;

    let tau1 = [
        tau1_base * constantes.tau1_mult[0],
        tau1_base * constantes.tau1_mult[1],
        tau1_base * constantes.tau1_mult[2],
    ];
    let tau2 = [
        tau2_base * constantes.tau2_mult[0],
        tau2_base * constantes.tau2_mult[1],
        tau2_base * constantes.tau2_mult[2],
    ];
    let tau3 = [
        tau3_base * constantes.tau3_mult[0],
        tau3_base * constantes.tau3_mult[1],
        tau3_base * constantes.tau3_mult[2],
    ];
    let dif = dif_base * constantes.difusion_base;
    let dec = dec_base * constantes.decaimiento;

    format!(
        r#"# EDEN Configuration File
# Generated: {}
# Name: "{}"
# Derived from seed with traits: {}

[cosmicas]
tau1 = [{:.6}, {:.6}, {:.6}]
tau2 = [{:.6}, {:.6}, {:.6}]
tau3 = [{:.6}, {:.6}, {:.6}]

difusion_base = {:.6}
decaimiento = {:.6}

[mar]
tamano = 64
hilos = 1

[seed_info]
nombre = "{}"
timestamp = {}
traits.resumen = "{}"
"#,
        timestamp,
        nombre,
        traits.resumen(),
        tau1[0],
        tau1[1],
        tau1[2],
        tau2[0],
        tau2[1],
        tau2[2],
        tau3[0],
        tau3[1],
        tau3[2],
        dif,
        dec,
        nombre,
        timestamp,
        traits.resumen(),
    )
}

// ============================================================================
// MAIN
// ============================================================================

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        println!(
            r#"EDEN Seed Generator v1.0

Usage:
    eden_seed_generator "<phrase>" [options]

Options:
    --output <path>    Output directory (default: ~/.eden/)
    --seed <path>      Custom seed file path
    --config <path>    Custom config file path
    --hex              Output seed as hex to stdout

Examples:
    eden_seed_generator "Mi universo personal"
    eden_seed_generator "Chaos Garden" --output ~/.eden/universes/chaos
    eden_seed_generator "Pacific Ocean" --hex

Trait Detection:
    - Caotic: caotico, chaos, random, turbio
    - Pacific: pacifico, peace, calm, tranquilo
    - Diverse: diverso, diversity, varied, many
    - Dark: oscuro, dark, shadow, night
    - Luminous: luminoso, light, bright, sol
    - Vital: vital, alive, vida, energia
    - Fragile: fragil, weak, delicate
"#
        );
        return;
    }

    let input = &args[1];
    let mut output_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(DEFAULT_EDEN_DIR);
    let mut seed_path = None;
    let mut config_path = None;
    let output_hex = args.contains(&"--hex".to_string());

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--output" => {
                if i + 1 < args.len() {
                    output_dir = PathBuf::from(&args[i + 1]);
                    i += 1;
                }
            }
            "--seed" => {
                if i + 1 < args.len() {
                    seed_path = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            "--config" => {
                if i + 1 < args.len() {
                    config_path = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    if let Err(e) = fs::create_dir_all(&output_dir) {
        eprintln!("Error creating output directory: {}", e);
        std::process::exit(1);
    }

    let mut salt = [0u8; 16];
    for (i, byte) in input.as_bytes().iter().enumerate() {
        salt[i % 16] = salt[i % 16].wrapping_add(*byte).wrapping_mul(0x9e as u8);
    }

    let seed = generar_seed(input, &salt);
    let traits = AnalisisTraits::analizar(input);
    let constantes = ConstantesAjustadas::desde_traits(&traits);

    if output_hex {
        println!("{}", seed_a_hex(&seed));
        return;
    }

    let seed_file = seed_path.unwrap_or_else(|| output_dir.join(DEFAULT_SEED_FILE));
    let config_file = config_path.unwrap_or_else(|| output_dir.join(DEFAULT_CONFIG_FILE));

    let seed_toml = generar_seed_toml(&seed, input, &traits);
    let config_toml = generar_config(&seed, input, &traits, &constantes);

    if let Err(e) = fs::write(&seed_file, &seed_toml) {
        eprintln!("Error writing seed file: {}", e);
        std::process::exit(1);
    }

    if let Err(e) = fs::write(&config_file, &config_toml) {
        eprintln!("Error writing config file: {}", e);
        std::process::exit(1);
    }

    println!(
        "EDEN Seed Generated Successfully

Name: \"{}\"
Seed: {} bytes ({} hex chars)
Traits: {}

Files created:
   Seed: {}
   Config: {}

Constants adjustment:
   Tau1: [{:.3}, {:.3}, {:.3}]
   Tau2: [{:.3}, {:.3}, {:.3}]
   Tau3: [{:.3}, {:.3}, {:.3}]
   Diffusion: {:.3}
   Decay: {:.3}
",
        input,
        seed.len(),
        seed.len() * 2,
        traits.resumen(),
        seed_file.display(),
        config_file.display(),
        constantes.tau1_mult[0],
        constantes.tau1_mult[1],
        constantes.tau1_mult[2],
        constantes.tau2_mult[0],
        constantes.tau2_mult[1],
        constantes.tau2_mult[2],
        constantes.tau3_mult[0],
        constantes.tau3_mult[1],
        constantes.tau3_mult[2],
        constantes.difusion_base,
        constantes.decaimiento,
    );
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analisis_traits_vacio() {
        let t = AnalisisTraits::analizar("");
        assert_eq!(t.caotico, 0.0);
        assert_eq!(t.pacifico, 0.0);
    }

    #[test]
    fn test_analisis_traits_caotico() {
        let t = AnalisisTraits::analizar("caotico y random");
        assert!(t.caotico > 0.2); // 0.25 per keyword
    }

    #[test]
    fn test_analisis_traits_pacifico() {
        let t = AnalisisTraits::analizar("pacifico y tranquilo");
        assert!(t.pacifico > 0.2); // 0.25 per keyword
    }

    #[test]
    fn test_analisis_traits_multiple() {
        let t = AnalisisTraits::analizar("caotico pero luminoso");
        assert!(t.caotico > 0.0);
        assert!(t.luminoso > 0.0);
    }

    #[test]
    fn test_resumen() {
        let mut t = AnalisisTraits::default();
        t.caotico = 0.75;
        t.luminoso = 0.6;
        let r = t.resumen();
        assert!(r.contains("caotic"));
        assert!(r.contains("luminous"));
    }

    #[test]
    fn test_seed_hex_conversion() {
        let seed = [
            0xAB, 0xCD, 0xEF, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x12, 0x34, 0x56,
            0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x12,
            0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE,
            0xF0, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x12, 0x34, 0x56, 0x78, 0x9A,
            0xBC, 0xDE, 0xF0, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x12, 0x34, 0x56,
            0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x12,
            0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE,
            0xF0, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x12, 0x34, 0x56, 0x78, 0x9A,
            0xBC, 0xDE, 0xF0, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x12, 0x34, 0x56,
            0x78, 0x9A,
        ];
        let hex = seed_a_hex(&seed);
        assert_eq!(hex.len(), 256);
        assert_eq!(&hex[0..4], "abcd");
    }

    #[test]
    fn test_hex_a_seed() {
        // 128 bytes = 256 hex chars
        let hex: String = (0..128).map(|i| format!("{:02x}", i as u8)).collect();
        assert_eq!(hex.len(), 256);
        let seed = hex_a_seed(&hex).unwrap();
        assert_eq!(seed[0], 0x00);
        assert_eq!(seed[127], 0x7F);
    }

    #[test]
    fn test_constantes_desde_traits() {
        let mut t = AnalisisTraits::default();
        t.caotico = 1.0;
        t.vital = 0.5;

        let c = ConstantesAjustadas::desde_traits(&t);
        assert!(c.tau1_mult[0] > 1.0);
        assert!(c.difusion_base > 0.5);
    }

    #[test]
    fn test_constantes_pacific() {
        let mut t = AnalisisTraits::default();
        t.pacifico = 1.0;

        let c = ConstantesAjustadas::desde_traits(&t);
        assert!(c.tau1_mult[0] < 1.0);
        assert!(c.difusion_base < 0.5);
    }

    #[test]
    fn test_generar_seed_deterministic() {
        let input = "test seed";
        let salt = [0u8; 16];
        let seed1 = generar_seed(input, &salt);
        let seed2 = generar_seed(input, &salt);
        assert_eq!(seed1, seed2);
    }

    #[test]
    fn test_generar_seed_different_inputs() {
        let salt = [0u8; 16];
        let seed1 = generar_seed("input1", &salt);
        let seed2 = generar_seed("input2", &salt);
        assert_ne!(seed1, seed2);
    }
}

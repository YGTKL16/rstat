use base64::{Engine as _, engine::general_purpose};
use clap::{Parser, Subcommand};
use ed25519_dalek::{Signer, SigningKey};
use rand::rngs::OsRng;
use serde::Serialize;

#[derive(Parser)]
#[command(
    name = "rstat-keygen",
    version,
    about = "Offline License Key Generator for rstat"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generates a new Ed25519 signing key and verifying key pair in hex.
    GenerateKeypair,
    /// Signs a licensee payload and generates a license.json.
    Sign {
        /// Name of the licensee
        #[arg(long)]
        name: String,
        /// Email of the licensee
        #[arg(long)]
        email: String,
        /// Order ID of the licensee
        #[arg(long)]
        order_id: String,
        /// Tier of the license (e.g. pro, enterprise)
        #[arg(long, default_value = "pro")]
        tier: String,
        /// Comma-separated features (e.g. spc,capability)
        #[arg(long)]
        features: String,
        /// Hex-encoded private (signing) key
        #[arg(long)]
        private_key: String,
    },
}

#[derive(Serialize)]
struct Licensee {
    name: String,
    email: String,
    order_id: String,
}

#[derive(Serialize)]
struct LicenseFile {
    licensee: Licensee,
    tier: String,
    features: Vec<String>,
    signature: String,
}

#[derive(Serialize)]
struct LicensePayload<'a> {
    licensee: &'a Licensee,
    tier: &'a str,
    features: &'a [String],
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::GenerateKeypair => {
            let mut csprng = OsRng;
            let signing_key = SigningKey::generate(&mut csprng);
            let verifying_key = signing_key.verifying_key();

            let private_key_hex = hex::encode(signing_key.to_bytes());
            let public_key_hex = hex::encode(verifying_key.to_bytes());

            println!("Private Key (Hex): {}", private_key_hex);
            println!("Public Key (Hex):  {}", public_key_hex);
        }
        Commands::Sign {
            name,
            email,
            order_id,
            tier,
            features,
            private_key,
        } => {
            // Parse private key
            let private_key_bytes = hex::decode(&private_key)?;
            let private_key_array: [u8; 32] = private_key_bytes
                .try_into()
                .map_err(|_| "Private key must be exactly 32 bytes (64 hex characters)")?;
            let signing_key = SigningKey::from_bytes(&private_key_array);

            // Setup structures
            let licensee = Licensee {
                name,
                email,
                order_id,
            };
            let feature_list: Vec<String> = features
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            // Create payload
            let payload = LicensePayload {
                licensee: &licensee,
                tier: &tier,
                features: &feature_list,
            };
            let payload_bytes = serde_json::to_vec(&payload)?;

            // Sign payload
            let signature = signing_key.sign(&payload_bytes);
            let signature_b64 = general_purpose::STANDARD.encode(signature.to_bytes());

            // Build license file
            let license_file = LicenseFile {
                licensee,
                tier,
                features: feature_list,
                signature: signature_b64,
            };

            // Print license JSON
            let json_out = serde_json::to_string_pretty(&license_file)?;
            println!("{}", json_out);
        }
    }

    Ok(())
}

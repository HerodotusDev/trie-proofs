// cli.rs
use alloy_primitives::hex::FromHex;
use alloy_primitives::B256;
use clap::{Parser, Subcommand};
use eth_trie_proofs::tx_trie::TxsMptHandler;
use serde::Serialize;
use serde_with::serde_as;

use eth_trie_proofs::tx_receipt_trie::TxReceiptsMptHandler;
use eth_trie_proofs::EthTrieError;

#[derive(Debug, Parser)]
#[command(name = "eth-trie-proof")]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Generate a MPT proof for a transaction")]
    Tx {
        /// Target transaction hash
        tx_hash: String,
        /// Ethereum node RPC URL
        rpc_url: Option<String>,
    },
    #[command(about = "Generate a MPT proof for a transaction receipt")]
    Receipt {
        /// Receipt transaction hash
        tx_hash: String,
        /// Ethereum node RPC URL
        rpc_url: Option<String>,
    },
}

#[serde_with::serde_as]
#[derive(Debug, Serialize)]
struct MptProof {
    root: B256,
    #[serde_as(as = "Vec<serde_with::hex::Hex>")]
    proof: Vec<Vec<u8>>,
    index: u64,
}

#[tokio::main]
async fn main() -> Result<(), EthTrieError> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Tx { tx_hash, rpc_url } => {
            generate_tx_proof(
                &tx_hash,
                rpc_url
                    .unwrap_or("https://ethereum-rpc.publicnode.com".parse().unwrap())
                    .as_str(),
            )
            .await?;
        }
        Commands::Receipt { tx_hash, rpc_url } => {
            generate_receipt_proof(
                &tx_hash,
                rpc_url
                    .unwrap_or("https://ethereum-rpc.publicnode.com".parse().unwrap())
                    .as_str(),
            )
            .await?;
        }
    }

    Ok(())
}

async fn generate_tx_proof(tx_hash: &str, rpc_url: &str) -> Result<(), EthTrieError> {
    let rpc_url = url::Url::parse(rpc_url).expect("Invalid URL");
    let mut txs_mpt_handler = TxsMptHandler::new(rpc_url)?;
    let tx_hash = B256::from_hex(tx_hash).unwrap();
    txs_mpt_handler.build_tx_tree_from_tx_hash(tx_hash).await?;
    let index = txs_mpt_handler.tx_hash_to_tx_index(tx_hash)?;
    let proof = txs_mpt_handler.get_proof(index)?;
    let root = txs_mpt_handler.get_root()?;

    let mpt_proof = MptProof { root, proof, index };
    println!("Generated TX Proof: ");
    println!("{}", serde_json::to_string(&mpt_proof).unwrap());
    Ok(())
}

async fn generate_receipt_proof(tx_hash: &str, rpc_url: &str) -> Result<(), EthTrieError> {
    let rpc_url = url::Url::parse(rpc_url).expect("Invalid URL");
    let mut tx_receipts_mpt_handler = TxReceiptsMptHandler::new(rpc_url)?;
    let tx_hash = B256::from_hex(tx_hash).unwrap();
    tx_receipts_mpt_handler
        .build_tx_receipt_tree_from_tx_hash(tx_hash)
        .await?;
    let index = tx_receipts_mpt_handler.tx_hash_to_tx_index(tx_hash).await?;
    let proof = tx_receipts_mpt_handler.get_proof(index)?;
    let root = tx_receipts_mpt_handler.get_root()?;

    let mpt_proof = MptProof { root, proof, index };
    println!("Generated Receipt Proof: ");
    println!("{}", serde_json::to_string(&mpt_proof).unwrap());
    Ok(())
}

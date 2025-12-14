// Standalone test binary for parakeet-rs transcription
// Replicates the parakeet-rs example to verify model behavior outside our app
//
// Usage: cargo run --example test_transcribe /path/to/audio.wav

use parakeet_rs::ParakeetTDT;
use std::env;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let audio_path = args.get(1).expect("Usage: test_transcribe <audio.wav>");

    // Model path: ~/Library/Application Support/heycat/models/parakeet-tdt/
    let model_dir = dirs::data_dir()
        .expect("Could not find data dir")
        .join("heycat")
        .join("models")
        .join("parakeet-tdt");

    println!("Loading model from: {:?}", model_dir);
    let start = Instant::now();

    let mut parakeet = ParakeetTDT::from_pretrained(
        model_dir.to_str().unwrap(),
        None,
    )?;

    println!("Model loaded in {:.2}s", start.elapsed().as_secs_f32());
    println!("\nTranscribing: {}", audio_path);

    let start = Instant::now();
    let result = parakeet.transcribe_file(audio_path, None)?;

    println!("\n=== result.text (BROKEN - parakeet-rs bug) ===");
    println!("{:?}", result.text);

    // WORKAROUND: Concatenate tokens without adding spaces between them
    // The tokens already have leading spaces where word boundaries are (from SentencePiece ▁)
    let fixed_text: String = result.tokens
        .iter()
        .map(|t| t.text.as_str())
        .collect();
    let fixed_text = fixed_text.trim();

    println!("\n=== FIXED text (concatenate tokens) ===");
    println!("{:?}", fixed_text);

    println!("\n=== result.tokens ({} total) ===", result.tokens.len());
    for (i, token) in result.tokens.iter().enumerate() {
        println!(
            "[{:3}] [{:.3}s - {:.3}s]: {:?}",
            i, token.start, token.end, token.text
        );
    }

    println!(
        "\n✓ Transcription completed in {:.2}s",
        start.elapsed().as_secs_f32()
    );

    Ok(())
}

//! Probes a local OpenAI-compatible endpoint **through unia's own transport**.
//!
//! The point is not to get an answer. The point is to find out which layer
//! refuses: the transport, the transcription, or the engine. A demo that cannot
//! distinguish those three has learned nothing, and a demo that *fabricates* past
//! them is worse than one that refuses.
//!
//! Usage: cargo run --release --example oc_shim_probe -- 127.0.0.1:8899

use unia::doubt::{Doubt, Resolver, Unresolvable};

fn main() {
    let authority = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8899".to_string());

    // The toll first, so a refusal is priced before anything is spent.
    let mut nuants = 10.0f64;
    println!("stock: {nuants} nuants");
    println!("toll of a consultation, at {} nuants / 1000 tokens:",
        unia::toll::NUANTS_PER_KILO_TOKEN);

    let doubt = Doubt::new(
        "what are you?",
        Unresolvable::NoRule,
    );
    println!("\nasking {authority} about: {:?}", doubt.question);
    println!("doubt signature: {}", doubt.signature());

    let r = match unia::resolve::HttpResolver::new(&authority, "llama-server") {
        Ok(r) => r,
        Err(e) => {
            println!("\nLAYER 1 refused — the authority: {e}");
            return;
        }
    };

    // Path as the OpenAI API wants it, not unia's own `/consult`.
    let mut r = r;
    r.path = "/v1/chat/completions".to_string();

    if std::env::args().any(|a| a == "--openai") {
        // The transpiler: unia's Doubt in the engine's own vocabulary.
        r.transcribe = Some(|d: &Doubt, model: &str| {
            serde_json::json!({
                "model": model,
                "messages": [{"role": "user", "content": d.question}],
                "max_tokens": 64,
            })
            .to_string()
        });
        println!("(transcribing into the OpenAI chat shape)");
    }

    match r.ask(&doubt) {
        Ok(answer) => {
            // Only reachable with a real measurement, which this transport does
            // not yet read. Print it, but say plainly that nothing was measured.
            println!("\nLAYER 3 answered: {answer}");
            let charged = unia::toll::charge(&mut nuants, None);
            println!("toll: {charged:?}  (an unmeasured answer is NOT free)");
            println!("stock now: {nuants} nuants");
        }
        Err(e) => {
            println!("\nLAYER 2 refused — the engine or the transcription: {e}");
            println!("This is the honest outcome: the transport reached the endpoint,");
            println!("and the endpoint declined. No sentence was invented, and the");
            println!("toll was not charged, because nothing was measured.");
        }
    }
}

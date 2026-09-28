#[cfg(test)]
mod tests {
    use std::fs;
    use unia::transducer::{SynapticTransducer, TransductionLayer};

    #[test]
    fn test_synaptic_transduction_flow() {
        let mut transducer = SynapticTransducer::new(TransductionLayer::LocalSovereign);
        let external_id = "external-llm-api-01";

        // 1. Shadow interactions
        transducer.shadow_resource(
            external_id,
            "What is a .ure?",
            "A Universal Resource Entity.",
            120,
            true,
        );
        transducer.shadow_resource(
            external_id,
            "How to synthesize?",
            "Use the Fluid Factory.",
            150,
            true,
        );

        // 2. Transduce to Mirror Actuator
        let mirror_id = transducer
            .transduce(external_id, None)
            .expect("Transduction failed");

        // 3. Verify the manifest landed where synthesis is documented to put it
        let file_path = unia::outdir::manifest_path(&mirror_id.to_string());
        assert!(fs::metadata(&file_path).is_ok());

        // 4. Mirroring Pattern Verification
        let is_valid = transducer.verify_mirror(
            mirror_id,
            external_id,
            "What is a .ure?",
            "A Universal Resource Entity.",
        );
        assert!(is_valid);

        // Cleanup
        let _ = fs::remove_file(file_path);
    }
}

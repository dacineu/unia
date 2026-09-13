#[cfg(test)]
mod tests {
    use askillify::wmis::WmisAdapter;
    use serde_json::json;

    #[test]
    fn test_ure_to_wmis_conversion() {
        let manifest = json!({
            "resource_id": "uuid-123-abc",
            "resource_type": "skill",
            "guidance": "Rust trait implementation expert",
            "complexity_score": 0.6
        });

        let wmis_res = WmisAdapter::from_ure(&manifest, "user_dacineu").expect("Conversion failed");

        assert_eq!(wmis_res.id, "uuid-123-abc");
        assert_eq!(wmis_res.owner, "user_dacineu");
        assert!(wmis_res.quality.qor > 0.8);
        assert!(wmis_res.capabilities[0].contains("Rust trait"));
    }
}

use pd_signature::cavage;

#[test]
fn basic() {
    const HEADER: &str = r#"keyId="Test",algorithm="rsa-sha256",headers="(request-target) host date",signature="qdx+H7PHHDZgy4y/Ahn9Tny9V3GP6YgBPyUXMmoxWtLbHpUnXS2mg2+SbrQDMCJypxBLSPQR2aAjn7ndmw2iicw3HMbe8VfEdKFYRqzic+efkb3nndiv/x1xSHDJWeSWkx3ButlYSuBskLu6kd9Fswtemr3lgdDEmn04swr2Os0=""#;

    let header = cavage::parse::parse(HEADER).unwrap();

    assert_eq!(header.key_id, "Test");
    assert_eq!(
        header.headers.collect::<Vec<_>>(),
        ["(request-target)", "host", "date"]
    );
    assert_eq!(
        header.signature,
        "qdx+H7PHHDZgy4y/Ahn9Tny9V3GP6YgBPyUXMmoxWtLbHpUnXS2mg2+SbrQDMCJypxBLSPQR2aAjn7ndmw2iicw3HMbe8VfEdKFYRqzic+efkb3nndiv/x1xSHDJWeSWkx3ButlYSuBskLu6kd9Fswtemr3lgdDEmn04swr2Os0="
    );
    assert_eq!(header.created, None);
    assert_eq!(header.expires, None);
}

#[test]
fn extended() {
    const HEADER: &str = r#"keyId="Test",algorithm="rsa-sha256",created=1402170695, expires=1402170699,headers="(request-target) (created) (expires) host date content-type digest content-length",signature="vSdrb+dS3EceC9bcwHSo4MlyKS59iFIrhgYkz8+oVLEEzmYZZvRs8rgOp+63LEM3v+MFHB32NfpB2bEKBIvB1q52LaEUHFv120V01IL+TAD48XaERZFukWgHoBTLMhYS2Gb51gWxpeIq8knRmPnYePbF5MOkR0Zkly4zKH7s1dE=""#;

    let header = cavage::parse::parse(HEADER).unwrap();

    assert_eq!(header.key_id, "Test");
    assert_eq!(
        header.headers.collect::<Vec<_>>(),
        [
            "(request-target)",
            "(created)",
            "(expires)",
            "host",
            "date",
            "content-type",
            "digest",
            "content-length"
        ]
    );
    assert_eq!(
        header.signature,
        "vSdrb+dS3EceC9bcwHSo4MlyKS59iFIrhgYkz8+oVLEEzmYZZvRs8rgOp+63LEM3v+MFHB32NfpB2bEKBIvB1q52LaEUHFv120V01IL+TAD48XaERZFukWgHoBTLMhYS2Gb51gWxpeIq8knRmPnYePbF5MOkR0Zkly4zKH7s1dE="
    );
    assert_eq!(header.created, Some(1402170695));
    assert_eq!(header.expires, Some(1402170699));
}

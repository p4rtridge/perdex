use pd_activitypub::webfinger::Webfinger;
use pd_core::federation::domain::account::AccountResolver;
use pd_http::Client;

#[tokio::test]
async fn basic() {
    let http_client = Client::builder().build().unwrap();
    let webfinger = Webfinger::builder().http_client(http_client).build();

    let resource = webfinger
        .resolve_account("Strandjunker", "mstdn.social")
        .await
        .expect("Failed to resolve account")
        .unwrap();

    assert_eq!(resource.username, "Strandjunker");
    assert_eq!(resource.domain, "mstdn.social");
    assert_eq!(resource.uri, "https://mstdn.social/users/Strandjunker");
}

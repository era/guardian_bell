use temp_dir::TempDir;
use tokio::runtime::Handle;
use tonic_health::pb::health_client::HealthClient;
use tokio::time::sleep;
use tokio::time::Duration;


fn start_grpc_server() {

   Handle::current().spawn(async {
        let logs = TempDir::new().unwrap();
        guardian_bell::app::App::run_server(8080, logs.path()).await.unwrap();
    });
    //FIXME
    sleep(Duration::from_secs(3)).await;
}

#[tokio::test]
async fn health_service() {
    start_grpc_server();
    let conn = tonic::transport::Endpoint::new("http://[::1]:8080").unwrap().connect().await.unwrap();
    let client = HealthClient::new(conn);
    // test the health service    
}
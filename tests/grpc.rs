use temp_dir::TempDir;
use tokio::runtime::Handle;
use tokio::time::sleep;
use tokio::time::Duration;
use tonic::transport::Channel;
use tonic_health::pb::health_check_response::ServingStatus;
use tonic_health::pb::health_client::HealthClient;

fn start_grpc_server() {
    Handle::current().spawn(async {
        let logs = TempDir::new().unwrap();
        guardian_bell::app::App::run_server(8080, logs.path())
            .await
            .unwrap();
    });
}

async fn setup() {
    start_grpc_server();
}

/// connects to our test server. Retries if the connection is not yet ready
async fn connect() -> Channel {
    let mut retry = 0;
    while true {
        let conn = tonic::transport::Endpoint::new("http://[::1]:8080")
        .unwrap()
        .connect()
        .await;
        match (conn, retry) {
            (Ok(conn), _) => return conn,
            (Err(e), i) => {
                if i <= 3 {                
                    sleep(Duration::from_secs(i)).await;
                    retry = 1 + retry
                } else {
                    panic!("could not connect {:?}", e);
                }
            },
            _ => panic!("should not be possible")
        };
    }
    panic!("impossible");
}

#[tokio::test]
async fn health_service() {
    setup().await;
    let mut client = HealthClient::new(connect().await);
    let request = tonic::Request::new(tonic_health::pb::HealthCheckRequest { service: "".into() });
    let response = client.check(request).await.unwrap();
    let response = response.into_inner();

    assert_eq!(
        ServingStatus::Serving,
        ServingStatus::from_i32(response.status).unwrap()
    );
}

// TODO send a request to mark the node as not_serving (so we can shutdown it)
// Test if SercingStatus return not_serving

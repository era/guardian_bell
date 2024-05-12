use proto::admin_client::AdminClient;
use proto::ShutdownRequest;
use std::sync::Once;
use temp_dir::TempDir;
use tokio::runtime::Handle;
use tokio::time::sleep;
use tokio::time::Duration;
use tonic::transport::Channel;
use tonic_health::pb::health_check_response::ServingStatus;
use tonic_health::pb::health_client::HealthClient;

pub mod proto {
    tonic::include_proto!("admin_service");
}

static INIT: Once = Once::new();

fn start_grpc_server() {
    INIT.call_once(|| {
        Handle::current().spawn(async {
            let logs = TempDir::new().unwrap();
            guardian_bell::app::App::run_server(8080, logs.path())
                .await
                .unwrap();
        });
    });
}

async fn setup() {
    start_grpc_server();
}

/// connects to our test server. Retries if the connection is not yet ready
async fn connect() -> Channel {
    let mut retry = 0;
    loop {
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
            }
        };
    }
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

#[tokio::test]
async fn shutdown_gracefully() {
    setup().await;
    let mut client = HealthClient::new(connect().await);

    let mut admin_client = AdminClient::new(connect().await);
    let request = tonic::Request::new(ShutdownRequest {});
    let _ = admin_client.shutdown(request).await.unwrap();
    // sleeps for one second to give shutdown the server in time
    sleep(Duration::from_secs(1)).await;

    // since we shutodwn the services, we are not able to receive requests anymore
    let request = tonic::Request::new(tonic_health::pb::HealthCheckRequest { service: "".into() });
    let result = client.check(request).await;
    assert!(result.is_err());
}

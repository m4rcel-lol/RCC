use tonic::{transport::Server, Request, Response, Status};
use rcc_proto::rcc::coordinator_server::{Coordinator, CoordinatorServer};
use rcc_proto::rcc::{ExecutorInfo, RegisterResponse, ExecutorStatus, HeartbeatAck};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct CoordinatorService {
    executors: Arc<Mutex<HashMap<String, ExecutorInfo>>>,
}

#[tonic::async_trait]
impl Coordinator for CoordinatorService {
    async fn register_executor(&self, request: Request<ExecutorInfo>) -> Result<Response<RegisterResponse>, Status> {
        let info = request.into_inner();
        println!("📝 RCCService: Registering Executor {}", info.id);
        self.executors.lock().unwrap().insert(info.id.clone(), info);
        Ok(Response::new(RegisterResponse { success: true }))
    }

    async fn heartbeat(&self, request: Request<ExecutorStatus>) -> Result<Response<HeartbeatAck>, Status> {
        // In a real scenario, we track health here
        Ok(Response::new(HeartbeatAck {}))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let addr = "0.0.0.0:50051".parse()?;
    let service = CoordinatorService::default();

    println!("🚀 RCCService listening on {}", addr);

    Server::builder()
        .add_service(CoordinatorServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

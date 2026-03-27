//! Actor system for Wrenflow hub.
//! Each actor owns its state and listens for DartSignals.

mod pipeline_actor;

use pipeline_actor::PipelineActor;
use tokio::spawn;

pub async fn create_actors() {
    let mut pipeline = PipelineActor::new();
    spawn(async move {
        pipeline.run().await;
    });
}

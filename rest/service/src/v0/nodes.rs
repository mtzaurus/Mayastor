use super::*;

struct Factory {}
impl HttpServiceFactory for Factory {
    fn register(self, config: &mut AppService) {
        get_node.register(config);
        get_nodes.register(config);
    }
}
pub(crate) fn factory() -> impl HttpServiceFactory {
    Factory {}
}

#[get("/v0/nodes")]
async fn get_nodes() -> impl Responder {
    match MessageBus::get_nodes().await {
        Ok(nodes) => HttpResponse::Ok().json(nodes),
        Err(error) => (RestError::from(error)).into(),
    }
}
#[get("/v0/nodes/{id}")]
async fn get_node(web::Path(node_id): web::Path<String>) -> impl Responder {
    match MessageBus::get_node(&node_id).await {
        Ok(node) => HttpResponse::Ok().json(node),
        Err(error) => (RestError::from(error)).into(),
    }
}

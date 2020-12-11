use super::*;

struct Factory {}
impl HttpServiceFactory for Factory {
    fn register(self, config: &mut AppService) {
        get_nexus_children.register(config);
        get_nexus_child.register(config);
        get_node_nexus_children.register(config);
        get_node_nexus_child.register(config);
        add_nexus_child.register(config);
        add_node_nexus_child.register(config);
        delete_nexus_child.register(config);
        delete_node_nexus_child.register(config);
    }
}
pub(crate) fn factory() -> impl HttpServiceFactory {
    Factory {}
}

#[get("/v0/nexuses/{nexus_id}/children")]
async fn get_nexus_children(
    web::Path(nexus_id): web::Path<String>,
) -> impl Responder {
    get_children_response(Filter::Nexus(nexus_id)).await
}
#[get("/v0/nodes/{node_id}/nexuses/{nexus_id}/children")]
async fn get_node_nexus_children(
    web::Path((node_id, nexus_id)): web::Path<(String, String)>,
) -> impl Responder {
    get_children_response(Filter::NodeNexus(node_id, nexus_id)).await
}

#[get("/v0/nexuses/{nexus_id}/children/{child_id:.*}")]
async fn get_nexus_child(
    web::Path((nexus_id, child_id)): web::Path<(String, String)>,
    req: HttpRequest,
) -> impl Responder {
    get_child_response(child_id, req, Filter::Nexus(nexus_id)).await
}
#[get("/v0/nodes/{node_id}/nexuses/{nexus_id}/children/{child_id:.*}")]
async fn get_node_nexus_child(
    web::Path((node_id, nexus_id, child_id)): web::Path<(
        String,
        String,
        String,
    )>,
    req: HttpRequest,
) -> impl Responder {
    get_child_response(child_id, req, Filter::NodeNexus(node_id, nexus_id))
        .await
}

#[put("/v0/nexuses/{nexus_id}/children/{child_id:.*}")]
async fn add_nexus_child(
    web::Path((nexus_id, child_id)): web::Path<(String, String)>,
    req: HttpRequest,
) -> impl Responder {
    add_child_filtered(child_id, req, Filter::Nexus(nexus_id)).await
}
#[put("/v0/nodes/{node_id}/nexuses/{nexus_id}/children/{child_id:.*}")]
async fn add_node_nexus_child(
    web::Path((node_id, nexus_id, child_id)): web::Path<(
        String,
        String,
        String,
    )>,
    req: HttpRequest,
) -> impl Responder {
    add_child_filtered(child_id, req, Filter::NodeNexus(node_id, nexus_id))
        .await
}

#[delete("/v0/nexuses/{nexus_id}/children/{child_id:.*}")]
async fn delete_nexus_child(
    web::Path((nexus_id, child_id)): web::Path<(String, String)>,
    req: HttpRequest,
) -> impl Responder {
    delete_child_filtered(child_id, req, Filter::Nexus(nexus_id)).await
}
#[delete("/v0/nodes/{node_id}/nexuses/{nexus_id}/children/{child_id:.*}")]
async fn delete_node_nexus_child(
    web::Path((node_id, nexus_id, child_id)): web::Path<(
        String,
        String,
        String,
    )>,
    req: HttpRequest,
) -> impl Responder {
    delete_child_filtered(child_id, req, Filter::NodeNexus(node_id, nexus_id))
        .await
}

async fn get_children_response(filter: Filter) -> impl Responder {
    match MessageBus::get_nexus(filter).await {
        Ok(nexus) => HttpResponse::Ok().json(nexus.children),
        Err(error) => (RestError::from(error)).into(),
    }
}

async fn get_child_response(
    child_id: String,
    req: HttpRequest,
    filter: Filter,
) -> impl Responder {
    let child_id = build_child_uri(child_id, req);
    let child = match MessageBus::get_nexus(filter).await {
        Ok(nexus) => find_nexus_child(&nexus, &child_id),
        Err(error) => Err(error),
    };
    match child {
        Ok(child) => HttpResponse::Ok().json(child),
        Err(error) => (RestError::from(error)).into(),
    }
}

fn find_nexus_child(nexus: &Nexus, child_uri: &str) -> Result<Child, BusError> {
    if let Some(child) = nexus.children.iter().find(|&c| c.uri == child_uri) {
        Ok(child.clone())
    } else {
        Err(BusError::NotFound)
    }
}

async fn add_child_filtered(
    child_id: String,
    req: HttpRequest,
    filter: Filter,
) -> impl Responder {
    let child_uri = build_child_uri(child_id, req);

    let nexus = match MessageBus::get_nexus(filter).await {
        Ok(nexus) => nexus,
        Err(error) => return (RestError::from(error)).into(),
    };

    let create = AddNexusChild {
        node: nexus.node,
        nexus: nexus.uuid,
        uri: child_uri,
        auto_rebuild: true,
    };
    match MessageBus::add_nexus_child(create).await {
        Ok(child) => HttpResponse::Ok().json(child),
        Err(error) => (RestError::from(error)).into(),
    }
}

async fn delete_child_filtered(
    child_id: String,
    req: HttpRequest,
    filter: Filter,
) -> impl Responder {
    let child_uri = build_child_uri(child_id, req);

    let nexus = match MessageBus::get_nexus(filter).await {
        Ok(nexus) => nexus,
        Err(error) => return (RestError::from(error)).into(),
    };

    let destroy = RemoveNexusChild {
        node: nexus.node,
        nexus: nexus.uuid,
        uri: child_uri,
    };
    match MessageBus::remove_nexus_child(destroy).await {
        Ok(_) => HttpResponse::Ok().json(()),
        Err(error) => (RestError::from(error)).into(),
    }
}

fn build_child_uri(child_id: String, req: HttpRequest) -> String {
    match url::Url::parse(&child_id) {
        Ok(_) => {
            if req.query_string().is_empty() {
                child_id
            } else {
                format!("{}?{}", child_id, req.query_string())
            }
        }
        _ => {
            // not a URL, it's probably legacy, default to AIO
            format!("aio://{}", child_id)
        }
    }
}

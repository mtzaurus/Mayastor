use super::*;

struct Factory {}
impl HttpServiceFactory for Factory {
    fn register(self, config: &mut AppService) {
        get_volumes.register(config);
        get_volume.register(config);
        get_node_volumes.register(config);
        get_node_volume.register(config);
        put_volume.register(config);
        del_nexus.register(config);
    }
}
pub(crate) fn factory() -> impl HttpServiceFactory {
    Factory {}
}

#[get("/v0/volumes")]
async fn get_volumes() -> impl Responder {
    match MessageBus::get_volumes(Filter::All).await {
        Ok(volumes) => HttpResponse::Ok().json(volumes),
        Err(error) => return (RestError::from(error)).into(),
    }
}

#[get("/v0/volumes/{volume_id}")]
async fn get_volume(web::Path(volume_id): web::Path<String>) -> impl Responder {
    match MessageBus::get_volume(Filter::Volume(volume_id)).await {
        Ok(volume) => HttpResponse::Ok().json(volume),
        Err(error) => return (RestError::from(error)).into(),
    }
}

#[get("/v0/nodes/{node_id}/volumes")]
async fn get_node_volumes(
    web::Path(node_id): web::Path<String>,
) -> impl Responder {
    match MessageBus::get_volumes(Filter::Node(node_id)).await {
        Ok(volumes) => HttpResponse::Ok().json(volumes),
        Err(error) => return (RestError::from(error)).into(),
    }
}
#[get("/v0/nodes/{node_id}/volumes/{volume_id}")]
async fn get_node_volume(
    web::Path((node_id, volume_id)): web::Path<(String, String)>,
) -> impl Responder {
    match MessageBus::get_volume(Filter::NodeVolume(node_id, volume_id)).await {
        Ok(volume) => HttpResponse::Ok().json(volume),
        Err(error) => return (RestError::from(error)).into(),
    }
}

#[put("/v0/volumes/{volume_id}")]
async fn put_volume(
    web::Path(volume_id): web::Path<String>,
    create: web::Json<CreateVolumeBody>,
) -> impl Responder {
    let create = create.into_inner().bus_request(volume_id);
    match MessageBus::create_volume(create).await {
        Ok(volume) => HttpResponse::Ok().json(volume),
        Err(error) => return (RestError::from(error)).into(),
    }
}

#[delete("/v0/volumes/{volume_id}")]
async fn del_nexus(web::Path(volume_id): web::Path<String>) -> impl Responder {
    let request = DestroyVolume {
        uuid: volume_id.to_string(),
    };
    match MessageBus::delete_volume(request).await {
        Ok(_) => HttpResponse::Ok().json(()),
        Err(error) => return (RestError::from(error)).into(),
    }
}

use crate::{
    database::DatabaseAccess, requests::DeclareCommunityRequest, responses::ErrorResponse,
    zksync::ZksyncApp,
};
use actix_web::{web, HttpResponse, Responder, Scope};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ServiceProvider<DB: DatabaseAccess> {
    db: Arc<DB>,
    zksync: Arc<ZksyncApp>,
}

impl<DB: 'static + DatabaseAccess> ServiceProvider<DB> {
    pub fn new(db: DB) -> Self {
        let zksync = ZksyncApp::new("incorrect_addr", "incorrect_addr");

        Self {
            db: Arc::new(db),
            zksync: Arc::new(zksync),
        }
    }

    pub async fn declare_community(
        provider: web::Data<Self>,
        request: web::Json<DeclareCommunityRequest>,
    ) -> impl Responder {
        let request = request.into_inner();

        match provider.db.declare_community(request.community).await {
            Ok(()) => HttpResponse::Ok().json(()),
            Err(err) => HttpResponse::BadRequest().json(ErrorResponse::error(&err.to_string())),
        }
    }

    // TODO: Subscribe (manual)

    // TODO: Subscribe (pre-sign txs)

    // TODO: Unsubscribe (what should this method do? provide a "change pubkey" tx?) Alternative -- this is a fully client-side function, provider has nothing to do with it.

    // TODO: Check amount of tokens granted to user

    // TODO: Request minting tx

    // TODO: Check subscription status

    pub fn into_web_scope(self) -> Scope {
        web::scope("api/v0.1/")
            .data(self)
            .service(web::resource("/declare_community").to(Self::declare_community))
    }
}

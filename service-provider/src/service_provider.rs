use crate::{
    database::{DatabaseAccess, Subscription},
    oracle::CommunityOracle,
    requests::{
        AddSubscriptionTxsRequest, DeclareCommunityRequest, GrantedTokensRequest,
        MintingSignatureRequest, RelatedCommunitiesRequest, SetSubscriptionDataRequest,
        SubscriptionCheckRequest,
    },
    responses::{ErrorResponse, RelatedCommunitiesResponse, SubscriptionCheckResponse},
    utils::response_from_error,
    zksync::ZksyncApp,
};
use actix_web::{web, HttpResponse, Scope};
use anyhow::Result;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ServiceProvider<DB: DatabaseAccess> {
    db: Arc<DB>,
    zksync: Arc<ZksyncApp>,
    oracle: Arc<CommunityOracle>,
}

impl<DB: 'static + DatabaseAccess> ServiceProvider<DB> {
    pub fn new(db: DB) -> Self {
        let zksync = ZksyncApp::new("incorrect_addr", "incorrect_addr");
        let oracle = CommunityOracle::new("incorrect_addr");

        Self {
            db: Arc::new(db),
            zksync: Arc::new(zksync),
            oracle: Arc::new(oracle),
        }
    }

    pub async fn declare_community(
        provider: web::Data<Self>,
        request: web::Json<DeclareCommunityRequest>,
    ) -> Result<HttpResponse> {
        let request = request.into_inner();

        provider.db.declare_community(request.community).await?;

        Ok(HttpResponse::Ok().json(()))
    }

    pub async fn related_communities(
        _provider: web::Data<Self>,
        request: web::Json<RelatedCommunitiesRequest>,
    ) -> Result<HttpResponse> {
        let _request = request.into_inner();

        // TODO: Stub
        let response = RelatedCommunitiesResponse {
            communities: Vec::new(),
        };

        Ok(HttpResponse::Ok().json(response))
    }

    pub async fn set_subscription_info(
        provider: web::Data<Self>,
        request: web::Json<SetSubscriptionDataRequest>,
    ) -> Result<HttpResponse> {
        let request = request.into_inner();

        let subscription = Subscription::new(request.community_name, request.subscription_wallet);

        provider
            .db
            .add_subscription(request.user, subscription)
            .await?;

        Ok(HttpResponse::Ok().json(()))
    }

    pub async fn add_subscription_txs(
        provider: web::Data<Self>,
        request: web::Json<AddSubscriptionTxsRequest>,
    ) -> Result<HttpResponse> {
        let request = request.into_inner();

        for subscription_tx in &request.txs {
            if let Err(_) = provider.zksync.check_subscription_tx(subscription_tx).await {
                let response = HttpResponse::BadRequest()
                    .json(ErrorResponse::error("Incorrect tx in request"));
                return Ok(response);
            }
        }

        provider
            .db
            .add_subscription_txs(request.user, &request.community_name, request.txs)
            .await?;

        let response = HttpResponse::Ok().json(());

        Ok(response)
    }

    // TODO: Unsubscribe (what should this method do? provide a "change pubkey" tx?) Alternative -- this is a fully client-side function, provider has nothing to do with it.

    pub async fn tokens_for_user(
        provider: web::Data<Self>,
        request: web::Json<GrantedTokensRequest>,
    ) -> Result<HttpResponse> {
        let request = request.into_inner();

        let response = provider.oracle.tokens_for_user(request).await?;

        Ok(response)
    }

    pub async fn sign_minting_tx(
        provider: web::Data<Self>,
        request: web::Json<MintingSignatureRequest>,
    ) -> Result<HttpResponse> {
        let request = request.into_inner();

        let response = provider.oracle.sign_minting_tx(request).await?;

        Ok(response)
    }

    pub async fn is_user_subscribed(
        provider: web::Data<Self>,
        request: web::Json<SubscriptionCheckRequest>,
    ) -> Result<HttpResponse> {
        let request = request.into_inner();

        let sub = match provider
            .db
            .get_subscription(request.user, &request.community_name)
            .await?
        {
            Some(community) => community,
            None => {
                return Ok(HttpResponse::Ok().json(SubscriptionCheckResponse { subscribed: false }))
            }
        };

        let subscribed = provider
            .zksync
            .is_user_subscribed(sub.subscription_wallet)
            .await?;

        Ok(HttpResponse::Ok().json(SubscriptionCheckResponse { subscribed }))
    }

    /// Wrapper around functions that return `anyhow::Result` which converts it to the `HttpResponse`.
    /// This decorator allows handler functions to return `Result` and use `?` for convenient error propagation.
    ///
    /// Wrapper functions must be `async`.
    pub async fn failable<F, Fut, R>(
        handler: F,
        provider: web::Data<Self>,
        request: web::Json<R>,
    ) -> HttpResponse
    where
        F: Fn(web::Data<Self>, web::Json<R>) -> Fut,
        Fut: std::future::Future<Output = Result<HttpResponse>>,
    {
        match handler(provider, request).await {
            Ok(response) => response,
            Err(error) => response_from_error(error),
        }
    }

    pub fn into_web_scope(self) -> Scope {
        web::scope("api/v0.1/")
            .data(self)
            .service(
                web::resource("/declare_community")
                    .to(|p, data| Self::failable(Self::declare_community, p, data)),
            )
            .service(
                web::resource("/is_user_subscribed")
                    .to(|p, data| Self::failable(Self::is_user_subscribed, p, data)),
            )
            .service(
                web::resource("/get_minting_signature")
                    .to(|p, data| Self::failable(Self::sign_minting_tx, p, data)),
            )
            .service(
                web::resource("/granted_tokens")
                    .to(|p, data| Self::failable(Self::tokens_for_user, p, data)),
            )
            .service(
                web::resource("/set_subscription_info")
                    .to(|p, data| Self::failable(Self::set_subscription_info, p, data)),
            )
            .service(
                web::resource("/add_subscription_txs")
                    .to(|p, data| Self::failable(Self::add_subscription_txs, p, data)),
            )
            .service(
                web::resource("/related_communities")
                    .to(|p, data| Self::failable(Self::related_communities, p, data)),
            )
    }
}

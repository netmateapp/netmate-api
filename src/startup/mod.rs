use axum::Router;
use tokio::net::TcpListener;

pub async fn startup() {
    // リクエストサイズを制限する
    let app = Router::new();
        //.merge(sign_up_route());

    // Brotli 圧縮を有効にする

    // rustlsなどでTLSを有効化

    /*特定のURLで実行されているElasticsearchのクライアント
        let transport = Transport::single_node("https://example.com")?;
    let client = Elasticsearch::new(transport);
     */
    
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    // let listener = TcpListener::bind("0.0.0.0:443").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/*
pub async fn startup(modules: Arc<Modules>) {
    let hc_router = Router::new()
        .route("/", get(hc))
        .route("/db", get(hc_db))
        .route("/dynamo", get(hc_dynamo));
    let stocks_router = Router::new()
        .route("/", post(create_stock))
        .route("/:id", get(stock_view));
    let market_kind_router = Router::new()
        .route("/", post(create_market_kind))
        .route("/:id", delete(delete_market_kind));
    let market_data_router = Router::new().route("/:stock_id", post(upload_market_data));

    let app = Router::new()
        .nest("/hc", hc_router)
        .nest("/stocks", stocks_router)
        .nest("/market_kind", market_kind_router)
        .nest("/market_data", market_data_router)
        .layer(AddExtensionLayer::new(modules));
}

pub fn init_app() {
    dotenv().ok();
    tracing_subscriber::fmt::init();
} */
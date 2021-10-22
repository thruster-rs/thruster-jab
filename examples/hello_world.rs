use std::sync::Arc;

use hyper::Body;
use log::info;
use thruster::{
    context::basic_hyper_context::HyperRequest, context::typed_hyper_context::TypedHyperContext,
    hyper_server::HyperServer, m, middleware_fn, App, MiddlewareNext, MiddlewareResult,
    ThrusterServer,
};
use thruster_jab::{fetch, provide, JabDI};

type Ctx = TypedHyperContext<RequestConfig>;

struct ServerConfig {
    di: Arc<JabDI>,
}

struct RequestConfig {
    di: Arc<JabDI>,
}

fn generate_context(request: HyperRequest, state: &ServerConfig, _path: &str) -> Ctx {
    Ctx::new(
        request,
        RequestConfig {
            di: state.di.clone(),
        },
    )
}

trait GreetingService {
    fn get_name(&self) -> String;
}

struct SpongebobGreeter;

impl GreetingService for SpongebobGreeter {
    fn get_name(&self) -> String {
        "Ahoy".to_string()
    }
}

#[middleware_fn]
async fn plaintext(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let val = format!(
        "{}, World!",
        fetch!(context.extra.di, dyn GreetingService).get_name()
    );
    context.body = Body::from(val);
    Ok(context)
}

#[middleware_fn]
async fn not_found(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let val = "Not found";
    context.body = Body::from(val);
    Ok(context)
}

fn app(jab_di: JabDI) -> App<HyperRequest, Ctx, ServerConfig> {
    let mut app = App::<HyperRequest, Ctx, ServerConfig>::create(
        generate_context,
        ServerConfig {
            di: Arc::new(jab_di),
        },
    );
    app.get("/plaintext", m![plaintext]);
    app.set404(m![not_found]);

    app
}

fn main() {
    env_logger::init();
    info!("Starting server...");

    let mut jab_di = JabDI::default();

    provide!(jab_di, dyn GreetingService, SpongebobGreeter);

    let server = HyperServer::new(app(jab_di));
    server.start("0.0.0.0", 4321);
}

#[cfg(test)]
mod tests {
    use hyper::{Body, Request};
    use thruster::testing;
    use tokio::runtime::Runtime;

    use crate::*;

    struct SquidwardGreeter;

    impl GreetingService for SquidwardGreeter {
        fn get_name(&self) -> String {
            "Go away".to_string()
        }
    }

    #[test]
    fn test_insertion() {
        let mut jab_di = JabDI::default();

        provide!(jab_di, dyn GreetingService, SquidwardGreeter);

        let _ = Runtime::new().unwrap().block_on(async {
            let app = app(jab_di).commit();

            let response = testing::request(
                &app,
                Request::builder()
                    .method("GET")
                    .uri("/plaintext")
                    .body(Body::from(""))
                    .unwrap(),
            )
            .await;

            assert_eq!(response.body_string(), "Go away, World!");
        });
    }
}

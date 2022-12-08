use poem::http::{StatusCode, Uri};
use poem::middleware::{AddData, AddDataEndpoint};
use poem::web::Data;
use poem::{handler, Body, EndpointExt, IntoResponse, Response, Route};
use tera::Context;

use crate::util::template_utils::{to_template_name, DEFAULT_TEMPLATE, TEMPLATES};

use super::template_utils::SinglePageApplication;

///  view: the template which renders the app
///
///  Full example (to render the `views/spa.html` template):
/// ```
///  app = app.nest("/my-spa", create_rust_app::render_single_page_application("spa.html"));
/// ```
pub fn render_single_page_application(view: &str) -> AddDataEndpoint<Route, SinglePageApplication> {
    let view = view.strip_prefix("/").unwrap_or(view);

    Route::new()
        .at("*", poem::get(render_spa_handler))
        .with(AddData::new(SinglePageApplication {
            view_name: view.to_string(),
        }))
}

#[handler]
async fn render_spa_handler(spa_info: Data<&SinglePageApplication>) -> impl IntoResponse {
    let content = TEMPLATES
        .render(spa_info.view_name.as_str(), &Context::new())
        .unwrap();
    template_response(content)
}

#[handler]
pub async fn render_views(uri: &Uri) -> impl IntoResponse {
    let path = uri.path();

    #[cfg(debug_assertions)]
    if path.eq("/__vite_ping") {
        println!("The vite dev server seems to be down...");
        return StatusCode::NOT_FOUND.into_response();
    }

    let mut template_path = to_template_name(path);
    let mut content_result = TEMPLATES.render(template_path, &Context::new());

    if content_result.is_err() {
        #[cfg(debug_assertions)]
        {
            // dev asset serving
            let asset_path = &format!("./frontend{path}");
            if std::path::PathBuf::from(asset_path).is_file() {
                println!("ASSET_FILE {path} => {asset_path}");

                return file_response(asset_path).await;
            }

            let public_path = &format!("./frontend/public{path}");
            if std::path::PathBuf::from(public_path).is_file() {
                println!("PUBLIC_FILE {path} => {public_path}");

                return file_response(public_path).await;
            }
        }

        #[cfg(not(debug_assertions))]
        {
            // production asset serving
            let static_path = &format!("./frontend/dist{path}");
            if std::path::PathBuf::from(static_path).is_file() {
                return file_response(static_path).await;
            }
        }

        content_result = TEMPLATES.render(DEFAULT_TEMPLATE, &Context::new());
        template_path = DEFAULT_TEMPLATE;
        if content_result.is_err() {
            // default template doesn't exist -- return 404 not found
            return StatusCode::NOT_FOUND.into();
        }
    }

    println!("TEMPLATE_FILE {path} => {template_path}");

    let content = content_result.unwrap();

    template_response(content)
}

fn template_response(content: String) -> Response {
    let mut content = content;
    #[cfg(debug_assertions)]
    {
        let inject: &str = r##"
        <!-- development mode -->
        <script type="module">
            import RefreshRuntime from 'http://localhost:21012/@react-refresh'
            RefreshRuntime.injectIntoGlobalHook(window)
            window.$RefreshReg$ = () => {}
            window.$RefreshSig$ = () => (type) => type
            window.__vite_plugin_react_preamble_installed__ = true
        </script>
        <script type="module" src="http://localhost:21012/src/dev.tsx"></script>
        "##;

        if content.contains("<body>") {
            content = content.replace("<body>", &format!("<body>{inject}"));
        } else {
            content = format!("{inject}{content}");
        }
    }

    Response::builder()
        .status(StatusCode::OK)
        .content_type("text/html")
        .body(content)
}

async fn file_response(path: &String) -> Response {
    let file = tokio::fs::read(path).await.unwrap();
    let content_type = mime_guess::from_path(path).first_raw();
    let mut response = Response::builder();
    if content_type.is_some() {
        response = response.content_type(content_type.unwrap());
    }
    response = response.status(StatusCode::OK);
    response.body(Body::from(file))
}
